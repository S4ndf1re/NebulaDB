use std::{collections::BinaryHeap, sync::Arc};

use rand::distributions::{Distribution, Uniform};

use crate::{index::Index, Result, ScoredVector, SimilarityMeasure, Vector};
use crate::id_provider::IdGuard;

use super::Layer;

const MAX_0: usize = 1_000;

pub struct HnswIndex {
    top_layer: usize,
    layers: Vec<Layer>,
}

impl HnswIndex {
    pub fn new(layers: usize, vec_len: usize) -> Self {
        let mut layer_vec = vec![];
        for _ in 0..layers {
            let mut id: IdGuard<usize> = IdGuard::default();
            id.set_value(usize::MAX);
            let v = Vector::new(Arc::new(id), vec![0.0; vec_len]);
            let layer = Layer::new(v);
            layer_vec.push(layer);
        }
        Self {
            top_layer: layers - 1,
            layers: layer_vec,
        }
    }

    /// get all neighbours of e in layer
    unsafe fn get_neighborhood(
        &self,
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
        neighborhood: Vec<*const Vector>,
    ) -> Result<()> {
        let layer = &mut self.layers[layer];
        layer.remove_connections_for_id(&(&*e).id)?;

        for n in neighborhood {
            // this is ok, since as of now, the code is private and the logic guarantees, that this
            // is dereferenced only when the vector is still in the layer
            layer.add_bidirectional_connection(Arc::clone(&(&*e).id), Arc::clone(&(*n).id))?;
        }

        Ok(())
    }

    unsafe fn search_layer(
        &self,
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
        // TODO: replace with heuristic
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

        let l = self.top_layer;
        let mut ep = vec![self.layers[l].get_vector_ptr(&self.layers[l].start_node)?];

        let new_layer = f64::floor(-f64::ln(uniform_rng.sample(&mut rng)) * m_l) as usize;

        for lc in (new_layer..l + 1).rev() {
            let w = self.search_layer(&q, &ep, 1, lc);
            ep = Self::select_neighbours(&q, &w, 1, lc);
        }

        for lc in (0..std::cmp::min(l, new_layer)).rev() {
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

    unsafe fn knn_search(&self, q: &Vector, k: usize, ef: usize) -> Result<Vec<*const Vector>> {
        let l = self.top_layer;
        let mut ep = vec![self.layers[l].get_vector_ptr(&self.layers[l].start_node)?];

        for lc in (0..l).rev() {
            let w = self.search_layer(q, &ep, ef, lc);
            ep = Self::select_neighbours(q, &w, 1, lc);
        }
        let w = self.search_layer(q, &ep, ef, 0);
        Ok(Self::select_neighbours(q, &w, k, 0))
    }
}

impl<S: SimilarityMeasure> Index<S> for HnswIndex {
    fn insert(&mut self, vector: Vector, _: usize) -> Result<()> {
        unsafe {
            self.insert_inner(vector, 32, 40, 100, 1.0) // TODO: use self parameters and use good defaults
        }
    }

    fn query(&self, q: &Vector, k: usize) -> Result<Vec<(f64, &Vector)>> {
        let w = unsafe {
            self.knn_search(q, k, 100)?
                .into_iter()
                .map(|v| {
                    let score = Vector::mul_unchecked(v, q);
                    (score, &*v)
                })
                .collect()
        };

        Ok(w)
    }

    fn create(vec_size: usize) -> Self {
        Self::new(4, vec_size)
    }
}
