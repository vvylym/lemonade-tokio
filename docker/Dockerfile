# Unified Dockerfile for Lemonade (load balancer + workers) using cargo-chef

# Stage 1: Builder - build the application
FROM rust:1.91-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release --bin lemonade

# Stage 2: Runtime - minimal final image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and configs
COPY --from=builder /app/target/release/lemonade /usr/local/bin/lemonade
COPY config /etc/lemonade

# Create non-root user
RUN useradd -r -s /bin/false lemonade && \
    chown -R lemonade:lemonade /app /etc/lemonade

USER lemonade

ENTRYPOINT ["lemonade"]
# Default command can be overridden per service
CMD ["--help"]

