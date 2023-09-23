use std::marker::PhantomData;

use rayon::{prelude::{IntoParallelRefIterator, ParallelIterator, IntoParallelIterator}, slice::ParallelSliceMut};

pub mod error;
pub use error::*;

pub mod options;
pub use options::*;

pub mod vector;
pub use vector::*;

pub mod similarity;
pub use similarity::*;

mod test;

pub struct Collection<S> {
    vec_len: usize,
    vectors: Vec<Vector>,
    _name: String,
    _phantom_s: PhantomData<S>,
}

impl<S> Collection<S>
where
    S: SimilarityMeasure,
{
    pub fn new(name: String, len: usize) -> Self {
        Self {
            vec_len: len,
            vectors: vec![],
            _name: name,
            _phantom_s: PhantomData {},
        }
    }

    pub fn upsert<P: AsRef<[Vector]>>(&mut self, points: P) -> Result<()> {
        for i in 0..points.as_ref().len() {
            let point = &points.as_ref()[i];
            if point.data.len() != self.vec_len {
                return Err(Error::VectorLengthInvalid {
                    index: i,
                    expected: self.vec_len,
                    found: point.data.len(),
                });
            }
        }

        for point in points.as_ref() {
            self.vectors.push(point.to_owned());
        }

        Ok(())
    }

    /// query all vectors and get most similar
    /// Note that this function may spawn threads (rayon)
    pub fn query<'a>(
        &'a self,
        ref_point: &Vector,
        options: QueryOptions,
    ) -> Vec<(f64, &'a Vector)> {
        // NOTE: Use Rayon to parrarelize computation
        let mut result: Vec<_> = self
            .vectors
            .par_iter()
            .map(|vec| (S::compare(ref_point, vec), vec))
            .collect();

        if options.ascending {
            result = result.into_par_iter().filter(|x| x.0 <= options.cutoff).collect();
            result.par_sort_by(|x, y| x.0.total_cmp(&y.0));
        } else {
            result = result.into_par_iter().filter(|x| x.0 >= options.cutoff).collect();
            result.par_sort_by(|x, y| y.0.total_cmp(&x.0));
        }

        if let Some(limit) = options.limit {
            result = result.into_iter().take(limit).collect();
        }

        result
    }
}
