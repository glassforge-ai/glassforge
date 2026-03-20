//! Safety controls: circuit breaker, rate limiter, budget enforcement.

pub mod scanner;

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Lock a mutex, falling back to the poisoned inner value.
fn lock_or_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Serializable snapshot of circuit breaker state for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub state: String,
    pub failure_count: u32,
    pub last_failure_epoch_ms: Option<u64>,
}

/// Circuit breaker error when the circuit is open.
#[derive(Debug, Clone)]
pub struct CircuitBreakerError;

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "circuit breaker open")
    }
}

impl std::error::Error for CircuitBreakerError {}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum CircuitState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

/// 3-state circuit breaker: Closed -> Open -> HalfOpen -> Closed.
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure: Mutex<Option<Instant>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new(5, 2, Duration::from_secs(60))
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: AtomicU8::new(CircuitState::Closed as u8),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure: Mutex::new(None),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    pub fn check(&self) -> Result<(), CircuitBreakerError> {
        match self.state() {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                let mut last = lock_or_recover(&self.last_failure);
                if let Some(instant) = *last {
                    if instant.elapsed() >= self.timeout {
                        *last = None;
                        self.state.store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                        self.success_count.store(0, Ordering::SeqCst);
                        return Ok(());
                    }
                }
                Err(CircuitBreakerError)
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    pub fn record_success(&self) {
        match self.state() {
            CircuitState::Closed => {}
            CircuitState::Open => {}
            CircuitState::HalfOpen => {
                let prev = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if prev >= self.success_threshold {
                    self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn record_failure(&self) {
        match self.state() {
            CircuitState::Closed => {
                let prev = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                if prev >= self.failure_threshold {
                    self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                    *lock_or_recover(&self.last_failure) = Some(Instant::now());
                }
            }
            CircuitState::Open => {}
            CircuitState::HalfOpen => {
                self.state.store(CircuitState::Open as u8, Ordering::SeqCst);
                *lock_or_recover(&self.last_failure) = Some(Instant::now());
            }
        }
    }

    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    pub fn export_state(&self) -> CircuitBreakerState {
        let state_name = match self.state() {
            CircuitState::Closed => "Closed",
            CircuitState::Open => "Open",
            CircuitState::HalfOpen => "HalfOpen",
        };
        let last_failure_ms = lock_or_recover(&self.last_failure)
            .map(|i| i.elapsed().as_millis() as u64);
        CircuitBreakerState {
            state: state_name.to_string(),
            failure_count: self.failure_count.load(Ordering::SeqCst),
            last_failure_epoch_ms: last_failure_ms,
        }
    }

    pub fn restore_state(&self, saved: &CircuitBreakerState) {
        let state_val = match saved.state.as_str() {
            "Open" => CircuitState::Open as u8,
            "HalfOpen" => CircuitState::HalfOpen as u8,
            _ => CircuitState::Closed as u8,
        };
        self.state.store(state_val, Ordering::SeqCst);
        self.failure_count.store(saved.failure_count, Ordering::SeqCst);

        if state_val == CircuitState::Open as u8 {
            if let Some(ms_ago) = saved.last_failure_epoch_ms {
                let elapsed = Duration::from_millis(ms_ago);
                if elapsed < self.timeout {
                    *lock_or_recover(&self.last_failure) =
                        Some(Instant::now() - (self.timeout - elapsed));
                } else {
                    self.state.store(CircuitState::HalfOpen as u8, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }

    pub fn reset(&self) {
        self.state.store(CircuitState::Closed as u8, Ordering::SeqCst);
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        *lock_or_recover(&self.last_failure) = None;
    }
}

/// Token-bucket rate limiter.
pub struct RateLimiter {
    tokens: AtomicU32,
    max_tokens: u32,
    refill_interval: Duration,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_interval: Duration) -> Self {
        Self {
            tokens: AtomicU32::new(max_tokens),
            max_tokens,
            refill_interval,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    pub fn try_acquire(&self) -> bool {
        let mut last = lock_or_recover(&self.last_refill);
        let now = Instant::now();
        let elapsed = last.elapsed();
        if elapsed >= self.refill_interval && self.refill_interval.as_nanos() > 0 {
            let refills = (elapsed.as_nanos() / self.refill_interval.as_nanos()) as u32;
            *last = now;
            drop(last);
            let current = self.tokens.load(Ordering::SeqCst);
            let added = refills.min(self.max_tokens.saturating_sub(current));
            self.tokens.fetch_add(added, Ordering::SeqCst);
        }
        self.tokens
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |t| {
                if t > 0 { Some(t - 1) } else { None }
            })
            .is_ok()
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_tokens", &self.max_tokens)
            .field("refill_interval", &self.refill_interval)
            .finish_non_exhaustive()
    }
}

/// Result of a budget check.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetStatus {
    Ok,
    Warning { current_cost: f64, threshold: f64 },
    Exceeded { current_cost: f64, limit: f64 },
}

/// Tracks cost/budget for agent usage.
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    warn: Option<f64>,
    limit: Option<f64>,
}

impl CostTracker {
    pub fn new(warn: Option<f64>, limit: Option<f64>) -> Self {
        Self { warn, limit }
    }

    pub fn check(&self, cost: f64) -> BudgetStatus {
        if let Some(limit) = self.limit {
            if cost >= limit {
                return BudgetStatus::Exceeded { current_cost: cost, limit };
            }
        }
        if let Some(warn) = self.warn {
            if cost >= warn {
                return BudgetStatus::Warning { current_cost: cost, threshold: warn };
            }
        }
        BudgetStatus::Ok
    }

    pub fn warn_threshold(&self) -> Option<f64> {
        self.warn
    }

    pub fn limit_threshold(&self) -> Option<f64> {
        self.limit
    }
}
