use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct Metadata {
    pub map: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Vector {
    pub data: Arc<[f64]>,
    pub metadata: Option<Metadata>,
}

impl Vector {
    pub fn new(data: &[f64], metadata: Option<Metadata>) -> Self {
        let mut v = Vec::new();
        v.reserve_exact(data.len());

        for i in data.as_ref() {
            v.push(*i);
        }

        let mut s = Vector {
            data: Arc::from(v),
            metadata,
        };
        s.normalize();

        s
    }

    pub fn get_size(&self) -> usize {
        self.data.len()
    }

    pub fn abs(&self) -> f64 {
        f64::sqrt(self * self)
    }

    pub fn normalize(&mut self) {
        let abs = self.abs();

        let mut vec = Vec::new();
        vec.reserve_exact(self.data.len());

        for num in self.data.iter() {
            vec.push(num / abs);
        }

        self.data = Arc::from(vec);
    }
}

impl std::ops::Mul for &Vector {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        assert!(self.data.len() == rhs.data.len());

        let mut sum = 0.0;
        for i in 0..self.data.len() {
            sum += self.data[i] * rhs.data[i];
        }

        sum
    }
}
