# Configuration Files

This directory contains configuration files for both workers and the load balancer. All configuration files use TOML format.

## Directory Structure

```
config/
├── worker/                    # Worker configuration files
│   ├── worker-1.toml
│   ├── worker-2.toml
│   ├── worker-3.toml
│   └── worker-4.toml
└── load-balancer/            # Load balancer configuration files
    ├── adaptive.toml
    ├── round-robin.toml
    ├── weighted-round-robin.toml
    ├── fastest-response-time.toml
    └── least-connections.toml
```

## Worker Configuration

Worker configuration files are located in the `worker/` directory. Each worker uses a unique port:

- **worker-1.toml**: Port `50510`
- **worker-2.toml**: Port `50520`
- **worker-3.toml**: Port `50530`
- **worker-4.toml**: Port `50540`

### Worker Config Structure

```toml
listen_address = "127.0.0.1:50510"
service_name = "lemonade-worker-1"
work_delay = 20
```

- `listen_address`: The socket address where the worker will listen (format: `IP:PORT`)
- `service_name`: A human-readable name for the worker service
- `work_delay`: The delay duration for processing work requests in milliseconds (u64)

### Using Worker Configs

```bash
# Start a worker using a config file
cargo run --release -- worker --config config/worker/worker-1.toml

# Or specify the framework explicitly
cargo run --release -- worker --framework actix --config config/worker/worker-1.toml
```

## Load Balancer Configuration

Load balancer configuration files are located in the `load-balancer/` directory. Each file represents a different load balancing strategy:

- **adaptive.toml**: Adaptive strategy that dynamically selects the best backend
- **round-robin.toml**: Round-robin strategy that distributes requests evenly
- **weighted-round-robin.toml**: Weighted round-robin with configurable backend weights
- **fastest-response-time.toml**: Routes to the backend with the fastest response time
- **least-connections.toml**: Routes to the backend with the fewest active connections

### Load Balancer Config Structure

```toml
[runtime]
metrics_cap = 100
health_cap = 50
drain_timeout_millis = 5000
background_timeout_millis = 1000
accept_timeout_millis = 2000

[proxy]
listen_address = "127.0.0.1:50501"
max_connections = 1000

strategy = "adaptive"

[[backends]]
id = 1
name = "worker-1"
address = "127.0.0.1:50510"

# ... more backends ...

[health]
interval = 30000  # milliseconds
timeout = 30000   # milliseconds

[metrics]
interval = 10000  # milliseconds
timeout = 10000   # milliseconds
```

#### Configuration Sections

- **`[runtime]`**: Runtime configuration
  - `metrics_cap`: Maximum capacity for metrics collection
  - `health_cap`: Maximum capacity for health checks
  - `drain_timeout_millis`: Timeout for draining connections during shutdown
  - `background_timeout_millis`: Timeout for background operations
  - `accept_timeout_millis`: Timeout for accepting new connections

- **`[proxy]`**: Proxy server configuration
  - `listen_address`: The socket address where the load balancer will listen
  - `max_connections`: Optional maximum number of concurrent connections

- **`strategy`**: Load balancing strategy (one of: `adaptive`, `round_robin`, `weighted_round_robin`, `fastest_response_time`, `least_connections`)

- **`[[backends]]`**: Array of backend worker configurations
  - `id`: Unique identifier for the backend (u8)
  - `name`: Optional human-readable name
  - `address`: Socket address of the backend worker (must match worker config)
  - `weight`: Optional weight for weighted strategies (u8, 1-255)

- **`[health]`**: Health check configuration
  - `interval`: Time between health checks (milliseconds)
  - `timeout`: Timeout for health check requests (milliseconds)

- **`[metrics]`**: Metrics collection configuration
  - `interval`: Time between metrics collection (milliseconds)
  - `timeout`: Timeout for metrics collection requests (milliseconds)

### Using Load Balancer Configs

```bash
# Start load balancer with a specific strategy
cargo run --release -- load-balancer --config config/load-balancer/adaptive.toml

# Or use round-robin
cargo run --release -- load-balancer --config config/load-balancer/round-robin.toml
```

## Complete Example

To run a complete setup with 4 workers and a load balancer:

### Terminal 1: Start Worker 1
```bash
cargo run --release -- worker  -f actix --config config/worker/worker-1.toml
```

### Terminal 2: Start Worker 2
```bash
cargo run --release -- worker -f axum --config config/worker/worker-2.toml
```

### Terminal 3: Start Worker 3
```bash
cargo run --release -- worker -f hyper --config config/worker/worker-3.toml
```

### Terminal 4: Start Worker 4
```bash
cargo run --release -- worker -f rocket --config config/worker/worker-4.toml
```

### Terminal 5: Start Load Balancer
```bash
cargo run --release -- load-balancer --config config/load-balancer/round-robin.toml
```

### Testing

Once all services are running, test the load balancer:

```bash
# Health check
curl http://127.0.0.1:50501/health

# Work endpoint (will be distributed across workers)
curl http://127.0.0.1:50501/work
```

## Notes

- All worker ports (50510-50540) must be available and not in use
- The load balancer listens on port `50501` (configurable in each config file)
- Maximum of 4 workers are configured in all load balancer configs
- Backend addresses in load balancer configs must match the `listen_address` in worker configs
- For `weighted_round_robin` strategy, ensure all backends have a `weight` field defined
- Health and metrics intervals/timeouts are specified in milliseconds

