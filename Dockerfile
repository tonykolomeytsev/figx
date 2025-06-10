# Builder
FROM rust:1.87.0-slim AS builder
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY app ./app
COPY crates ./crates
RUN cargo build --release

# Final image
FROM ghcr.io/linuxcontainers/debian-slim:12.5
COPY --from=builder /app/target/release/figx /usr/local/bin/figx
ENTRYPOINT ["/bin/sh", "-c"]
