#![cfg(test)]

use crate::{Collection, CosineSimilarity, QueryOptions, Vector};

#[test]
fn simple_cosine() {
    let vec1 = Vector::new(&[1.0, 0.0, 0.0], None);
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0], None);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0], None);

    assert!(1.0 - &vec1 * &vec1 <= f64::EPSILON);
    assert!(-1.0 - &vec1 * &vec1_inv <= f64::EPSILON);
    assert!(&vec1 * &vec2 <= f64::EPSILON);
}

#[test]
fn simple_insert() {
    let mut collection = Collection::<CosineSimilarity>::new("test".to_owned(), 3);

    let vec1 = Vector::new(&[1.0, 0.0, 0.0], None);
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0], None);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0], None);

    collection.upsert(&[vec1, vec1_inv, vec2]).unwrap();
}

#[test]
fn simple_insert_and_query() {
    let mut collection = Collection::<CosineSimilarity>::new("test".to_owned(), 3);

    let vec1 = Vector::new(&[1.0, 0.0, 0.0], None);
    let query = vec1.clone();
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0], None);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0], None);

    collection.upsert(&[vec1, vec1_inv, vec2]).unwrap();

    let result = collection.query(&query, QueryOptions::new());
    assert!(result.len() == 3);
    assert!(1.0 - result[0].0 <= f64::EPSILON);
    assert!(result[1].0 <= f64::EPSILON);
    assert!(-1.0 - result[2].0 <= f64::EPSILON);
}
