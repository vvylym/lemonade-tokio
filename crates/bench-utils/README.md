# bench-utils

Shared Criterion helpers for the benches.

## What lives here

- `get_url`: compose a request URL from a env-provided host/port.
- `start_server` / `stop_server`: spin up a binary via `cargo run --release`
  and tear it down cleanly once the benchmark finishes.
- `warm_up_server`: poll an endpoint until the server is ready.
- `bench_url`: register a Criterion bench that repeatedly issues GET requests.

Centralising these helpers keeps each benchmark focused on the scenario under
test (e.g. health or work endpoints) instead of duplicating bootstrapping code.

## Typical flow

```text
1. let mut server = start_server("worker-axum");
2. let url = get_url("ACTIX_WORKER_ADDRESS", "health");
3. assert!(warm_up_server(&url, 20, 2)); // retry up to 20 times
4. bench_url(c, "health", &url);
5. stop_server(&mut server);
```

Add `bench-utils` as a dev-dependency in the benchmark crate, import the
functions you need, and follow the sequence above for any new HTTP benchmark.
