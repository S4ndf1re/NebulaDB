#![feature(step_trait)]

use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

pub use index::*;
pub use error::*;
pub use hyperplane::*;
pub use options::*;
pub use payload_store::*;
pub use similarity::*;
use crate::annoy_index::IndexNode;
pub use vector::*;

use crate::id_provider::IdProvider;

pub mod util;

pub mod error;

pub mod options;

pub mod vector;

pub mod hyperplane;

pub mod index;
pub mod similarity;
pub mod hnsw_index;
pub mod annoy_index;

mod test;
pub mod payload_store;
pub(crate) mod id_provider;


#[derive(Clone)]
pub struct VectorInsert {
    pub id: usize,
    pub vec: Vector,
    pub payload: Option<JsonMap>,
}

impl From<Vector> for VectorInsert {
    fn from(value: Vector) -> Self {
        Self {
            id: value.id.deref().deref().clone(),
            vec: value,
            payload: None,
        }
    }
}


pub struct Index<S,I> {
    vec_len: usize,
    index: I,
    payload_store: PayloadStore,
    id_provider: IdProvider<usize>,
    _name: String,
    _phantom_s: PhantomData<S>,
}

impl<S, I> Index<S, I>
    where
        S: SimilarityMeasure,
        I: index::Index<S>
{
    pub fn new(name: String, len: usize) -> Self {
        Self {
            vec_len: len,
            index: I::create(),
            payload_store: PayloadStore::new(),
            id_provider: IdProvider::new(),
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
                if !self.id_provider.check_is_free(&point.id) {
                    return Err(Error::IdAlreadyExists(point.id));
                }
            }
        }

        for point in points.as_ref() {
            let mut point = point.clone();

            let id = if options.autoset_id {
                self.id_provider.claim_new_id()
            } else {
                self.id_provider.claim_if_free(&point.id).ok_or(Error::IdAlreadyExists(point.id))?
            };

            point.vec.id = Arc::clone(&id);

            self.index.insert(point.vec, options.limit);
            if point.payload.is_some() {
                self.payload_store.add_payload(id, point.payload.unwrap().to_owned());
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
        let mut result = self.index.query(ref_point, options.limit);

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
