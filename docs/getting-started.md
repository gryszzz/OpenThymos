---
layout: default
title: Get Started
eyebrow: 5 minutes · no keys required
subtitle: Boot the runtime, run an agent, point it at a local model. In that order.
permalink: /getting-started/
---

## 1 · Boot the runtime

```bash
git clone https://github.com/gryszzz/THYMOS.git
cd THYMOS/thymos
cargo run -p thymos-server
# → listening on http://localhost:3001
# → mock cognition (no key required)
```

The server boots with an in-memory ledger and a mock cognition. No API key
needed. Hit `http://localhost:3001/health` to confirm.

## 2 · Run your first agent

```bash
# In a second terminal
cargo run -p thymos-cli -- run "Set greeting to hello and read it back" \
    --provider mock
```

The mock cognition deterministically runs the IPC Triad end-to-end. You
will see a trajectory id, stream of intents, and a final answer.

## 3 · Drive it with a real model

Pick one. All three work with the same CLI, same ledger, same runtime.

### Anthropic (Opus 4.7)

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run -p thymos-server
cargo run -p thymos-cli -- run "..." --provider anthropic --model opus
```

### OpenAI (GPT-4o family)

```bash
OPENAI_API_KEY=sk-... cargo run -p thymos-server
cargo run -p thymos-cli -- run "..." --provider openai --model gpt-4o
```

### LM Studio (local, free, private)

Start LM Studio's local server (default `http://localhost:1234/v1`). Load a
tool-capable model — we recommend Qwen 2.5 Coder or a Llama 3.1 Instruct.

```bash
# No key needed — LM Studio ignores the Authorization header
OPENAI_BASE_URL=http://localhost:1234/v1 \
OPENAI_API_KEY=lm-studio \
cargo run -p thymos-server

cargo run -p thymos-cli -- run "..." \
    --provider local \
    --model qwen2.5-coder-32b-instruct
```

### Ollama

```bash
OPENAI_BASE_URL=http://localhost:11434/v1 \
OPENAI_API_KEY=ollama \
cargo run -p thymos-server

cargo run -p thymos-cli -- run "..." \
    --provider local \
    --model llama3.1:8b
```

## 4 · Production-shaped run

File-backed ledger, worker-sandboxed shell, persistent run store.

```bash
cargo build --release -p thymos-worker

THYMOS_RUNTIME_MODE=production \
THYMOS_LEDGER_PATH=thymos-ledger.db \
THYMOS_DB_PATH=thymos-runs.db \
THYMOS_GATEWAY_DB_PATH=thymos-gateway.db \
THYMOS_MARKETPLACE_DB_PATH=thymos-marketplace.db \
THYMOS_TOOL_FABRIC=worker \
THYMOS_WORKER_BIN=$PWD/target/release/thymos-worker \
cargo run -p thymos-server
```

## 5 · What to read next

- [Coding Agent]({{ '/coding-agent' | relative_url }}) — the flagship surface.
- [Architecture]({{ '/architecture' | relative_url }}) — IPC Triad, planes, invariants.
- [Secure Tool Fabric]({{ '/secure-tool-fabric' | relative_url }}) — how risky tools run.
- [API Reference]({{ '/api-reference' | relative_url }}) — HTTP surface.
- [Roadmap]({{ '/roadmap' | relative_url }}) — what's next.
