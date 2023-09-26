use std::{collections::HashMap, sync::Arc};


#[derive(Clone)]
pub struct Metadata {
    pub map: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Vector {
    pub data: Arc<[f64]>,
}

impl Vector {
    pub fn new(data: &[f64]) -> Self {
        let mut v = Vec::new();
        v.reserve_exact(data.len());

        for i in data.as_ref() {
            v.push(*i);
        }

        let mut s = Vector { data: Arc::from(v) };
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

impl From<Vec<f64>> for Vector {
    fn from(vec: Vec<f64>) -> Self {
        Self {
            data: Arc::from(vec),
        }
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

impl std::ops::Sub for &Vector {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.data.len() == rhs.data.len());

        let mut vec = Vec::new();
        vec.reserve_exact(self.data.len());

        for i in 0..self.data.len() {
            vec.push(self.data[i] - rhs.data[i]);
        }

        vec.into()
    }
}

impl std::ops::Add for &Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(self.data.len() == rhs.data.len());

        let mut vec = Vec::new();
        vec.reserve_exact(self.data.len());

        for i in 0..self.data.len() {
            vec.push(self.data[i] + rhs.data[i]);
        }

        vec.into()
    }
}

impl std::ops::MulAssign<f64> for Vector {
    fn mul_assign(&mut self, rhs: f64) {
        let mut vec_res = vec![];

        for i in self.data.iter() {
            vec_res.push(i * rhs);
        }

        self.data = Arc::from(vec_res);
    }
}

impl std::cmp::PartialEq  for Vector {
    fn eq(&self, other: &Self) -> bool {
        if self.data.len() != other.data.len() {
            return false
        }

        for i in 0..self.data.len() {
            if (self.data[i] - other.data[i]) > f64::EPSILON {
                return false
            }
        }
        
        true
    }
}
