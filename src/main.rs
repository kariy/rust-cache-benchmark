use memory_stats::memory_stats;
mod cache;
mod result;

use std::collections::HashSet;
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

fn bench_cache<C, F, T>(name: &str, cache: C, value_gen: F) -> BenchResult
where
    T: Clone,
    F: Fn(usize) -> T + Send + Sync,
    C: Cache<Item = T> + Clone + Send + Sync + 'static,
{
    let hit_counter = Arc::new(Mutex::new(0u64));
    let total_ops = NUM_THREADS * OPS_PER_THREAD;

    // Measure initial memory
    let initial_mem = memory_stats().map(|stats| stats.physical_mem).unwrap_or(0);
    let start = Instant::now();

    let keys = (0..NUM_THREADS)
        .into_par_iter()
        .map(|thread_id| {
            let cache = cache.clone();
            let mut local_unique_keys = HashSet::new();
            let hit_counter = Arc::clone(&hit_counter);
            let mut local_hits = 0;

            for i in 0..OPS_PER_THREAD {
                // This key generation strategy is designed for testing cache behavior with specific characteristics:
                //
                // 1. **Thread Isolation**:
                // - `thread_id * OPS_PER_THREAD` ensures each thread works on a different range of keys
                // - Prevents thread contention by giving each thread its own key space
                //
                // 2. **Cache Size Testing**:
                // - `% (CACHE_SIZE * 2)` creates a working set that's twice the cache size
                // - This ensures some keys will be evicted, testing cache replacement policies // - Creates a mix of cache hits and misses
                //
                // Example:
                //
                // If OPS_PER_THREAD = 5000 & CACHE_SIZE = 1000 :-
                // Thread 0: keys 0-4999 % 2000
                // Thread 1: keys 5000-9999 % 2000
                // Thread 2: keys 10000-14999 % 2000

                let key = (i + thread_id * OPS_PER_THREAD) % (CACHE_SIZE * 2);

                // Track unique keys
                local_unique_keys.insert(key);

                if let Some(_) = cache.get_key(&key) {
                    local_hits += 1;
                } else {
                    let value = value_gen(key);
                    cache.set_key(key, value);
                }
            }

            let mut hits = hit_counter.lock();
            *hits += local_hits;

            local_unique_keys
        })
        .collect::<Vec<_>>();

    let elapsed = start.elapsed();
    let hits = *hit_counter.lock();

    let unique_keys: HashSet<usize> = keys.into_iter().fold(HashSet::new(), |mut acc, keys| {
        acc.extend(keys);
        acc
    });
    let total_entries = unique_keys.len();

    // Drop unrelated objects to get more accurate memory reading
    drop(hit_counter);
    drop(unique_keys);

    // Measure final memory
    let final_mem = memory_stats().map(|stats| stats.physical_mem).unwrap_or(0);
    let memory_used = final_mem.saturating_sub(initial_mem);

    BenchResult {
        total_entries,
        name: name.to_string(),
        total_time: elapsed.as_millis(),
        hit_rate: hits as f64 / total_ops as f64,
        memory_mb: memory_used as f64 / 1024.0 / 1024.0,
        ops_per_sec: total_ops as f64 / elapsed.as_secs_f64(),
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

    // let json: Value = serde_json::from_str(include_str!("../fixtures/big.json")).unwrap();
    let value = |key: usize| format!("value_{key}");

    let results = pool.install(|| {
        let quick_cache = Arc::new(QuickCache::new(CACHE_SIZE));
        let quick_cache_result = bench_cache("quick_cache", quick_cache, value);

        let size = NonZeroUsize::new(CACHE_SIZE).unwrap();
        let lru_cache = Arc::new(Mutex::new(LruCache::new(size)));
        let lru_cache_result = bench_cache("lru", lru_cache, value);

        let cached = Arc::new(Mutex::new(SizedCache::with_size(CACHE_SIZE)));
        let cached_result = bench_cache("cached", cached, value);

        vec![quick_cache_result, lru_cache_result, cached_result]
    });

    println!("Results:");

    let table = Table::new(results).to_string();
    println!("{}", table);
}
