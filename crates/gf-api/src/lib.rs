//! GlassForge HTTP API: health, session CRUD, WebSocket event stream, embedded frontend.

pub mod error;
pub mod routes;
pub mod state;

pub use routes::router;
pub use state::AppState;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
    Router,
};
use http::{header::AUTHORIZATION, header::CONTENT_TYPE, Method, StatusCode};
use mime_guess::from_path;
use std::env;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

/// Embedded frontend assets (SvelteKit adapter-static output).
/// Path is relative to `crates/gf-api/Cargo.toml`.
/// Allow missing so the crate builds when frontend has not been built yet.
#[derive(rust_embed::RustEmbed)]
#[folder = "../../frontend/build"]
#[allow_missing = true]
struct FrontendAssets;

/// Build the application router with CORS, API routes, and embedded frontend fallback.
///
/// CORS origin: set `GF_CORS_ORIGIN` to a specific origin (e.g. `https://app.example.com`)
/// or leave unset for `*` (permissive, suitable for local dev).
pub fn app(state: AppState) -> Router {
    let cors_origin = env::var("GF_CORS_ORIGIN").unwrap_or_else(|_| "*".into());
    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];
    let headers = [CONTENT_TYPE, AUTHORIZATION];

    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(methods)
            .allow_headers(headers)
    } else {
        let origin = cors_origin
            .parse()
            .expect("GF_CORS_ORIGIN must be a valid HTTP header value");
        CorsLayer::new()
            .allow_origin(AllowOrigin::exact(origin))
            .allow_methods(methods)
            .allow_headers(headers)
    };

    Router::new()
        .nest("/api/v1", routes::router())
        .fallback(serve_embedded_fallback)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Serves embedded frontend files. Tries path, path/index.html, path.html; then SPA fallback.
async fn serve_embedded_fallback(request: Request) -> Response {
    if request.method() != Method::GET {
        return (StatusCode::METHOD_NOT_ALLOWED, Body::empty()).into_response();
    }

    let path = request.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    let candidates = [
        path.to_string(),
        format!("{}/index.html", path),
        format!("{}.html", path),
        "index.html".to_string(),
    ];

    for candidate in &candidates {
        if let Some(file) = FrontendAssets::get(candidate.as_str()) {
            let mime = from_path(candidate.as_str()).first_or_octet_stream();
            let value = match http::HeaderValue::try_from(mime.as_ref()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            return ([(CONTENT_TYPE, value)], file.data.to_vec()).into_response();
        }
    }

    // SPA fallback: serve index.html for client-side routing
    if let Some(index) = FrontendAssets::get("index.html") {
        let value = http::HeaderValue::from_static("text/html");
        return ([(CONTENT_TYPE, value)], index.data.to_vec()).into_response();
    }

    (StatusCode::NOT_FOUND, Body::empty()).into_response()
}

/// Run the server until Ctrl+C. Blocks until shutdown.
pub async fn serve_until_signal(app: Router, addr: SocketAddr) -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!(%local_addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received");
}
