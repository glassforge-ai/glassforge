//! Session CRUD: GET/DELETE /api/v1/sessions, GET /api/v1/sessions/:id, GET /api/v1/sessions/:id/events

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use gf_core::ids::SessionId;
use gf_db::{Session, StoredEvent};

use crate::error::{api_error, parse_uuid};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions))
        .route("/sessions/{id}", get(get_session).delete(delete_session))
        .route("/sessions/{id}/events", get(get_session_events))
}

async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<Session>>, axum::response::Response> {
    let sessions = state.session_repo.list().map_err(api_error)?;
    Ok(Json(sessions))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    let session = state.session_repo.get(&session_id).map_err(api_error)?;
    Ok(Json(session))
}

async fn get_session_events(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<StoredEvent>>, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    // Verify session exists
    state.session_repo.get(&session_id).map_err(api_error)?;
    let events = state.event_repo.query_by_session(&session_id).map_err(api_error)?;
    Ok(Json(events))
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, axum::response::Response> {
    let session_id = SessionId(parse_uuid(&id)?);
    state.session_repo.delete(&session_id).map_err(api_error)?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
