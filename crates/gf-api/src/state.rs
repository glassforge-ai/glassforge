//! Application state shared across handlers.

use gf_core::EventBus;
use gf_db::{DbPool, EventRepo, MigrationRunRepo, ScanRepo, SessionRepo};
use gf_persona::PersonaLoader;
use gf_process::BackendRegistry;
use gf_safety::{CircuitBreaker, CostTracker, RateLimiter};
use std::sync::Arc;

/// Shared state for the API server.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
    pub session_repo: Arc<SessionRepo>,
    pub event_repo: Arc<EventRepo>,
    pub scan_repo: Arc<ScanRepo>,
    pub migration_run_repo: Arc<MigrationRunRepo>,
    pub event_bus: Arc<EventBus>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub rate_limiter: Arc<RateLimiter>,
    pub cost_tracker: Arc<CostTracker>,
    pub backend_registry: Arc<BackendRegistry>,
    pub persona_loader: Arc<PersonaLoader>,
}

impl AppState {
    pub fn new(
        db: Arc<DbPool>,
        event_bus: Arc<EventBus>,
        circuit_breaker: Arc<CircuitBreaker>,
        rate_limiter: Arc<RateLimiter>,
        cost_tracker: Arc<CostTracker>,
        backend_registry: Arc<BackendRegistry>,
        persona_loader: Arc<PersonaLoader>,
    ) -> Result<Self, gf_core::ForgeError> {
        let conn = db.conn_arc()?;
        let session_repo = Arc::new(SessionRepo::new(conn.clone()));
        let event_repo = Arc::new(EventRepo::new(conn.clone()));
        let scan_repo = Arc::new(ScanRepo::new(conn.clone()));
        let migration_run_repo = Arc::new(MigrationRunRepo::new(conn));
        Ok(Self {
            db,
            session_repo,
            event_repo,
            scan_repo,
            migration_run_repo,
            event_bus,
            circuit_breaker,
            rate_limiter,
            cost_tracker,
            backend_registry,
            persona_loader,
        })
    }
}
