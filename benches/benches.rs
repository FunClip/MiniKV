use std::net::SocketAddr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Bencher};
use kvs::thread_pool::ThreadPool;
use kvs::{KvStore, KvsEngine, SledKvsEngine, KvsServer, KvsClient};
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use sloggers::Build;
use sloggers::null::NullLoggerBuilder;
use tempfile::TempDir;

const N_PAIRS: usize = 100;


pub fn write_bench_old(c: &mut Criterion) {
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

pub fn read_bench_old(c: &mut Criterion) {
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

criterion_group!(benches, write_bench_old, read_bench_old);
criterion_main!(benches);


fn generate_random_string(min: usize, max: usize) -> String {
    let len: usize = thread_rng().gen_range(min..max);
    Alphanumeric.sample_string(&mut thread_rng(), len)
}

fn generate_pairs(size: usize) -> Vec<(String, String)> {
    (0..size)
        .map(|_| {
            (
                generate_random_string(1, 100000),
                generate_random_string(1, 100000),
            )
        })
        .collect()
}

fn get_thread_num_inputs() -> Vec<usize> {
    let core_num = num_cpus::get() * 2;
    (0..10)
        .map(|n| 1 << n)
        .take_while(|n| n <= &core_num)
        .collect()
}

fn write_bench<E, T>(b: &mut Bencher, thread_num: usize)
    where
        E: KvsEngine,
        T: ThreadPool + Send + 'static,
{
    b.iter_batched(
        || {
            let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
            let e = E::open(tmp_dir.path().join("db")).expect("Fail in db initial");
            let logger = NullLoggerBuilder.build().unwrap();
            let thread_pool = T::new(thread_num as u32).expect("Fail in ThreadPool initial");

            let addr: SocketAddr = format!("127.0.0.1:{}", 14869 + thread_num).parse().unwrap();
            let mut server = KvsServer::new(&logger, e, thread_pool).expect("Fail in Server initial");

            let server = thread::spawn(move || {
                server.run(addr).unwrap();
            });

            thread::sleep(Duration::from_secs(1));

            let client_pool = T::new(num_cpus::get() as u32).unwrap();

            let datas: Vec<(String, String)> = generate_pairs(N_PAIRS);

            (server, client_pool, datas, addr, tmp_dir)
        },
        |(server, client_pool, datas, addr, _)|{
            let (sender, receiver) = channel();
            for (k, v) in datas {
                let sender_cp = sender.clone();
                client_pool.spawn(move || {
                    let mut client = KvsClient::new(addr).unwrap();
                    if let Ok(_) = client.set(k, v) {
                        while sender_cp.send(0).is_err() {}
                    }
                    else {
                        panic!("set error");
                    }
                });
            }

            for _ in 0..N_PAIRS {
                assert_eq!(receiver.recv().unwrap(), 0);
            }

            // TODO: add safe shutdown method for kvsServer
        },
        BatchSize::SmallInput);
}

fn read_bench<E, T>(b: &mut Bencher, thread_num: usize)
    where
        E: KvsEngine,
        T: ThreadPool + Send + 'static,
{
    b.iter_batched(
        || {

        },
        ||{

        },
        BatchSize::SmallInput);
}