use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{index::Index, Error, JsonMap, PayloadStore, Result, SimilarityMeasure, Vector};
use std::{
    collections::VecDeque,
    marker::PhantomData,
    mem,
    ops::DerefMut,
    sync::{Arc, Mutex, RwLock},
};

#[derive(Debug)]
pub struct InsertElement {
    pub vec: Vector,
    pub payload: Option<JsonMap>,
    pub limit: usize,
    pub answer_channel: oneshot::Sender<Result<()>>,
}

impl InsertElement {
    pub fn create_insert(
        vec: Vector,
        payload: Option<JsonMap>,
        limit: usize,
        answer_channel: oneshot::Sender<Result<()>>,
    ) -> Self {
        Self {
            vec,
            payload,
            limit,
            answer_channel,
        }
    }
}

pub struct QueryElement {
    pub vec: Vector,
    pub cutoff: f64,
    pub ascending: bool,
    pub limit: usize,
    pub answer_channel: oneshot::Sender<Result<Vec<(f64, Vector)>>>,
}

impl QueryElement {
    pub fn create_query(
        vec: Vector,
        cutoff: f64,
        ascending: bool,
        limit: usize,
        answer_channel: oneshot::Sender<Result<Vec<(f64, Vector)>>>,
    ) -> Self {
        Self {
            vec,
            cutoff,
            ascending,
            limit,
            answer_channel,
        }
    }
}

pub struct OperationQueue<I, S> {
    queue: RwLock<VecDeque<QueryElement>>,
    insert_queue: Mutex<VecDeque<InsertElement>>,
    index: RwLock<I>,
    payload_idx: RwLock<PayloadStore>,
    phantom_s: PhantomData<S>,
}

impl<I, S> OperationQueue<I, S>
where
    S: SimilarityMeasure,
    I: Index<S>,
{
    pub fn new(index: I, payload_idx: PayloadStore) -> Self {
        Self {
            queue: RwLock::new(VecDeque::new()),
            insert_queue: Mutex::new(VecDeque::new()),
            index: RwLock::new(index),
            payload_idx: RwLock::new(payload_idx),
            phantom_s: PhantomData,
        }
    }

    pub fn add_query(&self, query: QueryElement) -> Result<()> {
        let mut queue = self.queue.write().unwrap();
        queue.push_back(query);
        Ok(())
    }

    pub fn add_insert(&self, query: InsertElement) -> Result<()> {
        let mut queue = self.insert_queue.lock().unwrap();
        queue.push_back(query);
        Ok(())
    }

    fn insert_all(&self) -> Result<()> {
        // First lock this queue to be able to check if insert is ok at this point in time.
        // Also, this lock prevents an infinite wait for the index lock.
        // I know that this locking thing is a bit annoying, but i don't currently see another
        // option, because some of the locks need to bee RwLocks, while the insert_queue for
        // example can be just a Mutext. Also not all locks need to wait for eachother. Especially
        // both query and insert queues share only a small subset of operations.
        let queue = self.queue.read().map_err(|_| Error::MutexLockError)?;
        if !queue.is_empty() {
            return Ok(());
        }

        let mut ins_queue = self
            .insert_queue
            .lock()
            .map_err(|_| Error::MutexLockError)?;

        let mut index = self.index.write().map_err(|_| Error::MutexLockError)?;
        let mut payloads = self
            .payload_idx
            .write()
            .map_err(|_| Error::MutexLockError)?;
        let list = mem::replace(ins_queue.deref_mut(), VecDeque::new());

        for elem in list {
            if elem.payload.is_some() {
                payloads.add_payload(Arc::clone(&elem.vec.id), elem.payload.unwrap());
            }

            elem.answer_channel
                .send(index.insert(elem.vec, elem.limit))
                .map_err(|_e| Error::SendToQueryQueueChannelFailed)?;
        }

        ins_queue.clear();

        Ok(())
    }

    fn query_next(&self) -> Result<()> {
        // First aquire the first element of the queue and remove it from said.
        // After unlock the mutex again, in order to allow other threads to potentially handle the
        // next queue item
        // Also ignore the insert queue
        let mut queue = self.queue.write().map_err(|_| Error::MutexLockError)?;
        if let Some(query) = queue.pop_front() {
            drop(queue); // Drop the queue here to give back lock. Only hold a read lock
                         // after this point
            let _queue = self.queue.read().map_err(|_| Error::MutexLockError)?;

            // This lock is actually ok and will never wait, because the lock will only be
            // aquired after queue is also locked. This condition is NOT quaranteed by the rust compiler
            // though and is only preseved with code.
            let index = self.index.read().map_err(|_| Error::MutexLockError)?;

            let result = index.query(&query.vec, query.limit);
            let result = result.map(|x| x.into_iter().map(|x| (x.0, Clone::clone(x.1))).collect());
            let result = result.map(|r: Vec<(f64, Vector)>| {
                if query.ascending {
                    let mut result: Vec<(f64, Vector)> = r
                        .into_par_iter()
                        .filter(|x| x.0 <= query.cutoff && !x.0.is_nan())
                        .collect();
                    result.par_sort_by(|x, y| x.0.total_cmp(&y.0));
                    result
                } else {
                    let mut result: Vec<(f64, Vector)> = r
                        .into_par_iter()
                        .filter(|x| x.0 >= query.cutoff && !x.0.is_nan())
                        .collect();
                    result.par_sort_by(|x, y| y.0.total_cmp(&x.0));
                    result
                }
            });
            query
                .answer_channel
                .send(result)
                .map_err(|_e| Error::SendToQueryQueueChannelFailed)?;
        }
        Ok(())
    }

    /// work_queues allows for multithreaded querying of the database.
    /// In order for this to work, the queries and insert request need to be passed to the
    /// underlying queues.
    /// For this, the methods add_query and add_insert can be used.
    pub fn work_queues(&self) -> Result<()> {
        self.insert_all()?;
        self.query_next()
    }
}
