#![feature(step_trait)]

use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

pub use error::*;
pub use hyperplane::*;
pub use index::*;
pub use options::*;
pub use payload_store::*;
pub use query_queue::*;
pub use similarity::*;
pub use vector::*;

use crate::id_provider::IdProvider;

pub mod query_queue;

pub mod util;

pub mod error;

pub mod options;

pub mod vector;

pub mod hyperplane;

pub mod annoy_index;
pub mod hnsw;
pub mod index;
pub mod similarity;

pub(crate) mod id_provider;
pub mod payload_store;
mod test;

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

pub struct Index<S, I> {
    vec_len: usize,
    queue: OperationQueue<I, S>,
    id_provider: IdProvider<usize>,
    _name: String,
    _phantom_s: PhantomData<S>,
}

impl<S, I> Index<S, I>
where
    S: SimilarityMeasure,
    I: index::Index<S>,
{
    pub fn new(name: String, len: usize) -> Self {
        Self {
            vec_len: len,
            queue: OperationQueue::new(I::create(len), PayloadStore::new()),
            id_provider: IdProvider::new(),
            _name: name,
            _phantom_s: PhantomData {},
        }
    }

    pub fn upsert<P: AsRef<[VectorInsert]>>(
        &mut self,
        points: P,
        options: InsertOptions,
    ) -> Result<()> {
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

        let mut rx_list = vec![];

        for point in points.as_ref() {
            let mut point = point.clone();

            let id = if options.autoset_id {
                self.id_provider.claim_new_id()
            } else {
                self.id_provider
                    .claim_if_free(&point.id)
                    .ok_or(Error::IdAlreadyExists(point.id))?
            };

            point.vec.id = Arc::clone(&id);

            let (tx, rx) = oneshot::channel();
            rx_list.push(rx);
            let insert_element =
                InsertElement::create_insert(point.vec, point.payload, options.limit, tx);
            self.queue.add_insert(insert_element)?;
        }

        self.queue.work_queues()?; // TODO: multithreading
        for rx in rx_list {
            let _ = rx.recv()?;
        }

        Ok(())
    }

    /// query all vectors and get most similar
    /// Note that this function may spawn threads (rayon)
    /// Scores with value NaN are ignored
    pub fn query(&self, ref_point: &Vector, options: QueryOptions) -> Result<Vec<(f64, Vector)>> {
        // NOTE: Use Rayon to parallelize computation
        let (tx, rx) = oneshot::channel();
        let query_elem = QueryElement::create_query(
            ref_point.to_owned(),
            options.cutoff,
            options.ascending,
            options.limit,
            tx,
        );
        self.queue.add_query(query_elem)?;
        self.queue.work_queues()?; // TODO: multithreading

        let result = rx.recv()??;
        Ok(result)
    }
}
