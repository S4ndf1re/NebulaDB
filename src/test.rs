#![cfg(test)]

use crate::{CosineSimilarity, Index, InsertOptions, QueryOptions, Vector};

#[test]
fn simple_cosine() {
    let vec1 = Vector::new(&[1.0, 0.0, 0.0]);
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0]);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0]);

    assert!(1.0 - &vec1 * &vec1 <= f64::EPSILON);
    assert!(-1.0 - &vec1 * &vec1_inv <= f64::EPSILON);
    assert!(&vec1 * &vec2 <= f64::EPSILON);
}

#[test]
fn simple_insert() {
    let mut collection = Index::<CosineSimilarity>::new("test".to_owned(), 3);

    let vec1 = Vector::new(&[1.0, 0.0, 0.0]);
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0]);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0]);

    collection
        .upsert(&[vec1, vec1_inv, vec2], InsertOptions::default())
        .unwrap();
}

#[test]
fn simple_insert_and_query() {
    let mut collection = Index::<CosineSimilarity>::new("test".to_owned(), 3);

    let vec1 = Vector::new(&[1.0, 0.0, 0.0]);
    let query = vec1.clone();
    let vec1_inv = Vector::new(&[-1.0, 0.0, 0.0]);
    let vec2 = Vector::new(&[0.0, 1.0, 0.0]);

    collection
        .upsert(&[vec1, vec1_inv, vec2], InsertOptions::default())
        .unwrap();

    let result = collection.query(&query, QueryOptions::new());
    assert!(result.len() == 3);
    assert!(1.0 - result[0].0 <= f64::EPSILON);
    assert!(result[1].0 <= f64::EPSILON);
    assert!(-1.0 - result[2].0 <= f64::EPSILON);
}

#[test]
fn one_million_vectors_test() {
    let mut collection = Index::<CosineSimilarity>::new("test".to_owned(), 3);

    let mut points = vec![];
    for _ in 0..1_000_000 {
        let v = Vector::new(&[
            rand::random::<f64>(),
            rand::random::<f64>(),
            rand::random::<f64>(),
        ]);
        points.push(v);
    }

    let timer = std::time::Instant::now();

    collection.upsert(points, InsertOptions::default()).unwrap();

    println!("Elapsed build index: {} ms", timer.elapsed().as_millis());

    let timer = std::time::Instant::now();
    let v = Vector::new(&[
        rand::random::<f64>(),
        rand::random::<f64>(),
        rand::random::<f64>(),
    ]);
    let result_size = collection.query(&v, QueryOptions::default()).len();

    println!("Elapsed query: {} ns, for {} entries", timer.elapsed().as_nanos(), result_size);
}
