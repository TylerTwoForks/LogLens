FROM rust:1.93-slim-bookworm AS builder

ARG PROFILE=dev

WORKDIR /workspace
COPY . .
RUN if [ "$PROFILE" = "release" ]; then \
      cargo build --release -p loglens-api && \
      cp target/release/loglens-api /loglens-api; \
    else \
      cargo build -p loglens-api && \
      cp target/debug/loglens-api /loglens-api; \
    fi

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /loglens-api /usr/local/bin/loglens-api

EXPOSE 8080
CMD ["loglens-api", "serve"]
