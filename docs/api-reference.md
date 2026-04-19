---
layout: default
title: API Reference
eyebrow: HTTP surface
subtitle: axum over the Thymos runtime. JSON bodies, SSE streams, JWT or API-gateway auth.
permalink: /api-reference/
---

Base URL: `http://localhost:3001` (default)

## Health

### GET /health

Liveness probe. Always bypasses auth.

**Response** `200 OK`
```json
{ "status": "ok" }
```

---

## Runs

### POST /runs

Start a new agent run.

**Headers** (optional):
- `x-thymos-tenant-id` — tenant scoping
- `x-thymos-user-id` — user scoping
- `Authorization: Bearer <JWT>` — if JWT auth is enabled

**Request Body**
```json
{
  "task": "Set greeting to hello and read it back",
  "max_steps": 32,
  "tool_scopes": ["*"],
  "cognition": {
    "provider": "anthropic",
    "model": "claude-opus-4-7",
    "cache_prefix": true
  }
}
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `task` | string | required | Natural language task description |
| `max_steps` | u32 | 32 | Maximum agent loop iterations |
| `tool_scopes` | string[] | `["*"]` | Tool glob patterns |
| `cognition.provider` | string | "anthropic" | `anthropic`, `openai`, `local`, `mock` |
| `cognition.model` | string | provider default | Full id (`claude-opus-4-7`) or alias (`opus`, `sonnet`, `haiku`, `opus-4.6`) |
| `cognition.max_tokens` | u32 | provider default | Cap on response tokens |
| `cognition.thinking_budget_tokens` | u32 | null | Enable extended thinking with this budget (Anthropic 4.7+) |
| `cognition.cache_prefix` | bool | true | Mark the system+tools prefix with `cache_control` for prompt caching |
| `cognition.base_url` | string | null | Custom endpoint (for `local` provider) |

**Response** `202 Accepted`
```json
{
  "run_id": "uuid",
  "task": "Set greeting to hello and read it back",
  "status": "running"
}
```

### GET /runs/:id

Get run status and summary.

**Response** `200 OK`
```json
{
  "trajectory_id": "hex-string",
  "task": "...",
  "status": "completed",
  "summary": {
    "steps_executed": 3,
    "intents_submitted": 3,
    "commits": 2,
    "rejections": 0,
    "final_answer": "Done! The greeting is set to hello.",
    "terminated_by": "CognitionDone"
  }
}
```

Status values: `running`, `completed`, `failed`

### GET /runs/:id/events

SSE stream of ledger entries as they are committed.

**Event format:**
```
data: {"seq":0,"kind":"root","id":"abc123","detail":{...}}

data: {"seq":1,"kind":"commit","id":"def456","detail":{...}}
```

### GET /runs/:id/stream

SSE stream of cognition events (real-time tokens).

**Event types:**
- `token` — `{ "text": "..." }`
- `tool_use_start` — `{ "tool_use_id": "...", "name": "..." }`
- `tool_use_arg_delta` — `{ "tool_use_id": "...", "delta": "..." }`
- `tool_use_done` — `{ "tool_use_id": "..." }`
- `turn_complete` — `{ "stop_reason": "..." }`
- `error` — `{ "message": "..." }`

### GET /runs/:id/world

Current world state projection from the ledger.

**Response** `200 OK`
```json
{
  "resources": [
    { "kind": "kv", "id": "greeting", "version": 1, "value": "hello" }
  ]
}
```

### POST /runs/:id/resume

Resume a previously started or failed run. Body is the same as POST /runs.

**Response** `202 Accepted`
```json
{ "run_id": "...", "status": "resuming" }
```

### GET /runs/:id/delegations

List child trajectories spawned via delegation.

**Response** `200 OK`
```json
{
  "delegations": [
    {
      "child_trajectory_id": "hex",
      "task": "sub-task description",
      "final_answer": "result or null",
      "seq": 3
    }
  ]
}
```

---

## Approvals

### POST /runs/:id/approvals/:channel

Approve or deny a pending proposal (human-in-the-loop).

**Request Body**
```json
{ "approve": true }
```

**Response** `200 OK`
```json
{ "run_id": "...", "channel": "...", "approved": true }
```

Error responses: `404` (no pending approval), `410` (agent already terminated)

---

## Audit

### GET /audit/entries

Query ledger entries with optional filters. Supports JSON and CSV export.

**Query Parameters**
| Param | Type | Description |
|-------|------|-------------|
| `run_id` | string | Filter by run (resolves to trajectory) |
| `kind` | string | Entry kind: root, commit, rejection, pending_approval, delegation, branch |
| `from` | u64 | Unix timestamp lower bound |
| `to` | u64 | Unix timestamp upper bound |
| `format` | string | `json` (default) or `csv` |
| `limit` | u32 | Max entries (default 1000) |

**JSON Response** `200 OK`
```json
{
  "entries": [
    {
      "id": "hex",
      "trajectory_id": "hex",
      "seq": 0,
      "kind": "root",
      "payload": { "type": "root", "note": "..." },
      "created_at": 1713456789
    }
  ],
  "count": 1
}
```

**CSV Response** `200 OK` with `Content-Type: text/csv`

### GET /audit/entries/count

Count matching entries without fetching payloads. Same query params as above
(except `format` and `limit`).

**Response** `200 OK`
```json
{ "count": 42 }
```

---

## Usage

### GET /usage

Per-key usage statistics (only meaningful when API gateway is configured).

**Response** `200 OK`
```json
{ "message": "API gateway not configured", "stats": [] }
```

---

## Marketplace

### GET /marketplace/packages

List all published packages (latest versions).

### GET /marketplace/packages/:name

Get a specific package by name.

### POST /marketplace/packages

Publish a new package.

**Request Body**
```json
{
  "name": "my-tool",
  "version": "1.0.0",
  "description": "A useful tool",
  "author": "you",
  "kind": "manifest",
  "tags": ["utility"],
  "content": "base64-encoded content or JSON"
}
```

### DELETE /marketplace/packages/:name/:version

Unpublish a specific version.

### GET /marketplace/search?q=...&tag=...&kind=...&author=...

Search packages by text, tag, kind, or author.
