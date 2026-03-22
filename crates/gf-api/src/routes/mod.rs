//! API route handlers.

pub mod health;
pub mod sessions;
pub mod ws;

use axum::Router;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(sessions::routes())
        .merge(ws::routes())
}
