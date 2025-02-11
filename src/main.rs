mod cache;
mod result;

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use cache::Cache;
use cached::SizedCache;
use lru::LruCache;
use parking_lot::Mutex;
use quick_cache::sync::Cache as QuickCache;
use rayon::prelude::*;
use result::BenchResult;
use tabled::Table;

fn bench_cache<C>(name: &str, cache: C) -> BenchResult
where
    C: Cache + Clone + Send + Sync + 'static,
{
    let hit_counter = Arc::new(Mutex::new(0u64));
    let total_ops = NUM_THREADS * OPS_PER_THREAD;
    let start = Instant::now();

    (0..NUM_THREADS).into_par_iter().for_each(|thread_id| {
        let cache = cache.clone();
        let hit_counter = Arc::clone(&hit_counter);
        let mut local_hits = 0;

        for i in 0..OPS_PER_THREAD {
            let key = (i + thread_id * OPS_PER_THREAD) % (CACHE_SIZE * 2);

            if let Some(_) = cache.get_key(&key) {
                local_hits += 1;
            } else {
                cache.set_key(key, format!("value_{}", key));
            }
        }

        let mut hits = hit_counter.lock();
        *hits += local_hits;
    });

    let elapsed = start.elapsed();
    let hits = *hit_counter.lock();

    BenchResult {
        name: name.to_string(),
        total_time: elapsed.as_millis(),
        ops_per_sec: total_ops as f64 / elapsed.as_secs_f64(),
        hit_rate: hits as f64 / total_ops as f64,
    }
}

const CACHE_SIZE: usize = 10_000;
const NUM_THREADS: usize = 8;
const OPS_PER_THREAD: usize = 100_000;

fn main() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(NUM_THREADS)
        .build()
        .unwrap();

    println!("Running cache benchmarks...");
    println!("Configuration:");
    println!("  Cache size: {}", CACHE_SIZE);
    println!("  Threads: {}", NUM_THREADS);
    println!("  Operations per thread: {}", OPS_PER_THREAD);
    println!();

    let results = pool.install(|| {
        vec![
            bench_cache("quick_cache", Arc::new(QuickCache::new(CACHE_SIZE))),
            bench_cache(
                "lru",
                Arc::new(Mutex::new(LruCache::new(
                    NonZeroUsize::new(CACHE_SIZE).unwrap(),
                ))),
            ),
            bench_cache(
                "cached",
                Arc::new(Mutex::new(SizedCache::with_size(CACHE_SIZE))),
            ),
        ]
    });

    println!("Results:");

    let table = Table::new(results).to_string();
    println!("{}", table);
}
