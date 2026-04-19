---
layout: default
title: Architecture
---

# Architecture

Thymos treats the model as a bounded proposer instead of the authority.

## IPC Triad

1. Intent: cognition declares a desired action.
2. Proposal: runtime/compiler structure the action under writ and policy.
3. Commit: approved effects and observations are appended to the ledger.

## Runtime planes

- Execution kernel: commits, ledger, replay, world projection
- Cognition plane: provider adapters and eventual routing
- Secure tool fabric: high-risk tool execution behind worker boundaries
- Governance plane: policy, approvals, budgets, tenant boundaries
- Distribution plane: hosted control plane and self-hosted runtime nodes

## Current crates

- `thymos-core`
- `thymos-ledger`
- `thymos-policy`
- `thymos-tools`
- `thymos-worker`
- `thymos-compiler`
- `thymos-cognition`
- `thymos-runtime`
- `thymos-server`

The local repo has fuller technical docs in `docs/architecture.md`.
