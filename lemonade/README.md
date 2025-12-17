# Lemonade CLI

A unified command-line interface for running load balancers and worker servers.

## Overview

The `lemonade` binary provides a single entry point for:
- Running load balancers with configurable strategies
- Running worker servers with multiple HTTP framework options
- Managing configuration via files or environment variables

## Installation

Build from source:

```bash
cargo build --release
```

The binary will be available at `target/release/lemonade`.

## Usage

### Worker Command

Run a worker server with a specified HTTP framework:

```bash
lemonade worker [OPTIONS] --framework <FRAMEWORK>
```

**Frameworks:**
- `actix` or `actix-web`: Actix Web framework
- `axum`: Axum framework
- `hyper`: Hyper framework
- `rocket`: Rocket framework

**Options:**
- `-f, --framework <FRAMEWORK>`: Framework to use (required)
- `-c, --config <CONFIG_FILE>`: Path to configuration file (JSON or TOML)
- `-a, --address <LISTEN_ADDRESS>`: Listen address (e.g., `127.0.0.1:8080`)
- `-n, --name <SERVICE_NAME>`: Service name
- `-d, --delay <DELAY_MILLISECONDS>`: Work delay in milliseconds

**Examples:**

```bash
# Using command-line arguments
lemonade worker --framework actix \
  --address 127.0.0.1:4001 \
  --name worker-1 \
  --delay 20

# Using a configuration file
lemonade worker --framework axum --config worker.toml

# Using environment variables (with minimal CLI args)
lemonade worker --framework hyper
```

When using environment variables, the following are supported:
- `LEMONADE_WORKER_LISTEN_ADDRESS`: Listen address
- `LEMONADE_WORKER_SERVICE_NAME`: Service name
- `LEMONADE_WORKER_WORK_DELAY_MS`: Work delay in milliseconds

### Load Balancer Command

Run a load balancer:

```bash
lemonade load-balancer [OPTIONS]
```

**Options:**
- `-c, --config <CONFIG_FILE>`: Path to configuration file (JSON or TOML)

**Examples:**

```bash
# Using a configuration file
lemonade load-balancer --config lb.toml

# Using environment variables
LEMONADE_LB_LISTEN_ADDRESS=127.0.0.1:3000 \
LEMONADE_LB_STRATEGY=round_robin \
LEMONADE_LB_BACKEND_ADDRESSES=127.0.0.1:4001,127.0.0.1:4002 \
lemonade load-balancer
```

See the [lemonade-load-balancer README](../lemonade-load-balancer/README.md) for complete configuration options.

## Configuration Files

Both commands support JSON and TOML configuration files. Configuration files take precedence over environment variables, which take precedence over command-line arguments.

### Worker Configuration File Example

**TOML:**
```toml
listen_address = "127.0.0.1:4001"
service_name = "worker-1"
work_delay_ms = 20
```

**JSON:**
```json
{
  "listen_address": "127.0.0.1:4001",
  "service_name": "worker-1",
  "work_delay_ms": 20
}
```

### Load Balancer Configuration File Example

See the [lemonade-load-balancer README](../lemonade-load-balancer/README.md) for load balancer configuration file format.

## Complete Example: Full Stack

Start a complete load balancing system:

```bash
# Terminal 1: Start worker 1 (Hyper)
cargo run --release -- worker --framework hyper \
  --address 127.0.0.1:50510 --name worker-hyper --delay 20

# Terminal 2: Start worker 2 (Axum)
cargo run --release -- worker --framework axum \
  --address 127.0.0.1:50520 --name worker-axum --delay 20

# Terminal 3: Start worker 3 (Actix)
cargo run --release -- worker --framework actix \
  --address 127.0.0.1:50530 --name worker-actix --delay 20

# Terminal 4: Start worker 4 (Rocket)
cargo run --release -- worker --framework rocket \
  --address 127.0.0.1:50540 --name worker-rocket --delay 20

# Terminal 5: Start load balancer
cargo run --release -- load-balancer --config config/load-balancer.yaml

# Terminal 6: Test it
curl http://127.0.0.1:50501/health
curl http://127.0.0.1:50501/work
```

## Command Aliases

- `w` can be used instead of `worker`
- `lb` can be used instead of `load-balancer`

Examples:
```bash
lemonade w --framework actix --address 127.0.0.1:4001
lemonade lb --config config.toml
```

## Exit Codes

- `0`: Successful execution
- `1`: Configuration error
- `2`: Runtime error
- `130`: Interrupted by user (Ctrl-C)

## Logging

Set the `RUST_LOG` environment variable to control logging verbosity:

```bash
# Info level (default)
RUST_LOG=info lemonade load-balancer

# Debug level for specific crate
RUST_LOG=lemonade_load_balancer=debug lemonade lb

# Trace level for all components
RUST_LOG=trace lemonade worker --framework hyper
```

## Graceful Shutdown

Both workers and load balancers handle `SIGINT` (Ctrl-C) gracefully:

- **Load Balancer**: Drains active connections, stops accepting new connections, shuts down services
- **Workers**: Complete in-flight requests, close listeners cleanly

## Dependencies

- `clap`: Command-line argument parsing with derive macros
- `tokio`: Async runtime for all services
- `lemonade-service`: Shared service library for workers
- `lemonade-load-balancer`: Load balancer core library
- `lemonade-observability`: Tracing and observability
- `lemonade-worker-*`: Worker framework implementations (Actix, Axum, Hyper, Rocket)

## See Also

- [Root README](../README.md): Project overview and quick start
- [Load Balancer README](../lemonade-load-balancer/README.md): Detailed load balancer documentation
- [Service README](../lemonade-service/README.md): Worker service library documentation
- [Architecture Documentation](../docs/architecture.md): System architecture and design decisions

