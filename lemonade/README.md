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

## Aliases

- `w` can be used instead of `worker`
- `lb` can be used instead of `load-balancer`

## Dependencies

- `clap`: Command-line argument parsing
- `lemonade-service`: Shared service library
- `lemonade-load-balancer`: Load balancer library
- `lemonade-worker-*`: Worker framework implementations

