# Multi-stage build for the Thymos server.
#
# Stage 1 caches cargo's registry + built dependencies as a dedicated layer
# so that source-only edits don't invalidate third-party compilation.

FROM rust:1.90-slim AS builder

WORKDIR /build

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

# Copy the workspace manifests first to prime the dependency cache.
COPY Cargo.toml Cargo.lock ./
COPY crates crates

# Real build. With BuildKit enabled this layer reuses the cached cargo
# registry and target directory whenever only source changes.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release -p thymos-server -p thymos-cli \
 && cp target/release/thymos-server /usr/local/bin/thymos-server \
 && cp target/release/thymos        /usr/local/bin/thymos

# Stage 2: minimal runtime.
# Use trixie (glibc 2.39) to match the rust:1.90-slim builder. Bookworm
# (glibc 2.36) cannot load binaries linked against 2.39.
FROM debian:trixie-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/thymos-server /usr/local/bin/thymos-server
COPY --from=builder /usr/local/bin/thymos        /usr/local/bin/thymos

RUN mkdir -p /data

ENV THYMOS_DB_PATH=/data/thymos-runs.db \
    RUST_LOG=info

EXPOSE 3001

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -fsS http://localhost:3001/health || exit 1

CMD ["thymos-server"]
