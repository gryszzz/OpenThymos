# Contributing to Thymos

Thymos is a pre-alpha runtime. The codebase moves fast, but the invariants don't — correctness in the compiler / policy / ledger path is non-negotiable, because the whole system only has value if every commit is truly authorized, bounded, and replayable.

## Ways to help

1. **Run the demo and file issues.** `cargo run --example hello_triad -p thymos-runtime` from `thymos/`. Anything confusing, missing, or surprising is a filing.
2. **Try the HTTP server + `/runs` viewer.** Report latency, SSE reconnect behavior, policy UX, or anything that looks wrong.
3. **Write a tool.** Implement `ToolContract` in `thymos-tools`. Declare schema, effect class, risk class, and pre/postconditions. Open a PR.
4. **Harden an adapter.** The Anthropic adapter is the most mature; OpenAI and local are thinner. Tests for real-world transient-failure modes are welcome.

## Ground rules

- **Tests first on the ledger / compiler / policy path.** The IPC Triad is the spine. No untested changes there.
- **Keep PRs small and typed.** A single crate, a single responsibility. If a PR touches five crates, split it.
- **No silent bypass.** Don't add "trust me" modes that skip writ verification or policy evaluation. If you need to, it's a bug in the abstraction, not a feature.
- **Follow `rustfmt` and `clippy` defaults.** CI will run them.
- **Cargo features stay orthogonal.** `async`, `telemetry`, `postgres` — one concern per feature.
- **Don't land placeholder copy.** If you're adding docs/README/site copy, write what's true today, not what you hope is true next quarter.

## Local loop

```bash
# in thymos/
cargo build --workspace
cargo test --workspace
cargo run --example hello_triad -p thymos-runtime

# server + site
cargo run -p thymos-server                 # :3001
cd .. && npm install && npm run dev        # :3000, open /runs
```

## Scope

**In scope:** IPC Triad primitives, ledger backends, policy engines, tool contracts, cognition adapters, runtime orchestration, HTTP / CLI / SDK surfaces, observability.

**Out of scope for now:** hosted services, UI component libraries, paid plugins, marketing integrations, anything that requires a proprietary API.

## License

By contributing you agree your contributions are licensed under Apache-2.0. The Developer Certificate of Origin applies implicitly — sign-offs are encouraged but not required.
