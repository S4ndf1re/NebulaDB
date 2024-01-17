use std::cell::RefCell;
use std::hash::Hash;
use std::iter::Step;
use std::ops::Deref;
use std::sync::{Arc, Mutex};


pub trait Max {
    fn max() -> Self;
}

impl Max for usize {
    fn max() -> Self {
        usize::MAX
    }
}

pub type SharedIdGuard<I> = Arc<IdGuard<I>>;

#[derive(Debug)]
pub struct IdGuard<I>
    where I: Hash + PartialEq + Clone
{
    value: I,
    free: Option<Arc<Mutex<Vec<I>>>>,
}

impl<I> IdGuard<I>
    where I: Hash + PartialEq + Clone {
    pub fn new(id: I, free: Arc<Mutex<Vec<I>>>) -> Self {
        Self {
            value: id,
            free: Some(free),
        }
    }

    pub(crate) fn set_value(&mut self, new_value: I) {
        self.value = new_value;
    }
}

impl<I> Default for IdGuard<I>
    where I: Default + Hash + PartialEq + Clone
{
    fn default() -> Self {
        Self {
            value: I::default(),
            free: None,
        }
    }
}

impl<I> PartialEq for IdGuard<I>
    where I: Hash + PartialEq + Clone {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<I> Eq for IdGuard<I>
    where I: Hash + PartialEq + Clone {}

impl<I> Drop for IdGuard<I>
    where I: Hash + PartialEq + Clone {
    fn drop(&mut self) {
        self.free.as_mut().map(|free| free.lock().unwrap().push(self.value.clone()));
    }
}

impl<I> Deref for IdGuard<I>
    where I: Hash + PartialEq + Clone
{
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<I> Hash for IdGuard<I>
    where I: Hash + PartialEq + Clone {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

pub(crate) struct IdProvider<I> {
    next_free: RefCell<I>,
    free: Arc<Mutex<Vec<I>>>,
}


impl<I> IdProvider<I>
    where I: Hash + PartialEq + Step + Max + Default + PartialOrd {
    pub fn new() -> Self {
        Self {
            next_free: RefCell::new(I::default()),
            free: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Claim a free ID. Either a completely new ID is claimed, or a previously freed ID is claimed.
    pub fn claim_new_id(&self) -> SharedIdGuard<I> {
        let mut free_locked = self.free.lock().unwrap();
        if free_locked.is_empty() {
            let next = Step::forward(self.next_free.borrow().clone(), 1);
             if next == I::max() {
                 panic!("No more ids available"); // TODO change api to use Result
             }
            Arc::new(IdGuard::new(self.next_free.replace(next), Arc::clone(&self.free)))
        } else {
            Arc::new(IdGuard::new(free_locked.pop().unwrap(), Arc::clone(&self.free)))
        }
    }

    /// Claim a single ID if the id is free. Note however that this will hog a lot of memory, if the requested id is a lot bigger than the current next free id.
    /// If the id could not be claimed, None is returned.
    /// An id cannot be claimed, if it is currently claimed by another IdGuard.
    pub fn claim_if_free(&self, new_id: &I) -> Option<SharedIdGuard<I>> {
        if new_id >= self.next_free.borrow().deref() {
            let mut locked = self.free.lock().unwrap();
            // Free all ids from next_free to new_id, so that no ids are lost when calling claim_new_id next time
            for id in self.next_free.borrow().clone()..new_id.clone() {
                locked.push(id);
            }
            // now actually claim the new id and set the next free id to new_id (the claimed on) + 1
            let result = Arc::new(IdGuard::new(new_id.clone(), Arc::clone(&self.free)));
            self.next_free.replace(Step::forward(new_id.clone(), 1));
            Some(result)
        } else {
            // The id may only be claimed, if it is in the free list, because when next_free is larger then new_id,
            // then the id may already be claimed by another IdGuard
            let mut locked = self.free.lock().unwrap();
            if locked.contains(new_id) {
                let id_idx = locked.iter().position(|x| x == new_id).unwrap();
                locked.swap_remove(id_idx);
                Some(Arc::new(IdGuard::new(new_id.clone(), Arc::clone(&self.free))))
            } else {
                None
            }
        }
    }

    pub fn check_is_free(&self, id: &I) -> bool {
        if id >= self.next_free.borrow().deref() {
            true
        } else {
            let locked = self.free.lock().unwrap();
            locked.contains(id)
        }
    }
}
