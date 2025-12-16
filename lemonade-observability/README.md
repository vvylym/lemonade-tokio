# Lemonade Observability Library

Centralized observability library for the Lemonade load balancer and worker system, providing OpenTelemetry-compliant tracing and logging integration.

## Overview

The `lemonade-observability` crate provides a unified observability initialization system that:

- Initializes OpenTelemetry SDK with console output (ready for OTLP export)
- Bridges `tracing` to OpenTelemetry for distributed tracing
- Provides structured logging with JSON or compact format
- Supports environment-based log filtering
- Uses OpenTelemetry semantic conventions for interoperability

## Architecture

The observability system is designed to:

1. **Initialize per-service**: Each service's `run()` function initializes its own observability
2. **Propagate traces**: Distributed tracing across load balancer and workers
3. **Support OTLP export**: Future integration with OpenTelemetry Collector, Jaeger, Grafana, and Prometheus
4. **Use standard attributes**: Following OpenTelemetry semantic conventions for interoperability

## Usage

### Basic Initialization

```rust
use lemonade_observability::init_tracing;

// Initialize in each service's run() function
// Each service must provide its name and version
init_tracing("lemonade-load-balancer", "0.1.0")?;
```

**Convention**: Each service (load balancer and workers) calls `init_tracing()` in its own `run()` function, ensuring:
- Independent observability initialization per service
- Service-specific configuration
- Consistent service identification in traces

### Adding Tracing to Your Code

```rust
use tracing::{info, instrument};

#[instrument(fields(service.name = "my-service", backend.id = %backend_id))]
async fn my_function(backend_id: u8) {
    info!("Processing request");
    // Your code here
}
```

## Environment Variables

- `RUST_LOG` - Log level filter (default: `info`)
  - Format: `RUST_LOG=lemonade_load_balancer=debug,lemonade_worker_axum=trace`

Note: Service name and version are provided as function parameters, not environment variables. This ensures each service explicitly sets its own identity. Logs are output in JSON format for production-ready structured logging.

## OpenTelemetry Integration

The library uses OpenTelemetry SDK 0.31.0 and is architected to easily swap the console exporter for OTLP:

- **Current**: Console exporter (`opentelemetry-stdout`)
- **Future**: OTLP exporter (`opentelemetry-otlp`) for Jaeger/Grafana integration
- **Future**: Prometheus metrics integration via `ExternalMetricsService`

## Semantic Conventions

The library uses standard OpenTelemetry attributes:

- `service.name` - Service identifier (provided as parameter)
- `service.version` - Service version (provided as parameter)
- `http.method`, `http.route`, `http.status_code` - HTTP attributes
- `backend.id`, `backend.addr` - Backend identification
- `work.delay_ms`, `work.duration_ms` - Work execution metrics

## Future Enhancements

- OTLP export to OpenTelemetry Collector
- Prometheus metrics integration
- Distributed trace context propagation between load balancer and workers
- Grafana dashboard integration
- Jaeger trace visualization

## Dependencies

- `opentelemetry` 0.31.0 - OpenTelemetry API
- `opentelemetry_sdk` 0.31.0 - OpenTelemetry SDK
- `tracing-opentelemetry` 0.32.0 - Bridge between tracing and OpenTelemetry
- `tracing-subscriber` 0.3 - Tracing subscriber with fmt and env-filter
- `opentelemetry-stdout` 0.31.0 - Console span exporter
