use std::collections::HashMap;
use std::sync::Arc;

use crate::id_provider::{IdGuard, SharedIdGuard};

#[derive(Clone)]
pub struct Metadata {
    pub map: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Vector {
    pub id: Arc<IdGuard<usize>>,
    pub data: Vec<f64>,
}

impl Vector {
    pub fn new(id: SharedIdGuard<usize>, mut data: Vec<f64>) -> Self {
        data.shrink_to_fit();
        let mut s = Vector { id, data };
        s.normalize();

        s
    }

    pub fn new_with_default_id(mut data: Vec<f64>) -> Self {
        data.shrink_to_fit();
        let mut s = Vector {
            id: Arc::new(IdGuard::default()),
            data,
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

        self.data = vec;
    }

}

impl From<Vec<f64>> for Vector {
    fn from(mut vec: Vec<f64>) -> Self {
        vec.shrink_to_fit();
        Self {
            id: Arc::new(IdGuard::default()),
            data: vec,
        }
    }
}

impl std::ops::Mul for &Vector {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        assert_eq!(self.data.len(), rhs.data.len());

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
        assert_eq!(self.data.len(), rhs.data.len());

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
        assert_eq!(self.data.len(), rhs.data.len());

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

        self.data = vec_res;
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        if self.data.len() != other.data.len() {
            return false;
        }

        for i in 0..self.data.len() {
            if (self.data[i] - other.data[i]) > f64::EPSILON {
                return false;
            }
        }

        true
    }
}


impl Vector {
    pub unsafe fn mul_unchecked(a: *const Vector, b: *const Vector) -> f64 {
        let a = &*a;
        let b = &*b;

        a * b
    }
}
