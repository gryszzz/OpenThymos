//! Thymos CLI — interact with the Thymos server from the terminal.
//!
//! Usage:
//!     thymos run "Set greeting to hello"
//!     thymos run "Say hello" --model openai --max-steps 8
//!     thymos status <run-id>
//!     thymos stream <run-id>
//!     thymos world <run-id>
//!     thymos usage
//!     thymos health

use clap::{Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "thymos", about = "Thymos governed-cognition CLI")]
struct Cli {
    /// Server URL (default: http://localhost:3001).
    #[arg(long, env = "THYMOS_URL", default_value = "http://localhost:3001")]
    url: String,

    /// API key for authenticated requests.
    #[arg(long, env = "THYMOS_API_KEY")]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create and start a new agent run.
    Run {
        /// Task description.
        task: String,
        /// Maximum steps (default: 16).
        #[arg(long, default_value = "16")]
        max_steps: u32,
        /// Cognition provider: anthropic, openai, local, mock.
        #[arg(long, default_value = "anthropic")]
        provider: String,
        /// Model override.
        #[arg(long)]
        model: Option<String>,
        /// Tool scopes (comma-separated).
        #[arg(long)]
        scopes: Option<String>,
    },
    /// Get run status and summary.
    Status {
        /// Run ID.
        run_id: String,
    },
    /// Stream cognition events (tokens) in real-time.
    Stream {
        /// Run ID.
        run_id: String,
    },
    /// Get world state for a run.
    World {
        /// Run ID.
        run_id: String,
    },
    /// Show API gateway usage stats.
    Usage,
    /// Health check.
    Health,
    /// Approve or deny a pending proposal.
    Approve {
        /// Run ID.
        run_id: String,
        /// Approval channel name.
        channel: String,
        /// Deny instead of approve.
        #[arg(long)]
        deny: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::new();

    let result = match cli.command {
        Commands::Run {
            task,
            max_steps,
            provider,
            model,
            scopes,
        } => cmd_run(&client, &cli.url, cli.api_key.as_deref(), &task, max_steps, &provider, model, scopes).await,
        Commands::Status { run_id } => cmd_status(&client, &cli.url, cli.api_key.as_deref(), &run_id).await,
        Commands::Stream { run_id } => cmd_stream(&cli.url, &run_id).await,
        Commands::World { run_id } => cmd_world(&client, &cli.url, cli.api_key.as_deref(), &run_id).await,
        Commands::Usage => cmd_usage(&client, &cli.url, cli.api_key.as_deref()).await,
        Commands::Health => cmd_health(&client, &cli.url).await,
        Commands::Approve {
            run_id,
            channel,
            deny,
        } => cmd_approve(&client, &cli.url, cli.api_key.as_deref(), &run_id, &channel, !deny).await,
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn auth_headers(api_key: Option<&str>) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    if let Some(key) = api_key {
        headers.push(("Authorization".into(), format!("Bearer {key}")));
    }
    headers
}

async fn cmd_run(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    task: &str,
    max_steps: u32,
    provider: &str,
    model: Option<String>,
    scopes: Option<String>,
) -> Result<(), String> {
    let mut body = serde_json::json!({
        "task": task,
        "max_steps": max_steps,
        "cognition": {
            "provider": provider,
        },
    });
    if let Some(m) = model {
        body["cognition"]["model"] = serde_json::json!(m);
    }
    if let Some(s) = scopes {
        let scope_list: Vec<&str> = s.split(',').collect();
        body["tool_scopes"] = serde_json::json!(scope_list);
    }

    let mut req = client.post(format!("{url}/runs")).json(&body);
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;

    if status.is_success() || status.as_u16() == 202 {
        let run_id = body["run_id"].as_str().unwrap_or("unknown");
        println!("Run started: {run_id}");
        println!("  task: {task}");
        println!("  provider: {provider}");
        println!();
        println!("Poll status:  thymos status {run_id}");
        println!("Stream live:  thymos stream {run_id}");
    } else {
        println!("Error ({}): {}", status, serde_json::to_string_pretty(&body).unwrap());
    }
    Ok(())
}

async fn cmd_status(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    run_id: &str,
) -> Result<(), String> {
    let mut req = client.get(format!("{url}/runs/{run_id}"));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_stream(url: &str, run_id: &str) -> Result<(), String> {
    // Simple SSE client: read chunks from the stream endpoint.
    let resp = reqwest::get(format!("{url}/runs/{run_id}/stream"))
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        println!("Error: {}", serde_json::to_string_pretty(&body).unwrap());
        return Ok(());
    }

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Parse SSE lines.
        while let Some(pos) = buffer.find("\n\n") {
            let event_block = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            for line in event_block.lines() {
                if let Some(data) = line.strip_prefix("data:") {
                    let data = data.trim();
                    if let Ok(evt) = serde_json::from_str::<Value>(data) {
                        match evt["type"].as_str() {
                            Some("token") => {
                                print!("{}", evt["text"].as_str().unwrap_or(""));
                            }
                            Some("tool_use_start") => {
                                println!(
                                    "\n[tool: {}]",
                                    evt["tool"].as_str().unwrap_or("?")
                                );
                            }
                            Some("tool_use_done") => {
                                println!("[/tool]");
                            }
                            Some("turn_complete") => {
                                if let Some(answer) = evt["final_answer"].as_str() {
                                    println!("\n--- Final Answer ---");
                                    println!("{answer}");
                                }
                                println!("\n[turn complete: {} intents]", evt["intents_count"]);
                            }
                            Some("error") => {
                                eprintln!(
                                    "[error: {}]",
                                    evt["message"].as_str().unwrap_or("?")
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    println!();
    Ok(())
}

async fn cmd_world(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    run_id: &str,
) -> Result<(), String> {
    let mut req = client.get(format!("{url}/runs/{run_id}/world"));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_usage(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
) -> Result<(), String> {
    let mut req = client.get(format!("{url}/usage"));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_health(client: &reqwest::Client, url: &str) -> Result<(), String> {
    let resp = client
        .get(format!("{url}/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_approve(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    run_id: &str,
    channel: &str,
    approve: bool,
) -> Result<(), String> {
    let mut req = client
        .post(format!("{url}/runs/{run_id}/approvals/{channel}"))
        .json(&serde_json::json!({ "approve": approve }));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }

    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

// Needed for the streaming SSE client.
mod futures_util {
    pub use tokio_stream::StreamExt;
}
