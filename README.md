<div align="center">

<img src="thymos/Thymos-logo.PNG" alt="Thymos" width="128" height="128" />

# 𝚃𝙷𝚈𝙼𝙾𝚂
                                                   
**𝙰 𝚁𝚞𝚜𝚝 𝚛𝚞𝚗𝚝𝚒𝚖𝚎 𝚏𝚘𝚛 𝚐𝚘𝚟𝚎𝚛𝚗𝚎𝚍 𝙻𝙻𝙼 𝚊𝚐𝚎𝚗𝚝𝚜..**

*𝘾𝙤𝙜𝙣𝙞𝙩𝙞𝙤𝙣 𝙥𝙧𝙤𝙥𝙤𝙨𝙚𝙨. 𝙍𝙪𝙣𝙩𝙞𝙢𝙚 𝙙𝙚𝙘𝙞𝙙𝙚𝙨. 𝙇𝙚𝙙𝙜𝙚𝙧 𝙧𝙚𝙢𝙚𝙢𝙗𝙚𝙧𝙨.*

[![status](https://img.shields.io/badge/status-pre--alpha-orange)](#status)
[![rust](https://img.shields.io/badge/rust-1.80%2B-orange)](https://www.rust-lang.org)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![tests](https://img.shields.io/badge/tests-93%20passing-brightgreen)](#tests)

[Quickstart](#quickstart) · [Architecture](#architecture) · [Docs](thymos/docs) · [Examples](#examples) · [Status](#status)

</div>

---

## What it is

Most agent frameworks are a prompt loop plus a few tool calls. **Thymos treats a language model as a bounded proposer.** It emits typed **Intents**, the runtime compiles them into **Proposals**, a policy engine decides under a signed **Capability Writ**, and approved actions become **Commits** appended to an append-only, content-addressed **Trajectory Ledger**.

No Writ — no commit. No commit — no side effect. Every step is auditable, bounded, and replayable.

```
Cognition (LLM)          Runtime            Ledger
    │                       │                  │
    │──── Intent ───────────▶                  │
    │                       │── compile ──────▶│
    │                       │   Proposal       │
    │                       │── evaluate ─────▶│
    │                       │   (policy)       │
    │                       │── commit ───────▶│
    │                       │   Entry          │
    │◀── Observation ───────│                  │
```

---

## Why

A loop of prompts plus JSON tool calls has **no authorization layer** and **no durable record**. You can't prove what the agent tried to do, why the runtime let it, or what the world looked like at the moment it did. Thymos fixes both:

| Problem                                   | Thymos answer                                                            |
| ----------------------------------------- | ------------------------------------------------------------------------ |
| Agents mutate the world without authority | Every action is gated by an **Ed25519-signed Writ** with tool/budget/effect scopes |
| Decisions are opaque                      | **Policy engine** records allow / deny / suspend as first-class ledger entries     |
| History is a text log                     | **Content-addressed** (BLAKE3), **parent-chained** ledger on SQLite                |
| State drifts from reality                 | `World` is a **projection** — you replay commits to reconstruct it                 |
| Tool calls bypass contracts               | Tools declare schema, effect class, risk class, pre- and postconditions           |

---

## Quickstart

No API key required. Mock cognition ships with the runtime.

```bash
git clone https://github.com/gryszzz/THYMOS.git
cd THYMOS/thymos
cargo run --example hello_triad -p thymos-runtime
```

Expected output: a signed Writ, four Intents (one rejected by policy), and a final world projection — all in 170 lines of printed trajectory.

### Run the HTTP server

```bash
cargo run -p thymos-server
# server on http://localhost:3001
curl -X POST http://localhost:3001/runs \
  -H 'content-type: application/json' \
  -d '{"task": "Set greeting to hello and read it back",
       "cognition": {"provider": "mock"}}'
```

### Live trajectory viewer (Next.js)

```bash
# from repo root
npm install
npm run dev
# open http://localhost:3000/runs
```

### With a real model

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run -p thymos-server
# or
OPENAI_API_KEY=sk-... cargo run -p thymos-server
```

### With Docker

```bash
cd thymos
docker compose up --build
```

---

## Architecture

Eleven Rust crates, each with a single responsibility. State flows one direction: the ledger is the source of truth, every other surface is a projection or a decision.

```
thymos-core        types, hashing (BLAKE3), signing (Ed25519), invariants
thymos-ledger      append-only content-addressed store (SQLite; Postgres stub)
thymos-policy      policy trait + stock policies (writ authority, threshold)
thymos-tools       ToolContract trait + stock tools (kv, memory, shell, http, delegate, MCP)
thymos-compiler    Intent → Proposal compiler (writ resolution, policy eval, budget)
thymos-cognition   LLM adapters (Anthropic, OpenAI, local OpenAI-compat, mock)
thymos-runtime     agent loop, world projection, async streaming, approval channel
thymos-server      axum HTTP facade, SSE streams, JWT + API gateway, run store
thymos-marketplace tool-package registry
thymos-cli         command-line client
thymos-client      typed async Rust SDK
```

Full architecture notes: [`thymos/docs/architecture.md`](thymos/docs/architecture.md).
API reference: [`thymos/docs/api-reference.md`](thymos/docs/api-reference.md).
Getting started: [`thymos/docs/getting-started.md`](thymos/docs/getting-started.md).

---

## Core concepts

### Writ — signed capability

```rust
let writ = Writ::sign(WritBody {
    issuer: "root".into(),
    subject: "agent-1".into(),
    tool_scopes: vec![ToolPattern::exact("kv_*")],
    budget: Budget {
        tokens: 10_000,
        tool_calls: 10,
        wall_clock_ms: 60_000,
        usd_millicents: 0,
    },
    effect_ceiling: EffectCeiling::read_write_local(),
    time_window: TimeWindow { not_before: 0, expires_at: u64::MAX },
    delegation: DelegationBounds { max_depth: 1, may_subdivide: false },
    // ...
}, &root_key)?;
```

Child Writs can only be **subsets** of their parent. Delegation is bounded by construction.

### Intent → Proposal → Commit

```rust
let intent = Intent::new(IntentBody {
    kind: IntentKind::Act,
    target: "kv_set".into(),
    args: serde_json::json!({ "key": "foo", "value": "bar" }),
    rationale: "initialize greeting".into(),
    // ...
})?;

match run.submit(intent, &writ)? {
    Step::Committed(commit_id) => { /* ledger entry written */ }
    Step::Rejected(reason)      => { /* policy denied */ }
    Step::Suspended { channel, reason } => { /* awaiting human approval */ }
    Step::Delegated { child_trajectory_id, final_answer } => { /* subtask spawned */ }
}
```

### Trajectory — append-only ledger

Entry kinds: `root`, `commit`, `rejection`, `pending_approval`, `delegation`, `branch`. Every entry is BLAKE3-hashed; every commit carries its parent. World state is reconstructed by folding the chain — it is never stored separately.

---

## Cognition providers

| Provider   | Config                                                                     | Requires            |
| ---------- | -------------------------------------------------------------------------- | ------------------- |
| Anthropic  | `{"provider": "anthropic", "model": "claude-opus-4-7"}`                    | `ANTHROPIC_API_KEY` |
| OpenAI     | `{"provider": "openai", "model": "gpt-4o"}`                                | `OPENAI_API_KEY`    |
| Local      | `{"provider": "local", "base_url": "http://localhost:11434/v1"}`           | Ollama / vLLM / LM Studio |
| Mock       | `{"provider": "mock"}`                                                     | nothing             |

The Anthropic adapter has prompt-cache breakpoints, extended-thinking support (Opus 4.7+), internal-history trimming that preserves `tool_use` / `tool_result` pairing, transient-error retry with exponential backoff, and full `stop_reason` handling (`end_turn`, `tool_use`, `max_tokens`, `stop_sequence`, `pause_turn`, `refusal`).

---

## Examples

```bash
# Trivial KV triad — policy, budget, writ authority
cargo run --example hello_triad -p thymos-runtime

# Real model driving the triad (requires ANTHROPIC_API_KEY)
cargo run --example hello_llm_triad -p thymos-runtime
```

---

## Status

**Pre-alpha. Phase 1 reference implementation.**

Implemented and under test:

- [x] Core types: Writ, Intent, Proposal, Commit, Delta, World
- [x] Ed25519 Writ signing + verification
- [x] SQLite-backed content-addressed ledger with parent chain
- [x] Policy engine (allow / deny / suspend-for-approval)
- [x] ToolContract trait + stock tools (kv, memory, shell w/ sandbox, http, delegate, MCP bridge, manifest)
- [x] Intent → Proposal compiler with writ binding, policy eval, budget check
- [x] Sync + async agent loop with approval callback and cancellation
- [x] Multi-provider cognition (Anthropic, OpenAI, local, mock)
- [x] axum HTTP server with SSE token + ledger streaming
- [x] JWT (HS256, iss/aud) + API-gateway key auth with tenant isolation
- [x] SQLite-backed run persistence and resume
- [x] CLI + Rust client SDK
- [x] Marketplace (in-memory)
- [x] OpenTelemetry OTLP tracing
- [x] Docker + docker-compose (with optional Postgres + Jaeger profiles)

Not yet:

- [ ] Deterministic replay with compiler-version pinning
- [ ] Shadow branches for counterfactual execution
- [ ] Stratified memory
- [ ] Web-based trajectory debugger (beyond the current `/runs` viewer)
- [ ] Postgres ledger parity with SQLite
- [ ] Signed marketplace packages
- [ ] Distributed rate limiting

### Tests

```
93 passing across the workspace
├─ 22 server E2E (JWT, gateway, audit, runs, delegations, tenant isolation)
├─ 12 JWT auth E2E
├─ 12 Opus 4.6 → 4.7 migration regression harness
├─ ledger, policy, compiler, runtime unit suites
└─ cognition adapters (mock + anthropic + streaming)
```

```bash
cargo test --workspace
```

---

## Repo layout

```
THYMOS/
├── README.md                 ← you are here
├── src/                      ← Next.js marketing site + trajectory viewer (/runs)
├── public/                   ← site assets (thymos-mark.png)
├── docs/                     ← commercial notes
└── thymos/                   ← the Rust framework
    ├── Cargo.toml            ← workspace
    ├── crates/               ← 11 crates
    ├── docs/                 ← architecture, api-reference, getting-started
    ├── Dockerfile            ← multi-stage server image
    └── docker-compose.yml    ← server + optional otel/jaeger/postgres
```

---

## Contributing

Thymos is pre-alpha and moves fast. The best contribution right now is:

1. Run `cargo run --example hello_triad -p thymos-runtime` and open an issue with anything that felt unclear.
2. Try the server + `/runs` viewer and report latency, stream, or policy surprises.
3. Write a tool. Implement `ToolContract`, check in pre/postconditions, open a PR against `thymos-tools`.

Keep PRs small, typed, and backed by tests. The runtime is the kind of code where correctness matters more than throughput.

---

## License

Apache-2.0. See [LICENSE](LICENSE).

Thymos is a project of **Exponet Labs**.
