FROM rust:1.88 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/admin-backend /usr/local/bin/admin-backend
COPY migrations ./migrations
EXPOSE 8080
CMD ["admin-backend"]
