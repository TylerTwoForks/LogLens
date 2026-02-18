FROM rust:1.93-slim-bookworm AS builder

WORKDIR /workspace
COPY . .
RUN cargo build --release -p loglens-api

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /workspace/target/release/loglens-api /usr/local/bin/loglens-api

EXPOSE 8080
CMD ["loglens-api", "serve"]
