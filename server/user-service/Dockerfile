ARG RUST_VERSION=1.91

FROM rust:${RUST_VERSION}-slim-trixie AS chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked --version ^0.1
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked --bin user-service \
    && strip target/release/user-service

FROM debian:trixie-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system app \
    && useradd --system --gid app --home-dir /app --create-home app

WORKDIR /app

COPY --from=builder /app/target/release/user-service /usr/local/bin/user-service
COPY --from=builder /app/migrations ./migrations

RUN chown -R app:app /app
USER app:app

EXPOSE 8082

ENTRYPOINT ["/usr/local/bin/user-service"]