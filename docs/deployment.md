# Deployment Guide

This guide covers production deployment best practices for the Lemonade Load Balancer and worker services.

## Production Checklist

- [ ] Configure appropriate timeouts and connection limits
- [ ] Set up health check monitoring
- [ ] Configure log aggregation
- [ ] Set up metrics collection
- [ ] Configure graceful shutdown handling
- [ ] Test hot-reload in staging environment
- [ ] Set up alerting for backend failures
- [ ] Document rollback procedures
- [ ] Configure resource limits (file descriptors, memory)
- [ ] Test failure scenarios

## Configuration

### Load Balancer Production Config

Example `production.toml`:

```toml
[runtime]
metrics_cap = 10000
health_cap = 1000
drain_timeout_millis = 30000        # 30s for graceful drain
background_timeout_millis = 60000   # 60s for background tasks
accept_timeout_millis = 120000      # 120s for proxy shutdown
config_watch_interval_millis = 5000 # 5s for config changes

[proxy]
listen_address = "0.0.0.0:8080"
max_connections = 100000            # Adjust based on system limits

[strategy]
algorithm = "adaptive"              # Best for production

[[backends]]
id = 0
name = "backend-1"
address = "10.0.1.10:8080"
weight = 10

[[backends]]
id = 1
name = "backend-2"
address = "10.0.1.11:8080"
weight = 10

[[backends]]
id = 2
name = "backend-3"
address = "10.0.1.12:8080"
weight = 10

[health]
interval = "10s"                    # Frequent health checks
timeout = "3s"                      # Quick failure detection

[metrics]
interval = "30s"                    # Regular metrics aggregation
```

### Environment Variables for Production

```bash
# Log configuration
export RUST_LOG=info,lemonade_load_balancer=info

# Resource limits
ulimit -n 100000                    # File descriptor limit

# Optional: Use environment-based config
export LEMONADE_LB_LISTEN_ADDRESS=0.0.0.0:8080
export LEMONADE_LB_STRATEGY=adaptive
export LEMONADE_LB_BACKEND_ADDRESSES=10.0.1.10:8080,10.0.1.11:8080,10.0.1.12:8080
export LEMONADE_LB_MAX_CONNECTIONS=100000
```

## Deployment Methods

### Systemd Service

Create `/etc/systemd/system/lemonade-lb.service`:

```ini
[Unit]
Description=Lemonade Load Balancer
After=network.target

[Service]
Type=simple
User=lemonade
Group=lemonade
WorkingDirectory=/opt/lemonade
ExecStart=/opt/lemonade/bin/lemonade load-balancer --config /etc/lemonade/production.toml
Restart=always
RestartSec=10

# Resource limits
LimitNOFILE=100000
LimitNPROC=1000

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/lemonade

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=lemonade-lb

[Install]
WantedBy=multi-user.target
```

Start and enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable lemonade-lb
sudo systemctl start lemonade-lb
sudo systemctl status lemonade-lb

# View logs
sudo journalctl -u lemonade-lb -f
```

### Docker Deployment

#### Dockerfile for Load Balancer

```dockerfile
FROM rust:1.75-slim as builder

WORKDIR /build
COPY . .
RUN cargo build --release --bin lemonade

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/lemonade /usr/local/bin/lemonade
COPY config/load-balancer.toml /etc/lemonade/config.toml

USER nobody
EXPOSE 8080

CMD ["lemonade", "load-balancer", "--config", "/etc/lemonade/config.toml"]
```

Build and run:

```bash
# Build image
docker build -t lemonade-lb:latest -f Dockerfile.lb .

# Run container
docker run -d \
  --name lemonade-lb \
  -p 8080:8080 \
  -v /etc/lemonade/config.toml:/etc/lemonade/config.toml:ro \
  --restart unless-stopped \
  --ulimit nofile=100000:100000 \
  lemonade-lb:latest

# View logs
docker logs -f lemonade-lb
```

### Docker Compose

See the complete Docker Compose setup in the observability plan for multi-service deployment.

### Kubernetes Deployment

Example Kubernetes manifests:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: lemonade-lb
  labels:
    app: lemonade-lb
spec:
  replicas: 3
  selector:
    matchLabels:
      app: lemonade-lb
  template:
    metadata:
      labels:
        app: lemonade-lb
    spec:
      containers:
      - name: lemonade-lb
        image: lemonade-lb:latest
        ports:
        - containerPort: 8080
          name: http
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
        livenessProbe:
          tcpSocket:
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          tcpSocket:
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /etc/lemonade
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: lemonade-config
---
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: lemonade-lb
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    protocol: TCP
  selector:
    app: lemonade-lb
---
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: lemonade-config
data:
  config.toml: |
    [runtime]
    metrics_cap = 10000
    health_cap = 1000
    drain_timeout_millis = 30000
    background_timeout_millis = 60000
    accept_timeout_millis = 120000
    config_watch_interval_millis = 5000

    [proxy]
    listen_address = "0.0.0.0:8080"
    max_connections = 100000

    [strategy]
    algorithm = "adaptive"

    [[backends]]
    id = 0
    name = "backend-1"
    address = "backend-1:8080"

    [[backends]]
    id = 1
    name = "backend-2"
    address = "backend-2:8080"

    [health]
    interval = "10s"
    timeout = "3s"

    [metrics]
    interval = "30s"
```

Deploy:

```bash
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f configmap.yaml

# Check status
kubectl get pods -l app=lemonade-lb
kubectl logs -f deployment/lemonade-lb
```

## Monitoring and Observability

### Health Checks

The load balancer exposes health via TCP connection:

```bash
# Check if listening
nc -zv localhost 8080

# Or use health check script
#!/bin/bash
timeout 3 bash -c "</dev/tcp/localhost/8080" && echo "Healthy" || echo "Unhealthy"
```

### Logging

Configure structured logging with JSON output:

```bash
# Set log level
export RUST_LOG=info,lemonade_load_balancer=info

# JSON logs (future)
export LEMONADE_LOG_FORMAT=json
```

### Metrics

Key metrics to monitor:

- **Backend health**: Percentage of healthy backends
- **Request rate**: Requests per second per backend
- **Error rate**: Errors per second per backend
- **Latency**: P50, P95, P99 response times
- **Active connections**: Current connections per backend
- **Connection rate**: New connections per second

Access metrics (future Prometheus endpoint):

```bash
curl http://localhost:9090/metrics
```

### Alerting

Set up alerts for:

- All backends unhealthy
- High error rate (>5%)
- High latency (P95 > threshold)
- Connection limit reached
- Config reload failures

## Rolling Updates

### Hot Configuration Reload

Modify configuration file:

```bash
# 1. Update config file
vim /etc/lemonade/production.toml

# 2. Changes apply automatically within config_watch_interval_millis
# No restart required!
```

### Binary Updates with Zero Downtime

Using systemd:

```bash
# 1. Build new binary
cargo build --release

# 2. Copy to staging location
cp target/release/lemonade /opt/lemonade/bin/lemonade.new

# 3. Gracefully stop current instance
sudo systemctl stop lemonade-lb

# 4. Replace binary
sudo mv /opt/lemonade/bin/lemonade.new /opt/lemonade/bin/lemonade

# 5. Start new version
sudo systemctl start lemonade-lb
```

Using Docker:

```bash
# 1. Build new image
docker build -t lemonade-lb:v2 .

# 2. Update deployment (rolling update)
kubectl set image deployment/lemonade-lb lemonade-lb=lemonade-lb:v2

# Or with docker-compose
docker-compose up -d --no-deps --build lemonade-lb
```

## Backup and Disaster Recovery

### Configuration Backup

```bash
# Automated backup script
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
cp /etc/lemonade/production.toml /backup/lemonade-config-$DATE.toml

# Keep last 30 days
find /backup/ -name "lemonade-config-*" -mtime +30 -delete
```

### Rollback Procedures

If an update causes issues:

```bash
# 1. Revert to previous config
cp /backup/lemonade-config-TIMESTAMP.toml /etc/lemonade/production.toml

# 2. Revert binary if needed
sudo systemctl stop lemonade-lb
sudo cp /opt/lemonade/bin/lemonade.old /opt/lemonade/bin/lemonade
sudo systemctl start lemonade-lb

# 3. Verify service health
curl http://localhost:8080/health
```

## Security Considerations

### Network Security

- Run behind a firewall
- Use private networks for backend communication
- Consider TLS termination at proxy level (future feature)
- Implement rate limiting at edge

### Access Control

```bash
# Restrict file permissions
sudo chown -R lemonade:lemonade /opt/lemonade
sudo chmod 750 /opt/lemonade/bin/lemonade
sudo chmod 640 /etc/lemonade/production.toml
```

### Resource Limits

Prevent resource exhaustion:

```toml
[proxy]
max_connections = 100000            # Hard limit on concurrent connections

[runtime]
drain_timeout_millis = 30000        # Force close after 30s
```

System limits:

```bash
# /etc/security/limits.conf
lemonade soft nofile 100000
lemonade hard nofile 100000
lemonade soft nproc 4096
lemonade hard nproc 4096
```

## Performance Tuning

### TCP Tuning

```bash
# /etc/sysctl.conf
net.core.somaxconn = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_fin_timeout = 30
net.core.netdev_max_backlog = 65535
```

Apply:

```bash
sudo sysctl -p
```

### Load Balancer Tuning

```toml
[runtime]
# Reduce for faster hot-reload
config_watch_interval_millis = 1000

# Increase for more backends
metrics_cap = 100000
health_cap = 10000

# Adjust based on traffic patterns
drain_timeout_millis = 30000
```

### Strategy Selection

- **Round Robin**: Best for homogeneous backends
- **Least Connections**: Best for long-lived connections
- **Weighted Round Robin**: Best for heterogeneous backends
- **Fastest Response Time**: Best for backends with variable latency
- **Adaptive**: Best for production (combines multiple factors)

## Troubleshooting

### High CPU Usage

```bash
# Check for tight loops
perf top

# Profile with flamegraph
cargo flamegraph --bin lemonade
```

### Memory Leaks

```bash
# Check memory usage
top -p $(pgrep lemonade)

# Detailed memory analysis
valgrind --tool=massif target/release/lemonade lb
```

### Connection Issues

```bash
# Check connection counts
ss -s

# Check for TIME_WAIT connections
ss -tan state time-wait | wc -l

# Check backend connectivity
for addr in $(grep address config.toml | awk '{print $3}'); do
  echo "Testing $addr"
  nc -zv ${addr//\"/}
done
```

### Config Reload Not Working

```bash
# Check file watcher
sudo lsof | grep production.toml

# Check logs for errors
sudo journalctl -u lemonade-lb | grep -i config

# Verify config syntax
lemonade load-balancer --config production.toml --validate
```

## High Availability Setup

### Active-Passive

Use keepalived or similar for VIP failover:

```
     VIP: 10.0.0.100
          |
    ┌─────┴──────┐
    │            │
  Active      Standby
(lemonade-1) (lemonade-2)
    │            │
    └─────┬──────┘
          │
    Backend Pool
```

### Active-Active

Use DNS round-robin or L4 load balancer:

```
    DNS: lb.example.com
          |
    ┌─────┼──────┐
    │     │      │
  LB-1  LB-2   LB-3
    │     │      │
    └─────┼──────┘
          │
    Backend Pool
```

## Capacity Planning

### Estimating Load

- Concurrent connections per LB instance: ~10,000-50,000
- Throughput per LB instance: ~1M requests/second (proxy mode)
- Memory usage: ~100-500MB base + ~10KB per connection
- CPU usage: ~5-20% idle, scales with request rate

### Scaling Guidelines

- **Vertical**: Increase to 4-8 cores, 4-16GB RAM
- **Horizontal**: Add LB instances behind L4 balancer
- **Backend**: Monitor backend health, add capacity proactively

## Production Best Practices

1. **Always use configuration files** (not environment variables) in production
2. **Enable hot-reload** for zero-downtime updates
3. **Monitor all backends** with frequent health checks
4. **Set appropriate timeouts** based on traffic patterns
5. **Use adaptive strategy** for best automatic optimization
6. **Configure graceful shutdown** with adequate drain timeout
7. **Test failure scenarios** regularly (chaos engineering)
8. **Keep configs in version control** for auditability
9. **Automate deployment** with CI/CD pipelines
10. **Monitor continuously** and set up alerting

## Resources

- [Systemd Documentation](https://www.freedesktop.org/software/systemd/man/)
- [Docker Documentation](https://docs.docker.com/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [TCP Tuning Guide](https://www.kernel.org/doc/Documentation/networking/ip-sysctl.txt)
