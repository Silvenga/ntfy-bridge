FROM rust:1-slim-trixie AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src

RUN touch src/main.rs \
    && cargo test \
    && cargo build --release

FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ntfy-bridge /usr/local/bin/

EXPOSE 8080

CMD ["ntfy-bridge"]
