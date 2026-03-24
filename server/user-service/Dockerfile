FROM rust:slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Create dummy main for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true

COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/user-service /usr/local/bin/user-service
COPY migrations ./migrations

EXPOSE 8082

CMD ["user-service"]
