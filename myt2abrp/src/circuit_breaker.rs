// Circuit Breaker pattern implementation for resilient API calls
//
// Protects against cascading failures when downstream services are unavailable.
// Implements the classic three-state circuit breaker: Closed → Open → Half-Open

use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests are allowed
    Closed,
    /// Too many failures - requests fail fast without trying
    Open,
    /// Testing if service recovered - limited requests allowed
    HalfOpen,
}

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,
    /// Seconds to wait before attempting half-open
    pub timeout_seconds: u64,
    /// Number of consecutive successes needed to close circuit from half-open
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        CircuitBreakerConfig {
            failure_threshold: 5, // Open after 5 failures
            timeout_seconds: 60,  // Wait 60 seconds before retry
            success_threshold: 2, // Close after 2 successes
        }
    }
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Mutex<CircuitBreakerState>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    current_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: u64,
}

impl CircuitBreaker {
    fn new(config: CircuitBreakerConfig) -> Self {
        CircuitBreaker {
            config,
            state: Mutex::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: 0,
            }),
        }
    }

    /// Check if a request should be allowed through the circuit breaker
    pub fn can_attempt(&self) -> Result<(), CircuitBreakerError> {
        let mut state = self.state.lock().unwrap();
        let now = current_timestamp();

        match state.current_state {
            CircuitState::Closed => {
                debug!("Circuit breaker: CLOSED - allowing request");
                Ok(())
            }
            CircuitState::Open => {
                // Check if timeout has elapsed
                if now - state.last_failure_time >= self.config.timeout_seconds {
                    info!(
                        "Circuit breaker: OPEN → HALF_OPEN (timeout elapsed: {} seconds)",
                        now - state.last_failure_time
                    );
                    state.current_state = CircuitState::HalfOpen;
                    state.success_count = 0;
                    Ok(())
                } else {
                    let remaining = self.config.timeout_seconds - (now - state.last_failure_time);
                    debug!(
                        "Circuit breaker: OPEN - rejecting request (retry in {} seconds)",
                        remaining
                    );
                    Err(CircuitBreakerError::Open {
                        retry_after_seconds: remaining,
                    })
                }
            }
            CircuitState::HalfOpen => {
                debug!("Circuit breaker: HALF_OPEN - allowing test request");
                Ok(())
            }
        }
    }

    /// Record a successful call
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();

        match state.current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                if state.failure_count > 0 {
                    debug!(
                        "Circuit breaker: CLOSED - success recorded, resetting failure count from {}",
                        state.failure_count
                    );
                    state.failure_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                debug!(
                    "Circuit breaker: HALF_OPEN - success {}/{}",
                    state.success_count, self.config.success_threshold
                );

                if state.success_count >= self.config.success_threshold {
                    info!("Circuit breaker: HALF_OPEN → CLOSED (service recovered)");
                    state.current_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset if it does
                warn!("Circuit breaker: Unexpected success in OPEN state");
            }
        }
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let now = current_timestamp();

        match state.current_state {
            CircuitState::Closed => {
                state.failure_count += 1;
                state.last_failure_time = now;

                debug!(
                    "Circuit breaker: CLOSED - failure {}/{}",
                    state.failure_count, self.config.failure_threshold
                );

                if state.failure_count >= self.config.failure_threshold {
                    error!(
                        "Circuit breaker: CLOSED → OPEN (threshold reached: {} failures)",
                        state.failure_count
                    );
                    state.current_state = CircuitState::Open;
                    // Increment metrics if available
                    if let Some(metrics) = try_get_metrics() {
                        metrics.record_circuit_breaker_open();
                    }
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker: HALF_OPEN → OPEN (test request failed)");
                state.current_state = CircuitState::Open;
                state.failure_count = 1; // Reset count
                state.success_count = 0;
                state.last_failure_time = now;
            }
            CircuitState::Open => {
                // Already open, just update timestamp
                state.last_failure_time = now;
            }
        }
    }

    /// Get current circuit breaker state (test helper)
    #[cfg(test)]
    pub fn get_state(&self) -> CircuitState {
        self.state.lock().unwrap().current_state
    }

    /// Get current failure count (test helper)
    #[cfg(test)]
    pub fn get_failure_count(&self) -> u32 {
        self.state.lock().unwrap().failure_count
    }
}

/// Errors from circuit breaker
#[derive(Debug)]
pub enum CircuitBreakerError {
    Open { retry_after_seconds: u64 },
}

impl std::fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::Open {
                retry_after_seconds,
            } => {
                write!(
                    f,
                    "Circuit breaker is OPEN. Service unavailable. Retry after {} seconds.",
                    retry_after_seconds
                )
            }
        }
    }
}

impl std::error::Error for CircuitBreakerError {}

// Global circuit breaker instance for Toyota API
static TOYOTA_API_BREAKER: OnceLock<CircuitBreaker> = OnceLock::new();

/// Get the global Toyota API circuit breaker
pub fn toyota_api_breaker() -> &'static CircuitBreaker {
    TOYOTA_API_BREAKER.get_or_init(|| {
        info!("Initializing Toyota API circuit breaker with default config");
        CircuitBreaker::new(CircuitBreakerConfig::default())
    })
}

// Helper to get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Helper to try getting metrics (avoid circular dependency)
fn try_get_metrics() -> Option<&'static crate::metrics::Metrics> {
    Some(crate::metrics::metrics())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_to_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_seconds: 5,
            success_threshold: 2,
        };
        let breaker = CircuitBreaker::new(config);

        // Initial state is closed
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert!(breaker.can_attempt().is_ok());

        // Record failures
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
        assert_eq!(breaker.get_failure_count(), 1);

        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Closed);

        breaker.record_failure();
        // Should now be open
        assert_eq!(breaker.get_state(), CircuitState::Open);
        assert!(breaker.can_attempt().is_err());
    }

    #[test]
    fn test_circuit_breaker_success_resets_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_seconds: 5,
            success_threshold: 2,
        };
        let breaker = CircuitBreaker::new(config);

        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.get_failure_count(), 2);

        breaker.record_success();
        assert_eq!(breaker.get_failure_count(), 0);
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_seconds: 1, // Short timeout for testing
            success_threshold: 2,
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.get_state(), CircuitState::Open);

        // Wait for timeout (simulate with manual state change for testing)
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should transition to half-open
        assert!(breaker.can_attempt().is_ok());
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);

        // Record successes to close
        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitState::HalfOpen);

        breaker.record_success();
        assert_eq!(breaker.get_state(), CircuitState::Closed);
    }
}
