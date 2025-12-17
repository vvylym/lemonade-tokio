# Lemonade Load Balancer

A high-performance, production-ready TCP load balancer library built in Rust with support for multiple load balancing strategies, health checking, metrics collection, and dynamic configuration updates.

## Overview

`lemonade-load-balancer` is a library that implements a sophisticated load balancing system following clean architecture principles. It provides:

- **Multiple Load Balancing Strategies**: Choose from 5 different algorithms
- **Health Monitoring**: Automatic backend health checking
- **Performance Metrics**: Real-time metrics collection and analysis
- **Dynamic Configuration**: Hot-reload configuration without downtime
- **Graceful Shutdown**: Safe connection draining and resource cleanup
- **Concurrent State Management**: Lock-free state updates using `ArcSwap`

## Architecture

The library follows a clean architecture pattern with clear separation of concerns:

- **Ports**: Trait definitions for all services (`ConfigService`, `HealthService`, `MetricsService`, `ProxyService`, `StrategyService`)
- **Models**: Data structures for configuration, responses, and state
- **Types**: Core runtime types including state management, registries, and routing tables
- **App**: Main orchestrator that coordinates all services

## Key Components

### App

The `App` struct is the main entry point that:
- Initializes and coordinates all services
- Manages application state
- Handles graceful shutdown on Ctrl-C
- Ensures proper cleanup of all background tasks

### Shared Context

The `Context` struct is the central state coordinator that provides:
- **Thread-safe configuration access**: Atomic config updates via `ArcSwap`
- **Dynamic strategy switching**: Hot-swap strategies at runtime
- **Backend routing**: DashMap-based concurrent route table
- **Typed communication channels**: Separate channels for config, health, metrics, and proxy events
- **Migration support**: Graceful backend draining during config changes

All services interact with the shared context rather than maintaining their own state, ensuring consistency and reducing complexity.

### Configuration Service

The `ConfigService` trait provides:
- `watch_config()`: Monitor for configuration changes and update the context

The service automatically:
- Watches for file changes (when using file-based config)
- Migrates backends gracefully (draining old backends before removal)
- Updates strategy dynamically
- Notifies other services via config channel

Configuration includes:
- Runtime settings (timeouts, capacities, watch intervals)
- Proxy settings (listen address, max connections)
- Load balancing strategy
- Backend definitions
- Health check configuration
- Metrics configuration

### Proxy Service

The `ProxyService` trait handles:
- `accept_connections()`: Listens for incoming TCP connections and forwards them to selected backends

The proxy service:
- Accepts connections on a configured listen address
- Uses the selected strategy to pick a backend
- Forwards traffic to healthy backends only
- Tracks connection counts per backend
- Handles connection lifecycle events

### Strategy Service

The `StrategyService` trait provides:
- `strategy()`: Get the current strategy type
- `pick_backend()`: Select a backend based on the strategy

#### Available Strategies

1. **Adaptive**: Dynamically adjusts based on multiple factors
2. **Fastest Response Time**: Routes to the backend with the lowest response time
3. **Least Connections**: Routes to the backend with the fewest active connections
4. **Round Robin**: Distributes requests evenly in a circular fashion
5. **Weighted Round Robin**: Distributes requests based on backend weights

Strategies can be hot-swapped at runtime without service interruption.

### Health Service

The `HealthService` trait provides:
- `check_backend()`: Perform health check on a specific backend

The health service:
- Runs periodic health checks in background tasks
- Updates backend health atomically (no locks required)
- Listens for proxy-reported failures for immediate detection
- Avoids checking backends with active connections (reduces load)
- Defaults all backends to healthy on startup
- Supports configurable health check intervals and timeouts

### Metrics Service

The `MetricsService` trait provides:
- `collect_metrics()`: Collect metrics from backends in background

The metrics service:
- Aggregates metrics from backend atomic state
- Tracks requests, errors, latency per backend
- Runs periodic collection in background tasks
- Provides real-time metrics for strategy decisions
- Supports configurable collection intervals

### State Management

The `Context` struct manages all runtime state using lock-free concurrent data structures:

- **Config**: Shared configuration via `ArcSwap` for atomic updates
- **RouteTable**: `DashMap`-based concurrent map of backend ID → `Backend` state
- **Strategy**: Current load balancing strategy (swappable at runtime)
- **ChannelBundle**: Typed channels for inter-service communication
  - `config_tx/rx`: Configuration change events
  - `health_tx/rx`: Health check events
  - `metrics_tx/rx`: Metrics update events  
  - `connection_tx/rx`: Connection lifecycle events

The `Backend` struct contains:
- **Immutable metadata**: ID, name, address, weight
- **Atomic state**: Health, connections, requests, errors, latency (all lock-free)
- **Migration status**: Active or Draining

All state updates are atomic and lock-free using `Arc`, `ArcSwap`, `DashMap`, and atomic types (`AtomicBool`, `AtomicU64`, etc.), enabling high-performance concurrent access without locks.

## Usage

### Simple Usage

The easiest way to use the load balancer is via the `run()` function:

```rust
use lemonade_load_balancer::run;
use std::path::PathBuf;

// Run with a configuration file
run(Some(PathBuf::from("config.toml"))).await?;

// Or run with environment variables
run(None).await?;
```

The `run()` function automatically creates all necessary services and adapters:
- `NotifyConfigService` for configuration (with file watching if a file is provided)
- `BackendHealthService` for health checking
- `AggregatingMetricsService` for metrics collection
- `TokioProxyService` for TCP proxying

### Advanced Usage

For custom service implementations:

```rust
use lemonade_load_balancer::{App, prelude::*};
use std::sync::Arc;

// Create your service implementations
let config_service: Arc<dyn ConfigService> = /* your implementation */;
let health_service: Arc<dyn HealthService> = /* your implementation */;
let metrics_service: Arc<dyn MetricsService> = /* your implementation */;
let proxy_service: Arc<dyn ProxyService> = /* your implementation */;

// Create and run the app
let app = App::new(
    config_service,
    health_service,
    metrics_service,
    proxy_service,
).await;

app.run().await?;
```

## Configuration

The load balancer can be configured via:
1. **Configuration files** (JSON, YAML, or TOML) - supports hot-reload via file watching
2. **Environment variables** - prefixed with `LEMONADE_LB_`

### Configuration File Format

Configuration files support JSON, YAML, and TOML formats. When a configuration file is provided, the load balancer watches for changes and automatically reloads the configuration.

**TOML Example:**
```toml
[runtime]
metrics_cap = 1000
health_cap = 100
drain_timeout_millis = 5000
background_timeout_millis = 3000
accept_timeout_millis = 2000

[proxy]
listen_address = "127.0.0.1:3000"
max_connections = 10000

strategy = "round_robin"

[[backends]]
id = 0
name = "backend-1"
address = "127.0.0.1:4001"
weight = 1

[[backends]]
id = 1
name = "backend-2"
address = "127.0.0.1:4002"
weight = 2

[health]
interval_ms = 30000
timeout_ms = 5000

[metrics]
interval_ms = 10000
timeout_ms = 5000
```

**JSON Example:**
```json
{
  "runtime": {
    "metrics_cap": 1000,
    "health_cap": 100,
    "drain_timeout_millis": 5000,
    "background_timeout_millis": 3000,
    "accept_timeout_millis": 2000
  },
  "proxy": {
    "listen_address": "127.0.0.1:3000",
    "max_connections": 10000
  },
  "strategy": "round_robin",
  "backends": [
    {
      "id": 0,
      "name": "backend-1",
      "address": "127.0.0.1:4001",
      "weight": 1
    },
    {
      "id": 1,
      "name": "backend-2",
      "address": "127.0.0.1:4002",
      "weight": 2
    }
  ],
  "health": {
    "interval_ms": 30000,
    "timeout_ms": 5000
  },
  "metrics": {
    "interval_ms": 10000,
    "timeout_ms": 5000
  }
}
```

**YAML Example:**
```yaml
runtime:
  metrics_cap: 1000
  health_cap: 100
  drain_timeout_millis: 5000
  background_timeout_millis: 3000
  accept_timeout_millis: 2000

proxy:
  listen_address: "127.0.0.1:3000"
  max_connections: 10000

strategy: round_robin

backends:
  - id: 0
    name: backend-1
    address: "127.0.0.1:4001"
    weight: 1
  - id: 1
    name: backend-2
    address: "127.0.0.1:4002"
    weight: 2

health:
  interval_ms: 30000
  timeout_ms: 5000

metrics:
  interval_ms: 10000
  timeout_ms: 5000
```

### Environment Variables

When no configuration file is provided, the load balancer reads configuration from environment variables:

**Runtime Configuration:**
- `LEMONADE_LB_METRICS_CAP` (default: `100`)
- `LEMONADE_LB_HEALTH_CAP` (default: `50`)
- `LEMONADE_LB_DRAIN_TIMEOUT_MS` (default: `5000`)
- `LEMONADE_LB_BACKGROUND_TIMEOUT_MS` (default: `1000`)
- `LEMONADE_LB_ACCEPT_TIMEOUT_MS` (default: `2000`)

**Proxy Configuration:**
- `LEMONADE_LB_LISTEN_ADDRESS` (default: `127.0.0.1:3000`)
- `LEMONADE_LB_MAX_CONNECTIONS` (optional)

**Strategy:**
- `LEMONADE_LB_STRATEGY` (default: `round_robin`)
  - Options: `round_robin`, `least_connections`, `weighted_round_robin`, `fastest_response_time`, `adaptive`

**Backend Configuration:**
- `LEMONADE_LB_BACKEND_ADDRESSES`: Comma-separated list of backend addresses (e.g., `127.0.0.1:4001,127.0.0.1:4002`)

**Health Configuration:**
- `LEMONADE_LB_HEALTH_INTERVAL_MS` (default: `30000`)
- `LEMONADE_LB_HEALTH_TIMEOUT_MS` (default: `30000`)

**Metrics Configuration:**
- `LEMONADE_LB_METRICS_INTERVAL_MS` (default: `10000`)
- `LEMONADE_LB_METRICS_TIMEOUT_MS` (default: `10000`)

### Configuration Struct

The load balancer is configured via a `Config` struct:

```rust
Config {
    runtime: RuntimeConfig {
        metrics_cap: 1000,
        health_cap: 100,
        drain_timeout_millis: 5000,
        background_timeout_millis: 3000,
        accept_timeout_millis: 1000,
    },
    proxy: ProxyConfig {
        listen_address: "0.0.0.0:8080".parse()?,
        max_connections: Some(10000),
    },
    strategy: Strategy::RoundRobin,
    backends: vec![
        BackendMeta {
            id: 0,
            name: Some("backend-1".to_string()),
            address: BackendAddress::from("127.0.0.1:3001"),
            weight: Some(1),
        },
        // ... more backends
    ],
    health: HealthConfig { /* ... */ },
    metrics: MetricsConfig { /* ... */ },
}
```

## Error Handling

The library uses `thiserror` for comprehensive error handling:

- `ConfigError`: Configuration-related errors
- `HealthError`: Health checking errors
- `MetricsError`: Metrics collection errors
- `ProxyError`: Proxy/connection errors
- `StateError`: State management errors
- `StrategyError`: Strategy selection errors
- `Error`: Top-level error enum wrapping all errors

## Configuration Hot-Reload

When a configuration file is provided, the load balancer automatically watches for file changes and reloads the configuration without downtime. Changes to the following are supported:

- Backend list (additions, removals, modifications)
- Load balancing strategy
- Health check configuration
- Metrics configuration
- Proxy configuration

The configuration service uses file watching with debouncing to avoid excessive reloads during rapid file changes.

## Dependencies

- `arc-swap`: Lock-free atomic shared references for state management
- `async-trait`: For async trait definitions
- `dashmap`: Concurrent hash map for thread-safe data structures
- `notify`: File system event watching for configuration hot-reload
- `serde` / `serde_json` / `toml`: Configuration file parsing
- `thiserror`: For error handling
- `tokio`: Async runtime for networking and concurrency

## Features

- **High Performance**: Lock-free state management and efficient concurrent data structures
- **Production Ready**: Graceful shutdown, connection draining, and error handling
- **Extensible**: Clean architecture allows easy addition of new strategies and services
- **Dynamic**: Hot-reload configuration and strategy changes without downtime
- **Observable**: Built-in metrics and health monitoring
- **Type Safe**: Strong typing throughout with Rust's type system

## Use Cases

This load balancer is designed for:
- Distributing TCP traffic across multiple backend servers
- High-availability service architectures
- Microservices and service mesh implementations
- Performance testing and benchmarking
- Production-grade load balancing requirements

