//! Thymos server binary.
//!
//! Run:
//!     cargo run -p thymos-server
//! or with an LLM:
//!     ANTHROPIC_API_KEY=sk-ant-... cargo run -p thymos-server
//!
//! Then:
//!     curl -X POST http://localhost:3001/runs \
//!       -H 'content-type: application/json' \
//!       -d '{"task": "Set greeting to hello and read it back"}'

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use thymos_server::{app, auth, default_runtime, persistent_runtime, middleware, run_store, telemetry, AppState};

#[tokio::main]
async fn main() {
    telemetry::init();

    let runtime = if let Ok(path) = std::env::var("THYMOS_LEDGER_PATH") {
        eprintln!("ledger: {path}");
        persistent_runtime(&path)
    } else {
        eprintln!("ledger: in-memory (set THYMOS_LEDGER_PATH for persistence)");
        default_runtime()
    };

    // Optional: configure API gateway from environment.
    let gateway = if std::env::var("THYMOS_API_KEYS").is_ok() {
        let mut gw = middleware::ApiGateway::new();
        if let Ok(keys_str) = std::env::var("THYMOS_API_KEYS") {
            for entry in keys_str.split(',') {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() >= 4 {
                    gw.add_key(middleware::ApiKey {
                        key: parts[0].to_string(),
                        tenant_id: parts[1].to_string(),
                        name: parts[2].to_string(),
                        rate_limit_rpm: parts[3].parse().unwrap_or(60),
                    });
                }
            }
        }
        eprintln!("API gateway enabled");
        Some(Arc::new(gw))
    } else {
        None
    };

    // Persistent run store. Use THYMOS_DB_PATH or default to ./thymos-runs.db.
    let db_path = std::env::var("THYMOS_DB_PATH")
        .unwrap_or_else(|_| "thymos-runs.db".into());
    let run_store = match run_store::RunStore::open(&db_path) {
        Ok(store) => {
            eprintln!("run store: {db_path}");
            Some(Arc::new(store))
        }
        Err(e) => {
            eprintln!("warn: failed to open run store at {db_path}: {e}");
            None
        }
    };

    // Restore previously persisted runs into memory.
    let mut restored_runs = HashMap::new();
    if let Some(store) = &run_store {
        if let Ok(all) = store.load_all() {
            eprintln!("restored {} runs from disk", all.len());
            for (id, rec) in all {
                restored_runs.insert(id, rec);
            }
        }
    }

    // Optional: JWT auth from THYMOS_JWT_SECRET.
    let jwt_config = std::env::var("THYMOS_JWT_SECRET").ok().map(|secret| {
        eprintln!("JWT auth enabled");
        Arc::new(auth::JwtConfig::from_secret(secret.as_bytes()))
    });

    let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);

    let state = Arc::new(AppState {
        runtime,
        runs: Mutex::new(restored_runs),
        event_channels: Mutex::new(HashMap::new()),
        cognition_channels: Mutex::new(HashMap::new()),
        gateway,
        jwt_config,
        pending_approvals: Mutex::new(HashMap::new()),
        cancellation_tokens: Mutex::new(HashMap::new()),
        run_store,
        shutdown_tx,
        active_runs: AtomicU32::new(0),
        marketplace: Arc::new(Mutex::new(thymos_marketplace::Marketplace::new())),
    });

    let app = app(state.clone());
    let addr = "0.0.0.0:3001";
    eprintln!("thymos-server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");

    // Graceful shutdown: listen for SIGTERM/SIGINT.
    let state_for_shutdown = state.clone();
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let ctrl_c = tokio::signal::ctrl_c();
            #[cfg(unix)]
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("install SIGTERM handler");
            #[cfg(unix)]
            let terminate = sigterm.recv();
            #[cfg(not(unix))]
            let terminate = std::future::pending::<Option<()>>();

            tokio::select! {
                _ = ctrl_c => eprintln!("\nreceived SIGINT, shutting down..."),
                _ = terminate => eprintln!("\nreceived SIGTERM, shutting down..."),
            }

            // Signal all run handlers to stop accepting new work.
            let _ = state_for_shutdown.shutdown_tx.send(true);

            // Wait for active runs to drain (up to 30 seconds).
            let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
            loop {
                let active = state_for_shutdown.active_runs.load(Ordering::Relaxed);
                if active == 0 {
                    eprintln!("all runs drained, exiting cleanly");
                    break;
                }
                if tokio::time::Instant::now() >= deadline {
                    eprintln!("shutdown timeout: {} runs still active, forcing exit", active);
                    // Mark running runs as failed so they can be resumed.
                    let mut runs = state_for_shutdown.runs.lock().unwrap();
                    for (_id, rec) in runs.iter_mut() {
                        if rec.status == thymos_server::RunStatus::Running {
                            rec.status = thymos_server::RunStatus::Failed;
                        }
                    }
                    break;
                }
                eprintln!("waiting for {} active run(s) to complete...", active);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        })
        .await
        .expect("server failed");
}
