//! `wasm-engine` — wasmtime-based runtime that executes WASM/WASI modules
//! dispatched by [`wasm-operator`](https://github.com/pleme-io/wasm-operator).
//!
//! Reads its configuration from environment variables populated by the
//! operator on the spawned pod:
//!
//! | Env var                   | Source                                |
//! |---------------------------|---------------------------------------|
//! | `PROGRAM_MODULE_URL`      | `Program.spec.module.source`          |
//! | `PROGRAM_MODULE_BLAKE3`   | `Program.spec.module.blake3` (opt.)   |
//! | `PROGRAM_TRIGGER`         | one of `oneShot|cron|service|watch|event` |
//! | `PROGRAM_CONFIG`          | JSON-encoded `Program.spec.config`    |
//! | `PROGRAM_CAPABILITIES`    | newline-separated capability tokens   |
//!
//! Skeleton-quality: dispatches by trigger; the `service` shape spins
//! an axum server on `:8080`, every other shape currently logs and
//! exits. Production wasmtime + WASI component wiring lands in M1.

mod capabilities;
mod url;
mod wasi;

use anyhow::Context;
use std::env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    let module_url = env::var("PROGRAM_MODULE_URL")
        .context("PROGRAM_MODULE_URL must be set by wasm-operator")?;
    let trigger = env::var("PROGRAM_TRIGGER").unwrap_or_else(|_| "oneShot".to_string());

    info!(
        version = env!("CARGO_PKG_VERSION"),
        module_url = %module_url,
        trigger = %trigger,
        "wasm-engine starting"
    );

    // Parse the module URL — proves the URL grammar dispatch wires.
    match crate::url::parse(&module_url) {
        Ok(src) => info!(?src, "parsed module source"),
        Err(e) => warn!(error = %e, "failed to parse PROGRAM_MODULE_URL"),
    }

    // Parse capability tokens.
    let caps_raw = env::var("PROGRAM_CAPABILITIES").unwrap_or_default();
    let caps: Vec<String> = caps_raw
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect();
    let caps_set = capabilities::CapabilitySet::from_iter(caps);
    info!(count = caps_set.len(), "loaded capabilities");

    match trigger.as_str() {
        "service" => run_service().await,
        "watch" | "event" => run_long_running(&trigger).await,
        "cron" | "oneShot" => run_one_shot(&trigger).await,
        other => {
            warn!(trigger = %other, "unknown trigger; defaulting to oneShot");
            run_one_shot(other).await
        }
    }
}

/// `service` shape — start the axum HTTP server. Engine listens on
/// `0.0.0.0:8080` by default; override via `PROGRAM_SERVICE_PORT`.
async fn run_service() -> anyhow::Result<()> {
    let port: u16 = env::var("PROGRAM_SERVICE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();
    info!(%addr, "starting axum HTTP listener (skeleton)");

    let app = wasi::router();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// `watch` / `event` shape — long-running. Skeleton just logs + sleeps.
async fn run_long_running(trigger: &str) -> anyhow::Result<()> {
    info!(trigger, "long-running shape (skeleton): would attach informer / event subscription");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        info!(trigger, "heartbeat");
    }
}

/// `cron` / `oneShot` shape — execute once and exit.
async fn run_one_shot(trigger: &str) -> anyhow::Result<()> {
    info!(trigger, "one-shot shape (skeleton): would execute the WASM module once and exit");
    Ok(())
}
