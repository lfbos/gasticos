# Build stage
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml ./
COPY crates ./crates

# Build release binaries
RUN cargo build --release

# API runtime stage
FROM debian:bookworm-slim AS api

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api /usr/local/bin/api

EXPOSE 8080

CMD ["api"]

# Worker runtime stage
FROM debian:bookworm-slim AS worker

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/worker /usr/local/bin/worker

CMD ["worker"]

# Development stage (for docker-compose dev)
FROM rust:1.85-slim-bookworm AS dev

WORKDIR /app

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-watch for hot reloading
RUN cargo install cargo-watch

EXPOSE 8080

CMD ["cargo", "watch", "-x", "run -p api"]
