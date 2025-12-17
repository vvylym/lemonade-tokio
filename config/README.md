# Configuration Files

This directory contains configuration files for both workers and the load balancer. Configuration files support multiple formats: **JSON**, **YAML**, and **TOML**. Each worker and strategy uses a single format to demonstrate all supported formats.

## Directory Structure

```
config/
├── load-balancer.yaml        # Main load balancer configuration (YAML format)
├── worker-1.toml        # TOML format
│── worker-2.json         # JSON format
│── worker-3.yaml         # YAML format
│── worker-4.toml         # TOML format
└── worker-4.toml         # TOML format
```

## Supported Formats

All configuration files support three formats:
- **JSON** (`.json`) - JavaScript Object Notation
- **YAML** (`.yaml` or `.yml`) - YAML Ain't Markup Language
- **TOML** (`.toml`) - Tom's Obvious Minimal Language

The format is automatically detected based on the file extension.

## Worker Configuration

Worker configuration files are located in the `worker/` directory. Each worker uses a unique port and a different format:

- **worker-1.toml**: Port `50510` (TOML format)
- **worker-2.json**: Port `50520` (JSON format)
- **worker-3.yaml**: Port `50530` (YAML format)
- **worker-4.toml**: Port `50540` (TOML format)

### Worker Config Structure

The worker config structure is the same across all formats:

**TOML format:**
```toml
listen_address = "127.0.0.1:50510"
service_name = "lemonade-worker-1"
work_delay = 20
```

**JSON format:**
```json
{
  "listen_address": "127.0.0.1:50520",
  "service_name": "lemonade-worker-2",
  "work_delay": 20
}
```

**YAML format:**
```yaml
listen_address: "127.0.0.1:50530"
service_name: "lemonade-worker-3"
work_delay: 20
```

- `listen_address`: The socket address where the worker will listen (format: `IP:PORT`)
- `service_name`: A human-readable name for the worker service
- `work_delay`: The delay duration for processing work requests in milliseconds (u64)

### Using Worker Configs

```bash
# Start a worker using a config file (format is auto-detected from extension)
cargo run --release -- worker --config config/worker-1.toml
cargo run --release -- worker --config config/worker-2.json
cargo run --release -- worker --config config/worker-3.yaml

# Or specify the framework explicitly
cargo run --release -- worker --framework actix --config config/worker-1.toml
```

## Load Balancer Configuration

Load balancer configuration files are located in the `load-balancer/` directory. Each file represents a different load balancing strategy and uses different formats:

- **adaptive.toml**: Adaptive strategy that dynamically selects the best backend (TOML format)
- **round-robin.json**: Round-robin strategy that distributes requests evenly (JSON format)
- **weighted-round-robin.yaml**: Weighted round-robin with configurable backend weights (YAML format)
- **fastest-response-time.json**: Routes to the backend with the fastest response time (JSON format)
- **least-connections.yaml**: Routes to the backend with the fewest active connections (YAML format)

### Load Balancer Config Structure

The load balancer config structure is the same across all formats. Here are examples:

**TOML format:**
```toml
strategy = "adaptive"

[runtime]
metrics_cap = 100
health_cap = 50
drain_timeout_millis = 5000
background_timeout_millis = 1000
accept_timeout_millis = 2000

[proxy]
listen_address = "127.0.0.1:50501"
max_connections = 1000

[[backends]]
id = 1
name = "worker-1"
address = "127.0.0.1:50510"
```

**JSON format:**
```json
{
  "strategy": "round_robin",
  "runtime": {
    "metrics_cap": 100,
    "health_cap": 50,
    "drain_timeout_millis": 5000,
    "background_timeout_millis": 1000,
    "accept_timeout_millis": 2000
  },
  "proxy": {
    "listen_address": "127.0.0.1:50501",
    "max_connections": 1000
  },
  "backends": [
    {
      "id": 1,
      "name": "worker-1",
      "address": "127.0.0.1:50510"
    }
  ]
}
```

**YAML format:**
```yaml
strategy: "weighted_round_robin"
runtime:
  metrics_cap: 100
  health_cap: 50
  drain_timeout_millis: 5000
  background_timeout_millis: 1000
  accept_timeout_millis: 2000
proxy:
  listen_address: "127.0.0.1:50501"
  max_connections: 1000
backends:
  - id: 1
    name: "worker-1"
    address: "127.0.0.1:50510"
    weight: 10
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
# Start load balancer with the main configuration file (recommended)
cargo run --release -- load-balancer --config config/load-balancer.yaml
```

## Complete Example

To run a complete setup with 4 workers and a load balancer:

### Terminal 1: Start Worker 1 (TOML)
```bash
cargo run --release -- worker -f actix --config config/worker-1.toml
```

### Terminal 2: Start Worker 2 (JSON)
```bash
cargo run --release -- worker -f axum --config config/worker-2.json
```

### Terminal 3: Start Worker 3 (YAML)
```bash
cargo run --release -- worker -f hyper --config config/worker-3.yaml
```

### Terminal 4: Start Worker 4 (TOML)
```bash
cargo run --release -- worker -f rocket --config config/worker-4.toml
```

### Terminal 5: Start Load Balancer (YAML)
```bash
cargo run --release -- load-balancer --config config/load-balancer.yaml
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
- All three formats (JSON, YAML, TOML) are fully supported and tested
- The format is automatically detected from the file extension (`.json`, `.yaml`, `.yml`, `.toml`)

