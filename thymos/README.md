<div align="center">

<img src="Thymos-logo.PNG" alt="Thymos" width="128" height="128" />

# Thymos — Rust Framework

**The runtime for governed cognition.**

*Cognition proposes. Runtime decides. Ledger remembers.*

</div>

---

Thymos treats a language model as a bounded **proposer** against a ledgered, policy-governed runtime. The model emits typed **Intents**. The runtime compiles them into **Proposals**, evaluates policy under a signed **Capability Writ**, stages effects through typed **Tool Contracts**, and appends signed, content-addressed **Commits** to an append-only **Trajectory Ledger**.

This directory is the **Phase 1 reference implementation** in Rust — 11 crates, 93 passing tests.

## Quickstart

```bash
cargo run --example hello_triad -p thymos-runtime
```

Signs a Writ, submits four Intents (one rejected by policy), prints the full trajectory + world projection. No API key required.

```bash
# HTTP server (mock cognition, no key)
cargo run -p thymos-server
curl http://localhost:3001/health

# With a real model
ANTHROPIC_API_KEY=sk-ant-... cargo run -p thymos-server
OPENAI_API_KEY=sk-...       cargo run -p thymos-server

# CLI
cargo run -p thymos-cli -- run "Set greeting to hello" --provider mock

# Docker
docker compose up --build
```

See [`docs/getting-started.md`](docs/getting-started.md) for a full tour.

## Architecture

```
Cognition (LLM, or anything implementing the Cognition trait)
      │  Intent
      ▼
Compiler ── resolve writ, typecheck, policy eval, budget check ───▶ Proposal
      │
      ▼
Runtime ── stage, invoke tool contract, verify postconditions ────▶ Commit
      │
      ▼
Ledger (SQLite, append-only, content-addressed via BLAKE3, parent-chained)
```

See [`docs/architecture.md`](docs/architecture.md).

## Crates

| Crate              | Role                                                                 |
| ------------------ | -------------------------------------------------------------------- |
| `thymos-core`      | Types, hashing (BLAKE3), signing (Ed25519), invariants               |
| `thymos-ledger`    | Append-only content-addressed store (SQLite; Postgres stub)          |
| `thymos-policy`    | Policy trait + stock policies                                        |
| `thymos-tools`     | `ToolContract` trait + stock tools (kv, memory, shell, http, delegate, MCP, manifest) |
| `thymos-compiler`  | Intent → Proposal compiler                                           |
| `thymos-cognition` | LLM adapters (Anthropic, OpenAI, local OpenAI-compat, mock)          |
| `thymos-runtime`   | Agent loop, world projection, async streaming, approval channel     |
| `thymos-server`    | axum HTTP facade, SSE, JWT + gateway auth, run store                |
| `thymos-marketplace` | In-memory tool-package registry                                    |
| `thymos-cli`       | Command-line client                                                  |
| `thymos-client`    | Typed async Rust SDK                                                 |

## Status

Pre-alpha. The IPC Triad, signed Writs, ledger, policy, tool contracts, sync + async agent loops, HTTP server with SSE streaming, JWT + API-gateway auth, and tenant isolation are implemented and tested. See the top-level [README](../README.md#status) for the full checklist.

## Tests

```bash
cargo test --workspace
```

93 tests currently pass, including a 12-test cognition migration regression harness (Opus 4.6 → 4.7), a 12-test JWT auth E2E suite, and a 22-test server E2E suite with tenant-isolation checks.

## License

Apache-2.0.
