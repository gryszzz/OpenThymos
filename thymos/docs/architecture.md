# Thymos Architecture

Thymos is a **governed-cognition runtime** for AI agents. It enforces the
**IPC Triad** — Intent, Proposal, Commit — so that every action an agent takes
is proposed, evaluated against policy, and recorded in an immutable ledger
before it takes effect.

## Core Concepts

### IPC Triad

```
Cognition (LLM)          Runtime            Ledger
    |                       |                  |
    |--- Intent ----------->|                  |
    |                       |-- compile ------>|
    |                       |   Proposal       |
    |                       |-- evaluate ----->|
    |                       |   (policy)       |
    |                       |-- commit ------->|
    |                       |   Entry          |
    |<-- Observation -------|                  |
```

1. **Intent**: The cognition layer (LLM) declares what it wants to do — which
   tool to call, with what arguments.
2. **Proposal**: The compiler converts the intent into a structured proposal
   containing a delta (world-state changes) and an observation (tool output).
3. **Commit**: If policy approves, the proposal becomes a commit — an immutable
   ledger entry. If policy rejects, a rejection entry is recorded instead.

### Writs (Capability Tokens)

Every agent run is governed by a **Writ** — a signed capability token that
specifies:

- **Tool scopes**: which tools the agent may use (glob patterns)
- **Budget**: token/tool-call/time/cost limits
- **Effect ceiling**: read-only, read-write-local, or read-write-network
- **Time window**: validity period
- **Delegation bounds**: max depth and whether the agent may subdivide

Writs are signed with Ed25519. Child writs can only be subsets of their parent.

### Ledger

The ledger is append-only, content-addressed (BLAKE3), and parent-chained.
Entry kinds:

| Kind | Description |
|------|-------------|
| `root` | First entry in a trajectory |
| `commit` | Approved action with delta + observations |
| `rejection` | Policy denied the proposal |
| `pending_approval` | Waiting for human approval |
| `delegation` | Child trajectory spawned |
| `branch` | Forked from another trajectory |

Backends: SQLite (default, zero-config) or Postgres (multi-node).

### World Projection

The current state of the world is computed by replaying commits from the
ledger. Resources are versioned and typed (kind + id). This makes the system
deterministic and auditable — you can always reconstruct what the agent saw at
any point.

## Crate Map

```
thymos/
├── thymos-core        Core types: ContentHash, Writ, Budget, Delta, Commit
├── thymos-ledger      Append-only ledger (SQLite / Postgres backends)
├── thymos-policy      Policy engine (trait + stock policies)
├── thymos-tools       Tool registry, stock tools (shell, KV, HTTP, delegate)
├── thymos-compiler    Intent → Proposal compiler
├── thymos-cognition   LLM adapters (Anthropic, OpenAI, Local, Mock)
├── thymos-runtime     Agent loop, world projection, async streaming
├── thymos-server      HTTP API (axum), auth, SSE streaming, run persistence
├── thymos-marketplace Tool package registry
├── thymos-cli         Command-line client
└── thymos-client      Rust HTTP client SDK
```

## Data Flow (Server)

```
HTTP POST /runs {task, cognition}
  │
  ├─ Mint Writ (server-side)
  ├─ Build Cognition (Anthropic/OpenAI/Mock)
  ├─ Spawn async agent loop
  │     │
  │     ├─ cognition.next_action(history) → Intent
  │     ├─ compiler.compile(intent) → Proposal
  │     ├─ policy.evaluate(proposal, writ) → Allow/Deny/Suspend
  │     │     ├─ Allow → commit to ledger, observe, continue
  │     │     ├─ Deny → record rejection, continue
  │     │     └─ Suspend → park oneshot, wait for HTTP approval
  │     └─ loop until: CognitionDone / MaxSteps / BudgetExhausted
  │
  ├─ SSE /runs/:id/stream  (cognition tokens in real-time)
  ├─ SSE /runs/:id/events  (ledger entries)
  └─ GET /runs/:id         (final summary)
```

## Authentication

Two layers, both optional:

1. **API Gateway**: key-based auth with per-key rate limiting and tenant
   isolation. Configured via `THYMOS_API_KEYS` env var.
2. **JWT Auth**: HS256 tokens with claims (sub, tenant_id, roles). Configured
   via `THYMOS_JWT_SECRET` env var.

Both can be bypassed for development. The `/health` endpoint always skips auth.

## Persistence

- **Run Store**: SQLite database (`THYMOS_DB_PATH`) tracking run metadata
  (task, status, summary). Runs are restored into memory on server startup.
- **Ledger**: SQLite or Postgres. The ledger is the source of truth for all
  agent actions and can be queried via the audit API.
