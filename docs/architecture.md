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

**Purpose**: Monitors configuration changes and triggers context updates.

**Interface**:
```rust
#[async_trait]
trait ConfigService: Send + Sync {
    async fn watch_config(&self, ctx: &Arc<Context>) -> Result<()>;
}
```

**Responsibilities**:
- Watch for configuration changes (file watching for File source, no-op for Environment source)
- Call `ctx.migrate()` when config changes
- Emit `ConfigEvent::Migrated` or `ConfigEvent::ListenAddressChanged` to notify services
- Debounce rapid file changes

**Implementation**: `NotifyConfigService` uses the `notify` crate for file watching.

**Key Features**:
- Automatic source detection (File vs Environment) from Config
- Hot-reload with configurable watch interval
- Graceful backend migration via `Context::migrate()`

### HealthService

**Purpose**: Performs health checks on individual backends.

**Interface**:
```rust
#[async_trait]
trait HealthService: Send + Sync {
    async fn check_backend(&self, backend: Arc<Backend>, timeout: Duration) -> Result<()>;
}
```

**Responsibilities**:
- Perform single backend health check (TCP connection attempt)
- Update backend's atomic health state: `backend.set_health(alive)`
- Update last health check timestamp
- Run periodic checks in background tasks

**Implementation**: `BackendHealthService` performs TCP health checks.

**Key Features**:
- Defaults backends to healthy on startup
- Avoids checking backends with active connections (reduces load)
- Listens for `BackendFailureEvent` from proxy for immediate detection
- Configurable interval and timeout per config

### MetricsService

**Purpose**: Aggregates metrics from backend atomic state.

**Interface**:
```rust
#[async_trait]
trait MetricsService: Send + Sync {
    async fn collect_metrics(&self, ctx: &Arc<Context>) -> Result<()>;
}
```

**Responsibilities**:
- Run periodic background collection
- Read atomic metrics from each `Backend` instance
- Aggregate backend-level metrics for observability
- Prepare snapshots for strategy decisions

**Implementation**: `AggregatingMetricsService` reads from `Backend` atomic fields.

**Key Features**:
- Lock-free metric reading via atomics
- Each backend tracks: requests, errors, latency, connections
- Metrics updated directly by proxy service on per-request basis
- Configurable collection interval

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
    // Shared configuration (ArcSwap for lock-free updates)
    config: ArcSwap<Config>,
    
    // Concurrent backend routing table (DashMap for lock-free concurrent access)
    route_table: Arc<RouteTable>,  // DashMap<BackendId, Arc<Backend>>
    
    // Current strategy (swappable at runtime)
    strategy: ArcSwap<Arc<dyn StrategyService>>,
    
    // Typed communication channels
    channels: Arc<ChannelBundle>,
    
    // Connection draining coordination
    notify: Arc<Notify>,
    
    // Shutdown coordination
    shutdown_tx: broadcast::Sender<()>,
}
```

### Backend Structure

Each backend in the `RouteTable` contains both metadata and atomic runtime state:

```rust
pub struct Backend {
    // Immutable metadata
    id: BackendId,
    name: Option<String>,
    address: SocketAddr,
    weight: Option<u8>,
    
    // Atomic runtime state (lock-free)
    alive: AtomicBool,                // Health status
    last_health_check_ms: AtomicU64,  // Last health check timestamp
    active_connections: AtomicUsize,   // Current connection count
    total_requests: AtomicU64,        // Total requests handled
    total_errors: AtomicU64,          // Total errors encountered
    total_latency_ms: AtomicU64,      // Cumulative latency
    last_metrics_update_ms: AtomicU64,// Last metrics update
    status: AtomicU8,                 // Active(0) or Draining(1)
}
```

### Context Components Explained

#### 1. Config (`config`)

**Purpose**: Shared configuration accessible to all services.

**Design Choice**: Uses `ArcSwap` for atomic config updates without locks.

**Operations**:
- Load current config snapshot
- Update config atomically (triggers migration)
- Access config fields (timeouts, intervals, etc.)

**Key Features**: 
- Config source tracking (File vs Environment)
- Drain timeout configuration
- Hot-reload support via file watching

#### 2. RouteTable (`route_table`)

**Purpose**: Maps backend IDs to `Backend` instances with unified state.

**Design Choice**: Uses `DashMap` for lock-free concurrent access - multiple services can read/write simultaneously without blocking.

**Operations**:
- Lookup backend by ID: `O(1)` concurrent access
- Iterate over all backends
- Filter healthy backends
- Add/remove backends during migration

**Key Advantages**:
- No separate health/connection/metrics registries needed
- Atomic operations on individual backend state
- True concurrent access without read locks

#### 3. Strategy (`strategy`)

**Purpose**: Current load balancing strategy implementation.

**Design Choice**: Wrapped in `ArcSwap<Arc<dyn StrategyService>>` to allow hot-swapping strategies without blocking.

**Operations**:
- Load current strategy
- Swap to new strategy (e.g., when config changes)
- Execute `pick_backend()` for request routing

**Key Features**:
- Runtime strategy switching
- No service restart required
- Strategies access backends via Context

#### 4. ChannelBundle (`channels`)

**Purpose**: Typed communication channels for inter-service communication.

**Design Choice**: Separate channels for each event type ensures type safety and allows services to only receive events they need.

**Channels**:
- `config_tx/rx`: `broadcast::Receiver<ConfigEvent>` - Config change notifications
- `health_tx/rx`: `broadcast::Receiver<HealthEvent>` - Health check results
- `metrics_tx/rx`: `mpsc::Receiver<MetricsEvent>` - Metrics collection (unused in current implementation)
- `connection_tx/rx`: `broadcast::Receiver<ConnectionEvent>` - Connection lifecycle events
- `failure_tx/rx`: `mpsc::Receiver<BackendFailureEvent>` - Proxy-reported failures

**Key Features**:
- Type-safe event passing
- Services subscribe only to needed channels
- Broadcast for one-to-many, mpsc for point-to-point

#### 5. Connection Draining (`notify`)

**Purpose**: Coordinates graceful connection draining during backend migration and shutdown.

**Design Choice**: Uses `tokio::sync::Notify` for lightweight wake-up notifications.

**Operations**:
- `wait_for_drain()`: Async wait for active connections to complete
- `notify_one()`: Wake up one waiter when connection closes
- Used during backend removal and shutdown

**Key Features**:
- Zero allocation for notify
- Efficient for infrequent wake-ups
- Works with drain timeout

#### 6. Shutdown Coordination (`shutdown_tx`)

**Purpose**: Broadcasts shutdown signal to all services.

**Design Choice**: Uses `broadcast` channel for one-to-many notification.

**Operations**:
- Send shutdown signal: `shutdown_tx.send(())`
- Subscribe: `shutdown_tx.subscribe()`
- All services listen for shutdown

**Shutdown Sequence**:
1. App receives Ctrl-C (SIGINT)
2. Broadcasts shutdown signal
3. Services stop background tasks
4. Active connections drain (with timeout)
5. Cleanup and exit

### Context Access Patterns

The context provides several types of access methods:

1. **Configuration Access**: Lock-free config reading
   ```rust
   pub fn config(&self) -> Arc<Config>
   pub fn set_config(&self, config: Config)  // Triggers migration
   pub fn drain_timeout(&self) -> Duration
   ```

2. **Backend Access**: Concurrent via DashMap
   ```rust
   pub fn get_backend(&self, id: BackendId) -> Option<Arc<Backend>>
   pub fn get_all_backends(&self) -> Vec<Arc<Backend>>
   pub fn healthy_backends(&self) -> Vec<Arc<Backend>>
   ```

3. **Strategy Access**: Lock-free strategy loading/swapping
   ```rust
   pub fn strategy(&self) -> Arc<dyn StrategyService>
   pub fn set_strategy(&self, strategy: Arc<dyn StrategyService>)
   ```

4. **Channel Access**: Direct channel access
   ```rust
   pub fn channels(&self) -> &Arc<ChannelBundle>
   pub fn config_rx(&self) -> broadcast::Receiver<ConfigEvent>
   ```

5. **Migration**: Coordinated state updates
   ```rust
   pub async fn migrate(&self, new_config: Config) -> Result<()>
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

