//! Map ForgeError to HTTP response and shared error helpers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use gf_core::error::ForgeError;

/// Convert ForgeError to an HTTP response.
pub fn api_error(e: ForgeError) -> Response {
    let status = e.http_status();
    if status.is_server_error() {
        tracing::error!(
            http_status = %status,
            error_code = e.error_code(),
            error = %e,
            "API server error"
        );
    }
    let body = serde_json::json!({
        "error": e.to_string(),
        "code": e.error_code(),
        "retriable": e.is_retriable(),
    });
    (status, Json(body)).into_response()
}

/// Parse a UUID string, returning a 400 BAD_REQUEST response on failure.
#[allow(clippy::result_large_err)]
pub fn parse_uuid(s: &str) -> Result<uuid::Uuid, Response> {
    uuid::Uuid::parse_str(s).map_err(|_| {
        let body = serde_json::json!({
            "error": "invalid uuid",
            "code": "VALIDATION_ERROR",
            "retriable": false,
        });
        (StatusCode::BAD_REQUEST, Json(body)).into_response()
    })
}
