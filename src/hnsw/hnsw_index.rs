use std::{
    collections::BinaryHeap,
    collections::{HashMap, HashSet},
    ops::Deref,
    sync::Arc,
};

use rand::distributions::{Distribution, Uniform};

use crate::{id_provider::IdGuard, Error, Result, Vector};

pub struct ScoredVector {
    pub score: f64,
    pub vector: *const Vector,
}

impl ScoredVector {
    fn new(score: f64, vector: *const Vector) -> Self {
        Self { score, vector }
    }
}

impl PartialEq for ScoredVector {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredVector {}

impl PartialOrd for ScoredVector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for ScoredVector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.total_cmp(&other.score)
    }
}

pub struct Layer {
    nodes: HashMap<Arc<IdGuard<usize>>, Vector>,
    connections: HashMap<Arc<IdGuard<usize>>, HashSet<Arc<IdGuard<usize>>>>,
}

impl Layer {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
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

const MAX_0: usize = 1_000;

pub struct HnswIndex {
    top_layer: usize,
    layers: Vec<Layer>,
}

impl HnswIndex {
    pub fn new(layers: usize) -> Self {
        let mut layer_vec = vec![];
        for _ in 0..layers {
            layer_vec.push(Layer::new());
        }
        Self {
            top_layer: layers,
            layers: layer_vec,
        }
    }

    /// get all neighbours of e in layer
    unsafe fn get_neighborhood<'a>(
        &'a self,
        e: *const Vector,
        layer: usize,
    ) -> Result<Vec<*const Vector>> {
        let layer = &self.layers[layer];
        let connections = layer.get_connections_for_id(&(&*e).id)?;

        // funky looking, but this works due to collect being able to convert Vec<Result<_>> to
        // Result<Vec<_>>
        connections
            .into_iter()
            .map(|c| layer.get_vector_ptr(c))
            .collect()
    }

    /// set all neighbours of e in layer to neighboorhood
    unsafe fn set_neighborhood(
        &mut self,
        e: *const Vector,
        layer: usize,
        neighboorhood: Vec<*const Vector>,
    ) -> Result<()> {
        let layer = &mut self.layers[layer];
        layer.remove_connections_for_id(&(&*e).id)?;

        for n in neighboorhood {
            // this is ok, since as of now, the code is private and the logic guarantees, that this
            // is dereferenced only when the vector is still in the layer
            layer.add_bidirectional_connection(Arc::clone(&(&*e).id), Arc::clone(&(*n).id))?;
        }

        Ok(())
    }

    unsafe fn search_layer<'a>(
        &'a self,
        q: *const Vector,
        eq: &Vec<*const Vector>,
        ef: usize,
        lc: usize,
    ) -> Vec<*const Vector> {
        let mut visited = eq.clone();

        let mut candidates = BinaryHeap::new();
        for e in eq {
            let score = Vector::mul_unchecked(*e, q);
            candidates.push(ScoredVector::new(-score, *e));
        }

        let mut w = BinaryHeap::new();
        for e in eq {
            let score = Vector::mul_unchecked(*e, q);
            w.push(ScoredVector::new(score, *e));
        }

        while !candidates.is_empty() {
            // select closest element from candidates to q
            let c = candidates.pop().unwrap().vector;

            // select most distant element in w from q
            let f = w.peek().expect("this may never be empty");
            let f_vec = f.vector;

            if Vector::mul_unchecked(c, q) > Vector::mul_unchecked(f_vec, q) {
                break;
            }

            for e in self.get_neighborhood(c, lc).unwrap() {
                if !visited.contains(&e) {
                    visited.push(e);
                    let eq_score = Vector::mul_unchecked(e, q);
                    if eq_score < Vector::mul_unchecked(f_vec, q) || w.len() < ef {
                        w.push(ScoredVector::new(eq_score, e));
                        candidates.push(ScoredVector::new(-eq_score, e));
                        if w.len() > ef {
                            w.pop();
                        }
                    }
                }
            }
        }
        
        return w.into_sorted_vec().into_iter().map(|v| v.vector).collect();
    }

    unsafe fn select_neighbours<'a>(
        q: *const Vector,
        w: &Vec<*const Vector>,
        m: usize,
        _lc: usize,
    ) -> Vec<*const Vector> {
        // TODO: replace with heursitic
        // TODO: make w a priority queue (Heap)
        let mut tmp_res = vec![];
        for e in w.iter() {
            let dst = Vector::mul_unchecked(q, *e);
            tmp_res.push((dst, *e));
        }

        tmp_res.sort_by(|v1, v2| v1.0.total_cmp(&v2.0));
        tmp_res.into_iter().take(m).map(|v| v.1).collect()
    }

    unsafe fn insert_inner(
        &mut self,
        q: Vector,
        m: usize,
        m_max: usize,
        ef_construction: usize,
        m_l: f64,
    ) -> Result<()> {
        let mut rng = rand::thread_rng();
        let uniform_rng = Uniform::new(0.0, 1.0);

        let mut ep = vec![];
        let l = self.top_layer;
        let new_layer = f64::floor(-f64::ln(uniform_rng.sample(&mut rng)) * m_l) as usize;

        for lc in (new_layer..l + 1).rev() {
            let w = self.search_layer(&q, &ep, 1, lc);
            ep = Self::select_neighbours(&q, &w, 1, lc);
        }

        for lc in (0..l.min(new_layer)).rev() {
            self.layers[lc].add_vector(q.clone())?;
            let w = self.search_layer(&q, &ep, ef_construction, lc);
            let neighbors = Self::select_neighbours(&q, &w, m, lc);
            for n in &neighbors {
                self.layers[lc]
                    .add_bidirectional_connection(Arc::clone(&q.id), Arc::clone(&(&**n).id))?;
            }
            for e in neighbors {
                let e_neighbours = self.get_neighborhood(e, lc)?;
                let m_max = if lc == 0 { MAX_0 } else { m_max };
                if e_neighbours.len() < m_max {
                    let e_new_neighbours = Self::select_neighbours(e, &e_neighbours, m_max, lc);
                    self.set_neighborhood(e, lc, e_new_neighbours)?;
                }
            }
        }

        Ok(())
    }

    pub fn insert(&mut self, q: Vector) -> Result<()> {
        unsafe {
            self.insert_inner(q, 32, 40, 0, 1.0) // TODO: use self parameters and use good defaults
        }
    }
}
