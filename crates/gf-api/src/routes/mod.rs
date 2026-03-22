//! API route handlers.

pub mod health;
pub mod migrate;
pub mod scan;
pub mod sessions;
pub mod verify;
pub mod ws;

use axum::Router;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(scan::routes())
        .merge(migrate::routes())
        .merge(verify::routes())
        .merge(sessions::routes())
        .merge(ws::routes())
}
