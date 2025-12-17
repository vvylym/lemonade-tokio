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
    cargo run --release -- lb -c config/load-balancer.yaml

# Run worker Actix-web
run-worker-actix:
    cargo run --release -- w -f actix -c config/worker-1.toml

# Run worker Axum
run-worker-axum:
    cargo run --release -- w -f axum -c config/worker-2.json

# Run worker Hyper
run-worker-hyper:
    cargo run --release -- w -f hyper -c config/worker-3.yaml

# Run worker Rocket
run-worker-rocket:
    cargo run --release -- w -f rocket -c config/worker-4.toml
