# Build stage
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/ktunbot
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn lib() {}" > src/lib.rs
RUN cargo build --release || true
RUN rm -rf src

COPY src ./src
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl python3 && \
    rm -rf /var/lib/apt/lists/*

RUN groupadd -g 10001 ktunbot && \
    useradd -u 10001 -g ktunbot -s /bin/bash -m ktunbot

WORKDIR /app
RUN chown -R ktunbot:ktunbot /app

COPY --from=builder /usr/src/ktunbot/target/release/ktunbot /usr/local/bin/ktunbot

USER ktunbot

ENV RUST_LOG=info
CMD ["ktunbot"]
