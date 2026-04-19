# Thymos Monster Runtime Roadmap

This document turns the current Thymos implementation into an execution plan for
building the strongest possible code-agent runtime:

- sovereign over execution
- portable across premium, cheap, and local models
- safe enough for real code operations
- monetizable as both open core and enterprise control plane

It is written against the repo as it exists now.

## Current Reality

Thymos already has the right kernel:

- cognition is a bounded proposer, not the authority
- the runtime owns execution, policy, and ledger writes
- every action becomes an Intent -> Proposal -> Commit flow
- runs can stream, suspend for approval, resume, and branch
- local OpenAI-compatible models are already supported

Relevant crates:

- `thymos-core`: identities, writs, hashes, commits, deltas
- `thymos-cognition`: Anthropic, OpenAI, local OpenAI-compatible, mock
- `thymos-compiler`: authority, budget, precondition, and policy gates
- `thymos-runtime`: run loop, branching, delegation, world projection
- `thymos-tools`: tool contracts, shell/http tools, manifest tools, MCP loading
- `thymos-ledger`: SQLite default, async Postgres implementation
- `thymos-server`: HTTP API, SSE, approvals, resume, cancellation, auth
- `thymos-marketplace`: in-memory package registry

## Product Thesis

Thymos should become:

**The governed execution substrate for code agents.**

Not "another coding agent."

The product is the runtime beneath any agent:

- bring your own model
- bring your own planner
- keep execution, policy, and audit under Thymos

## First User Wedge

The first wedge is not general AI automation. It is:

**Teams running coding agents against sensitive codebases that need policy, audit, approvals, and model portability.**

Ideal first users:

- platform teams standardizing agent execution
- security-conscious engineering orgs
- enterprises wanting local or mixed-model operation
- teams already using Claude Code, Codex, Cursor, or internal agents

First promise:

**Run code agents with less trust in the model and more trust in the runtime.**

## Monetization Path

### Open core

- core runtime
- local deployment
- SQLite and Postgres ledger support
- basic tool registry
- local model support

### Paid control plane

- run dashboard
- approval inbox
- policy management
- audit retention and export
- usage controls and quotas
- package trust and install workflows

### Enterprise

- SSO, SCIM, RBAC
- air-gapped deployment
- HSM-backed signing
- private package registry
- compliance packs
- support and onboarding

## Target Architecture

The monster-product version should have five planes.

### 1. Execution kernel

Keep the existing IPC triad and ledger as the permanent core.

Needed additions:

- stronger replay guarantees
- stable schema/version migration story
- cross-run and cross-tenant audit querying
- deterministic execution receipts

### 2. Cognition plane

Today Thymos selects one provider per run. It should become a router.

Target:

- planner model
- executor model
- verifier model
- cost/latency/quality routing
- cheap/local default with premium fallback

### 3. Secure tool fabric

This is the most important missing layer.

Target:

- sandboxed tool workers
- capability-scoped filesystem mounts
- network egress controls
- browser/code/shell worker classes
- signed execution receipts

Current status:

- Slice 2 scaffold is in place
- `shell` and `http` now route through a tool-fabric seam
- `thymos-worker` provides a subprocess worker boundary
- the shell contract emits THYMOS-native execution receipts instead of being
  only a raw terminal wrapper

### 4. Governance plane

Target:

- policy packs
- approval workflows
- tenant and role boundaries
- run budgets and kill switches
- compliance and forensic export

### 5. Distribution plane

Target:

- hosted control plane
- self-hosted runtime nodes
- private/public tool marketplace
- package signing and trust policy

## Hard Truths From The Current Codebase

These are the main gaps between the current implementation and the monster version.

### Gap 1: execution is still too trusted

`thymos-tools` can execute shell commands and HTTP calls directly in-process.

That is excellent for a reference implementation and insufficient for a
production-grade code-agent runtime.

Primary files:

- `crates/thymos-tools/src/lib.rs`
- `crates/thymos-server/src/lib.rs`

### Gap 2: the control plane is still phase-one

The package marketplace is in-memory.
The API gateway key store and rate limiting are in-memory.

Primary files:

- `crates/thymos-marketplace/src/lib.rs`
- `crates/thymos-server/src/middleware.rs`

### Gap 3: Postgres exists, but is not the operational default

The repo has a meaningful async Postgres ledger implementation, but the runtime
surface still centers SQLite and the top-level docs understate Postgres.

Primary files:

- `crates/thymos-ledger/src/postgres.rs`
- `crates/thymos-server/src/lib.rs`
- `crates/thymos-server/src/main.rs`
- `README.md`
- `docs/getting-started.md`

### Gap 4: policy is credible but still shallow

The policy engine is composable, but current stock policy is intentionally small.
The monster version needs policy packs, execution-risk policy, sandbox policy,
model-routing policy, and approval policy.

Primary files:

- `crates/thymos-policy/src/lib.rs`
- `crates/thymos-compiler/src/lib.rs`

### Gap 5: local models are supported, but routing is primitive

Local models already work through OpenAI-compatible endpoints, which is a great
base. The next step is not "more providers"; it is routing and specialization.

Primary files:

- `crates/thymos-cognition/src/lib.rs`
- `crates/thymos-cognition/src/openai.rs`
- `docs/getting-started.md`

## Build Sequence

The order matters. This sequence is optimized for product leverage, not just
technical neatness.

## Phase 0: Lock The Kernel

Goal:

- make the current core explicit, stable, and easier to extend safely

Deliverables:

- versioned architectural ADR for the execution kernel
- stronger invariants around replay and schema evolution
- explicit "trusted reference mode" vs "production mode"

Files to touch:

- `README.md`
- `docs/architecture.md`
- `docs/getting-started.md`
- `crates/thymos-core/src/*.rs`
- `crates/thymos-ledger/src/lib.rs`

Acceptance criteria:

- a run recorded on one build can be replayed and validated on the next build
- ledger format versioning is explicit
- production-vs-reference tradeoffs are documented

## Phase 1: Production Ledger And State

Goal:

- make persistence and multi-node operation real

Deliverables:

- first-class Postgres ledger path
- persistent API key store
- persistent marketplace backing store
- consistent run metadata in Postgres or shared durable store

Files to touch:

- `crates/thymos-ledger/src/postgres.rs`
- `crates/thymos-ledger/src/lib.rs`
- `crates/thymos-server/src/main.rs`
- `crates/thymos-server/src/lib.rs`
- `crates/thymos-server/src/middleware.rs`
- `crates/thymos-marketplace/src/lib.rs`
- `docker-compose.yml`

Acceptance criteria:

- two server instances can operate against the same ledger backend
- auth/package metadata survives restart
- Postgres becomes documented production default

## Phase 2: Secure Tool Workers

Goal:

- move dangerous execution out of the server process

Deliverables:

- worker interface for tool execution
- worker classes: shell, http, browser, filesystem, git
- receipt model for worker execution
- sandbox policy and egress control

Files to touch:

- `crates/thymos-tools/src/lib.rs`
- new crate: `crates/thymos-worker`
- new crate: `crates/thymos-sandbox`
- `crates/thymos-runtime/src/lib.rs`
- `crates/thymos-server/src/lib.rs`

Acceptance criteria:

- shell and http execution no longer run directly in the server process in production mode
- every external tool execution emits a receipt with worker identity, timing, and policy context
- filesystem/network access can be constrained per run or per tool

## Phase 3: Policy Packs And Governance

Goal:

- turn policy from code-level hooks into product capability

Deliverables:

- policy pack abstraction
- approval rules by tool, tenant, cost, path, network target, and risk class
- operator-facing approval metadata
- policy trace visibility in API responses and UI

Files to touch:

- `crates/thymos-policy/src/lib.rs`
- `crates/thymos-compiler/src/lib.rs`
- `crates/thymos-server/src/lib.rs`
- new docs under `docs/policies/`

Acceptance criteria:

- an operator can apply a predefined policy pack to a tenant or run class
- approval requirements can be declared without modifying Rust source per customer
- policy traces are visible and auditable

## Phase 4: Cognition Routing For Cheap And Local Models

Goal:

- make model choice an execution policy, not a hardcoded run setting

Deliverables:

- multi-model router
- task-class model profiles
- verifier/fallback flow
- cheap/local-first mode

Files to touch:

- `crates/thymos-cognition/src/lib.rs`
- `crates/thymos-cognition/src/openai.rs`
- `crates/thymos-cognition/src/anthropic.rs`
- new modules:
  - `crates/thymos-cognition/src/router.rs`
  - `crates/thymos-cognition/src/profile.rs`

Acceptance criteria:

- a run can use one model for planning and another for execution or verification
- local OpenAI-compatible models are a first-class documented deployment mode
- fallback rules are explicit and test-covered

## Phase 5: Control Plane UX

Goal:

- make Thymos usable as a real platform

Deliverables:

- run dashboard
- live token stream and ledger stream
- world-state browser
- approval inbox
- usage and tenant dashboards

Suggested implementation:

- keep Rust server as control API
- add a separate frontend app rather than pushing UI into the server crate

Files to touch:

- `crates/thymos-server/src/lib.rs`
- `crates/thymos-client/src/lib.rs`
- new directory: `apps/thymos-console`

Acceptance criteria:

- operators can inspect any run without touching the DB
- approvals can be handled from the UI
- model/provider, policy trace, and tool receipts are visible per step

## Phase 6: Package Trust And Marketplace

Goal:

- turn tools into a governed distribution ecosystem

Deliverables:

- persistent package registry
- package signing
- trust policy and install constraints
- private registry mode

Files to touch:

- `crates/thymos-marketplace/src/lib.rs`
- `crates/thymos-tools/src/lib.rs`
- `crates/thymos-server/src/marketplace_api.rs`
- new crate: `crates/thymos-signing`

Acceptance criteria:

- tools can be published, signed, and installed with integrity verification
- orgs can allow only signed packages from trusted publishers
- private marketplace works for self-hosted customers

## Phase 7: Native Multi-Agent Runtime

Goal:

- make delegation and branching a flagship feature

Deliverables:

- first-class planner/executor/verifier patterns
- branch comparison and merge semantics
- trajectory-level evals
- budget-aware delegation strategy

Files to touch:

- `crates/thymos-runtime/src/agent.rs`
- `crates/thymos-runtime/src/agent_async.rs`
- `crates/thymos-runtime/src/lib.rs`
- `crates/thymos-core/src/writ.rs`
- `crates/thymos-server/src/lib.rs`

Acceptance criteria:

- parent and child trajectories can be inspected and compared in product surfaces
- delegation budgets and depth are enforced and observable
- multi-agent runs are replayable and auditable like single-agent runs

## First Three Execution Slices

These should be implemented in order.

### Slice 1: Production mode foundation

- add explicit runtime mode config: `reference` vs `production`
- make Postgres ledger selectable and documented as primary production path
- persist API keys and marketplace state

Why first:

- it upgrades the repo from credible demo to credible platform base

### Slice 2: Tool worker abstraction

- extract shell/http execution behind a worker boundary
- add execution receipts
- add sandbox config primitives

Why second:

- this is the core trust unlock for code-agent use cases

### Slice 3: Model router

- add cognition routing profiles
- local-first and cheap-first execution strategy
- premium verifier fallback

Why third:

- this is where "run on my own terms" becomes true in practice

## Immediate Repo Tasks

These are the concrete next tasks to open as implementation tickets.

1. Add `RuntimeMode` and production config plumbing.
2. Add persistent `ApiGatewayStore` abstraction and replace in-memory-only keys.
3. Add persistent `MarketplaceStore` abstraction and replace in-memory-only registry.
4. Add a `ToolExecutor` abstraction so `ToolContract` execution can be routed to workers.
5. Implement production `ShellWorker` and `HttpWorker` behind that abstraction.
6. Add `CognitionRouter` with profiles like `cheap_first`, `local_first`, `premium_safe`.
7. Add a lightweight web console for runs, approvals, and trajectory inspection.

## Definition Of Success

Thymos wins if users can say:

- "I can run code agents without trusting the model with raw authority."
- "I can switch between Anthropic, OpenAI, Ollama, or vLLM without changing the runtime."
- "I can prove what happened in every run."
- "I can let teams move fast without giving uncontrolled shell access to an LLM."

That is the line between a sharp framework and a category-defining runtime.
