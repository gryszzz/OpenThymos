<div align="center">

<img src="thymos/Thymos-logo.PNG" alt="Thymos" width="128" height="128" />

# 𝚃𝙷𝚈𝙼𝙾𝚂
                                                   
**𝚃𝚑𝚎 𝚐𝚘𝚟𝚎𝚛𝚗𝚎𝚍 𝚛𝚞𝚗𝚝𝚒𝚖𝚎 𝚏𝚘𝚛 𝚌𝚘𝚍𝚎 𝚊𝚐𝚎𝚗𝚝𝚜.**

*𝘾𝙤𝙜𝙣𝙞𝙩𝙞𝙤𝙣 𝙥𝙧𝙤𝙥𝙤𝙨𝙚𝙨. 𝙍𝙪𝙣𝙩𝙞𝙢𝙚 𝙙𝙚𝙘𝙞𝙙𝙚𝙨. 𝙇𝙚𝙙𝙜𝙚𝙧 𝙧𝙚𝙢𝙚𝙢𝙗𝙚𝙧𝙨.*

[![status](https://img.shields.io/badge/status-pre--alpha-orange)](#status)
[![rust](https://img.shields.io/badge/rust-1.80%2B-orange)](https://www.rust-lang.org)
[![license](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![tests](https://img.shields.io/badge/tests-passing-brightgreen)](#tests)

[Site](https://gryszzz.github.io/THYMOS/) · [Quickstart](#quickstart) · [Coding agent](#coding-agent) · [Architecture](#architecture) · [Docs](docs) · [Status](#status)

</div>

---

## What it is

Most agent frameworks are a prompt loop plus a few tool calls. **Thymos treats a language model as a bounded proposer.** It emits typed **Intents**, the runtime compiles them into **Proposals**, a policy engine decides under a signed **Capability Writ**, and approved actions become **Commits** appended to an append-only, content-addressed **Trajectory Ledger**.

No Writ — no commit. No commit — no side effect. Every step is auditable, bounded, and replayable.

The first surface built on top of this is a **coding agent**: read files, patch files, list, map, grep, run tests — every call typed, path-confined, ledgered, and model-agnostic across Anthropic, OpenAI, LM Studio, Hugging Face Router, Ollama, and any OpenAI-compatible local backend.

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

## Use cases

Concrete things people build on Thymos today:

- **Code-refactor bot with audit trail.** Point the coding agent at a monorepo; every file read, patch, and test run becomes a ledger entry. Review the full trajectory before merging — including the proposals the policy engine *denied*.
- **On-call runbook automation.** An agent reads logs and proposes remediation. The Writ scopes it to read-only on prod and write on staging; policy suspends risky actions for human approval. Every attempted action is preserved even if blocked — the ledger is your postmortem.
- **Compliance-sensitive workflows.** Tamper-evident BLAKE3 hash chain + Ed25519-signed Writs mean you can *prove* what an agent did, under what authority, against what world state. Works for PII redaction pipelines, regulated financial tasks, and anything a SOC-2 auditor asks about.
- **Local-only dev loop.** Pair the `lmstudio` provider with a local Qwen / DeepSeek Coder model. Zero cost, zero data egress, full ledger — good for sensitive repos where you can't send source to a hosted model.
- **Multi-model A/B.** Run the same task twice via shadow branches — once with Claude, once with Qwen via HF Router — then diff the ledgers to compare approach, cost, and outcome side-by-side.

---

## Quickstart

Three steps. Mock cognition ships with the runtime, so step 1 needs no API key.

### 1. Smoke test (no model, no config)

```bash
git clone https://github.com/gryszzz/THYMOS.git
cd THYMOS/thymos
cargo run --example hello_triad -p thymos-runtime
```

Prints a signed Writ, four Intents (one denied by policy), and the final world projection.

### 2. Run the server with a real model

Copy `.env.example` to `.env` and fill in **one** of the provider blocks below. The server auto-loads `.env` — no need to `export` anything.

```bash
cp .env.example .env
# edit .env, then:
cargo run -p thymos-server           # listens on :3001
```

| Provider      | What to put in `.env`                                          |
| ------------- | -------------------------------------------------------------- |
| Anthropic     | `ANTHROPIC_API_KEY=sk-ant-...`                                 |
| OpenAI        | `OPENAI_API_KEY=sk-...`                                        |
| Hugging Face  | `HF_TOKEN=hf_...` *(needs "Inference Providers" scope)*        |
| LM Studio     | nothing — defaults to `http://localhost:1234/v1`               |
| Ollama / vLLM | `OPENAI_BASE_URL=http://localhost:11434/v1` (use `--provider local`) |

Then kick off a run (picks the provider per request, so one server can serve all of them):

```bash
curl -X POST http://localhost:3001/runs \
  -H 'content-type: application/json' \
  -d '{"task":"Set greeting to hello and read it back",
       "cognition":{"provider":"huggingface"}}'
```

Or via the CLI:

```bash
cargo run -p thymos-cli -- run "explain the ledger module" --provider huggingface
```

### 3. (optional) Live trajectory viewer

```bash
npm install && npm run dev    # from repo root
# open http://localhost:3000/runs
```

### With Docker

```bash
cd thymos
docker compose up --build     # image carries full OCI labels (title, version, revision, source)
```

---

## Coding agent

The runtime ships with a typed coding-tool surface. Each tool is path-confined
to the configured allowed roots, returns a structured observation, and runs
under the same writ/policy/ledger pipeline as every other Thymos action.

| Tool          | Effect    | Risk    | What it does                                                      |
| ------------- | --------- | ------- | ----------------------------------------------------------------- |
| `repo_map`    | Read      | Low     | Top-level layout, build markers, Cargo workspace members          |
| `list_files`  | Read      | Low     | Bounded directory walk (skips `target`, `node_modules`, `.git`)   |
| `fs_read`     | Read      | Low     | File read with optional line range and a hard byte cap            |
| `grep`        | Read      | Low     | Substring scan with optional extension filter                     |
| `fs_patch`    | Write     | Medium  | `write` (full overwrite) or `replace` (unique-anchor substitute)  |
| `test_run`    | External  | Medium  | Auto-detects `cargo` / `npm` / `pytest` / `go` and runs the suite |

Drive it through the HTTP server with whichever model you prefer:

```bash
# LM Studio (free, local — just load a model in LM Studio and start its server)
curl -X POST http://localhost:3001/runs \
  -H 'content-type: application/json' \
  -d '{
    "task": "Add a retry helper to crates/thymos-cognition/src/openai.rs and a unit test.",
    "tool_scopes": ["repo_map","fs_read","fs_patch","grep","test_run"],
    "cognition": { "provider": "lmstudio" }
  }'
```

The `replace` patch mode rejects anchors that appear zero or more than once,
so the model can't quietly clobber unrelated code. Every patch and every test
run becomes a ledger commit you can replay or audit.

---

## Architecture

Twelve Rust crates, each with a single responsibility. State flows one direction: the ledger is the source of truth, every other surface is a projection or a decision.

```
thymos-core        types, hashing (BLAKE3), signing (Ed25519), invariants
thymos-ledger      append-only content-addressed store (SQLite; Postgres stub)
thymos-policy      policy trait + stock policies (writ authority, threshold)
thymos-tools       ToolContract trait + stock tools (kv, memory, shell, http, delegate, MCP, coding)
thymos-compiler    Intent → Proposal compiler (writ resolution, policy eval, budget)
thymos-cognition   LLM adapters (Anthropic, OpenAI, LM Studio, Hugging Face, local, mock)
thymos-runtime     agent loop, world projection, async streaming, approval channel
thymos-server      axum HTTP facade, SSE streams, JWT + API gateway, run store
thymos-worker      subprocess sandbox for risky tool execution
thymos-marketplace tool-package registry
thymos-cli         command-line client
thymos-client      typed async Rust SDK
```

Full architecture notes: [`docs/architecture.md`](docs/architecture.md).
API reference: [`docs/api-reference.md`](docs/api-reference.md).
Getting started: [`docs/getting-started.md`](docs/getting-started.md).
Coding agent surface: [`docs/coding-agent.md`](docs/coding-agent.md).
Secure tool fabric: [`docs/secure-tool-fabric.md`](docs/secure-tool-fabric.md).

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

| Provider     | Config                                                                            | Requires                            |
| ------------ | --------------------------------------------------------------------------------- | ----------------------------------- |
| Anthropic    | `{"provider": "anthropic", "model": "claude-opus-4-7"}`                           | `ANTHROPIC_API_KEY`                 |
| OpenAI       | `{"provider": "openai", "model": "gpt-4o"}`                                       | `OPENAI_API_KEY`                    |
| LM Studio    | `{"provider": "lmstudio", "model": "qwen2.5-coder-32b-instruct"}`                 | LM Studio running on `:1234`        |
| Hugging Face | `{"provider": "huggingface", "model": "Qwen/Qwen2.5-Coder-32B-Instruct"}`         | `HF_TOKEN`                          |
| Local        | `{"provider": "local", "base_url": "http://localhost:11434/v1"}`                  | Ollama / vLLM / llama.cpp           |
| Mock         | `{"provider": "mock"}`                                                            | nothing                             |

Full provider matrix and recommended models: [`docs/providers.md`](docs/providers.md).

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
- [x] **Coding tool surface** (`fs_read`, `fs_patch`, `list_files`, `repo_map`, `grep`, `test_run`) with path confinement
- [x] Intent → Proposal compiler with writ binding, policy eval, budget check
- [x] Sync + async agent loop with approval callback and cancellation
- [x] Multi-provider cognition (Anthropic, OpenAI, LM Studio, Hugging Face Router, local, mock)
- [x] axum HTTP server with SSE token + ledger streaming
- [x] JWT (HS256, iss/aud) + API-gateway key auth with tenant isolation
- [x] SQLite-backed run persistence and resume
- [x] CLI + Rust client SDK
- [x] Marketplace (in-memory + SQLite) with Ed25519-signed packages and trusted-publisher gate
- [x] OpenTelemetry OTLP tracing
- [x] Docker + docker-compose (with optional Postgres + Jaeger profiles)
- [x] Deterministic replay with compiler-version pinning (`thymos_ledger::replay`)
- [x] Shadow branches for counterfactual execution (`POST /runs/:id/branch`)
- [x] Stratified memory (`working` / `episodic` / `semantic` strata)
- [x] Web trajectory debugger: scrubber + "branch from here" in `/runs/:id`
- [x] Postgres ledger parity with SQLite (including `query_entries` / `count_entries`)
- [x] Distributed rate limiting (SQL-backed shared counter across server nodes)

Not yet:

- [ ] Multi-region Postgres replication topology
- [ ] First-party dashboard for marketplace trust + package publishing

### Tests

```
~100 passing across the workspace
├─ 22 server E2E (JWT, gateway, audit, runs, delegations, tenant isolation)
├─ 12 JWT auth E2E
├─ 12 Opus 4.6 → 4.7 migration regression harness
├─  5 coding tool sandbox + patch tests
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
├── docs/                     ← Jekyll site published to gryszzz.github.io/THYMOS/
│   ├── index.md              ← landing
│   ├── coding-agent.md       ← typed coding-tool surface
│   ├── architecture.md       ← IPC triad, ledger, planes
│   ├── secure-tool-fabric.md ← worker boundary + capability profiles
│   ├── getting-started.md    ← five-step quickstart
│   ├── api-reference.md      ← HTTP surface
│   └── roadmap.md            ← Now / Next / After / Later
├── src/                      ← Next.js marketing site + /runs trajectory viewer
├── public/                   ← site assets (thymos-mark.png)
└── thymos/                   ← the Rust framework
    ├── Cargo.toml            ← workspace
    ├── crates/               ← 12 crates
    ├── .env.example          ← copy to .env for provider tokens (auto-loaded)
    ├── Dockerfile            ← multi-stage server image (OCI labels + build-arg revision/version)
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
