//! Health check endpoint.

use axum::{routing::get, Json, Router};
use serde::Serialize;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub cli_available: bool,
    pub cli_command: String,
}

static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

fn uptime_secs() -> u64 {
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs()
}

pub async fn health() -> Json<HealthResponse> {
    let cli_command = std::env::var("GF_CLI_COMMAND").unwrap_or_else(|_| "claude".into());
    let cli_available = std::process::Command::new(&cli_command)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    Json(HealthResponse {
        status: if cli_available { "ok".into() } else { "degraded".into() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: uptime_secs(),
        cli_available,
        cli_command,
    })
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
