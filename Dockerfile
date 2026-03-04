FROM rust:1.85-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system app \
    && useradd --system --gid app --create-home app

COPY --from=builder /app/target/release/bench-server /usr/local/bin/bench-server

USER app
WORKDIR /home/app

ENV BENCH_HOST=0.0.0.0 \
    BENCH_PORT=8080 \
    RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["bench-server"]
