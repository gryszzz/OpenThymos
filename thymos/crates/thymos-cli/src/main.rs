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
        /// Cognition provider: anthropic, openai, local, lmstudio, huggingface, mock.
        #[arg(long, default_value = "anthropic")]
        provider: String,
        /// Model override.
        #[arg(long)]
        model: Option<String>,
        /// Tool scopes (comma-separated).
        #[arg(long)]
        scopes: Option<String>,
        /// After starting the run, stream cognition events until it completes.
        #[arg(long, short = 'f')]
        follow: bool,
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
    /// Run history operations (list / show / diff).
    Runs {
        #[command(subcommand)]
        action: RunsAction,
    },
}

#[derive(Subcommand)]
enum RunsAction {
    /// List recent runs.
    Ls {
        /// Filter by status (running, completed, failed).
        #[arg(long)]
        status: Option<String>,
        /// Page size (default: 50, max 200).
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination.
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// Show full record + ledger entries for one run.
    Show {
        /// Run ID.
        run_id: String,
    },
    /// Diff the ledger entries of two runs (counts + final-answer compare).
    Diff {
        /// Source run ID.
        a: String,
        /// Target run ID.
        b: String,
    },
}

#[tokio::main]
async fn main() {
    // Load .env from CWD or any parent dir so THYMOS_URL / THYMOS_API_KEY
    // and provider tokens are picked up without needing to `source` manually.
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let client = reqwest::Client::new();

    let result = match cli.command {
        Commands::Run {
            task,
            max_steps,
            provider,
            model,
            scopes,
            follow,
        } => {
            cmd_run(
                &client,
                &cli.url,
                cli.api_key.as_deref(),
                &task,
                max_steps,
                &provider,
                model,
                scopes,
                follow,
            )
            .await
        }
        Commands::Status { run_id } => {
            cmd_status(&client, &cli.url, cli.api_key.as_deref(), &run_id).await
        }
        Commands::Stream { run_id } => cmd_stream(&cli.url, &run_id).await,
        Commands::World { run_id } => {
            cmd_world(&client, &cli.url, cli.api_key.as_deref(), &run_id).await
        }
        Commands::Usage => cmd_usage(&client, &cli.url, cli.api_key.as_deref()).await,
        Commands::Health => cmd_health(&client, &cli.url).await,
        Commands::Approve {
            run_id,
            channel,
            deny,
        } => {
            cmd_approve(
                &client,
                &cli.url,
                cli.api_key.as_deref(),
                &run_id,
                &channel,
                !deny,
            )
            .await
        }
        Commands::Runs { action } => match action {
            RunsAction::Ls {
                status,
                limit,
                offset,
            } => cmd_runs_ls(
                &client,
                &cli.url,
                cli.api_key.as_deref(),
                status.as_deref(),
                limit,
                offset,
            )
            .await,
            RunsAction::Show { run_id } => {
                cmd_runs_show(&client, &cli.url, cli.api_key.as_deref(), &run_id).await
            }
            RunsAction::Diff { a, b } => {
                cmd_runs_diff(&client, &cli.url, cli.api_key.as_deref(), &a, &b).await
            }
        },
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

async fn json_body_or_error(resp: reqwest::Response) -> Result<Value, String> {
    let status = resp.status();
    let body: Value = resp.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!(
            "HTTP {}: {}",
            status,
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| body.to_string())
        ));
    }
    Ok(body)
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
    follow: bool,
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
        let run_id = body["run_id"].as_str().unwrap_or("unknown").to_string();
        println!("Run started: {run_id}");
        println!("  task: {task}");
        println!("  provider: {provider}");
        println!();
        if follow {
            println!("--- streaming ---");
            cmd_stream(url, &run_id).await?;
            // Print final status once the stream closes.
            return cmd_status(client, url, api_key, &run_id).await;
        }
        println!("Poll status:  thymos status {run_id}");
        println!("Stream live:  thymos stream {run_id}");
    } else {
        println!(
            "Error ({}): {}",
            status,
            serde_json::to_string_pretty(&body).unwrap()
        );
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
    let body = json_body_or_error(resp).await?;
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
                                println!("\n[tool: {}]", evt["tool"].as_str().unwrap_or("?"));
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
                                eprintln!("[error: {}]", evt["message"].as_str().unwrap_or("?"));
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
    let body = json_body_or_error(resp).await?;
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
    let body = json_body_or_error(resp).await?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_health(client: &reqwest::Client, url: &str) -> Result<(), String> {
    let resp = client
        .get(format!("{url}/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let body = json_body_or_error(resp).await?;
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
    let body = json_body_or_error(resp).await?;
    println!("{}", serde_json::to_string_pretty(&body).unwrap());
    Ok(())
}

async fn cmd_runs_ls(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    status: Option<&str>,
    limit: u32,
    offset: u32,
) -> Result<(), String> {
    let mut endpoint = format!("{url}/runs?limit={limit}&offset={offset}");
    if let Some(s) = status {
        endpoint.push_str(&format!("&status={s}"));
    }
    let mut req = client.get(&endpoint);
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body = json_body_or_error(resp).await?;

    let runs = body["runs"].as_array().cloned().unwrap_or_default();
    let total = body["total"].as_u64().unwrap_or(0);
    if runs.is_empty() {
        println!("(no runs)");
        return Ok(());
    }
    println!(
        "{:<14}  {:<10}  {}",
        "RUN ID", "STATUS", "TASK"
    );
    for r in &runs {
        let id = r["run_id"].as_str().unwrap_or("?");
        let st = r["status"].as_str().unwrap_or("?");
        let task = r["task"].as_str().unwrap_or("");
        let id_short: String = id.chars().take(12).collect();
        let task_short: String = task.chars().take(56).collect();
        println!("{id_short:<14}  {st:<10}  {task_short}");
    }
    println!();
    println!("({} of {total} shown, offset {offset})", runs.len());
    Ok(())
}

async fn cmd_runs_show(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    run_id: &str,
) -> Result<(), String> {
    // Status block.
    let mut req = client.get(format!("{url}/runs/{run_id}"));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let rec = json_body_or_error(resp).await?;
    println!("=== Run {run_id} ===");
    println!("{}", serde_json::to_string_pretty(&rec).unwrap_or_default());

    // Audit entries.
    let mut req = client.get(format!("{url}/audit/entries?run_id={run_id}&limit=200"));
    for (k, v) in auth_headers(api_key) {
        req = req.header(&k, &v);
    }
    let resp = req.send().await.map_err(|e| e.to_string())?;
    let body = json_body_or_error(resp).await?;
    let entries = body["entries"].as_array().cloned().unwrap_or_default();
    println!("\n=== Ledger ({} entries) ===", entries.len());
    for e in &entries {
        let seq = e["seq"].as_u64().unwrap_or(0);
        let kind = e["kind"].as_str().unwrap_or("?");
        let id = e["id"].as_str().unwrap_or("");
        let id_short: String = id.chars().take(12).collect();
        println!("  #{seq:<4} {kind:<18} {id_short}");
    }
    Ok(())
}

async fn cmd_runs_diff(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    a: &str,
    b: &str,
) -> Result<(), String> {
    async fn summary(
        client: &reqwest::Client,
        url: &str,
        api_key: Option<&str>,
        run_id: &str,
    ) -> Result<(Value, Value), String> {
        let mut req = client.get(format!("{url}/runs/{run_id}"));
        for (k, v) in auth_headers(api_key) {
            req = req.header(&k, &v);
        }
        let rec = json_body_or_error(req.send().await.map_err(|e| e.to_string())?).await?;
        let mut req = client.get(format!("{url}/audit/entries?run_id={run_id}&limit=2000"));
        for (k, v) in auth_headers(api_key) {
            req = req.header(&k, &v);
        }
        let entries = json_body_or_error(req.send().await.map_err(|e| e.to_string())?).await?;
        Ok((rec, entries))
    }

    let (rec_a, ent_a) = summary(client, url, api_key, a).await?;
    let (rec_b, ent_b) = summary(client, url, api_key, b).await?;

    fn count_kind(entries: &Value, kind: &str) -> usize {
        entries["entries"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|e| e["kind"].as_str() == Some(kind))
                    .count()
            })
            .unwrap_or(0)
    }

    let commits_a = count_kind(&ent_a, "commit");
    let commits_b = count_kind(&ent_b, "commit");
    let rej_a = count_kind(&ent_a, "rejection");
    let rej_b = count_kind(&ent_b, "rejection");
    let final_a = rec_a["summary"]["final_answer"].as_str().unwrap_or("");
    let final_b = rec_b["summary"]["final_answer"].as_str().unwrap_or("");

    println!("           {:<22} {:<22} delta", a, b);
    println!(
        "status     {:<22} {:<22}",
        rec_a["status"].as_str().unwrap_or("?"),
        rec_b["status"].as_str().unwrap_or("?")
    );
    println!(
        "commits    {commits_a:<22} {commits_b:<22} {:+}",
        commits_b as i64 - commits_a as i64
    );
    println!(
        "rejections {rej_a:<22} {rej_b:<22} {:+}",
        rej_b as i64 - rej_a as i64
    );
    println!();
    if final_a == final_b {
        println!("final_answer: identical");
    } else {
        println!("final_answer DIFFERS:");
        println!("  a: {final_a}");
        println!("  b: {final_b}");
    }
    Ok(())
}

// Needed for the streaming SSE client.
mod futures_util {
    pub use tokio_stream::StreamExt;
}
