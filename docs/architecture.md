---
layout: default
title: Architecture
eyebrow: System design
subtitle: Thymos treats the language model as a bounded proposer, not the authority. Everything else follows from that.
permalink: /architecture/
---

## The IPC Triad

```
Cognition  ─Intent─▶  Compiler  ─Proposal─▶  Runtime  ─Commit─▶  Ledger
                        ▲                      │
                        └──── Policy / Writ ───┘
```

1. **Intent.** Cognition declares a desired action — a tool target plus
   arguments plus rationale. No side effects.
2. **Proposal.** The compiler resolves the writ, typechecks the intent against
   the tool's schema, evaluates policy, and enforces budget / time-window
   constraints. Output is a `Staged`, `Rejected`, or `Suspended` proposal.
3. **Commit.** The runtime executes the tool, verifies post-conditions, and
   appends a content-addressed commit to the trajectory ledger.

## Planes

- **Execution kernel** — `thymos-core`, `thymos-runtime`, `thymos-ledger`.
  Hashing (BLAKE3), signing (Ed25519), world projection, append-only commits.
- **Cognition plane** — `thymos-cognition`. Pluggable `Cognition` trait with
  Anthropic, OpenAI, OpenAI-compatible (local), and mock adapters. Async
  streaming via `StreamingCognition`.
- **Tool plane** — `thymos-tools`. Typed `ToolContract`s: `fs_*`, `repo_map`,
  `grep`, `list_files`, `test_run`, `shell`, `http`, `kv_*`, `memory_*`,
  `delegate`, `manifest`, `mcp_bridge`.
- **Secure fabric** — `thymos-worker`. Subprocess boundary for risky tools
  with capability profiles and execution receipts.
- **Governance plane** — `thymos-policy` + `thymos-core::writ`. Signed
  capability writs, policy engine, approval channels.
- **Distribution plane** — `thymos-server`, `thymos-cli`, `thymos-client`,
  `thymos-marketplace`. axum HTTP facade, SSE streaming, JWT auth, API
  gateway, tenant isolation.

## Crates

| Crate                | Role                                                                 |
| -------------------- | -------------------------------------------------------------------- |
| `thymos-core`        | Types, hashing, signing, invariants                                  |
| `thymos-ledger`      | Append-only content-addressed store (SQLite; Postgres stub)          |
| `thymos-policy`      | Policy trait + stock policies                                        |
| `thymos-tools`       | `ToolContract` trait + stock coding / shell / http / mcp tools       |
| `thymos-worker`      | Subprocess worker boundary for the secure tool fabric                |
| `thymos-compiler`    | Intent → Proposal compiler                                           |
| `thymos-cognition`   | LLM adapters (Anthropic, OpenAI, local, mock)                        |
| `thymos-runtime`     | Agent loop, world projection, async streaming, approval channel     |
| `thymos-server`      | axum HTTP facade, SSE, JWT + gateway auth, run store                |
| `thymos-marketplace` | In-memory tool-package registry                                     |
| `thymos-cli`         | `thymos code run`, `thymos runs …`, in-process agent driver          |
| `thymos-client`      | Typed async Rust SDK                                                 |

## Invariants

- **Cognition never mutates state.** It only emits Intents.
- **Proposals are typed.** A rejected proposal is a first-class outcome fed
  back to cognition, not an exception.
- **Commits are hash-addressed.** The ledger chain is the single source of
  truth; replaying it yields byte-for-byte identical world state.
- **Writs are signed.** Forged or expired writs fail at the compiler before
  any tool runs.
- **Tools declare their deltas.** Postconditions are checked before commit,
  not after a regretful write.

## World projection

The world is not stored — it is projected. On any step, the runtime folds
commits from the ledger into a typed `World` map of resource keys → versioned
values. Branches recursively fold their ancestor up to the branch point.
Projection is pure.
