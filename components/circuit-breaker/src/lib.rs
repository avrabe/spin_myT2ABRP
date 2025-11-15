#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::circuit_breaker::breaker::{BreakerError, CircuitState, Guest};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

struct Component;

/// Configuration for circuit breaker behavior
#[derive(Debug, Clone, Copy)]
struct Config {
    /// Number of consecutive failures before opening circuit
    failure_threshold: u32,
    /// Seconds to wait before attempting half-open
    timeout_seconds: u64,
    /// Number of consecutive successes needed to close circuit from half-open
    success_threshold: u32,
}

/// Circuit breaker implementation
struct CircuitBreaker {
    config: Config,
    state: Mutex<CircuitBreakerState>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    current_state: InternalState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InternalState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    fn new(config: Config) -> Self {
        CircuitBreaker {
            config,
            state: Mutex::new(CircuitBreakerState {
                current_state: InternalState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: 0,
            }),
        }
    }

    fn can_attempt(&self) -> Result<(), BreakerError> {
        let mut state = self.state.lock().unwrap();
        let now = current_timestamp();

        match state.current_state {
            InternalState::Closed => Ok(()),
            InternalState::Open => {
                // Check if timeout has elapsed
                if now - state.last_failure_time >= self.config.timeout_seconds {
                    state.current_state = InternalState::HalfOpen;
                    state.success_count = 0;
                    Ok(())
                } else {
                    let remaining = self.config.timeout_seconds - (now - state.last_failure_time);
                    Err(BreakerError {
                        retry_after_seconds: remaining,
                    })
                }
            }
            InternalState::HalfOpen => Ok(()),
        }
    }

    fn record_success(&self) {
        let mut state = self.state.lock().unwrap();

        match state.current_state {
            InternalState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            InternalState::HalfOpen => {
                state.success_count += 1;

                if state.success_count >= self.config.success_threshold {
                    state.current_state = InternalState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                }
            }
            InternalState::Open => {
                // Shouldn't happen, but reset if it does
            }
        }
    }

    fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        let now = current_timestamp();

        match state.current_state {
            InternalState::Closed => {
                state.failure_count += 1;
                state.last_failure_time = now;

                if state.failure_count >= self.config.failure_threshold {
                    state.current_state = InternalState::Open;
                }
            }
            InternalState::HalfOpen => {
                state.current_state = InternalState::Open;
                state.failure_count = 1;
                state.success_count = 0;
                state.last_failure_time = now;
            }
            InternalState::Open => {
                // Already open, just update timestamp
                state.last_failure_time = now;
            }
        }
    }

    fn get_state(&self) -> CircuitState {
        let state = self.state.lock().unwrap();
        match state.current_state {
            InternalState::Closed => CircuitState::Closed,
            InternalState::Open => CircuitState::Open,
            InternalState::HalfOpen => CircuitState::HalfOpen,
        }
    }

    fn get_failure_count(&self) -> u32 {
        self.state.lock().unwrap().failure_count
    }
}

// Global circuit breaker instance
static BREAKER: OnceLock<CircuitBreaker> = OnceLock::new();

fn get_breaker() -> &'static CircuitBreaker {
    BREAKER.get_or_init(|| {
        // Default configuration
        let config = Config {
            failure_threshold: 5,
            timeout_seconds: 60,
            success_threshold: 2,
        };
        CircuitBreaker::new(config)
    })
}

impl Guest for Component {
    fn can_attempt() -> Result<(), BreakerError> {
        get_breaker().can_attempt()
    }

    fn record_success() {
        get_breaker().record_success()
    }

    fn record_failure() {
        get_breaker().record_failure()
    }

    fn get_state() -> CircuitState {
        get_breaker().get_state()
    }

    fn get_failure_count() -> u32 {
        get_breaker().get_failure_count()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_to_open() {
        let config = Config {
            failure_threshold: 3,
            timeout_seconds: 5,
            success_threshold: 2,
        };
        let breaker = CircuitBreaker::new(config);

        // Initial state is closed
        assert!(matches!(breaker.get_state(), CircuitState::Closed));
        assert!(breaker.can_attempt().is_ok());

        // Record failures
        breaker.record_failure();
        assert_eq!(breaker.get_failure_count(), 1);

        breaker.record_failure();
        breaker.record_failure();

        // Should now be open
        assert!(matches!(breaker.get_state(), CircuitState::Open));
        assert!(breaker.can_attempt().is_err());
    }

    #[test]
    fn test_circuit_breaker_success_resets_failures() {
        let config = Config {
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
        assert!(matches!(breaker.get_state(), CircuitState::Closed));
    }

    #[test]
    fn test_circuit_breaker_half_open_to_closed() {
        let config = Config {
            failure_threshold: 2,
            timeout_seconds: 1,
            success_threshold: 2,
        };
        let breaker = CircuitBreaker::new(config);

        // Open the circuit
        breaker.record_failure();
        breaker.record_failure();
        assert!(matches!(breaker.get_state(), CircuitState::Open));

        // Wait for timeout
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should transition to half-open
        assert!(breaker.can_attempt().is_ok());
        assert!(matches!(breaker.get_state(), CircuitState::HalfOpen));

        // Record successes to close
        breaker.record_success();
        breaker.record_success();
        assert!(matches!(breaker.get_state(), CircuitState::Closed));
    }
}
