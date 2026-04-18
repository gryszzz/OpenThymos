# Getting Started with Thymos

## Prerequisites

- Rust 1.80+ (`rustup update stable`)
- An LLM API key (Anthropic or OpenAI) — optional, mock cognition works without one

## Build

```bash
cd thymos
cargo build --release
```

## Run the Server

### Minimal (mock cognition, no auth)

```bash
cargo run -p thymos-server
```

Server starts on `http://localhost:3001`.

### With Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo run -p thymos-server
```

### With OpenAI

```bash
OPENAI_API_KEY=sk-... cargo run -p thymos-server
```

### With JWT Auth

```bash
THYMOS_JWT_SECRET=my-secret cargo run -p thymos-server
```

### With API Gateway

```bash
THYMOS_API_KEYS="key1:tenant1:MyKey:60" cargo run -p thymos-server
```

Format: `api_key:tenant_id:key_name:rate_limit_rpm` (comma-separated for multiple keys)

## Your First Run

### Using curl

```bash
# Health check
curl http://localhost:3001/health

# Start a run with mock cognition (no API key needed)
curl -X POST http://localhost:3001/runs \
  -H 'content-type: application/json' \
  -d '{"task": "Set greeting to hello and read it back", "cognition": {"provider": "mock"}}'

# Check run status (replace RUN_ID with the actual ID)
curl http://localhost:3001/runs/RUN_ID

# View world state
curl http://localhost:3001/runs/RUN_ID/world
```

### Using the CLI

```bash
# Build the CLI
cargo build -p thymos-cli --release

# Health check
./target/release/thymos health

# Start a run
./target/release/thymos run "Set greeting to hello" --cognition mock

# Check status
./target/release/thymos status RUN_ID

# Stream tokens in real-time
./target/release/thymos stream RUN_ID

# View world state
./target/release/thymos world RUN_ID
```

### Using the Rust Client SDK

```rust
use thymos_client::ThymosClient;

#[tokio::main]
async fn main() -> Result<(), thymos_client::Error> {
    let client = ThymosClient::new("http://localhost:3001");

    // Health check
    let health = client.health().await?;
    println!("Server: {}", health.status);

    // Create a run
    let run = client.create_run("Say hello", None).await?;
    println!("Run ID: {}", run.run_id);

    // Poll until complete
    let result = client.poll_run(&run.run_id, 200, 50).await?;
    println!("Status: {:?}", result.status);
    println!("Summary: {:?}", result.summary);

    Ok(())
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ANTHROPIC_API_KEY` | Anthropic API key for Claude models | — |
| `OPENAI_API_KEY` | OpenAI API key | — |
| `OPENAI_BASE_URL` | Custom OpenAI-compatible endpoint | `https://api.openai.com/v1` |
| `THYMOS_JWT_SECRET` | HS256 secret for JWT auth | — (auth disabled) |
| `THYMOS_API_KEYS` | API gateway keys (see format above) | — (gateway disabled) |
| `THYMOS_DB_PATH` | SQLite path for run persistence | `thymos-runs.db` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry collector endpoint | — (tracing disabled) |

## Docker

```bash
# Build + run the server (mock cognition, no API key)
docker compose up --build

# With an API key
ANTHROPIC_API_KEY=sk-ant-... docker compose up --build

# With observability stack (Jaeger + OTel collector)
docker compose --profile observability up --build

# With Postgres (for ledger development)
docker compose --profile postgres up --build
```

### Local models via Ollama

```bash
# Bring up Ollama alongside the server.
docker compose --profile local up --build

# Pull a model into the Ollama volume.
docker compose exec ollama ollama pull llama3

# Start a run against it.
curl -X POST http://localhost:3001/runs \
  -H 'content-type: application/json' \
  -d '{
    "task": "Set greeting to hello",
    "cognition": {
      "provider": "local",
      "base_url": "http://ollama:11434/v1",
      "model": "llama3"
    }
  }'
```

## Running Tests

```bash
# Full workspace
cargo test --workspace

# Single crate
cargo test -p thymos-core
cargo test -p thymos-server
```

## Cognition Providers

| Provider | Config | Requires |
|----------|--------|----------|
| `anthropic` | `{"provider": "anthropic", "model": "claude-opus-4-7"}` | `ANTHROPIC_API_KEY` |
| `openai` | `{"provider": "openai", "model": "gpt-4o"}` | `OPENAI_API_KEY` |
| `local` | `{"provider": "local", "base_url": "http://localhost:11434/v1"}` | Ollama/vLLM running |
| `mock` | `{"provider": "mock"}` | Nothing |

The mock provider always returns a `done` action after one turn — useful for
testing and development.
