# Lemonade Tokio Justfile

default:
    @just --list

# Build release binary
build:
    cargo build --release

# Run all tests
test:
    cargo test --workspace --all-targets --all-features

# Generate coverage report
coverage:
    cargo llvm-cov --workspace --all-targets --all-features --summary-only
    @echo "📄 Report: target/llvm-cov/html/index.html"

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all --check

# Run clippy
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Run all checks (CI equivalent)
check: fmt-check clippy test
    @echo "✅ All checks passed"

# Security audit
audit:
    cargo audit

# Run cargo deny
deny:
    cargo deny check

# Run load balancer
run-lb:
    cargo run --release -- load-balancer --config config/load-balancer.yaml

# Run Actix-web worker
run-worker-actix:
    cargo run --release -- worker --framework actix --config config/worker-1.toml

# Run Axum worker
run-worker-axum:
    cargo run --release -- worker --framework axum --config config/worker-2.json

# Run RHyperocket worker
run-worker-hyper:
    cargo run --release -- worker --framework hyper --config config/worker-3.yaml

# Run Rocket worker
run-worker-rocket:
    cargo run --release -- worker --framework rocket --config config/worker-4.toml


# Benchmark load balancer
bench-lb:
    @echo "🚀 Benchmarking load balancer..."
    LEMONADE_BENCH_ADDRESS="localhost:50501" \
    cargo bench -p lemonade --bench lemonade_benchmark

# Benchmark worker 1 (Actix)
bench-actix:
    @echo "🚀 Benchmarking worker-1 (Actix)..."
    LEMONADE_BENCH_ADDRESS="localhost:50510" \
    cargo bench -p lemonade --bench lemonade_benchmark

# Benchmark worker 2 (Axum)
bench-axum:
    @echo "🚀 Benchmarking worker-2 (Axum)..."
    LEMONADE_BENCH_ADDRESS="localhost:50520" \
    cargo bench -p lemonade --bench lemonade_benchmark

# Benchmark worker 3 (Hyper)
bench-hyper:
    @echo "🚀 Benchmarking worker-3 (Hyper)..."
    LEMONADE_BENCH_ADDRESS="localhost:50530" \
    cargo bench -p lemonade --bench lemonade_benchmark

# Benchmark worker 4 (Rocket)
bench-rocket:
    @echo "🚀 Benchmarking worker-4 (Rocket)..."
    LEMONADE_BENCH_ADDRESS="localhost:50540" \
    cargo bench -p lemonade --bench lemonade_benchmark
