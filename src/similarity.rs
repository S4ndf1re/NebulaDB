use crate::Vector;

pub trait SimilarityMeasure {
    /// compare to Vectors using any measure. 0.0 is considered as unequal,
    /// 1 is considered as equal.
    /// -1 is considered as opposite.
    fn compare(vec1: &Vector, vec2: &Vector) -> f64;
}

pub struct CosineSimilarity {}

impl SimilarityMeasure for CosineSimilarity {
    fn compare(vec1: &Vector, vec2: &Vector) -> f64 {
        vec1 * vec2
    }
}
