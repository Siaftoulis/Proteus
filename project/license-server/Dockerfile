FROM rust:1.81-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p license-server --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/license-server /app/license-server
EXPOSE 3000
CMD ["/app/license-server"]