use crate::{SimilarityMeasure, Vector, Result};


pub(crate) struct ScoredVector {
    pub score: f64,
    pub vector: *const Vector,
}

impl ScoredVector {
    pub(crate) fn new(score: f64, vector: *const Vector) -> Self {
        Self { score, vector }
    }
}

impl PartialEq for ScoredVector {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredVector {}

impl PartialOrd for ScoredVector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for ScoredVector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.total_cmp(&other.score)
    }
}

pub trait Index<S: SimilarityMeasure> {
    fn insert(&mut self, vector: Vector, limit: usize) -> Result<()>;
    fn query<'a>(&'a self, query: &Vector, count: usize) -> Result<Vec<(f64, &'a Vector)>>;
    fn create(vec_size: usize) -> Self;
}
