# Build
FROM rust:1.88 as builder
WORKDIR /app

# Pour le submodule dans le build docker: on copie tout le repo incluant contracts/
COPY . .
RUN cargo build --release

# Run
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/guard-ai-backend /app/guard-ai-backend

ENV PORT=8080
EXPOSE 8080
CMD ["/app/guard-ai-backend"]
