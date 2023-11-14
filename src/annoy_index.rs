use crate::{Hyperplane, SimilarityMeasure, util, Vector};

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
    pub fn query<S>(&self, query: &Vector, count: usize) -> Vec<(f64, &Vector)>
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
