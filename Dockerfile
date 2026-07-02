FROM rust:1.96.1-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y cmake clang libclang-dev && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release --bin pingora-example

FROM debian:trixie-20260623-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pingora-example /app/
EXPOSE 9999
CMD ["/app/pingora-example"]
