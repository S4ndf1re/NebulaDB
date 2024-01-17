use std::collections::HashMap;

use crate::id_provider::SharedIdGuard;

pub type JsonMap = HashMap<String, String>;

pub struct PayloadStore {
    payloads: HashMap<SharedIdGuard<usize>, JsonMap>,
}

impl PayloadStore {
    pub fn new() -> Self {
        Self {
            payloads: HashMap::new(),
        }
    }

    /// Add payload the id.
    /// If a payload existed before, return the old payload
    pub fn add_payload(&mut self, id: SharedIdGuard<usize>, payload: JsonMap) -> Option<JsonMap> {
        self.payloads.insert(id, payload)
    }

    /// Remove the possibly existing payload from the store
    pub fn remove_payload(&mut self, id: &SharedIdGuard<usize>) -> Option<JsonMap> {
        self.payloads.remove(id)
    }

    /// Check if payload does exist for id
    pub fn contains_payload(&mut self, id: &SharedIdGuard<usize>) -> bool {
        self.payloads.contains_key(id)
    }

    pub fn find_by_payload(&self, payload: &JsonMap) -> Vec<SharedIdGuard<usize>> {
        let mut result = vec![];
        for (k, v) in &self.payloads {
            let mut is_match = true;
            for (key, value) in payload {
                if v.get(key) != Some(value) {
                    is_match = false;
                    break;
                }
            }

            if is_match {
                result.push(k.clone());
            }
        }
        result
    }
}
