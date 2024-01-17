use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use crate::{id_provider::IdGuard, Error, Result, Vector};

pub struct Layer {
    pub start_node: Arc<IdGuard<usize>>,
    nodes: HashMap<Arc<IdGuard<usize>>, Vector>,
    connections: HashMap<Arc<IdGuard<usize>>, HashSet<Arc<IdGuard<usize>>>>,
}

impl Layer {
    pub fn new(start_vector: Vector) -> Self {
        let mut nodes = HashMap::new();

        let start_node = Arc::clone(&start_vector.id);
        nodes.insert(Arc::clone(&start_vector.id), start_vector);

        let mut connections = HashMap::new();
        connections.insert(Arc::clone(&start_node), HashSet::new());

        Self {
            start_node,
            nodes,
            connections,
        }
    }

    pub fn add_vector(&mut self, v: Vector) -> Result<()> {
        let id = Arc::clone(&v.id.clone());
        self.nodes.insert(Arc::clone(&id), v);
        self.connections.insert(id, HashSet::new());
        Ok(())
    }

    pub fn add_connection(
        &mut self,
        id1: Arc<IdGuard<usize>>,
        id2: Arc<IdGuard<usize>>,
    ) -> Result<()> {
        if !self.nodes.contains_key(&id1) {
            return Err(Error::IdDoesNotExist(*id1.deref().deref()));
        } else if !self.nodes.contains_key(&id2) {
            return Err(Error::IdDoesNotExist(*id2.deref().deref()));
        }

        // NOTE: if the connection list is not existent, insert a new one
        let mut connections = self.connections.get_mut(&id1);
        if connections.is_none() {
            self.connections.insert(Arc::clone(&id1), HashSet::new());
            connections = self.connections.get_mut(&id1);
        }
        let connections = connections.unwrap();

        if connections.contains(&id2) {
            return Ok(()); // TODO: consider returning an error????
        }

        connections.insert(Arc::clone(&id2));

        Ok(())
    }

    pub fn add_bidirectional_connection(
        &mut self,
        id1: Arc<IdGuard<usize>>,
        id2: Arc<IdGuard<usize>>,
    ) -> Result<()> {
        self.add_connection(Arc::clone(&id1), Arc::clone(&id2))?;
        self.add_connection(id2, id1)?;
        Ok(())
    }

    pub fn remove_connections_for_id(&mut self, id: &Arc<IdGuard<usize>>) -> Result<()> {
        let connections = self.connections.get_mut(id);
        if let None = connections {
            return Ok(());
        }

        *connections.unwrap() = HashSet::new();
        Ok(())
    }

    pub fn get_vector_ptr(&self, id: &Arc<IdGuard<usize>>) -> Result<*const Vector> {
        if !self.nodes.contains_key(id) {
            return Err(Error::IdDoesNotExist(*id.deref().deref()));
        }

        Ok(&self.nodes[id] as *const Vector)
    }

    pub fn get_connections_for_id(
        &self,
        id: &Arc<IdGuard<usize>>,
    ) -> Result<Vec<&Arc<IdGuard<usize>>>> {
        if !self.nodes.contains_key(id) {
            return Err(Error::IdDoesNotExist(*id.deref().deref()));
        }

        let mut result = vec![];
        for c in self
            .connections
            .get(id)
            .ok_or(Error::IdDoesNotExist(*id.deref().deref()))?
        {
            result.push(c)
        }

        Ok(result)
    }
}
