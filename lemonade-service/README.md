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

The service is configured via a `Config` struct that requires:
- `service_name`: A unique identifier for the service instance
- `work_delay`: A `Duration` specifying how long work operations should take

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

```rust
use lemonade_service::{service::WorkerServiceImpl, config::Config};
use std::time::Duration;

// Create configuration
let config = Config {
    service_name: "worker-1".to_string(),
    work_delay: Duration::from_millis(100),
};

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
- `serde`: For serialization/deserialization of models
- `thiserror`: For error handling

## Use Cases

This service is designed for:
- Load balancer testing and benchmarking
- Simulating backend services with configurable response times
- Health check endpoint implementations
- Service mesh and microservices development
