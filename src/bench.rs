extern crate test;

use std::rand::{task_rng, Rng};

use super::partitioning::{Partitioning};


#[bench]
fn find_random_key_10(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let seq = rng.gen_iter::<uint>();

    for i in seq.take(10u) {
        p.insert(i);
    }

    b.iter(|| {
        let mut rng = task_rng();
        p.find(rng.gen());
    })
}

#[bench]
fn find_random_key_100(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let seq = rng.gen_iter::<uint>();

    for i in seq.take(100u) {
        p.insert(i);
    }

    b.iter(|| {
        let mut rng = task_rng();
        p.find(rng.gen());
    })
}

#[bench]
fn find_random_key_1000(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let seq = rng.gen_iter::<uint>();

    for i in seq.take(1000u) {
        p.insert(i);
    }

    b.iter(|| {
        let mut rng = task_rng();
        p.find(rng.gen());
    })
}

#[bench]
fn find_random_key_10000(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let seq = rng.gen_iter::<uint>();

    for i in seq.take(10000u) {
        p.insert(i);
    }

    b.iter(|| {
        let mut rng = task_rng();
        p.find(rng.gen());
    })
}

#[bench]
fn find_random_key_100000(b: &mut test::Bencher) {
    let mut p: Partitioning<uint> = Partitioning::new(64, 4);

    let mut rng = task_rng();
    let seq = rng.gen_iter::<uint>();

    for i in seq.take(100000u) {
        p.insert(i);
    }

    b.iter(|| {
        let mut rng = task_rng();
        p.find(rng.gen());
    })
}
