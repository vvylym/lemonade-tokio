# Load Balancer Performance Benchmarking Project

A high-performance, asynchronous TCP load balancer written in Rust with comprehensive benchmarking capabilities. This project compares different load balancing algorithms and worker implementations (Actix Web vs Axum) to evaluate performance characteristics.

## Overview

This project implements a production-ready load balancer with multiple load balancing algorithms and two different HTTP worker implementations. The primary goal is to benchmark and compare:

1. **Load Balancing Algorithms**: Round Robin, Least Connections, Weighted Round Robin, and Adaptive
2. **Worker Frameworks**: Actix Web and Axum HTTP servers
3. **Performance Metrics**: Throughput, latency, and resource utilization


## Features

### Load Balancer
- **Multiple Algorithms**: Round Robin, Least Connections, Weighted Round Robin, and Adaptive
- **Health Checking**: Automatic backend health monitoring with configurable intervals
- **Connection Tracking**: Real-time connection count tracking per backend
- **Graceful Shutdown**: Clean shutdown handling for all components
- **Metrics**: Prometheus-compatible metrics collection
- **Asynchronous I/O**: Built on Tokio for high-performance, non-blocking operations

### Worker Servers
- **Actix Web Worker**: Full-featured HTTP server using Actix Web framework
- **Axum Worker**: Modern HTTP server using Axum framework
- **Unified Metrics**: Shared metrics library for consistent monitoring
- **Health Endpoints**: `/health` for health checks
- **Work Endpoints**: `/work` for simulating processing workloads

### Shared Components
- **Worker Metrics Library**: Unified metrics interface for all workers
- **Prometheus Integration**: Built-in Prometheus metrics export
- **Comprehensive Testing**: Unit tests and integration tests

## Prerequisites

- **Rust**: Version 1.70 or later (Edition 2024)
- **Cargo**: Latest stable version
- **Just** (optional): For convenient command execution (`cargo install just`)

## Quick Start

### 1. Clone and Build

```bash
git clone <repository-url>
cd lb
cargo build --release
```

### 2. Start Workers

In separate terminals:

```bash
# Terminal 1: Start Actix worker
just worker-actix
# or
ACTIX_WORKER_PORT=4001 cargo run -p worker-actix --release

# Terminal 2: Start Axum worker
just worker-axum
# or
AXUM_WORKER_PORT=4002 cargo run -p worker-axum --release
```

### 3. Start Load Balancer

```bash
# Terminal 3: Start load balancer
just load-balancer
# or
LB_BACKEND_ADDRESSES=127.0.0.1:4001,127.0.0.1:4002 \
LB_LOAD_BALANCING_ALGORITHM=round-robin \
cargo run -p load-balancer --release
```

### 4. Test the Setup

```bash
# Health check through load balancer
curl http://127.0.0.1:4000/health

# Work endpoint through load balancer
curl "http://127.0.0.1:4000/work"
```
