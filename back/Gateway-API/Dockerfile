# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS builder

WORKDIR /workspace

COPY . .

RUN --mount=type=cache,id=gateway-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-target,target=/workspace/target \
    cargo build --release --bin Gateway \
    && cp /workspace/target/release/Gateway /usr/local/bin/Gateway

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/Gateway /usr/local/bin/Gateway
EXPOSE 8080 9091

CMD ["Gateway"]
