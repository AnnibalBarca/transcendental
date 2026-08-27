# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS builder

WORKDIR /workspace

COPY . .

RUN --mount=type=cache,id=room-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-target,target=/workspace/target \
    cargo build --release --bin Room \
    && cp /workspace/target/release/Room /usr/local/bin/Room

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/Room /usr/local/bin/Room
LABEL description="Transcendence Room API Service"

CMD ["Room"]
