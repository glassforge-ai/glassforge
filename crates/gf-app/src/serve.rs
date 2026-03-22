//! `glassforge serve` — start the HTTP server with embedded SvelteKit frontend.
//!
//! Bootstraps the full runtime: DB, migrations, EventBus, BatchWriter,
//! safety controls, backend registry, and persona loader. When gf-api is
//! implemented it will provide the real router; until then a minimal
//! placeholder Axum app is used.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use gf_core::EventBus;
use gf_db::{BatchWriter, DbPool, Migrator};
use gf_persona::PersonaLoader;
use gf_process::{BackendRegistry, ClaudeBackend};
use gf_safety::{CircuitBreaker, CostTracker, RateLimiter};
use tracing::info;

/// Default database path: `~/.glassforge/glassforge.db`.
fn default_db_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(format!("{}/.glassforge/glassforge.db", home))
}

/// Default personas directory: `personas/` relative to cwd.
fn default_personas_dir() -> PathBuf {
    PathBuf::from("personas")
}

/// Entry-point for `glassforge serve`.
pub async fn cmd_serve(host: &str, port: u16, personas_dir: Option<&str>) -> ExitCode {
    // -----------------------------------------------------------------------
    // 1. Database
    // -----------------------------------------------------------------------
    let db_path = env::var("GLASSFORGE_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_db_path());

    if let Some(parent) = db_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Error: cannot create database directory {}: {e}", parent.display());
            return ExitCode::from(1);
        }
    }

    info!(path = %db_path.display(), "opening database");
    let db = match DbPool::new(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error: failed to open database: {e}");
            return ExitCode::from(1);
        }
    };

    // Run migrations.
    {
        let conn = match db.connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: failed to get database connection: {e}");
                return ExitCode::from(1);
            }
        };
        let migrator = Migrator::new(&conn);
        match migrator.apply_pending() {
            Ok(applied) => {
                if applied > 0 {
                    info!(count = applied, "migrations applied");
                }
            }
            Err(e) => {
                eprintln!("Error: migration failed: {e}");
                return ExitCode::from(1);
            }
        }
    }

    let db = Arc::new(db);

    // -----------------------------------------------------------------------
    // 2. EventBus
    // -----------------------------------------------------------------------
    let (event_bus, mut persist_rx) = EventBus::new(1024, 256);
    let event_bus = Arc::new(event_bus);

    // -----------------------------------------------------------------------
    // 3. BatchWriter — persist events from the mpsc channel
    // -----------------------------------------------------------------------
    let conn_arc = match db.conn_arc() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: failed to create batch writer connection: {e}");
            return ExitCode::from(1);
        }
    };
    let batch_writer = Arc::new(BatchWriter::spawn(conn_arc));
    let bw = Arc::clone(&batch_writer);
    tokio::spawn(async move {
        while let Some(event) = persist_rx.recv().await {
            if let Err(e) = bw.write(event) {
                tracing::warn!(error = %e, "batch writer: failed to queue event");
            }
        }
        info!("persistence channel closed, stopping event persistence");
    });
    info!("event persistence wired (BatchWriter <- mpsc guaranteed channel)");

    // -----------------------------------------------------------------------
    // 4. Safety controls
    // -----------------------------------------------------------------------
    let rate_limit_max: u32 = env::var("GLASSFORGE_RATE_LIMIT_MAX")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let rate_limit_refill_ms: u64 = env::var("GLASSFORGE_RATE_LIMIT_REFILL_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000);

    let rate_limiter = Arc::new(RateLimiter::new(
        rate_limit_max,
        std::time::Duration::from_millis(rate_limit_refill_ms),
    ));
    let circuit_breaker = Arc::new(CircuitBreaker::default());

    let budget_warn: Option<f64> = env::var("GLASSFORGE_BUDGET_WARN")
        .ok()
        .and_then(|s| s.parse().ok());
    let budget_limit: Option<f64> = env::var("GLASSFORGE_BUDGET_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok());
    let cost_tracker = Arc::new(CostTracker::new(budget_warn, budget_limit));

    // -----------------------------------------------------------------------
    // 5. Backend registry
    // -----------------------------------------------------------------------
    let mut backend_registry = BackendRegistry::new("claude");
    backend_registry.register(Box::new(ClaudeBackend::new()));
    let backend_registry = Arc::new(backend_registry);

    // -----------------------------------------------------------------------
    // 6. Persona loader
    // -----------------------------------------------------------------------
    let personas_path = personas_dir
        .map(PathBuf::from)
        .unwrap_or_else(default_personas_dir);
    let persona_loader = if personas_path.is_dir() {
        match PersonaLoader::load_from_dir(&personas_path) {
            Ok(loader) => {
                info!(count = loader.all().len(), dir = %personas_path.display(), "personas loaded");
                Arc::new(loader)
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load personas, continuing with empty set");
                Arc::new(PersonaLoader::load_from_dir(std::path::Path::new("/dev/null"))
                    .unwrap_or_else(|_| PersonaLoader::empty()))
            }
        }
    } else {
        info!(dir = %personas_path.display(), "personas directory not found, skipping");
        Arc::new(PersonaLoader::empty())
    };

    // -----------------------------------------------------------------------
    // 7. Build AppState + Axum app via gf-api
    // -----------------------------------------------------------------------
    let state = match gf_api::AppState::new(
        Arc::clone(&db),
        Arc::clone(&event_bus),
        Arc::clone(&circuit_breaker),
        Arc::clone(&rate_limiter),
        Arc::clone(&cost_tracker),
        Arc::clone(&backend_registry),
        persona_loader,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: failed to create app state: {e}");
            return ExitCode::from(1);
        }
    };

    let app = gf_api::app(state);

    // -----------------------------------------------------------------------
    // 8. Start server
    // -----------------------------------------------------------------------
    let addr: SocketAddr = match format!("{host}:{port}").parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: invalid bind address {host}:{port}: {e}");
            return ExitCode::from(1);
        }
    };

    info!(%addr, "GlassForge server listening");
    println!("GlassForge server listening on http://{addr}");

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error: failed to bind {addr}: {e}");
            return ExitCode::from(1);
        }
    };

    // Serve until Ctrl+C.
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        eprintln!("Error: server exited with error: {e}");
    }

    // -----------------------------------------------------------------------
    // 9. Graceful shutdown
    // -----------------------------------------------------------------------
    info!("shutting down");

    // Flush the batch writer.
    match Arc::try_unwrap(batch_writer) {
        Ok(bw) => {
            if let Err(e) = bw.shutdown() {
                tracing::warn!(error = %e, "batch writer shutdown error");
            }
        }
        Err(arc) => {
            tracing::warn!("batch writer: extra refs held, dropping");
            drop(arc);
        }
    }

    info!("shutdown complete");
    ExitCode::SUCCESS
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    info!("shutdown signal received");
}
