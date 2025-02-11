use tabled::Tabled;

#[derive(Clone, Tabled)]
pub struct BenchResult {
    #[tabled(rename = "Type")]
    pub name: String,
    #[tabled(rename = "Hit Rate", format("{:.2}", self.hit_rate * 100.0))]
    pub hit_rate: f64,
    #[tabled(rename = "Ops/sec", format("{:.3}", self.ops_per_sec ))]
    pub ops_per_sec: f64,
    /// The total time (in msl)
    #[tabled(rename = "Time (ms)")]
    pub total_time: u128,
}
