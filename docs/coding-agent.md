---
layout: default
title: Coding Agent
eyebrow: Primary surface
subtitle: A coding agent that plans, edits, tests, and retries — under a signed writ, with every action on the ledger.
permalink: /coding-agent/
---

Thymos ships with a first-class coding agent surface. Under the hood it is the
same IPC Triad — cognition proposes, runtime decides, ledger remembers — with
a focused tool surface optimized for repository work.

## The loop

```
plan   →  edit   →   test   →   retry-on-fail
  │        │           │
 repo_map fs_patch   test_run
 fs_read  list_files shell(inspect)
 grep
```

Every step produces a commit. Every commit is content-addressed. Every tool
invocation is scoped by a signed writ. The model never touches the
filesystem directly — it emits Intents that the runtime evaluates, executes,
and records.

## The coding tool surface

| Tool         | Effect   | Risk    | Does                                                              |
|--------------|----------|---------|-------------------------------------------------------------------|
| `fs_read`    | read     | low     | Return file content with sha256. Missing file → `{exists:false}`. |
| `fs_patch`   | write    | medium  | Exact-match edit. Verifies pre-hash, emits post-hash.             |
| `list_files` | read     | low     | Glob-filtered walk. Respects `.gitignore` and `.thymosignore`.    |
| `repo_map`   | read     | low     | Typed summary of repo shape: languages, sizes, hashes, tree.      |
| `grep`       | read     | low     | Pattern search over the repo. Structured matches back.            |
| `test_run`   | external | medium  | Runs a test profile (`cargo`, `npm`, `pytest`, ...). Typed result. |
| `shell`      | external | high    | Capability-profiled secure shell. Inspect / build / mutate.       |

Each tool is a `ToolContract` with an input schema, effect class, risk class,
preconditions, and postconditions. The runtime rejects any invocation outside
the writ's scope.

## One command

```bash
thymos code run "add a retry to the HTTP client" \
    --provider lmstudio \
    --model qwen2.5-coder-32b-instruct
```

This boots an in-process runtime, mints a coding writ, and drives the model
through the loop above. Output streams live. A failed `test_run` feeds back as
a typed observation — the model gets a structured view of the failure and
proposes a fix on the next turn.

## Safety by default

- File writes go through `fs_patch`, not raw `shell rm/mv`.
- `fs_patch` verifies the file's current hash before applying the edit.
  Concurrent-edit races are caught, not clobbered.
- The shell tool runs under a capability profile; `inspect` is the default,
  `mutate` is opt-in, `networked` is its own profile.
- Any tool can be wired through the worker-backed secure fabric — a
  subprocess boundary with timeout enforcement and execution receipts.

## The coding writ

The default writ for `thymos code run`:

```text
scopes:        fs_read, fs_patch, list_files, repo_map, grep,
               test_run, shell(inspect)
budget:        200k tokens · 64 tool calls · 600s wall clock
effect ceiling: read+write local, no network
time window:   now → now + 10 min
delegation:    max_depth 2
```

Override with `thymos writ mint ...` or by passing a signed writ from your
control plane.

## Model-agnostic

Point `--provider lmstudio` (or `ollama`, or any OpenAI-compatible endpoint)
and the full governance layer applies identically. The `OpenAiCognition`
adapter accepts a `base_url` and auto-negotiates tool-calling. Swap models
mid-project without re-plumbing anything.

## Replayable

Every run appends to the append-only trajectory ledger. Run
`thymos runs show <id> --diff` to get a unified diff of what changed.
Run `thymos runs resume <id>` to pick up from the last commit.
Every byte is hash-addressable. Every commit has a parent. The ledger is
the source of truth — nothing lives only in memory.
