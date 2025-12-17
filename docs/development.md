# Development Guide

This guide covers development workflow, testing strategies, and contribution guidelines for the Lemonade Load Balancer project.

## Getting Started

### Prerequisites

- **Rust**: 1.75+ with Edition 2024 support
- **Cargo**: Latest stable version
- **Tools**: 
  - `cargo-llvm-cov` for coverage reports
  - `cargo-nextest` (optional) for faster test execution

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/vvylym/lemonade-tokio.git
cd lemonade-tokio

# Build all crates
cargo build --workspace --all-targets --all-features

# Run tests
cargo test --workspace --all-targets --all-features

# Check code quality
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

## Project Structure

```
lemonade-tokio/
├── lemonade/                      # CLI binary
├── lemonade-load-balancer/        # Load balancer core library
│   ├── src/
│   │   ├── config/               # Configuration management
│   │   ├── health/               # Health checking
│   │   ├── metrics/              # Metrics collection
│   │   ├── proxy/                # TCP proxying (hot path)
│   │   ├── strategy/             # Load balancing strategies
│   │   ├── types/                # Core types (Context, Backend, etc.)
│   │   ├── app.rs               # Application orchestrator
│   │   ├── error.rs             # Error types
│   │   └── lib.rs               # Public API
│   └── tests/                    # Integration tests
├── lemonade-service/              # Worker service library
├── lemonade-observability/        # Observability (tracing/metrics)
├── lemonade-worker-*/             # Worker implementations (4 frameworks)
├── config/                        # Example configuration files
└── docs/                          # Documentation
```

## Development Workflow

### Standard Development Loop

```bash
# 1. Make changes to code

# 2. Format code
cargo fmt --all

# 3. Run clippy
cargo clippy --workspace --all-features --all-targets -- -D warnings

# 4. Run tests with coverage
cargo llvm-cov --workspace --all-targets --all-features --summary-only

# 5. Run full test suite
cargo test --workspace --all-targets --all-features

# Complete check (CI equivalent)
cargo fmt --all && \
  cargo clippy --workspace --all-features --all-targets -- -D warnings && \
  cargo llvm-cov --summary-only --workspace --all-targets --all-features
```

### Coverage Requirements

**Minimum coverage**: 85% across all crates

Generate detailed coverage report:

```bash
# HTML report
cargo llvm-cov --workspace --all-targets --all-features --html
open target/llvm-cov/html/index.html

# LCOV for CI
cargo llvm-cov --workspace --all-targets --all-features --lcov --output-path lcov.info
```

## Testing Strategy

### Test Pyramid

1. **Unit Tests**: Test individual functions and methods
2. **Integration Tests**: Test service interactions
3. **End-to-End Tests**: Test complete request flows

### Test Organization

- **Unit tests**: Inline with source code in `#[cfg(test)] mod tests`
- **Integration tests**: `tests/` directory with realistic scenarios
- **Helper utilities**: `tests/common/` module for shared test code

### Writing Tests

#### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_backend_health_update() {
        let config = BackendConfig {
            id: 0,
            name: Some("test".into()),
            address: "127.0.0.1:8080".parse().unwrap(),
            weight: None,
        };
        let backend = Backend::new(config);
        
        // Backend starts healthy
        assert!(backend.is_alive());
        
        // Mark unhealthy
        backend.set_health(false);
        assert!(!backend.is_alive());
    }
}
```

#### Integration Test Example

```rust
#[tokio::test]
async fn test_round_robin_distribution() {
    // Create context with 3 backends
    let ctx = create_test_context(3).await;
    
    // Create round robin strategy
    let strategy = RoundRobinStrategy::new();
    
    // Pick backends 10 times
    let mut picks = Vec::new();
    for _ in 0..10 {
        let backend = strategy.pick_backend(ctx.clone()).await.unwrap();
        picks.push(backend.id());
    }
    
    // Verify round robin pattern
    assert_eq!(picks[0], picks[3]);  // Pattern repeats
    assert_eq!(picks[1], picks[4]);
}
```

### Testing Best Practices

1. **Use realistic data**: Create helpers in `tests/common/fixtures.rs`
2. **Test error paths**: Don't just test happy path
3. **Test concurrency**: Use multiple tasks to verify thread safety
4. **Clean up resources**: Use Drop guards or defer cleanup
5. **Deterministic tests**: Avoid flaky tests with proper synchronization

### Hot Reload Testing

Test hot reload manually:

```bash
# Terminal 1: Start load balancer
cargo run --release -- load-balancer --config config/load-balancer.yaml

# Terminal 2: Monitor logs
tail -f /tmp/lemonade-lb.log

# Terminal 3: Modify config file
vim config/load-balancer.yaml
# Change strategy or backends, save

# Verify changes applied within 1 second (config_watch_interval_millis)
```

## Code Style and Guidelines

### Rust Edition 2024 Features

Leverage Edition 2024 features for better performance:

- `if let` chains for cleaner conditionals
- Pattern matching improvements
- Improved type inference

### Error Handling

- Use `thiserror` for custom error types
- Propagate errors with `?` operator
- Add context with `.map_err()` where helpful
- Log errors at appropriate levels

### Async Patterns

- Prefer structured concurrency with `tokio::spawn`
- Use `select!` for cancellation
- Leverage `JoinSet` for managing multiple tasks
- Avoid blocking operations in async code

### Lock-Free Patterns

- Use `Arc` for shared ownership
- Use `ArcSwap` for atomic updates to shared state
- Use `DashMap` for concurrent hash maps
- Use atomic types (`AtomicBool`, `AtomicU64`) for counters

### Documentation

- Document all public APIs with rustdoc comments
- Include examples in documentation
- Document error conditions
- Explain design decisions in module-level docs

Example:

```rust
/// Selects a backend using round robin algorithm.
///
/// # Arguments
///
/// * `ctx` - Shared context containing backend routing table
///
/// # Returns
///
/// Returns the selected backend or an error if no healthy backends are available.
///
/// # Examples
///
/// ```no_run
/// use lemonade_load_balancer::strategy::RoundRobinStrategy;
///
/// let strategy = RoundRobinStrategy::new();
/// let backend = strategy.pick_backend(ctx).await?;
/// ```
#[instrument(skip(ctx))]
async fn pick_backend(&self, ctx: Arc<Context>) -> Result<Arc<Backend>> {
    // Implementation
}
```

## Adding New Features

### Adding a New Load Balancing Strategy

1. Create new file: `lemonade-load-balancer/src/strategy/adapters/my_strategy.rs`
2. Implement `StrategyService` trait
3. Add to `Strategy` enum in `models.rs`
4. Add tests in `tests/strategy/test_my_strategy.rs`
5. Document in README

Example:

```rust
use crate::prelude::*;
use async_trait::async_trait;

pub struct MyStrategy {
    // Strategy state
}

#[async_trait]
impl StrategyService for MyStrategy {
    fn strategy(&self) -> Strategy {
        Strategy::MyStrategy
    }
    
    async fn pick_backend(&self, ctx: Arc<Context>) -> Result<Arc<Backend>> {
        // Implementation
    }
}
```

### Adding a New Worker Framework

1. Create new crate: `lemonade-worker-myframework/`
2. Add dependency on `lemonade-service`
3. Implement HTTP server with `/health` and `/work` endpoints
4. Add to workspace in root `Cargo.toml`
5. Add CLI support in `lemonade/src/lib.rs`

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run -- load-balancer --config config.toml
```

### Trace Specific Modules

```bash
RUST_LOG=lemonade_load_balancer::proxy=trace,lemonade_load_balancer::strategy=debug \
  cargo run -- load-balancer
```

### Debugging Integration Tests

```bash
# Run specific test with logs
RUST_LOG=debug cargo test test_name -- --nocapture

# Run with tokio-console (requires tokio-console feature)
tokio-console &
cargo test --features tokio-console
```

### Performance Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
cargo flamegraph --bin lemonade -- load-balancer --config config.toml

# Memory profiling with valgrind
valgrind --tool=massif target/release/lemonade load-balancer
```

## Contribution Guidelines

### Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes with tests and documentation
4. Ensure all checks pass:
   ```bash
   cargo fmt --all && \
   cargo clippy --workspace --all-features --all-targets -- -D warnings && \
   cargo test --workspace --all-targets --all-features && \
   cargo llvm-cov --summary-only --workspace --all-targets --all-features
   ```
5. Commit with descriptive message
6. Push and create pull request
7. Address review feedback

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Example:

```
feat(strategy): add weighted least connections strategy

Implements a new load balancing strategy that combines least connections
with backend weights for more sophisticated load distribution.

Closes #123
```

### Code Review Checklist

- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Coverage maintained at 85%+
- [ ] No clippy warnings
- [ ] Formatted with `cargo fmt`
- [ ] Error handling appropriate
- [ ] No breaking API changes (or documented)

## Common Issues and Solutions

### Issue: Tests failing intermittently

**Solution**: Likely timing issue. Increase delays in tests or add proper synchronization:

```rust
// Bad: Flaky
tokio::time::sleep(Duration::from_millis(10)).await;

// Good: Deterministic
rx.recv().await.unwrap();
```

### Issue: High memory usage

**Solution**: Check for Arc cycles or leaked tasks. Use weak references where appropriate.

### Issue: Clippy warnings after merge

**Solution**: Rebase on main and run clippy:

```bash
git rebase main
cargo clippy --workspace --all-features --all-targets --fix
```

## Release Process

1. Update version in all `Cargo.toml` files
2. Update CHANGELOG.md
3. Run full test suite
4. Create git tag: `git tag v0.1.0`
5. Push tag: `git push origin v0.1.0`
6. Create GitHub release

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)
- [async-trait Documentation](https://docs.rs/async-trait)
- [ArcSwap Documentation](https://docs.rs/arc-swap)
- [DashMap Documentation](https://docs.rs/dashmap)

## Getting Help

- Open an issue on GitHub
- Check existing documentation in `docs/`
- Review architecture.md for design decisions
- Ask questions in pull requests
