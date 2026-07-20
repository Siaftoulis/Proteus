FROM rust:1.81-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p auth-server --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/auth-server /app/auth-server
EXPOSE 3001
CMD ["/app/auth-server"]