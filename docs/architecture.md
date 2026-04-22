---
layout: default
title: Architecture
eyebrow: System design
subtitle: Thymos is built around a shared runtime, a bounded agent loop, and one observable execution session per run.
permalink: /architecture/
---

## The big idea

Thymos separates **proposing work** from **executing work**.

The model proposes the next move.

The runtime owns:

- authority
- tool execution
- failure handling
- logging
- replay
- completion state

That separation is what lets multiple interfaces attach to the same backend run without inventing their own version of reality.

## The execution flow

```
Cognition -> Intent -> Proposal -> Execution -> Result
```

### Intent

The model declares what it wants to do next.

### Proposal

The runtime compiles the intent under the current writ, checks policy, and decides whether the action is allowed, rejected, or suspended for approval.

### Execution

If approved, the runtime invokes the real tool and captures the observed outcome.

### Result

The runtime records what actually happened: commit, rejection, suspension, failure, delegation, or completion.

## The agent loop

The loop is intentionally simple:

1. build context
2. ask cognition for the next step
3. execute allowed work
4. observe the outcome
5. feed the outcome back into the next step
6. continue until complete, blocked, expired, or cancelled

This is how Thymos behaves like an agent without giving the model direct authority over the world.

## The shared execution session

Each run produces a live execution session with:

- current status
- current phase
- operator state
- counters for commits, rejections, failures, approvals, and recoveries
- final answer
- execution log

The web console, CLI, shell, and VS Code sidebar all consume that same runtime session.

## The main planes

### Cognition plane

`thymos-cognition`

Responsible for turning context into proposed next actions. Supports hosted and local providers.

### Runtime plane

`thymos-runtime`

Responsible for running the agent loop, handling approvals, projecting world state, and turning runtime outcomes back into typed history.

### Governance plane

`thymos-policy` and writ handling in `thymos-core`

Responsible for capability enforcement, policy checks, budgets, time windows, and approval boundaries.

### Tool plane

`thymos-tools`

Responsible for typed tools such as file reads, file patches, repo mapping, grep, tests, shell, HTTP, memory, and delegation.

### Ledger plane

`thymos-ledger`

Responsible for durable trajectory history and replayable state.

### Surface plane

`thymos-server`, `thymos-cli`, `clients/vscode`, and the web app

Responsible for exposing the same backend run to different operator surfaces.

## Core invariants

- The model never directly mutates the world.
- Authority is checked before execution happens.
- Runtime truth is observable through structured events.
- Failed execution is part of the run history, not hidden control flow.
- Clients read shared run state instead of maintaining separate agent state.
- The ledger remains the durable record of what happened.

## Why this matters

Without a shared runtime, every surface becomes its own mini-agent product.

With Thymos:

- a task started in the CLI can be reviewed in the browser
- an approval can be handled in VS Code
- the same run can be resumed later
- the system can expose one authoritative execution log

That is the architectural point of the product.
