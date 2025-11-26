//! Helper utilities shared across the Criterion benches in this workspace.
//!
//! The worker benches all follow the same pattern:
//! 1. Spawn the server binary under test with [`start_server`].
//! 2. Probe the server until it is ready using [`warm_up_server`].
//! 3. Drive Criterion against one or more HTTP endpoints via [`benchmark_server`].
//! 4. Tear the server back down with [`stop_server`].
//!
//! Keeping these helpers here avoids copy‑pasting the bootstrapping logic
//! into every benchmark file and makes it easy to introduce new benches—
//! import this crate, call into the helpers, and focus on the benchmarked
//! request/response logic itself.

use criterion::Criterion;
use reqwest::Client;
use std::{
    process::{Child, Command, Stdio},
    time::Duration,
};
use tokio::{runtime::Runtime, time::sleep};

/// Compose a benchmark URL from an address stored in an env var.
///
/// The benches set (for example) `WORKER_ADDR=127.0.0.1:4000` before running.
/// Passing `("WORKER_ADDR", "health")` yields `http://127.0.0.1:4000/health`.
pub fn get_url(env_key: &str, endpoint: &str) -> String {
    let address = std::env::var(env_key).unwrap_or_else(|_| panic!("Failed to get {}", env_key));
    format!("http://{}/{}", address, endpoint)
}

/// Run a server binary in release mode using Cargo.
///
/// This keeps stdout/stderr quiet so the benchmark output stays clean.
pub fn start_server(package_name: &str) -> Child {
    Command::new("cargo")
        .args(["run", "-p", package_name, "--release"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to start {}", package_name))
}

/// Poll the server until it responds (or we exhaust retries).
pub fn warm_up_server(url: &str, retries: u8, timeout: u64) -> bool {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let client = Client::new();
        for _ in 0..retries {
            if client
                .get(url)
                .timeout(Duration::from_secs(timeout))
                .send()
                .await
                .is_ok()
            {
                return true;
            }
            sleep(Duration::from_millis(500)).await;
        }
        false
    })
}

/// Register a Criterion benchmark that repeatedly performs a GET request.
pub fn benchmark_server(c: &mut Criterion, id: &str, url: &str) {
    c.bench_function(id, |b| {
        let rt = Runtime::new().unwrap();
        let client = Client::new();
        b.iter(|| {
            rt.block_on(async {
                let _ = client.get(url).send().await;
            });
        });
    });
}

/// Stop the spawned server that was started via [`start_server`].
pub fn stop_server(server: &mut Child) {
    let _ = server.kill();
}
