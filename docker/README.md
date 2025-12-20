# Docker Setup

This directory contains Docker configurations for running the Lemonade Load Balancer with a full observability stack.

## Quick Start

```bash
# Build and start all services
docker compose -f docker/docker-compose.yml up -d

# View logs
docker compose -f docker/docker-compose.yml logs -f

# Stop all services
docker compose -f docker/docker-compose.yml down
```

## Services

### Application Services (host ports and commands)

- **load-balancer** (50501)  
  CMD: `lemonade load-balancer --config /etc/lemonade/load-balancer.yaml`
- **worker-actix** (50510)  
  CMD: `lemonade worker --framework actix --config /etc/lemonade/worker-1.toml`
- **worker-axum** (50520)  
  CMD: `lemonade worker --framework axum --config /etc/lemonade/worker-2.json`
- **worker-hyper** (50530)  
  CMD: `lemonade worker --framework hyper --config /etc/lemonade/worker-3.yaml`
- **worker-rocket** (50540)  
  CMD: `lemonade worker --framework rocket --config /etc/lemonade/worker-4.toml`

Configs are mounted read-only from the repository root `config/` directory to `/etc/lemonade` in each container, so you can edit host configs without rebuilding images.

### Observability Stack (host ports in 50600–50700 range)

- **otel-collector**: 50601 (gRPC), 50602 (HTTP), 50603 (Prometheus metrics)
- **jaeger**: 50610 (UI), 50611 (HTTP collector)
- **prometheus**: 50620
- **grafana**: 50630

## Access Points (host)

- **Load Balancer**: http://localhost:50501
- **Jaeger UI**: http://localhost:50610
- **Prometheus**: http://localhost:50620
- **Grafana**: http://localhost:50630 (admin/admin)

## Configuration

### OpenTelemetry Collector

Configuration: `otel-collector/config.yaml`

- Receives OTLP traces and metrics from services
- Exports traces to Jaeger
- Exports metrics to Prometheus
- Exposes Prometheus metrics endpoint on port 8888

### Prometheus

Configuration: `prometheus/prometheus.yml`

Scrapes metrics from:
- Prometheus itself
- OpenTelemetry Collector
- Load balancer (if metrics endpoint available)
- All workers (if metrics endpoints available)

### Grafana

Pre-configured with:
- Prometheus datasource
- Jaeger datasource
- Dashboard provisioning

## Building Images

```bash
# Build all images
docker compose -f docker/docker-compose.yml build

# Build specific service
docker compose -f docker/docker-compose.yml build load-balancer
```

## Development

All Dockerfiles use `cargo-chef` for efficient layer caching:

1. **Planner stage**: Analyzes dependencies
2. **Cacher stage**: Builds and caches dependencies
3. **Builder stage**: Builds the application
4. **Runtime stage**: Minimal final image

This approach significantly speeds up rebuilds when only source code changes.

