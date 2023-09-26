use crate::Vector;

pub struct Hyperplane {
    base: Vector,
    normal: Vector,
}

impl Hyperplane {
    pub fn new(base: Vector, normal: Vector) -> Self {
        Self { base, normal }
    }
}

impl From<(&Vector, &Vector)> for Hyperplane {
    fn from((a, b): (&Vector, &Vector)) -> Self {
        let mut normal = b - a;
        normal *= 0.5;

        let base = a + &normal;
        normal.normalize();

        Self { base, normal }
    }
}

impl std::cmp::PartialEq<Vector> for Hyperplane {
    fn eq(&self, other: &Vector) -> bool {
        (&self.normal * &(other - &self.base)).abs() < f64::EPSILON
    }
}

impl std::cmp::PartialOrd<Vector> for Hyperplane {
    fn partial_cmp(&self, other: &Vector) -> Option<std::cmp::Ordering> {
        let angle = &self.normal * &(other - &self.base);

        if angle.abs() < f64::EPSILON {
            Some(std::cmp::Ordering::Less)
        } else if angle > f64::EPSILON {
            Some(std::cmp::Ordering::Greater)
        } else {
            Some(std::cmp::Ordering::Equal)
        }
    }
}
