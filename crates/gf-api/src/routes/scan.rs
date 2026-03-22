//! Scan API: POST /scans (trigger scan), GET /scans (list), GET /scans/:id

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/scans", post(create_scan).get(list_scans))
        .route("/scans/{id}", get(get_scan))
}

#[derive(Deserialize)]
struct CreateScanRequest {
    repo_path: String,
}

async fn create_scan(
    State(state): State<AppState>,
    Json(body): Json<CreateScanRequest>,
) -> Result<Json<gf_db::Scan>, axum::response::Response> {
    // Create pending scan record.
    let scan = state.scan_repo.create(&body.repo_path).map_err(api_error)?;
    let scan_id = scan.id.clone();
    let repo_path = body.repo_path.clone();

    // Update to running.
    state.scan_repo.update_status(&scan_id, "running").map_err(api_error)?;

    // Run scanner in background.
    let scan_repo = state.scan_repo.clone();
    let scanner_bin = find_scanner_binary();

    tokio::spawn(async move {
        match run_scanner_subprocess(&scanner_bin, &repo_path).await {
            Ok(artifact_json) => {
                // Parse score and finding count from artifact.
                let (score, finding_count) = extract_scan_summary(&artifact_json);
                if let Err(e) = scan_repo.update_completed(&scan_id, score, finding_count, &artifact_json) {
                    tracing::error!(error = %e, scan_id, "failed to update scan as completed");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, scan_id, "scan failed");
                if let Err(db_err) = scan_repo.update_failed(&scan_id, &e) {
                    tracing::error!(error = %db_err, scan_id, "failed to update scan as failed");
                }
            }
        }
    });

    // Return the scan immediately (pending/running).
    let scan = state.scan_repo.get(&scan.id).map_err(api_error)?;
    Ok(Json(scan))
}

async fn list_scans(
    State(state): State<AppState>,
) -> Result<Json<Vec<gf_db::Scan>>, axum::response::Response> {
    let scans = state.scan_repo.list().map_err(api_error)?;
    Ok(Json(scans))
}

async fn get_scan(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<gf_db::Scan>, axum::response::Response> {
    let scan = state.scan_repo.get(&id).map_err(api_error)?;
    Ok(Json(scan))
}

// ---------------------------------------------------------------------------
// Scanner subprocess
// ---------------------------------------------------------------------------

fn find_scanner_binary() -> String {
    // Try relative path first, then PATH.
    let relative = "scanner/.build/release/glassforge";
    if std::path::Path::new(relative).exists() {
        return relative.to_string();
    }
    // Try arm64 variant.
    let arm64 = "scanner/.build/arm64-apple-macosx/release/glassforge";
    if std::path::Path::new(arm64).exists() {
        return arm64.to_string();
    }
    // Fall back to PATH.
    "glassforge-scanner".to_string()
}

async fn run_scanner_subprocess(scanner_bin: &str, repo_path: &str) -> Result<String, String> {
    let output = tokio::process::Command::new(scanner_bin)
        .arg("analyze")
        .arg(repo_path)
        .arg("--output-dir")
        .arg("/tmp")
        .arg("--artifact-name")
        .arg(format!("gf-scan-{}.json", uuid::Uuid::new_v4()))
        .output()
        .await
        .map_err(|e| format!("failed to run scanner: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("scanner exited with {}: {}", output.status, stderr));
    }

    // The scanner writes the artifact to the specified output path.
    // We need to read it back. Parse stdout for the artifact path.
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Look for "Artifact written to" line.
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("Artifact written to ") {
            let path = path.trim();
            return tokio::fs::read_to_string(path)
                .await
                .map_err(|e| format!("failed to read artifact at {path}: {e}"));
        }
    }

    Err("scanner did not output artifact path".to_string())
}

fn extract_scan_summary(artifact_json: &str) -> (Option<i64>, Option<i64>) {
    let v: serde_json::Value = match serde_json::from_str(artifact_json) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };
    let score = v
        .get("readiness")
        .and_then(|r| r.get("score"))
        .and_then(|s| s.as_i64());

    // Count findings: deprecated_apis + liquidGlass findings.
    let dep_count = v
        .get("findings")
        .and_then(|f| f.get("deprecated_apis"))
        .and_then(|a| a.as_array())
        .map(|a| a.len() as i64)
        .unwrap_or(0);
    let lg_count = v
        .get("liquidGlass")
        .and_then(|lg| lg.get("findings"))
        .and_then(|a| a.as_array())
        .map(|a| a.len() as i64)
        .unwrap_or(0);
    let finding_count = Some(dep_count + lg_count);

    (score, finding_count)
}
