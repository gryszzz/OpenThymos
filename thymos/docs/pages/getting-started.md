---
layout: default
title: Getting Started
---

# Getting Started

## Run locally

```bash
cargo run -p thymos-server
```

## Production-shaped local run

```bash
THYMOS_RUNTIME_MODE=production \
THYMOS_LEDGER_PATH=thymos-ledger.db \
THYMOS_DB_PATH=thymos-runs.db \
THYMOS_GATEWAY_DB_PATH=thymos-gateway.db \
THYMOS_MARKETPLACE_DB_PATH=thymos-marketplace.db \
THYMOS_TOOL_FABRIC=worker \
THYMOS_WORKER_BIN=$PWD/target/release/thymos-worker \
cargo run -p thymos-server
```

## Build the worker

```bash
cargo build --release -p thymos-worker
```

## Why worker mode matters

Worker mode pushes `shell` and `http` execution behind the secure tool fabric
instead of running them directly inside the main runtime process.

For the fuller local guide, see `docs/getting-started.md` in the repo.
