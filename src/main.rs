use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Instant;

use cached::Cached;
use cached::SizedCache;
use lru::LruCache;
use parking_lot::Mutex;
use quick_cache::sync::Cache as QuickCache;
use rayon::prelude::*;
use tabled::{Table, Tabled};

const CACHE_SIZE: usize = 10_000;
const NUM_THREADS: usize = 8;
const OPS_PER_THREAD: usize = 100_000;

#[derive(Clone, Tabled)]
struct BenchResult {
    #[tabled(rename = "Type")]
    name: String,
    #[tabled(rename = "Hit Rate", format("{:.2}", self.hit_rate * 100.0))]
    hit_rate: f64,
    #[tabled(rename = "Ops/sec", format("{:.3}", self.ops_per_sec ))]
    ops_per_sec: f64,
    /// The total time (in msl)
    #[tabled(rename = "Time (ms)")]
    total_time: u128,
}

fn bench_quick_cache() -> BenchResult {
    let cache = Arc::new(QuickCache::new(CACHE_SIZE));
    let hit_counter = Arc::new(Mutex::new(0u64));
    let total_ops = NUM_THREADS * OPS_PER_THREAD;
    let start = Instant::now();

    (0..NUM_THREADS).into_par_iter().for_each(|thread_id| {
        let cache = Arc::clone(&cache);
        let hit_counter = Arc::clone(&hit_counter);
        let mut local_hits = 0;

        for i in 0..OPS_PER_THREAD {
            let key = (i + thread_id * OPS_PER_THREAD) % (CACHE_SIZE * 2);

            if let Some(_) = cache.get(&key) {
                local_hits += 1;
            } else {
                cache.insert(key, format!("value_{}", key));
            }
        }

        let mut hits = hit_counter.lock();
        *hits += local_hits;
    });

    let elapsed = start.elapsed();
    let hits = *hit_counter.lock();

    BenchResult {
        name: "quick_cache".to_string(),
        total_time: elapsed.as_millis(),
        ops_per_sec: total_ops as f64 / elapsed.as_secs_f64(),
        hit_rate: hits as f64 / total_ops as f64,
    }
}

fn bench_lru() -> BenchResult {
    let size = NonZeroUsize::new(CACHE_SIZE).unwrap();
    let cache = Arc::new(Mutex::new(LruCache::new(size)));
    let hit_counter = Arc::new(Mutex::new(0u64));
    let total_ops = NUM_THREADS * OPS_PER_THREAD;
    let start = Instant::now();

    (0..NUM_THREADS).into_par_iter().for_each(|thread_id| {
        let cache = cache.clone();
        let hit_counter = Arc::clone(&hit_counter);
        let mut local_hits = 0;

        for i in 0..OPS_PER_THREAD {
            let key = (i + thread_id * OPS_PER_THREAD) % (CACHE_SIZE * 2);

            let mut cache = cache.lock();
            if let Some(_) = cache.get(&key) {
                local_hits += 1;
            } else {
                cache.put(key, format!("value_{}", key));
            }
        }

        let mut hits = hit_counter.lock();
        *hits += local_hits;
    });

    let elapsed = start.elapsed();
    let hits = *hit_counter.lock();

    BenchResult {
        name: "lru".to_string(),
        total_time: elapsed.as_millis(),
        ops_per_sec: total_ops as f64 / elapsed.as_secs_f64(),
        hit_rate: hits as f64 / total_ops as f64,
    }
}

fn bench_cached() -> BenchResult {
    let cache = Arc::new(Mutex::new(SizedCache::with_size(CACHE_SIZE)));
    let hit_counter = Arc::new(Mutex::new(0u64));
    let total_ops = NUM_THREADS * OPS_PER_THREAD;
    let start = Instant::now();

    (0..NUM_THREADS).into_par_iter().for_each(|thread_id| {
        let cache = cache.clone();
        let hit_counter = Arc::clone(&hit_counter);
        let mut local_hits = 0;

        for i in 0..OPS_PER_THREAD {
            let key = (i + thread_id * OPS_PER_THREAD) % (CACHE_SIZE * 2);

            let mut cache = cache.lock();
            if let Some(_) = cache.cache_get(&key) {
                local_hits += 1;
            } else {
                cache.cache_set(key, format!("value_{}", key));
            }
        }

        let mut hits = hit_counter.lock();
        *hits += local_hits;
    });

    let elapsed = start.elapsed();
    let hits = *hit_counter.lock();

    BenchResult {
        name: "cached".to_string(),
        total_time: elapsed.as_millis(),
        ops_per_sec: total_ops as f64 / elapsed.as_secs_f64(),
        hit_rate: hits as f64 / total_ops as f64,
    }
}

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

    let results = pool.install(|| vec![bench_quick_cache(), bench_lru(), bench_cached()]);

    println!("Results:");

    let table = Table::new(results).to_string();
    println!("{}", table);
}
