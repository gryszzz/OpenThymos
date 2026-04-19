---
layout: default
title: Roadmap
eyebrow: Direction
subtitle: Bring your own model. Keep execution under Thymos.
permalink: /roadmap/
---

The product line runs top-to-bottom: the kernel is the moat, the fabric is
the seatbelt, the coding agent is the first surface, the control plane is the
revenue model.

## Now · pre-alpha

- IPC Triad: Intent → Proposal → Commit.
- Content-addressed, parent-chained SQLite ledger.
- Signed Ed25519 writs, budget + time-window + delegation bounds.
- Cognition: Anthropic (Opus 4.7 default), OpenAI, local OpenAI-compatible, mock.
- Async streaming runtime with SSE, approval channels, cancellation, resume.
- Server with JWT, API-gateway, tenant isolation, persistent run store.
- Secure tool fabric seam — subprocess worker, shell capability profiles, HTTP allowlists.
- 93 tests passing across the workspace.

## Next · Slice 2 (in progress)

- **Coding agent surface** — `fs_read`, `fs_patch`, `list_files`, `repo_map`,
  `grep`, `test_run` as typed `ToolContract`s.
- **In-process CLI** — `thymos code run` drives the runtime without needing
  the HTTP server.
- **Streaming for OpenAI** — SSE parser for `/chat/completions`.
- **Tool-protocol fallback** — JSON-block parser for local models without
  native function calling.
- **Run history CLI** — `thymos runs ls`, `show`, `diff`, `resume`, `cancel`
  reading the existing SQLite run store.
- **Retry-on-fail meta-loop** — plan → edit → test → retry, bounded.

## After · Slice 3

- Postgres ledger path (sync adapter today, async tomorrow).
- Container- or microVM-backed workers.
- Model routing across premium / cheap / local tiers based on task class.
- Control plane: multi-tenant writ issuance, budget enforcement, billing.
- Marketplace v1 — signed tool manifests, verified publishers.
- Trusted delegation: child writs signed by the subject, not the runtime.

## Later

- Browser and code-exec worker classes.
- Signed worker attestation.
- Formal ledger replay verification binary.
- First-party TUI.

## Principles we're not bending

- The model is a proposer, never the authority.
- Every effect is a typed, ledgered commit.
- Every capability is a signed, scoped, budgeted writ.
- Every risky tool runs behind a boundary it cannot choose.
