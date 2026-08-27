# syntax=docker/dockerfile:1

FROM rust:1.97.1-bookworm AS builder

WORKDIR /workspace

COPY . .

RUN --mount=type=cache,id=auth-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=ws-target,target=/workspace/target \
    cargo build --release --bin Auth \
    && cp /workspace/target/release/Auth /usr/local/bin/Auth

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/Auth /usr/local/bin/Auth
EXPOSE 9092
LABEL description="Transcendence Auth Service - Workers process requests via Redis"

CMD ["Auth"]
