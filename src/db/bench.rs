extern crate test;
extern crate rand;

use self::rand::{thread_rng, Rng};

use db::{Database, SubstitutionDB, DeletionDB};

#[bench]
fn insert_large_value(b: &mut test::Bencher) {
    let mut p: DeletionDB<Vec<u8>> = Database::new(100, 4);

    let mut rng = thread_rng();
    let mut value = Vec::with_capacity(100);
    for _ in (0..100) {
        value.push(rng.gen());
    }

    b.iter(|| {
        p.insert(value.clone());
    })
}

#[bench]
fn insert_new_value(b: &mut test::Bencher) {
    let mut p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();

    b.iter(|| {
        p.insert(value);
    })
}

#[bench]
fn insert_existing_value(b: &mut test::Bencher) {
    let mut p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.insert(value);
    })
}

#[bench]
fn find_existing_value(b: &mut test::Bencher) {
    let mut p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.get(&value);
    })
}

#[bench]
fn find_missing_value(b: &mut test::Bencher) {
    let p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();

    b.iter(|| {
        p.get(&value);
    })
}

#[bench]
fn remove_missing_value(b: &mut test::Bencher) {
    let mut p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();

    b.iter(|| {
        p.remove(&value);
    })
}

#[bench]
fn remove_existing_value(b: &mut test::Bencher) {
    let mut p: SubstitutionDB<usize> = Database::new(64, 4);

    let mut rng = thread_rng();
    let value = rng.gen();
    p.insert(value);

    b.iter(|| {
        p.remove(&value);
    })
}

