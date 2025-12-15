# Lemonade Service

A worker service library that provides health checking and work execution capabilities for load balancer testing and development.

## Overview

`lemonade-service` is a Rust library that implements a simple worker service pattern. It provides two main capabilities:

- **Health Checks**: Verify that the service is properly configured and ready to handle work
- **Work Execution**: Perform configurable work tasks with a specified delay

## Architecture

The library follows a clean architecture pattern with clear separation of concerns:

- **Ports**: Trait definitions (`HealthService`, `WorkService`) that define the service interface
- **Models**: Data structures for requests and responses (`HealthResponse`, `WorkResponse`)
- **Service Implementation**: `WorkerServiceImpl` that implements both health and work traits

## Key Components

### Configuration

The service can be configured via:
1. **Configuration files** (JSON or TOML)
2. **Environment variables** (prefixed with `LEMONADE_WORKER_`)
3. **Programmatic configuration** via the `Config` struct

The `Config` struct requires:
- `listen_address`: The address to listen on (as a `WorkerAddress`)
- `service_name`: A unique identifier for the service instance
- `work_delay`: A `Duration` specifying how long work operations should take

#### Configuration Files

**TOML Example:**
```toml
listen_address = "127.0.0.1:4001"
service_name = "worker-1"
work_delay_ms = 20
```

**JSON Example:**
```json
{
  "listen_address": "127.0.0.1:4001",
  "service_name": "worker-1",
  "work_delay_ms": 20
}
```

Note: `work_delay` in the struct is a `Duration`, but in configuration files it's specified as `work_delay_ms` (milliseconds).

#### Environment Variables

- `LEMONADE_WORKER_LISTEN_ADDRESS` (default: `127.0.0.1:50200`)
- `LEMONADE_WORKER_SERVICE_NAME` (default: `lemonade-worker`)
- `LEMONADE_WORKER_WORK_DELAY_MS` (default: `20`)

The `ConfigBuilder` automatically loads from `.env` files if present (via `dotenv`).

### Health Service

The `HealthService` trait provides:
- `health_check()`: Returns a `HealthResponse` indicating service status and name

A service is considered healthy when:
- The service name is not empty
- The work delay is non-zero

### Work Service

The `WorkService` trait provides:
- `work()`: Executes work by sleeping for the configured delay, then returns a `WorkResponse` with:
  - Status (success/failure)
  - Service name
  - Duration in milliseconds

## Usage Example

### Using Configuration Builder

```rust
use lemonade_service::{service::WorkerServiceImpl, config::ConfigBuilder};
use std::path::PathBuf;

// Load from environment variables
let config = ConfigBuilder::from_env()?;

// Or load from a configuration file
let config = ConfigBuilder::from_file(Some(PathBuf::from("worker.toml")))?;

// Create service instance
let service = WorkerServiceImpl::new(config);

// Perform health check
let health = service.health_check().await?;

// Execute work
let work_result = service.work().await?;
```

### Programmatic Configuration

```rust
use lemonade_service::{service::WorkerServiceImpl, config::{Config, WorkerAddress}};
use std::time::Duration;

// Create configuration programmatically
let config = Config::new(
    WorkerAddress::parse("127.0.0.1:4001")?,
    "worker-1",
    Duration::from_millis(100),
);

// Create service instance
let service = WorkerServiceImpl::new(config);

// Perform health check
let health = service.health_check().await?;

// Execute work
let work_result = service.work().await?;
```

## Error Handling

The library uses `thiserror` for error handling with distinct error types:
- `HealthError`: Errors during health checks
- `WorkError`: Errors during work execution

## Dependencies

- `async-trait`: For async trait definitions
- `dotenv`: For loading environment variables from `.env` files
- `serde` / `serde_json` / `toml`: For serialization/deserialization of models and configuration files
- `thiserror`: For error handling

## Integration with Workers

This library is used by all worker implementations:
- `lemonade-worker-actix` (Actix Web framework)
- `lemonade-worker-axum` (Axum framework)
- `lemonade-worker-hyper` (Hyper framework)
- `lemonade-worker-rocket` (Rocket framework)

Each worker wraps `WorkerServiceImpl` and exposes:
- `GET /health` endpoint mapped to `health_check()`
- `GET /work` endpoint mapped to `work()`

## Use Cases

This service is designed for:
- **Load balancer testing**: Provides consistent backend behavior for testing load balancing strategies
- **Benchmarking**: Configurable work delays allow testing under different latency scenarios
- **Framework comparison**: Same service logic across different web frameworks enables fair comparisons
- **Health check patterns**: Demonstrates health check endpoint implementation
- **Service mesh development**: Useful for microservices and service mesh testing

## Testing

Run the test suite:

```bash
cargo test --package lemonade-service
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
