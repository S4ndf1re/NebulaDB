use std::marker::PhantomData;

use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub use error::*;
pub use hyperplane::*;
pub use options::*;
pub use payload_store::*;
pub use similarity::*;
pub use vector::*;

pub mod util;

pub mod error;

pub mod options;

pub mod vector;

pub mod hyperplane;

pub mod similarity;

mod test;
pub mod payload_store;
pub(crate) mod id_provider;


pub struct VectorInsert {
    pub vec: Vector,
    pub payload: Option<JsonMap>,
}

/// Annoy Index node
/// Left node means, all vectors are below the hyperplane
/// Right means, that all vectors are above the hyperplane
pub enum IndexNode {
    Leaf {
        vectors: Vec<Vector>,
    },
    Node {
        left: Box<IndexNode>,
        right: Box<IndexNode>,
        total: usize,
        hyperplane: Hyperplane,
    },
}

impl IndexNode {
    pub fn new_leaf(vectors: Vec<Vector>) -> Self {
        Self::Leaf { vectors }
    }

    pub fn new_node(left: Box<IndexNode>, right: Box<IndexNode>, hyperplane: Hyperplane) -> Self {
        let mut node = Self::Node {
            left,
            right,
            total: 0,
            hyperplane,
        };

        node.update_count();
        node
    }

    pub fn update_count(&mut self) -> usize {
        match self {
            IndexNode::Leaf { vectors } => vectors.len(),
            IndexNode::Node {
                left,
                right,
                total,
                hyperplane: _,
            } => {
                *total = left.update_count() + right.update_count();
                *total
            }
        }
    }

    pub fn insert(&mut self, vector: Vector, limit: usize) {
        match self {
            IndexNode::Leaf { vectors } => {
                vectors.push(vector);

                if vectors.len() > limit {
                    let ((_, a), (_, b)) = util::two_randoms(&vectors);
                    let plane = Hyperplane::from((a, b));

                    let mut left_list = vec![];
                    let mut right_list = vec![];

                    let vectors = std::mem::take(vectors);
                    let total = vectors.len();

                    for v in vectors.into_iter() {
                        if plane <= v {
                            right_list.push(v);
                        } else {
                            left_list.push(v);
                        }
                    }

                    *self = IndexNode::Node {
                        left: IndexNode::Leaf { vectors: left_list }.into(),
                        right: IndexNode::Leaf {
                            vectors: right_list,
                        }
                            .into(),
                        total,
                        hyperplane: plane,
                    }
                }
            }
            IndexNode::Node {
                left,
                right,
                total,
                hyperplane,
            } => {
                *total += 1;
                if *hyperplane <= vector {
                    right.insert(vector, limit);
                } else {
                    left.insert(vector, limit);
                }
            }
        }
    }

    /// Query all embeddings using annoy index
    /// NOTE: the sort order is ascending, meaning the last values are the closest to the query
    fn query<S>(&self, query: &Vector, count: usize) -> Vec<(f64, &Vector)>
        where
            S: SimilarityMeasure,
    {
        match self {
            IndexNode::Leaf { vectors } => {
                let mut vectors: Vec<_> = vectors
                    .into_iter()
                    .map(|v| (S::compare(query, v), v))
                    .collect();
                vectors.sort_by(|a, b| a.0.total_cmp(&b.0));
                vectors.into_iter().take(count).collect()
            }
            IndexNode::Node {
                left,
                right,
                total,
                hyperplane,
            } => {
                if total > &count {
                    if hyperplane <= query {
                        right.query::<S>(query, count)
                    } else {
                        left.query::<S>(query, count)
                    }
                } else {
                    let left_res = left.query::<S>(query, count);
                    let right_res = right.query::<S>(query, count);
                    let result = util::merge_by(left_res, right_res);
                    result
                }
            }
        }
    }
}

pub struct Index<S> {
    vec_len: usize,
    index: IndexNode,
    payload_store: PayloadStore,
    _name: String,
    _phantom_s: PhantomData<S>,
}

impl<S> Index<S>
    where
        S: SimilarityMeasure,
{
    pub fn new(name: String, len: usize) -> Self {
        Self {
            vec_len: len,
            index: IndexNode::Leaf { vectors: vec![] },
            payload_store: PayloadStore::new(),
            _name: name,
            _phantom_s: PhantomData {},
        }
    }

    pub fn upsert<P: AsRef<[VectorInsert]>>(&mut self, points: P, options: InsertOptions) -> Result<()> {
        for i in 0..points.as_ref().len() {
            let point = &points.as_ref()[i];
            if point.vec.data.len() != self.vec_len {
                return Err(Error::VectorLengthInvalid {
                    index: i,
                    expected: self.vec_len,
                    found: point.vec.data.len(),
                });
            }

            if options.with_payload && point.payload.is_none() {
                return Err(Error::MissingPayload);
            } else if !options.with_payload && point.payload.is_some() {
                return Err(Error::ProvidedPayload);
            }

            if !options.autoset_id {
                todo!("check for only unique and not inserted ids")
            }
        }

        for point in points.as_ref() {
            self.index.insert(point.vec.to_owned(), options.limit);
            if point.payload.is_some() {
                self.payload_store.add_payload(point.vec.id, point.payload.unwrap().to_owned());
            }
        }

        Ok(())
    }

    /// query all vectors and get most similar
    /// Note that this function may spawn threads (rayon)
    pub fn query(
        &self,
        ref_point: &Vector,
        options: QueryOptions,
    ) -> Vec<(f64, &Vector)> {
        // NOTE: Use Rayon to parallelize computation
        let mut result = self.index.query::<S>(ref_point, options.limit);

        if options.ascending {
            result = result
                .into_par_iter()
                .filter(|x| x.0 <= options.cutoff)
                .collect();
            result.par_sort_by(|x, y| x.0.total_cmp(&y.0));
        } else {
            result = result
                .into_par_iter()
                .filter(|x| x.0 >= options.cutoff)
                .collect();
            result.par_sort_by(|x, y| y.0.total_cmp(&x.0));
        }

        result
    }
}
