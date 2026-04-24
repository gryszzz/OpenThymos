<div align="center">

<img src="thymos/Thymos-logo.PNG" alt="Thymos" width="128" height="128" />

# THYMOS

**Provider-neutral execution framework for agentic software.**

*Intent -> Proposal -> Execution -> Result*

[Website](https://gryszzz.github.io/OpenThymos/) · [Docs](docs) · [Wiki](https://github.com/gryszzz/OpenThymos/wiki) · [Rust Workspace](thymos) · [Getting Started](docs/getting-started.md)

</div>

---

## What Thymos Is

Thymos is not just a prompt wrapper or a tool-calling shell.

It is a **shared execution framework** for agentic work. A user can start a task from the CLI, a VS Code sidebar, a web console, or a terminal session and attach all of those surfaces to the **same live run**.

Each run moves through a structured flow:

`Intent -> Proposal -> Execution -> Result`

The model proposes work. The runtime decides whether that work is allowed under a signed writ, executes real tools, observes results, records everything in a durable ledger, and keeps going until the task is complete, blocked, or explicitly cancelled.

The core idea is simple:

`Model output becomes governed execution, not direct authority.`

## What Makes It Different

- **One runtime, many surfaces.** CLI, VS Code, terminal, and web console reflect the same backend run state.
- **Live by default.** The console consumes execution-session SSE snapshots, cognition streams, and periodic snapshot refreshes so users can watch real work while it happens.
- **Agentic by design.** Thymos plans, executes, observes, retries, and adapts instead of stopping after every tool call.
- **Real execution only.** File reads, patches, tests, shell calls, approvals, failures, and recoveries are all real runtime events.
- **Fully observable.** Every run has a live execution session, clear status, and replayable execution log.
- **Controlled, not chaotic.** Signed writs, policy checks, budgets, approvals, and typed tools keep autonomy bounded.
- **Model-flexible.** Anthropic, OpenAI, LM Studio, Hugging Face, Ollama, and other OpenAI-compatible backends can all drive the same runtime.

## Start In 5 Minutes

### 1. Start the backend runtime

```bash
git clone https://github.com/gryszzz/OpenThymos.git
cd OpenThymos/thymos
cargo run -p thymos-server
```

By default this starts Thymos on `http://localhost:3001` with mock cognition, so you can test the full runtime with no API key.

Runtime probes:

```bash
curl http://localhost:3001/health
curl http://localhost:3001/ready
```

### 2. Pick the surface you want to use

**Web console**

```bash
cd ..
npm install
npm run dev
```

Open `http://localhost:3000/runs`.

Validate the static GitHub Pages export and the markdown docs before push:

```bash
npm run site:check
```

Preview the exported Pages build locally:

```bash
npm run pages:preview
```

**CLI**

```bash
cd thymos
cargo run -p thymos-cli -- run "Inspect the repo and explain the runtime" --provider mock --follow
```

**VS Code**

Build the extension in [`thymos/clients/vscode`](thymos/clients/vscode), launch it in Extension Development Host, and point it at `http://localhost:3001`.

### 3. Use a real model when you are ready

Examples:

```bash
# Anthropic
ANTHROPIC_API_KEY=... cargo run -p thymos-server

# OpenAI
OPENAI_API_KEY=... cargo run -p thymos-server

# Local OpenAI-compatible server
OPENAI_BASE_URL=http://localhost:1234/v1 OPENAI_API_KEY=local cargo run -p thymos-server
```

For production-shaped deployments, also configure:

```bash
THYMOS_RUNTIME_MODE=production
THYMOS_BIND_ADDR=0.0.0.0:3001
THYMOS_LEDGER_PATH=/var/lib/thymos/thymos-ledger.db
THYMOS_DB_PATH=/var/lib/thymos/thymos-runs.db
THYMOS_GATEWAY_DB_PATH=/var/lib/thymos/thymos-gateway.db
THYMOS_MARKETPLACE_DB_PATH=/var/lib/thymos/thymos-marketplace.db
THYMOS_ALLOWED_ORIGINS=https://your-console.example.com
THYMOS_TOOL_FABRIC=worker
THYMOS_WORKER_BIN=/usr/local/bin/thymos-worker
THYMOS_MAX_CONCURRENT_RUNS_GLOBAL=100
THYMOS_MAX_CONCURRENT_RUNS_PER_TENANT=20
```

In production mode the server validates these settings at startup and refuses unsafe defaults such as in-process shell/http execution or missing browser origin policy.

## Production Readiness

Thymos is built to be usable without a hosted model and configurable for real deployments:

- **Default cognition is `mock`.** The framework boots, creates runs, streams state, and exercises the operator loop without any AI provider key.
- **Provider choice is explicit.** OpenAI, local OpenAI-compatible servers, LM Studio, Hugging Face, Anthropic, and mock cognition all use the same runtime contract.
- **Runtime state is live.** `/runs/:id/execution/stream` is the authoritative operator stream, `/runs/:id/stream` exposes cognition events, and the web console refreshes snapshots if a stream reconnects.
- **Production mode is guarded.** `THYMOS_RUNTIME_MODE=production` requires persistent database paths, worker-backed tool execution, explicit CORS origins, and valid concurrency limits.
- **Verification is scripted.** Run `npm run verify` for the web/docs surface and `cargo test --workspace` from `thymos` for the Rust runtime.

## The Operator Experience

Thymos is meant to feel like an elite coding runtime, not a chatbot.

When you submit a task, the runtime should:

1. Understand the goal.
2. Plan the next step.
3. Choose an allowed tool.
4. Execute the tool for real.
5. Observe the outcome.
6. Recover from failures when possible.
7. Keep working until the task is actually resolved.

That work is exposed as a shared **execution session** with:

- current status
- active phase
- operator-readable execution log
- commit / rejection / failure counters
- final answer
- replayable world state

## The Main Surfaces

### Web Console

The web app gives you a premium operator view of the runtime: live execution session, execution log, raw cognition stream, world replay, and branching.

### CLI

The CLI is for terminal-first users who want to launch, follow, inspect, diff, resume, cancel, and review runs from the shell.

### VS Code Sidebar

The extension gives you a persistent Thymos console inside the editor, plus approval review and diff inspection without losing the shared runtime state.

### System Terminal / Shell

The interactive shell lets users stay in a terminal workflow while still using the same live run model and approval loop.

## Core Concepts

### Intent

What the model wants to do next.

### Proposal

What the runtime has compiled and policy-checked under the current writ.

### Execution

The real tool invocation and observed result.

### Result

The recorded outcome: commit, rejection, suspension, delegation, failure, or completion.

### Writ

A signed capability document that defines what the agent may do, for how long, with what tool scopes and budget.

### Trajectory Ledger

The append-only record of what really happened during the run.

### Execution Session

The live runtime state shared across CLI, VS Code, terminal, and web surfaces.

## Repository Map

- [`thymos`](thymos) — Rust runtime, server, CLI, worker, and core crates
- [`src`](src) — Thymos web app and operator console
- [`docs`](docs) — GitHub Pages documentation site
- [`wiki`](wiki) — source pages mirrored into the GitHub wiki

## Where To Read Next

- [Getting Started](docs/getting-started.md)
- [Interfaces](docs/interfaces.md)
- [Architecture](docs/architecture.md)
- [Coding Agent](docs/coding-agent.md)
- [API Reference](docs/api-reference.md)
- [Providers](docs/providers.md)

## GitHub Wiki

The project wiki lives at:

`https://github.com/gryszzz/OpenThymos/wiki`

The markdown source for those pages is also kept in [`wiki`](wiki) so the public docs and the wiki can stay aligned.
