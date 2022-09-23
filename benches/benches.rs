use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, Bencher, Criterion};
use kvs::thread_pool::{SharedQueueThreadPool, ThreadPool, RayonThreadPool};
use kvs::{KvStore, KvsClient, KvsEngine, KvsServer, SledKvsEngine};
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};
use sloggers::null::NullLoggerBuilder;
use sloggers::Build;
use tempfile::TempDir;

const N_PAIRS: usize = 100;

pub fn write_bench_group(c: &mut Criterion) {
    let mut g = c.benchmark_group("concurrency write bench");
    g.sample_size(10);

    let thread_nums = get_thread_num_inputs();
    for thread_num in thread_nums {
        g.bench_with_input(
            format!("write_queued_kvstore_{}_threads", &thread_num),
            &thread_num,
            write_bench::<KvStore, SharedQueueThreadPool>,
        );
        g.bench_with_input(
            format!("write_rayon_kvstore_{}_threads", &thread_num),
            &thread_num,
            write_bench::<KvStore, RayonThreadPool>,
        );
        g.bench_with_input(
            format!("write_rayon_sled_{}_threads", &thread_num),
            &thread_num,
            write_bench::<SledKvsEngine, RayonThreadPool>,
        );
    }
}

pub fn read_bench_group(c: &mut Criterion) {
    let mut g = c.benchmark_group("concurrency read bench");
    g.sample_size(10);

    let thread_nums = get_thread_num_inputs();
    for thread_num in thread_nums {
        g.bench_with_input(
            format!("read_queued_kvstore_{}_threads", &thread_num),
            &thread_num,
            read_bench::<KvStore, SharedQueueThreadPool>,
        );
        g.bench_with_input(
            format!("read_rayon_kvstore_{}_threads", &thread_num),
            &thread_num,
            write_bench::<KvStore, RayonThreadPool>,
        );
        g.bench_with_input(
            format!("read_rayon_sled_{}_threads", &thread_num),
            &thread_num,
            write_bench::<SledKvsEngine, RayonThreadPool>,
        );
    }
}

criterion_group!(benches, write_bench_group, read_bench_group);
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

fn write_bench<E, T>(b: &mut Bencher, thread_num: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    b.iter_batched(
        || {
            let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
            let e = E::open(tmp_dir.path().join("db")).expect("Fail in db initial");
            let logger = NullLoggerBuilder.build().unwrap();
            let thread_pool = T::new(*thread_num as u32).expect("Fail in ThreadPool initial");

            let mut server =
                KvsServer::new(logger, e, thread_pool, "127.0.0.1:0").expect("Fail in Server initial");
            let addr = server.get_address();

            let server = thread::spawn(move || {
                server.run().unwrap();
            });

            thread::sleep(Duration::from_secs(1));

            let client_pool = T::new(num_cpus::get() as u32).unwrap();

            let datas: Vec<(String, String)> = generate_pairs(N_PAIRS);

            (server, client_pool, datas, addr, tmp_dir)
        },
        |(_, client_pool, datas, addr, _)| {
            let (sender, receiver) = channel();
            for (k, v) in datas {
                let sender_cp = sender.clone();
                client_pool.spawn(move || {
                    let mut client = KvsClient::new(addr).unwrap();
                    if let Ok(_) = client.set(k, v) {
                        while sender_cp.send(0).is_err() {}
                    } else {
                        panic!("set error");
                    }
                });
            }

            for _ in 0..N_PAIRS {
                assert_eq!(receiver.recv().unwrap(), 0);
            }
        },
        BatchSize::SmallInput,
    );
}

fn read_bench<E, T>(b: &mut Bencher, thread_num: &usize)
where
    E: KvsEngine,
    T: ThreadPool + Send + 'static,
{
    b.iter_batched(
        || {
            let tmp_dir = TempDir::new().expect("Fail in creating temporary directory");
            let e = E::open(tmp_dir.path().join("db")).expect("Fail in db initial");
            let logger = NullLoggerBuilder.build().unwrap();
            let thread_pool = T::new(*thread_num as u32).expect("Fail in ThreadPool initial");

            let datas: Vec<(String, String)> = generate_pairs(N_PAIRS);

            for (k, v) in datas.clone() {
                e.set(k, v).expect("Fail in insert kv to sled db");
            }

            let mut server =
                KvsServer::new(logger, e, thread_pool, "127.0.0.1:0").expect("Fail in Server initial");
            let addr = server.get_address();

            let server = thread::spawn(move || {
                server.run().unwrap();
            });

            thread::sleep(Duration::from_secs(1));

            let client_pool = T::new(num_cpus::get() as u32).unwrap();

            let index_list: Vec<usize> = (0..1000)
                .map(|_| thread_rng().gen_range(0..N_PAIRS))
                .collect();

            (server, client_pool, datas, addr, tmp_dir, index_list)
        },
        |(_, client_pool, datas, addr, _, index_list)| {
            let (sender, receiver) = channel();
            for i in index_list {
                let sender_cp = sender.clone();
                let (k, v) = datas[i].clone();
                client_pool.spawn(move || {
                    let mut client = KvsClient::new(addr).unwrap();
                    if let Ok(value) = client.get(k) {
                        assert_eq!(v.as_str(), value.unwrap().as_str());
                        while sender_cp.send(0).is_err() {}
                    } else {
                        panic!("get error value of given key")
                    }
                });
            }

            for _ in 0..1000 {
                assert_eq!(receiver.recv().unwrap(), 0);
            }
        },
        BatchSize::SmallInput,
    );
}
