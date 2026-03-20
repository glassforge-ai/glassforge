//! Error hierarchy and result type.

use http::StatusCode;
use thiserror::Error;

/// Stratified error types for the platform.
#[derive(Debug, Error)]
pub enum ForgeError {
    // -- Client errors (4xx) --
    #[error("validation error: {0}")]
    Validation(String),

    #[error("{entity} not found: {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    // -- Domain errors (4xx) --
    #[error("budget exceeded: cost {cost:.2} >= limit {limit:.2}")]
    BudgetExceeded { cost: f64, limit: f64 },

    #[error("approval required: {approval_type}")]
    ApprovalRequired { approval_type: String },

    #[error("rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    // -- Retriable errors (5xx) --
    #[error("CLI unavailable: {0}")]
    CliUnavailable(String),

    #[error("circuit breaker open, resets in {reset_in_ms}ms")]
    CircuitOpen { reset_in_ms: u64 },

    #[error("operation timed out: {0}")]
    Timeout(String),

    // -- System errors (5xx) --
    #[error("database error: {0}")]
    Database(Box<dyn std::error::Error + Send + Sync>),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("process error: {0}")]
    Process(String),
}

impl ForgeError {
    /// Whether the client should retry this request.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::CliUnavailable(_)
                | Self::CircuitOpen { .. }
                | Self::Timeout(_)
                | Self::RateLimited { .. }
        )
    }

    /// Map to HTTP status code.
    pub fn http_status(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BudgetExceeded { .. } => StatusCode::PAYMENT_REQUIRED,
            Self::ApprovalRequired { .. } => StatusCode::FORBIDDEN,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::CliUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::CircuitOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Process(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Machine-readable error code for API consumers.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound { .. } => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::BudgetExceeded { .. } => "BUDGET_EXCEEDED",
            Self::ApprovalRequired { .. } => "APPROVAL_REQUIRED",
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::CliUnavailable(_) => "CLI_UNAVAILABLE",
            Self::CircuitOpen { .. } => "CIRCUIT_OPEN",
            Self::Timeout(_) => "TIMEOUT",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Process(_) => "PROCESS_ERROR",
        }
    }
}

pub type ForgeResult<T> = Result<T, ForgeError>;
