# Lemonade Load Balancer Architecture

This document provides a detailed overview of the Lemonade Load Balancer architecture, design decisions, and implementation details.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Clean Architecture Pattern](#clean-architecture-pattern)
3. [The App Orchestrator](#the-app-orchestrator)
4. [Service Traits (Ports)](#service-traits-ports)
5. [Shared Context](#shared-context)
6. [Design Decisions](#design-decisions)
7. [Suggestions for Improvements](#suggestions-for-improvements)

## Architecture Overview

The Lemonade Load Balancer follows a **clean architecture** pattern with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                      App (Orchestrator)                  │
│  Coordinates all services and manages application state  │
└─────────────────────────────────────────────────────────┘
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Config     │  │    Health    │  │   Metrics    │
│   Service    │  │   Service    │  │   Service    │
└──────────────┘  └──────────────┘  └──────────────┘
        │                 │                 │
        └─────────────────┼─────────────────┘
                          │
                          ▼
                  ┌──────────────┐
                  │   Context    │
                  │ (Shared State)│
                  └──────────────┘
                          │
                          ▼
                  ┌──────────────┐
                  │    Proxy     │
                  │   Service    │
                  └──────────────┘
```

## Clean Architecture Pattern

The load balancer uses the **Port/Adapter** pattern (also known as Hexagonal Architecture):

- **Ports**: Trait definitions that define the interface contracts
- **Adapters**: Concrete implementations of these traits
- **Domain Logic**: Business logic that operates on the ports

This pattern provides:
- **Testability**: Easy to mock services for testing
- **Extensibility**: New implementations can be added without changing core logic
- **Separation of Concerns**: Clear boundaries between different responsibilities

### Port Modules

Each service has a `port.rs` file defining the trait interface:

- `config/port.rs`: `ConfigService` trait
- `health/port.rs`: `HealthService` trait
- `metrics/port.rs`: `MetricsService` trait
- `proxy/port.rs`: `ProxyService` trait
- `strategy/port.rs`: `StrategyService` trait

### Adapter Modules

Concrete implementations live in `adapters/` or `impls/` directories:

- `config/impls/notify.rs`: File-watching configuration service
- `health/adapters/backend.rs`: Backend health checking implementation
- `metrics/adapters/aggregating.rs`: Metrics aggregation implementation
- `proxy/adapters/tokio_proxy.rs`: Tokio-based TCP proxy implementation
- `strategy/adapters/`: Various load balancing strategy implementations

## The App Orchestrator

The `App` struct is the main orchestrator that coordinates all services. It follows a simple lifecycle:

```rust
pub struct App {
    config_service: Arc<dyn ConfigService>,
    health_service: Arc<dyn HealthService>,
    metrics_service: Arc<dyn MetricsService>,
    proxy_service: Arc<dyn ProxyService>,
}
```

### App Lifecycle

1. **Initialization**: Creates a `Context` from the initial configuration
2. **Service Startup**: Spawns background tasks for each service
3. **Runtime**: Waits for shutdown signal (Ctrl-C)
4. **Shutdown**: Gracefully drains connections and stops all services

### Service Coordination

The App uses a macro-based approach to spawn and manage background services:

```rust
let config_handle = spawn_background_handle!(self.config_service, &ctx);
let health_handle = spawn_background_handle!(self.health_service, &ctx);
let metrics_handle = spawn_background_handle!(self.metrics_service, &ctx);
let accept_handle = self.proxy_service.accept_connections(&ctx);
```

Each service receives a shared `Arc<Context>` for accessing and updating shared state.

## Service Traits (Ports)

### ConfigService

**Purpose**: Manages configuration and supports hot-reload.

**Interface**:
```rust
trait ConfigService {
    fn snapshot(&self) -> Config;
    async fn start(&self, ctx: Arc<Context>) -> Result<(), ConfigError>;
    async fn shutdown(&self) -> Result<(), ConfigError>;
}
```

**Responsibilities**:
- Provide initial configuration snapshot
- Monitor configuration changes (file watching, environment variables)
- Update the shared context when configuration changes
- Coordinate strategy updates when backends or strategy type changes

**Implementation**: `NotifyConfigService` uses the `notify` crate for file watching with debouncing.

### HealthService

**Purpose**: Monitors backend health status.

**Interface**:
```rust
trait HealthService {
    async fn start(&self, ctx: Arc<Context>) -> Result<(), HealthError>;
    async fn shutdown(&self) -> Result<(), HealthError>;
}
```

**Responsibilities**:
- Periodically check backend health
- Update the health registry in the shared context
- Only mark backends as healthy if they respond within the timeout

**Implementation**: `BackendHealthService` performs TCP health checks at configurable intervals.

### MetricsService

**Purpose**: Collects and aggregates performance metrics.

**Interface**:
```rust
trait MetricsService {
    async fn snapshot(&self) -> Result<MetricsSnapshot, MetricsError>;
    async fn start(&self, ctx: Arc<Context>) -> Result<(), MetricsError>;
    async fn shutdown(&self) -> Result<(), MetricsError>;
}
```

**Responsibilities**:
- Collect metrics events from the proxy service
- Aggregate metrics (response times, connection counts)
- Provide metrics snapshots for strategy decisions
- Update the metrics registry in the shared context

**Implementation**: `AggregatingMetricsService` aggregates metrics from events sent via channels.

### ProxyService

**Purpose**: Handles TCP connection proxying.

**Interface**:
```rust
trait ProxyService {
    async fn accept_connections(&self, ctx: &Arc<Context>) -> Result<(), ProxyError>;
}
```

**Responsibilities**:
- Accept incoming TCP connections
- Use the strategy service to select a backend
- Forward traffic to selected backends
- Track connection lifecycle events
- Emit metrics and connection events

**Implementation**: `TokioProxyService` uses Tokio for async TCP operations.

### StrategyService

**Purpose**: Selects backends based on the configured load balancing strategy.

**Interface**:
```rust
trait StrategyService {
    fn strategy(&self) -> Strategy;
    async fn pick_backend(&self, ctx: Arc<Context>) -> Result<BackendMeta, StrategyError>;
}
```

**Responsibilities**:
- Select a backend from healthy backends
- Use strategy-specific logic (round robin, least connections, etc.)
- Access shared context for metrics, health, and connection data

**Available Strategies**:
- `RoundRobin`: Circular distribution
- `LeastConnections`: Fewest active connections
- `WeightedRoundRobin`: Weighted circular distribution
- `FastestResponseTime`: Lowest response time
- `Adaptive`: Multi-factor decision making

## Shared Context

The `Context` struct is the central shared state that all services access. It uses **lock-free concurrent data structures** for high performance.

### Context Structure

```rust
pub struct Context {
    // State registries (ArcSwap for lock-free updates)
    route_table: ArcSwap<RouteTable>,
    strategy: ArcSwap<Arc<dyn StrategyService>>,
    connections: ArcSwap<ConnectionRegistry>,
    health: ArcSwap<HealthRegistry>,
    metrics: ArcSwap<MetricsSnapshot>,
    channels: ArcSwap<ChannelBundle>,
    
    // Channel version tracking
    channel_version_tx: watch::Sender<u64>,
    
    // Connection draining
    notify: Notify,
    
    // Timeout configurations
    drain_timeout_duration: Duration,
    background_handle_timeout: Duration,
    accept_handle_timeout: Duration,
    
    // Event receivers
    metrics_rx: ArcSwap<MpscReceiver<MetricsEvent>>,
    health_rx: ArcSwap<MpscReceiver<HealthEvent>>,
    
    // Shutdown coordination
    shutdown_tx: BroadcastSender<()>,
}
```

### Context Components Explained

#### 1. RouteTable (`route_table`)

**Purpose**: Maps backend IDs to backend metadata.

**Design Choice**: Uses `ArcSwap` for lock-free atomic updates when backends are added/removed.

**Operations**:
- Lookup by ID or index
- Filter healthy backends
- Update when configuration changes

**Improvement Suggestion**: Consider using a concurrent hash map (like `DashMap`) if frequent lookups by name are needed.

#### 2. Strategy (`strategy`)

**Purpose**: Current load balancing strategy implementation.

**Design Choice**: Wrapped in `Arc<Arc<dyn StrategyService>>` to allow hot-swapping strategies without blocking.

**Operations**:
- Load current strategy
- Swap to new strategy (e.g., when config changes)

**Improvement Suggestion**: Consider strategy versioning to ensure consistency during updates.

#### 3. ConnectionRegistry (`connections`)

**Purpose**: Tracks active connection counts per backend.

**Design Choice**: Uses atomic operations for lock-free connection counting.

**Operations**:
- Increment/decrement connection counts
- Get current counts for strategy decisions
- Migrate counts when backends are reconfigured

**Improvement Suggestion**: Add connection metadata (e.g., connection age) for more sophisticated strategies.

#### 4. HealthRegistry (`health`)

**Purpose**: Tracks health status of each backend.

**Design Choice**: Uses atomic flags for lock-free health status updates.

**Operations**:
- Update health status
- Check if backend is healthy
- Migrate health status when backends are reconfigured

**Improvement Suggestion**: Add health history (e.g., consecutive failures) for better failure detection.

#### 5. MetricsSnapshot (`metrics`)

**Purpose**: Current aggregated metrics per backend.

**Design Choice**: Snapshot pattern - metrics are aggregated periodically and stored as a snapshot.

**Operations**:
- Update metrics snapshot
- Access metrics for strategy decisions
- Reset/clear metrics

**Improvement Suggestion**: Consider time-series metrics storage for historical analysis and better adaptive strategies.

#### 6. ChannelBundle (`channels`)

**Purpose**: Communication channels for services to send events.

**Design Choice**: Uses `ArcSwap` to allow hot-reloading channels when configuration changes.

**Operations**:
- Send metrics events
- Send health events
- Update channels (with version tracking)

**Improvement Suggestion**: Consider bounded channels with backpressure handling for overload scenarios.

#### 7. Channel Version Tracking (`channel_version_tx`)

**Purpose**: Notifies services when channels are updated.

**Design Choice**: Uses `watch` channel for lightweight version notifications.

**Operations**:
- Subscribe to version changes
- Notify on channel updates

**Improvement Suggestion**: Add version-based event filtering to prevent processing stale events.

#### 8. Connection Draining (`notify`)

**Purpose**: Coordinates graceful connection draining during shutdown.

**Design Choice**: Uses `Notify` for lightweight wake-up notifications.

**Operations**:
- Wait for connections to close
- Notify when connection closes

**Improvement Suggestion**: Add connection timeout tracking to force-close stuck connections.

#### 9. Timeout Configurations

**Purpose**: Configurable timeouts for different operations.

**Design Choice**: Stored directly in context for easy access.

**Operations**:
- Get timeout values
- Update timeouts (requires `&mut self`)

**Improvement Suggestion**: Make timeouts updatable without requiring `&mut` (e.g., use `ArcSwap<Duration>`).

#### 10. Event Receivers (`metrics_rx`, `health_rx`)

**Purpose**: Services consume events from these receivers.

**Design Choice**: Uses `ArcSwap` to allow hot-reloading receivers when channels change.

**Operations**:
- Clone receiver for service consumption
- Update receiver when channels change

**Improvement Suggestion**: Consider using `broadcast` channels for multiple consumers if needed.

#### 11. Shutdown Coordination (`shutdown_tx`)

**Purpose**: Broadcasts shutdown signal to all services.

**Design Choice**: Uses `broadcast` channel for one-to-many notification.

**Operations**:
- Send shutdown signal
- Subscribe to shutdown notifications

**Improvement Suggestion**: Add shutdown phases (e.g., stop accepting, drain connections, cleanup) for more controlled shutdown.

### Context Access Patterns

The context provides three types of access methods:

1. **Loaders**: Return `Arc<T>` for read access (lock-free)
   ```rust
   pub fn routing_table(&self) -> Arc<RouteTable>
   pub fn strategy(&self) -> Arc<Arc<dyn StrategyService>>
   ```

2. **Swappers**: Update state atomically (lock-free)
   ```rust
   pub fn set_routing_table(&self, rt: Arc<RouteTable>)
   pub fn set_strategy(&self, strategy: Arc<dyn StrategyService>)
   ```

3. **Getters**: Return owned or cloned values
   ```rust
   pub fn healthy_backends(&self) -> Vec<BackendMeta>
   pub fn drain_timeout(&self) -> Duration
   ```

## Design Decisions

### 1. Lock-Free State Management

**Decision**: Use `ArcSwap` for all shared state instead of `Mutex` or `RwLock`.

**Rationale**:
- Eliminates lock contention
- Better performance under high concurrency
- Atomic updates without blocking readers

**Trade-offs**:
- Slightly more memory overhead (multiple `Arc` instances)
- Readers may see slightly stale data (acceptable for this use case)

### 2. Snapshot Pattern for Metrics

**Decision**: Store metrics as periodic snapshots rather than real-time updates.

**Rationale**:
- Reduces contention on metrics data
- Strategies read consistent snapshots
- Simpler aggregation logic

**Trade-offs**:
- Slight delay in metrics visibility
- May miss very short-lived spikes

### 3. Channel-Based Event System

**Decision**: Use channels for service-to-service communication.

**Rationale**:
- Decouples services
- Backpressure handling
- Easy to test and mock

**Trade-offs**:
- Event ordering guarantees depend on channel type
- Potential for event loss if channels are full

### 4. Hot-Reloadable Configuration

**Decision**: Support runtime configuration updates without restart.

**Rationale**:
- Zero-downtime updates
- Better operational flexibility
- Production-friendly

**Trade-offs**:
- More complex state migration logic
- Need to handle partial updates gracefully

### 5. Strategy as Trait Object

**Decision**: Use `dyn StrategyService` trait objects instead of enums.

**Rationale**:
- Easy to add new strategies
- Hot-swappable strategies
- Clean separation of concerns

**Trade-offs**:
- Slight performance overhead (dynamic dispatch)
- Less compile-time optimization

## Suggestions for Improvements

### 1. Observability

**Current State**: Basic metrics collection.

**Improvements**:
- Add structured logging with tracing
- Export metrics in Prometheus format
- Add distributed tracing support
- Health check endpoint with detailed status

### 2. Performance

**Current State**: Good performance with lock-free structures.

**Improvements**:
- Connection pooling for backend connections
- Batch metrics aggregation
- Optimize hot paths with profiling
- Consider SIMD for metrics calculations

### 3. Reliability

**Current State**: Basic error handling and graceful shutdown.

**Improvements**:
- Circuit breaker pattern for unhealthy backends
- Retry logic with exponential backoff
- Connection timeout tracking
- Health check failure threshold (N consecutive failures)

### 4. Configuration

**Current State**: File and environment variable support.

**Improvements**:
- Configuration validation on startup
- Configuration schema/documentation
- Support for remote configuration (e.g., etcd, Consul)
- Configuration versioning

### 5. Testing

**Current State**: Unit tests and integration tests.

**Improvements**:
- Property-based testing for state transitions
- Chaos testing for failure scenarios
- Load testing with realistic traffic patterns
- Fuzzing for input validation

### 6. State Management

**Current State**: Lock-free with `ArcSwap`.

**Improvements**:
- Add state versioning for consistency checks
- Implement state snapshots for debugging
- Add state migration utilities
- Consider event sourcing for auditability

### 7. Strategy Enhancements

**Current State**: Five basic strategies.

**Improvements**:
- Machine learning-based adaptive strategy
- Geographic routing
- Session affinity/sticky sessions
- Custom strategy plugins

### 8. Security

**Current State**: Basic TCP proxying.

**Improvements**:
- TLS termination
- Rate limiting
- IP whitelisting/blacklisting
- DDoS protection

### 9. Documentation

**Current State**: Code documentation and READMEs.

**Improvements**:
- Architecture decision records (ADRs)
- API documentation
- Deployment guides
- Performance tuning guides

### 10. Context Improvements

**Specific to Context struct**:

1. **Timeout Updates**: Make timeouts updatable without `&mut`:
   ```rust
   timeouts: ArcSwap<Timeouts>
   ```

2. **Connection Metadata**: Track more than just counts:
   ```rust
   connections: ArcSwap<ConnectionRegistry<ConnectionMetadata>>
   ```

3. **Health History**: Track health over time:
   ```rust
   health: ArcSwap<HealthRegistry<HealthHistory>>
   ```

4. **Metrics Time-Series**: Store historical metrics:
   ```rust
   metrics: ArcSwap<MetricsTimeSeries>
   ```

5. **State Versioning**: Add version numbers for consistency:
   ```rust
   state_version: AtomicU64
   ```

6. **Shutdown Phases**: More granular shutdown control:
   ```rust
   shutdown_phase: ArcSwap<ShutdownPhase>
   ```

## Conclusion

The Lemonade Load Balancer architecture provides a solid foundation with clean separation of concerns, high performance through lock-free data structures, and extensibility through the Port/Adapter pattern. The shared `Context` design balances performance, flexibility, and maintainability, with clear opportunities for enhancement as the system evolves.

