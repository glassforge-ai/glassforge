//! Verify API: POST /verify — compare two scan artifacts.

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::error::api_error;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/verify", post(verify))
}

#[derive(Deserialize)]
struct VerifyRequest {
    before_scan_id: String,
    after_scan_id: String,
}

#[derive(Serialize)]
struct VerifyResponse {
    before_score: Option<i64>,
    after_score: Option<i64>,
    delta: i64,
    before_findings: Option<i64>,
    after_findings: Option<i64>,
    resolved: i64,
}

async fn verify(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, axum::response::Response> {
    let before = state.scan_repo.get(&body.before_scan_id).map_err(api_error)?;
    let after = state.scan_repo.get(&body.after_scan_id).map_err(api_error)?;

    if before.status != "completed" {
        return Err(api_error(gf_core::ForgeError::Validation(
            "before scan is not completed".into(),
        )));
    }
    if after.status != "completed" {
        return Err(api_error(gf_core::ForgeError::Validation(
            "after scan is not completed".into(),
        )));
    }

    let delta = after.score.unwrap_or(0) - before.score.unwrap_or(0);
    let resolved = before.finding_count.unwrap_or(0) - after.finding_count.unwrap_or(0);

    Ok(Json(VerifyResponse {
        before_score: before.score,
        after_score: after.score,
        delta,
        before_findings: before.finding_count,
        after_findings: after.finding_count,
        resolved,
    }))
}
