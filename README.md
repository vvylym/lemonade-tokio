# Lemonade Load Balancer

A high-performance, asynchronous TCP load balancer written in Rust with support for multiple load balancing strategies and HTTP worker frameworks.

## Quick Start

### Prerequisites

- Rust 1.70+ (Edition 2024)
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
  --address 127.0.0.1:4001 --name worker-1 --delay 20

# Axum worker
cargo run --release -- worker --framework axum \
  --address 127.0.0.1:4002 --name worker-2 --delay 20
```

Supported frameworks: `actix`, `axum`, `hyper`, `rocket`

### Running Load Balancer

```bash
# Using environment variables
LEMONADE_LB_LISTEN_ADDRESS=127.0.0.1:3000 \
LEMONADE_LB_STRATEGY=round_robin \
LEMONADE_LB_BACKEND_ADDRESSES=127.0.0.1:4001,127.0.0.1:4002 \
cargo run --release -- load-balancer

# Using configuration file
cargo run --release -- load-balancer --config config.toml
```

### Testing

```bash
# Health check
curl http://127.0.0.1:3000/health

# Work endpoint
curl http://127.0.0.1:3000/work
```

## Features

- **5 Load Balancing Strategies**: Round Robin, Least Connections, Weighted Round Robin, Fastest Response Time, Adaptive
- **4 Worker Frameworks**: Actix Web, Axum, Hyper, Rocket
- **Production Ready**: Health checking, metrics collection, dynamic configuration, graceful shutdown
- **Clean Architecture**: Port/Adapter pattern for extensibility

## Documentation

- **[Architecture Documentation](docs/architecture.md)**: Detailed architecture, design decisions, and implementation details
- **[Load Balancer README](lemonade-load-balancer/README.md)**: Complete load balancer library documentation
- **[Service README](lemonade-service/README.md)**: Worker service library documentation
- **[CLI README](lemonade/README.md)**: Command-line interface documentation

## Project Structure

```
lemonade-tokio/
├── lemonade/                    # CLI binary
├── lemonade-load-balancer/      # Load balancer library
├── lemonade-service/            # Shared service library
├── lemonade-worker-*/          # Worker framework implementations
└── docs/                        # Architecture and design documentation
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
