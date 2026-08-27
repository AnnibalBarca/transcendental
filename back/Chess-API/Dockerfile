# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS builder

WORKDIR /workspace

COPY . .

RUN --mount=type=cache,id=chess-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-target,target=/workspace/target \
    cargo build --release --bin chess \
    && cp /workspace/target/release/chess /usr/local/bin/chess

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/chess /usr/local/bin/chess
LABEL description="Transcendence Chess Service - Redis worker"

CMD ["chess"]
