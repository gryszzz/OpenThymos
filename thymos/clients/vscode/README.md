# Thymos VSCode Extension

Run a Thymos governed-cognition agent from inside VSCode. The extension talks
directly to the Thymos HTTP API and uses the native VSCode diff viewer for
approval review.

## Commands

| Command | What it does |
|---|---|
| `Thymos: Open Shell` | Opens an integrated terminal running `thymos shell` with `THYMOS_URL` + workspace + preset pre-configured |
| `Thymos: Run Task (auto)` | Prompts for a task, POSTs `/runs`, polls the ledger, and opens a **native VSCode diff editor** for each pending `fs_patch` approval before prompting y/N |
| `Thymos: Health Check` | `GET /health`, shown as a VSCode notification |
| `Thymos: Review Pending Approval` | Takes a run id and walks pending approvals with diff review |

A status-bar item shows live server health; click it to refresh.

## Settings

- `thymos.url` — server URL (default `http://localhost:3001`)
- `thymos.apiKey` — bearer token (optional)
- `thymos.cliPath` — path to the `thymos` binary (default: on PATH)
- `thymos.provider` — default cognition provider
- `thymos.preset` — `code` (default) uses the coding toolkit + `max_steps=64`

## Build

```bash
cd clients/vscode
npm install
npm run compile
```

Then either:

- **Dev**: open this folder in VSCode and hit F5 — a new Extension Host
  window launches with Thymos loaded.
- **Package**: `npx vsce package` produces `thymos-vscode-0.0.1.vsix`. Install
  it via the command palette: *Extensions: Install from VSIX…*.

## Prerequisites

A running Thymos server. In a separate terminal:

```bash
cargo run -p thymos-server
```

And the CLI on PATH (or adjust `thymos.cliPath`):

```bash
cargo install --path crates/thymos-cli
```
