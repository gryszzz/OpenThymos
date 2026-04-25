<div align="center">

<img src="thymos/Thymos-logo.PNG" alt="Thymos" width="132" height="132" />

# THYMOS

### Governed execution runtime for AI agents.

**Intent -> Proposal -> Execution -> Result**

Thymos turns model output into typed, policy-checked, ledgered execution.<br />
One runtime. Many surfaces. Real tools. Replayable history.

<p>
  <a href="https://gryszzz.github.io/OpenThymos/"><strong>Website</strong></a>
  ·
  <a href="docs/getting-started.md"><strong>Get Started</strong></a>
  ·
  <a href="docs/architecture.md"><strong>Architecture</strong></a>
  ·
  <a href="docs/api-reference.md"><strong>API</strong></a>
  ·
  <a href="https://github.com/gryszzz/OpenThymos/wiki"><strong>Wiki</strong></a>
</p>

<p>
  <img alt="Runtime" src="https://img.shields.io/badge/runtime-Rust-111827?style=for-the-badge" />
  <img alt="Console" src="https://img.shields.io/badge/console-Next.js-0f172a?style=for-the-badge" />
  <img alt="Mode" src="https://img.shields.io/badge/cognition-local%20%2B%20hosted-0b3b5a?style=for-the-badge" />
  <img alt="License" src="https://img.shields.io/badge/license-Apache--2.0-1f2937?style=for-the-badge" />
</p>

</div>

---

## The Signal

Thymos is not a chatbot shell with a few tools bolted on.

It is an execution layer for agentic software. A model proposes work, but the
runtime owns authority, policy, tool execution, observation, recovery, and the
durable record of what happened.

```text
              model output
                   |
                   v
        +----------------------+
        | typed Intent         |
        +----------+-----------+
                   |
                   v
        +----------------------+
        | Proposal + policy    |
        | writ + budget check  |
        +----------+-----------+
                   |
                   v
        +----------------------+
        | real tool execution  |
        | shell, files, HTTP   |
        +----------+-----------+
                   |
                   v
        +----------------------+
        | ledgered Result      |
        | replayable world     |
        +----------------------+
```

The core idea is simple:

> Model output becomes governed execution, not direct authority.

## High-Tech Capabilities

| Capability | What it gives you |
| --- | --- |
| **Shared runtime state** | CLI, VS Code, shell, and web console can attach to the same live run. |
| **Typed action pipeline** | Every move flows through Intent, Proposal, Execution, and Result. |
| **Signed authority** | Writs define who can do what, for how long, with which tool scopes and budgets. |
| **Policy-gated effects** | Runtime checks happen before tools touch files, shell, HTTP, or state. |
| **Live operator feed** | SSE streams expose cognition, execution sessions, approvals, failures, and completion. |
| **Replayable trajectory ledger** | Runs become durable history, not terminal smoke. |
| **Provider-neutral cognition** | Anthropic, OpenAI, LM Studio, Hugging Face, Ollama, local OpenAI-compatible servers, and mock runs can drive the same contract. |
| **Production guardrails** | Production mode refuses unsafe defaults such as missing origin policy or in-process tool fabric. |

## Launch In 5 Minutes

### 0. Install the terminal tools

```bash
./scripts/install.sh
export PATH="$HOME/.local/bin:$PATH"
source "$HOME/.config/thymos/thymos.env"
thymos doctor
```

The installer builds and installs:

| Binary | Purpose |
| --- | --- |
| `thymos` | Branded CLI, doctor dashboard, interactive shell, run controls. |
| `thymos-server` | Local runtime server. |
| `thymos-worker` | Worker process for safer shell / HTTP tool execution. |

### 1. Boot the runtime

```bash
git clone https://github.com/gryszzz/OpenThymos.git
cd OpenThymos/thymos
cargo run -p thymos-server
```

The server starts at `http://localhost:3001` with mock cognition by default,
so you can exercise the full loop without an API key.

```bash
curl http://localhost:3001/health
curl http://localhost:3001/ready
```

### 2. Open the operator console

```bash
cd ..
npm install
npm run dev
```

Open:

```text
http://localhost:3000/runs
```

Terminal-first:

```bash
thymos config
thymos shell
```

### 3. Fire a mock run

```bash
cd thymos
cargo run -p thymos-cli -- run "Inspect the repo and explain the runtime" --provider mock --follow
```

You now have a real Thymos run flowing through the runtime with live status,
execution state, and replayable output.

## Choose Your Control Surface

| Surface | Best for | Start here |
| --- | --- | --- |
| **Web console** | Live operator view, execution log, world replay, branching | `npm run dev`, then open `/runs` |
| **CLI** | Terminal-first launch, follow, status, world, diff, resume, cancel | `cargo run -p thymos-cli -- --help` |
| **VS Code sidebar** | Editor-native approvals and run visibility | [`thymos/clients/vscode`](thymos/clients/vscode) |
| **System shell** | Persistent terminal workflow against the shared runtime | `cargo run -p thymos-cli -- shell` |

## Real Model Mode

Use mock mode for zero-key local validation. When you are ready, point the same
runtime at hosted or local cognition.

```bash
# Anthropic
ANTHROPIC_API_KEY=... cargo run -p thymos-server

# OpenAI
OPENAI_API_KEY=... cargo run -p thymos-server

# Local OpenAI-compatible server
OPENAI_BASE_URL=http://localhost:1234/v1 OPENAI_API_KEY=local cargo run -p thymos-server
```

## Production Profile

For production-shaped deployments, configure persistent stores, explicit
browser origins, worker-backed tool execution, and bounded concurrency.

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

In production mode the server validates these settings at startup and refuses
unsafe defaults.

## Operator Experience

When you submit a task, the runtime is designed to keep the work moving:

1. Understand the goal.
2. Plan the next step.
3. Choose an allowed tool.
4. Execute the tool for real.
5. Observe the result.
6. Recover from failures when possible.
7. Continue until the task is complete, blocked, or cancelled.

Every run exposes a shared **execution session** with status, phase, active
tool, counters, final answer, execution log, and replayable world state.

## Core Concepts

| Concept | Meaning |
| --- | --- |
| **Intent** | What cognition wants to do next. |
| **Proposal** | What the runtime has compiled and policy-checked under the current writ. |
| **Execution** | The real tool invocation and observed result. |
| **Result** | The recorded outcome: commit, rejection, suspension, delegation, failure, or completion. |
| **Writ** | A signed capability document with scopes, budget, effect ceiling, and time window. |
| **Trajectory ledger** | The append-only record of what actually happened during the run. |
| **Execution session** | The live runtime state shared across CLI, VS Code, terminal, and web surfaces. |

## Verify The Stack

```bash
# Local tooling, docs hygiene, and GitHub Pages configuration
npm run doctor

# Web app, static export, and docs
npm run verify

# Rust runtime, server, CLI, worker, ledger, marketplace, and tools
cd thymos
cargo test --workspace
```

## Repository Map

| Path | Purpose |
| --- | --- |
| [`thymos`](thymos) | Rust runtime, server, CLI, worker, policy, ledger, marketplace, and core crates. |
| [`src`](src) | Next.js web app and operator console. |
| [`docs`](docs) | GitHub Pages documentation site. |
| [`wiki`](wiki) | Source pages mirrored into the GitHub wiki. |
| [`thymos/clients/vscode`](thymos/clients/vscode) | VS Code sidebar client. |

## GitHub Pages Site

The public site is deployed with GitHub Pages:

- Website: `https://gryszzz.github.io/OpenThymos/`
- Source: [`docs`](docs)
- Static console export: generated by the Pages workflow from the Next.js app

No custom domain is configured for this repository.

## Read Next

- [Getting Started](docs/getting-started.md)
- [Interfaces](docs/interfaces.md)
- [Architecture](docs/architecture.md)
- [Coding Agent](docs/coding-agent.md)
- [API Reference](docs/api-reference.md)
- [Providers](docs/providers.md)
- [Launch Playbook](docs/launch-playbook.md)

## GitHub Wiki

The project wiki lives at:

`https://github.com/gryszzz/OpenThymos/wiki`

The markdown source for those pages is also kept in [`wiki`](wiki) so the
public docs and the wiki can stay aligned.
