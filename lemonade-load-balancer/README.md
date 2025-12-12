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

### Configuration Service

The `ConfigService` trait provides:
- `snapshot()`: Get current configuration
- `start()`: Begin monitoring for configuration changes
- `shutdown()`: Stop configuration monitoring

Configuration includes:
- Runtime settings (timeouts, capacities)
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
- `start()`: Begin health checking backends
- `shutdown()`: Stop health checking

The health service:
- Periodically checks backend health
- Updates health registry with backend status
- Only routes traffic to healthy backends
- Supports configurable health check intervals

### Metrics Service

The `MetricsService` trait provides:
- `snapshot()`: Get current metrics snapshot
- `start()`: Begin metrics collection
- `shutdown()`: Stop metrics collection

The metrics service:
- Tracks response times per backend
- Monitors connection counts
- Collects performance data
- Provides metrics snapshots for strategy decisions

### State Management

The `State` struct manages all runtime state using lock-free concurrent data structures:

- **RouteTable**: Maps backend IDs to backend metadata
- **Strategy**: Current load balancing strategy implementation
- **ConnectionRegistry**: Tracks active connections per backend
- **HealthRegistry**: Tracks health status per backend
- **MetricsSnapshot**: Performance metrics per backend
- **ChannelBundle**: Communication channels for services

State updates are atomic and lock-free using `ArcSwap`, enabling high-performance concurrent access.

## Usage Example

```rust
use lemonade_load_balancer::{App, prelude::*};
use std::sync::Arc;
use std::net::SocketAddr;

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
).await?;

app.run().await?;
```

## Configuration

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

## Dependencies

- `arc-swap`: Lock-free atomic shared references for state management
- `async-trait`: For async trait definitions
- `dashmap`: Concurrent hash map for thread-safe data structures
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

