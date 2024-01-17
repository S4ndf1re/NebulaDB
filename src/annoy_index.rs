use crate::{index::Index, util, Hyperplane, Result, SimilarityMeasure, Vector};

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
}

impl<S: SimilarityMeasure> Index<S> for IndexNode {
    fn create(_: usize) -> Self {
        return Self::Leaf { vectors: vec![] };
    }

    fn query<'a>(&'a self, query: &Vector, count: usize) -> Result<Vec<(f64, &'a Vector)>> {
        match self {
            IndexNode::Leaf { vectors } => {
                let mut vectors: Vec<_> = vectors
                    .into_iter()
                    .map(|v| (S::compare(query, v), v))
                    .collect();
                vectors.sort_by(|a, b| a.0.total_cmp(&b.0));
                Ok(vectors.into_iter().take(count).collect())
            }
            IndexNode::Node {
                left,
                right,
                total,
                hyperplane,
            } => {
                if total > &count {
                    if hyperplane <= query {
                        Index::<S>::query(right.as_ref(), query, count)
                    } else {
                        Index::<S>::query(left.as_ref(), query, count)
                    }
                } else {
                    let left_res = Index::<S>::query(left.as_ref(), query, count)?;
                    let right_res = Index::<S>::query(right.as_ref(), query, count)?;
                    let result = util::merge_by(left_res, right_res);
                    Ok(result)
                }
            }
        }
    }

    fn insert(&mut self, vector: Vector, limit: usize) -> Result<()> {
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
                Ok(())
            }
            IndexNode::Node {
                left,
                right,
                total,
                hyperplane,
            } => {
                *total += 1;
                if *hyperplane <= vector {
                    Index::<S>::insert(right.as_mut(), vector, limit)?;
                } else {
                    Index::<S>::insert(left.as_mut(), vector, limit)?;
                }
                Ok(())
            }
        }
    }
}
