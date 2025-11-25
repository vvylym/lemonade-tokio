# Justfile for Lemonade Project

## Install just: cargo install just
## Run commands: just <command>

# Default recipe - show available commands
default:
    @just --list

# ============================================================
# Server Commands
# ============================================================

# Run Actix Web worker
worker-actix:
    @echo "Starting Actix Web worker..."
    cargo run -p worker-actix

# Run Actix Web worker
worker-axum:
    @echo "Starting Axum worker..."
    cargo run -p worker-axum

# Run Load Balancer (requires workers to be running)
load-balancer:
    @echo "Starting Load Balancer ..."
    cargo run -p load-balancer

# ============================================================
# Development Commands
# ============================================================

# Format (rustfmt)
setup-dev:
    @echo "Formatting all crates..."
    cargo install cargo-nextest --locked

# Format (rustfmt)
format:
    @echo "Formatting all crates..."
    cargo fmt

# Linting (clippy)
lint:
    @echo "Linting all crates..."
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests for all projects
test:
    @echo "Running tests for all crates..."
    cargo nextest run --workspace --all-features
