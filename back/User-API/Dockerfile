# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS builder

WORKDIR /workspace

COPY . .

RUN --mount=type=cache,id=user-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-target,target=/workspace/target \
    cargo build --release --bin User \
    && cp /workspace/target/release/User /usr/local/bin/User

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/User /usr/local/bin/User
LABEL description="Transcendence User Service - Redis worker"

CMD ["User"]
