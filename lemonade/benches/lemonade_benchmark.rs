//! Benchmark module for Lemonade Tokio
//!
use criterion::{Criterion, criterion_group, criterion_main};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;

/// Benchmark the server
fn bench_server(c: &mut Criterion) {
    let bench_address = std::env::var("LEMONADE_BENCH_ADDRESS")
        .expect("LEMONADE_BENCH_ADDRESS must be set");
    let bench_total_requests: u64 = std::env::var("LEMONADE_BENCH_TOTAL_REQUESTS")
        .expect("LEMONADE_BENCH_TOTAL_REQUESTS must be set")
        .parse()
        .expect("LEMONADE_BENCH_TOTAL_REQUESTS must be a number");
    let bench_concurrency: u64 = std::env::var("LEMONADE_BENCH_CONCURRENCY")
        .expect("LEMONADE_BENCH_CONCURRENCY must be set")
        .parse()
        .expect("LEMONADE_BENCH_CONCURRENCY must be a number");
    let bench_timeout: u64 = std::env::var("LEMONADE_BENCH_TIMEOUT_MS")
        .expect("LEMONADE_BENCH_TIMEOUT_MS must be set")
        .parse()
        .expect("LEMONADE_BENCH_TIMEOUT_MS must be a number");

    eprintln!("Starting benchmark for {}", bench_address);
    eprintln!("  Total Requests: {}", bench_total_requests);
    eprintln!("  Concurrency: {}", bench_concurrency);
    eprintln!("  Timeout: {}ms", bench_timeout);

    let stats = Arc::new(Mutex::new(RequestStats::new()));
    let timeout_duration = Duration::from_millis(bench_timeout);

    // Create runtime once, outside the benchmark iteration
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Create HTTP client with timeout configured
    let client = reqwest::Client::builder()
        .timeout(timeout_duration)
        .build()
        .expect("Failed to create HTTP client");

    let url = format!("http://{}/work", bench_address);

    c.bench_function(&format!("Lemonade Benchmark - {}", bench_address), |b| {
        let stats = stats.clone();
        let client = client.clone();
        let url = url.clone();

        b.iter(|| {
            // Create a fresh semaphore for each iteration to properly limit concurrency
            let semaphore = Arc::new(Semaphore::new(bench_concurrency as usize));

            rt.block_on(async {
                // Create a semaphore to limit concurrent requests
                let mut handles = Vec::new();

                // Spawn all requests, but limit concurrency using semaphore
                for _ in 0..bench_total_requests {
                    let client = client.clone();
                    let url = url.clone();
                    let stats = stats.clone();
                    let semaphore = semaphore.clone();

                    // Acquire permit before spawning to limit concurrency
                    let permit = semaphore.acquire_owned().await.unwrap();

                    handles.push(tokio::spawn(async move {
                        let start = std::time::Instant::now();
                        let result = client.get(&url).send().await;
                        let duration = start.elapsed();
                        drop(permit); // Release permit after request completes

                        match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    stats.lock().unwrap().record_success(duration);
                                } else {
                                    stats.lock().unwrap().record_failure(duration);
                                }
                            }
                            Err(_) => {
                                // Record failure (timeout or other error)
                                stats.lock().unwrap().record_failure(duration);
                            }
                        }
                    }));
                }

                // Wait for all requests to complete
                for handle in handles {
                    let _ = handle.await;
                }
            });
        });
    });

    let final_stats = stats.lock().unwrap();
    final_stats.print_summary(&format!("Lemonade Benchmark - {}", bench_address));
}

/// Tracks success and failure counts for benchmark requests with response time statistics
pub struct RequestStats {
    /// Number of successful requests
    pub success: u64,
    /// Number of failed requests
    pub failed: u64,
    /// Success response times
    success_times: Mutex<Vec<Duration>>,
    /// Failure response times
    failure_times: Mutex<Vec<Duration>>,
}

impl Default for RequestStats {
    fn default() -> Self {
        Self {
            success: 0,
            failed: 0,
            success_times: Mutex::new(Vec::new()),
            failure_times: Mutex::new(Vec::new()),
        }
    }
}

impl Clone for RequestStats {
    fn clone(&self) -> Self {
        Self {
            success: self.success,
            failed: self.failed,
            success_times: Mutex::new(self.success_times.lock().unwrap().clone()),
            failure_times: Mutex::new(self.failure_times.lock().unwrap().clone()),
        }
    }
}

impl RequestStats {
    /// Create a new request stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful request
    pub fn record_success(&mut self, duration: Duration) {
        self.success += 1;
        if let Ok(mut times) = self.success_times.lock() {
            times.push(duration);
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self, duration: Duration) {
        self.failed += 1;
        if let Ok(mut times) = self.failure_times.lock() {
            times.push(duration);
        }
    }

    /// Total number of requests
    pub fn total(&self) -> u64 {
        self.success + self.failed
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.success as f64 / total as f64) * 100.0
        }
    }

    /// Calculate statistics for a vector of durations
    /// Returns (avg, median, min, max, p95, p99)
    fn calculate_stats(times: &[Duration]) -> (f64, f64, f64, f64, f64, f64) {
        if times.is_empty() {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let mut sorted: Vec<f64> =
            times.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted.len();
        let min = sorted[0];
        let max = sorted[len - 1];
        let sum: f64 = sorted.iter().sum();
        let avg = sum / len as f64;

        // Median calculation
        let median = if len.is_multiple_of(2) {
            // Even length: average of two middle values
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            // Odd length: middle value
            sorted[len / 2]
        };

        // Percentile calculations
        let p95_idx = ((len as f64) * 0.95).ceil() as usize - 1;
        let p99_idx = ((len as f64) * 0.99).ceil() as usize - 1;
        let p95 = sorted[p95_idx.min(len - 1)];
        let p99 = sorted[p99_idx.min(len - 1)];

        (avg, median, min, max, p95, p99)
    }

    /// Print the summary of the request stats
    pub fn print_summary(&self, bench_name: &str) {
        let total = self.total();
        let success_rate = self.success_rate();

        // Get all response times (success + failure)
        let all_times: Vec<Duration> = {
            let success_times = self.success_times.lock().unwrap();
            let failure_times = self.failure_times.lock().unwrap();
            let mut all = success_times.clone();
            all.extend_from_slice(&failure_times);
            all
        };

        let (avg_ms, median_ms, min_ms, max_ms, p95_ms, p99_ms) =
            Self::calculate_stats(&all_times);

        // Calculate success and failure specific stats
        let (
            success_avg,
            success_median,
            success_min,
            success_max,
            success_p95,
            success_p99,
        ) = {
            let times = self.success_times.lock().unwrap();
            Self::calculate_stats(&times)
        };

        let (
            failure_avg,
            failure_median,
            failure_min,
            failure_max,
            failure_p95,
            failure_p99,
        ) = {
            let times = self.failure_times.lock().unwrap();
            Self::calculate_stats(&times)
        };

        eprintln!("\n{}", "=".repeat(80));
        eprintln!("{}", bench_name);
        eprintln!("{}", "=".repeat(80));
        eprintln!("Total Requests: {}", total);
        eprintln!("  Success: {} ({:.2}%)", self.success, success_rate);
        eprintln!("  Failed: {} ({:.2}%)", self.failed, 100.0 - success_rate);
        eprintln!();
        eprintln!("Response Times (All Requests):");
        eprintln!("  Avg:    {:.2}ms", avg_ms);
        eprintln!("  Median: {:.2}ms", median_ms);
        eprintln!("  P95:    {:.2}ms", p95_ms);
        eprintln!("  P99:    {:.2}ms", p99_ms);
        eprintln!("  Min:    {:.2}ms", min_ms);
        eprintln!("  Max:    {:.2}ms", max_ms);
        eprintln!();
        eprintln!("Success Response Times:");
        eprintln!("  Avg:    {:.2}ms", success_avg);
        eprintln!("  Median: {:.2}ms", success_median);
        eprintln!("  P95:    {:.2}ms", success_p95);
        eprintln!("  P99:    {:.2}ms", success_p99);
        eprintln!("  Min:    {:.2}ms", success_min);
        eprintln!("  Max:    {:.2}ms", success_max);
        eprintln!();
        if self.failed > 0 {
            eprintln!("Failure Response Times:");
            eprintln!("  Avg:    {:.2}ms", failure_avg);
            eprintln!("  Median: {:.2}ms", failure_median);
            eprintln!("  P95:    {:.2}ms", failure_p95);
            eprintln!("  P99:    {:.2}ms", failure_p99);
            eprintln!("  Min:    {:.2}ms", failure_min);
            eprintln!("  Max:    {:.2}ms", failure_max);
        }
        eprintln!("{}", "=".repeat(80));
    }
}

criterion_group!(benches, bench_server);
criterion_main!(benches);
