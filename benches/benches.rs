use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use tempfile::TempDir;

fn generate_random_string(min: usize, max: usize) -> String {
    let len: usize = thread_rng().gen_range(min..max);
    Alphanumeric.sample_string(&mut thread_rng(), len)
}

pub fn write_bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("write bench");

    g.bench_function("kvs write", |b| {
        b.iter_batched(
            || {
                let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
                let e =
                    KvStore::open(tmp_dir.path().join("kvs_db")).expect("Fail in kvs db initial");
                let datas: Vec<(String, String)> = (0..100)
                    .map(|_| {
                        (
                            generate_random_string(1, 100000),
                            generate_random_string(1, 100000),
                        )
                    })
                    .collect();

                (e, datas, tmp_dir)
            },
            |(mut e, data, _)| {
                for (k, v) in data {
                    e.set(k, v).expect("Fail in insert kv to kvs db");
                }
            },
            BatchSize::SmallInput,
        );
    });

    g.bench_function("sled write", |b| {
        b.iter_batched(
            || {
                let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
                let e = SledKvsEngine::open(tmp_dir.path().join("sled_db"))
                    .expect("Fail in sled db initial");
                let datas: Vec<(String, String)> = (0..100)
                    .map(|_| {
                        (
                            generate_random_string(1, 100000),
                            generate_random_string(1, 100000),
                        )
                    })
                    .collect();

                (e, datas, tmp_dir)
            },
            |(mut e, data, _)| {
                for (k, v) in data {
                    e.set(k, v).expect("Fail in insert kv to sled db");
                }
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn read_bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("read bench");

    g.bench_function("kvs read", |b| {
        b.iter_batched(
            || {
                let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
                let mut e =
                    KvStore::open(tmp_dir.path().join("kvs_db")).expect("Fail in kvs db initial");
                let datas: Vec<(String, String)> = (0..100)
                    .map(|_| {
                        (
                            generate_random_string(1, 100000),
                            generate_random_string(1, 100000),
                        )
                    })
                    .collect();

                for (k, v) in datas.clone() {
                    e.set(k, v).expect("Fail in insert kv to kvs db");
                }

                let index_list: Vec<usize> =
                    (0..1000).map(|_| thread_rng().gen_range(0..100)).collect();

                (e, datas, index_list, tmp_dir)
            },
            |(mut e, data, index_list, _)| {
                for i in index_list {
                    let (k, v) = &data[i];
                    let value = e.get(k.to_owned()).expect("Fail in get key from kvs");
                    assert_eq!(v.as_str(), value.unwrap().as_str());
                }
            },
            BatchSize::SmallInput,
        );
    });

    g.bench_function("sled read", |b| {
        b.iter_batched(
            || {
                let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
                let mut e = SledKvsEngine::open(tmp_dir.path().join("sled_db"))
                    .expect("Fail in db initial");
                let datas: Vec<(String, String)> = (0..100)
                    .map(|_| {
                        (
                            generate_random_string(1, 100000),
                            generate_random_string(1, 100000),
                        )
                    })
                    .collect();
                for (k, v) in datas.clone() {
                    e.set(k, v).expect("Fail in insert kv to sled db");
                }

                let index_list: Vec<usize> =
                    (0..1000).map(|_| thread_rng().gen_range(0..100)).collect();

                (e, datas, index_list, tmp_dir)
            },
            |(mut e, data, index_list, _)| {
                for i in index_list {
                    let (k, v) = &data[i];
                    let value = e.get(k.to_owned()).expect("Fail in get key from sled");
                    assert_eq!(v.as_str(), value.unwrap().as_str());
                }
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, write_bench, read_bench);
criterion_main!(benches);
