//! WASI HTTP server boilerplate (component-model).
//!
//! For `service`-shape Programs, the engine starts an axum HTTP server
//! that forwards inbound requests to the WASM component's
//! `wasi:http/incoming-handler` export. For `controller`/`event`/`job`
//! shapes, this module is unused.
//!
//! Skeleton: returns a tiny axum router that responds 200 OK with a
//! JSON envelope describing the engine state. Real component dispatch
//! lands in M1 alongside the wasmtime + wasmtime-wasi wiring.

use axum::{response::Json, routing::get, Router};
use serde_json::json;

#[must_use]
pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "engine": "wasm-engine",
        "version": env!("CARGO_PKG_VERSION"),
        "module_url": std::env::var("PROGRAM_MODULE_URL").unwrap_or_default(),
        "trigger": std::env::var("PROGRAM_TRIGGER").unwrap_or_default(),
        "status": "skeleton-ready",
    }))
}

async fn healthz() -> &'static str { "ok" }
async fn readyz()  -> &'static str { "ready" }
