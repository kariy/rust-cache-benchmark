use tabled::Tabled;

#[derive(Clone, Tabled)]
pub struct BenchResult {
    #[tabled(rename = "Type")]
    pub name: String,
    #[tabled(rename = "Hit Rate", format("{:.2}", self.hit_rate * 100.0))]
    pub hit_rate: f64,
    #[tabled(rename = "Ops/sec", format("{:.3}", self.ops_per_sec ))]
    pub ops_per_sec: f64,
    /// Total number of entries in the cache.
    #[tabled(rename = "Total Entries", format("{}", self.total_entries))]
    pub total_entries: usize,
    /// The total time, in milliseconds, to execute all of the operations.
    #[tabled(rename = "Time (ms)")]
    pub total_time: u128,
    /// The amount of memory used, in megabytes, by cache at the end of the operations.
    #[tabled(rename = "Memory (MB)", format("{:.2}", self.memory_mb))]
    pub memory_mb: f64,
}
