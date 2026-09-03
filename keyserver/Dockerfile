# syntax=docker/dockerfile:1
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/keyserver /usr/local/bin/keyserver
EXPOSE 3000
CMD ["keyserver"]
