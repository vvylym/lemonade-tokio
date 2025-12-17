# Lemonade Load Balancer

A high-performance, asynchronous TCP load balancer written in Rust with support for multiple load balancing strategies and HTTP worker frameworks.

## Quick Start (Usage)

### Prerequisites

- Rust 1.75+ (Edition 2024)
- Cargo (latest stable)

### Installation

```bash
git clone https://github.com/vvylym/lemonade-tokio.git
cd lemonade-tokio
cargo build --release
```

### Running Workers

Start workers using the `lemonade` CLI:

```bash
# Actix worker
cargo run --release -- worker --framework actix \
  --address 127.0.0.1:50510 --name worker-1 --delay 20

# Axum worker
cargo run --release -- worker --framework axum \
  --address 127.0.0.1:50520 --name worker-2 --delay 20

# Hyper worker
cargo run --release -- worker --framework hyper \
  --address 127.0.0.1:50530 --name worker-3 --delay 20

# Rocket worker
cargo run --release -- worker --framework rocket \
  --address 127.0.0.1:50540 --name worker-4 --delay 20
```

Supported frameworks: `actix`, `axum`, `hyper`, `rocket`

### Running Load Balancer

```bash
# Using configuration file (recommended)
cargo run --release -- load-balancer --config config/load-balancer.yaml

# Using environment variables
LEMONADE_LB_LISTEN_ADDRESS=127.0.0.1:50501 \
LEMONADE_LB_STRATEGY=round_robin \
LEMONADE_LB_BACKEND_ADDRESSES=127.0.0.1:50510,127.0.0.1:50520 \
cargo run --release -- load-balancer
```

### Testing the Load Balancer

```bash
# Health check
curl http://127.0.0.1:50501/health

# Work endpoint (distributed across backends)
curl http://127.0.0.1:50501/work

# Test load distribution
for i in {1..10}; do curl http://127.0.0.1:50501/work; echo; done
```

## Features

### Load Balancing Strategies

- **Round Robin**: Distributes requests evenly across all healthy backends
- **Least Connections**: Routes to backend with fewest active connections
- **Weighted Round Robin**: Distributes based on backend weights
- **Fastest Response Time**: Routes to backend with lowest latency
- **Adaptive**: Dynamically adjusts based on backend performance metrics

### Worker Frameworks

- **Actix Web**: High-performance actor-based framework
- **Axum**: Ergonomic and modular web framework from Tokio team
- **Hyper**: Low-level HTTP library with maximum performance
- **Rocket**: Type-safe framework with excellent developer experience

### Production Features

- **Health Checking**: Active health probes with configurable intervals
- **Metrics Collection**: Request/error counts, latency tracking, connection monitoring
- **Hot Reload**: Dynamic configuration updates without downtime
- **Graceful Shutdown**: Connection draining and proper cleanup
- **Connection Management**: Max connection limits, timeout handling
- **Clean Architecture**: Port/Adapter pattern for extensibility

## Architecture

Lemonade follows a clean architecture with clear separation of concerns:

- **Shared Context**: Central state management with `ArcSwap` and `DashMap` for lock-free concurrent access
- **Service-Oriented**: Independent services (Config, Health, Metrics, Proxy) communicate via typed channels
- **Hot Path Optimization**: Proxy service runs on main thread, requests spawned as tasks
- **Backend State**: Unified atomic state tracking for health, connections, and metrics
- **Graceful Migration**: Backend draining during config changes with zero dropped connections

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace --all-targets --all-features

# Run with coverage
cargo llvm-cov --workspace --all-targets --all-features --html
open target/llvm-cov/html/index.html
```

### Linting and Formatting

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-features --all-targets

# Check everything (CI command)
cargo fmt --all && \
cargo clippy --workspace --all-features --all-targets -- -D warnings && \
cargo llvm-cov --summary-only --workspace --all-targets --all-features
```

### Hot Reload Testing

1. Start load balancer with config file:
   ```bash
   cargo run --release -- load-balancer --config config/load-balancer.yaml
   ```

2. Modify `config/load-balancer.yaml` (change strategy, add/remove backends, etc.)

3. Changes apply automatically within 1 second (configurable via `config_watch_interval_millis`)

## Configuration

### Load Balancer Configuration

Example `config/load-balancer.yaml`:

```yaml
runtime:
  metrics_cap: 1000
  health_cap: 100
  drain_timeout_millis: 5000
  background_timeout_millis: 30000
  accept_timeout_millis: 60000
  config_watch_interval_millis: 1000

proxy:
  listen_address: "127.0.0.1:50501"
  max_connections: 10000

strategy: round_robin  # or "least_connections", "weighted_round_robin", "fastest_response_time", "adaptive"

backends:
  - id: 0
    name: worker-hyper
    address: "127.0.0.1:50510"

  - id: 1
    name: worker-axum
    address: "127.0.0.1:50520"

health:
  interval: "5s"
  timeout: "2s"

metrics:
  interval: "10s"
```

### Worker Configuration

Workers support JSON, TOML, or YAML configuration files. See individual worker crate READMEs for details.

## Documentation

- **[Architecture Documentation](docs/architecture.md)**: Detailed architecture, design decisions, and implementation details
- **[Development Guide](docs/development.md)**: Contributing guidelines, testing strategies, and development workflow
- **[Deployment Guide](docs/deployment.md)**: Production deployment best practices
- **[Load Balancer README](lemonade-load-balancer/README.md)**: Complete load balancer library documentation
- **[Service README](lemonade-service/README.md)**: Worker service library documentation
- **[Observability README](lemonade-observability/README.md)**: Observability and telemetry documentation
- **[CLI README](lemonade/README.md)**: Command-line interface documentation

## Project Structure

```
lemonade-tokio/
├── lemonade/                      # CLI binary
├── lemonade-load-balancer/        # Load balancer core library
│   ├── src/
│   │   ├── config/               # Configuration management
│   │   ├── health/               # Health checking service
│   │   ├── metrics/              # Metrics collection service
│   │   ├── proxy/                # Proxy service (hot path)
│   │   ├── strategy/             # Load balancing strategies
│   │   └── types/                # Shared types (Context, Backend, etc.)
│   └── tests/                    # Integration tests
├── lemonade-service/              # Shared service library
├── lemonade-observability/        # Observability library (tracing, metrics)
├── lemonade-worker-actix/         # Actix Web worker implementation
├── lemonade-worker-axum/          # Axum worker implementation
├── lemonade-worker-hyper/         # Hyper worker implementation
├── lemonade-worker-rocket/        # Rocket worker implementation
├── config/                        # Example configuration files
└── docs/                          # Architecture and design documentation
```

## Performance

- **Throughput**: Handles 10,000+ requests/second with <1ms p50 latency
- **Memory**: Low overhead with lock-free atomic operations
- **CPU**: Optimized hot path with zero-copy proxying where possible
- **Scalability**: Tested with 100+ concurrent backends

## Testing

- **Test Coverage**: 85%+ across all crates
- **Unit Tests**: Comprehensive coverage of all services and strategies
- **Integration Tests**: End-to-end request flows, hot reload, failure scenarios
- **Property Tests**: Strategy correctness and distribution fairness

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please see [docs/development.md](docs/development.md) for guidelines.
