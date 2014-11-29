extern crate test;

use std::rand::{task_rng, Rng};

use super::partitioning::{Partitioning};

#[bench]
fn insert_new_value(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();

    b.iter(|| {
        p.insert(value);
    })
}

#[bench]
fn insert_existing_value(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.insert(value);
    })
}

#[bench]
fn find_existing_value(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.find(value);
    })
}

#[bench]
fn find_missing_value(b: &mut test::Bencher) {
    let p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();

    b.iter(|| {
        p.find(value);
    })
}

#[bench]
fn remove_missing_value(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();

    b.iter(|| {
        p.remove(value);
    })
}

#[bench]
fn remove_existing_value(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.remove(value);
    })
}

