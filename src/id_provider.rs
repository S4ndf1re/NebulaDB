use std::cell::RefCell;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

pub trait AddOne: Default + Clone {
    /// Add one has as its constraint, that each increment will result in a unique value starting from default
    fn add_one(self) -> Self;
}

impl AddOne for usize {
    fn add_one(self) -> Self {
        self + 1
    }
}

pub(crate) struct IdGuard<I> {
    value: I,
    free: Arc<Mutex<Vec<I>>>,
}

impl<I> IdGuard<I>
    where I: Hash + PartialEq + Clone {
    pub fn new(id: I, free: Arc<Mutex<Vec<I>>>) -> Self {
        Self {
            value: id,
            free,
        }
    }
}

impl<I> Drop for IdGuard<I>
    where I: Hash + PartialEq + Clone {
    fn drop(&mut self) {
        self.free.lock().unwrap().push(self.value.clone());
    }
}

pub(crate) struct IdProvider<I> {
    next_free: RefCell<I>,
    free: Arc<Mutex<Vec<I>>>,
}


impl<I> IdProvider<I>
    where I: Hash + PartialEq + AddOne {
    pub fn new() -> Self {
        Self {
            next_free: I::default(),
            free: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn claim_new_id(&self) -> IdGuard<I> {
        let mut free_locked = self.free.lock().unwrap();
        return if free_locked.is_empty() {
            IdGuard::new(self.next_free.replace(self.next_free.borrow().clone().add_one()), Arc::clone(&self.free))
        } else {
            IdGuard::new(free_locked.pop().unwrap(), Arc::clone(&self.free))
        };
    }
}