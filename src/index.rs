use crate::{SimilarityMeasure, Vector};



pub trait Index<S: SimilarityMeasure> {
    fn insert(&mut self, vector: Vector, limit: usize);
    fn query(&self, query: &Vector, count: usize) -> Vec<(f64, &Vector)>;
    fn create() -> Self;
}
