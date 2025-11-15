// Retry Strategy Utilities
//
// Pure retry logic for exponential backoff and retry decision making.
// Zero dependencies on Spin SDK or any platform-specific APIs.

#[allow(warnings)]
mod bindings;

use bindings::exports::toyota::retry::strategy::{Guest, RetryConfig};

// =============================================================================
// Constants
// =============================================================================

const DEFAULT_MAX_ATTEMPTS: u32 = 3;
const DEFAULT_INITIAL_DELAY_MS: u64 = 100;
const DEFAULT_MAX_DELAY_MS: u64 = 10_000; // 10 seconds
const DEFAULT_MULTIPLIER: f64 = 2.0;

// =============================================================================
// Retry Logic Functions
// =============================================================================

/// Calculate exponential backoff delay for a given attempt
fn calculate_backoff_internal(attempt: u32, config: &RetryConfig) -> u64 {
    if attempt == 0 {
        return 0;
    }

    // Calculate exponential delay: initial_delay * multiplier^(attempt-1)
    let exponent = (attempt - 1) as f64;
    let delay = config.initial_delay_ms as f64 * config.multiplier.powf(exponent);

    // Cap at max_delay_ms
    delay.min(config.max_delay_ms as f64) as u64
}

/// Determine if a request should be retried
fn should_retry_internal(error_message: &str, attempt: u32, max_attempts: u32) -> bool {
    // Check if we've exceeded max attempts
    if attempt >= max_attempts {
        return false;
    }

    // Don't retry if circuit breaker is open
    if error_message.contains("Circuit breaker is OPEN")
        || error_message.contains("circuit breaker")
    {
        return false;
    }

    // Don't retry client errors (4xx)
    if error_message.contains("HTTP 4") || error_message.contains("400-series") {
        return false;
    }

    // Retry server errors (5xx), network errors, timeouts
    true
}

/// Check if an HTTP status code is retryable
fn is_retryable_status_internal(status_code: u32) -> bool {
    match status_code {
        // 2xx Success - don't retry
        200..=299 => false,
        // 3xx Redirection - don't retry
        300..=399 => false,
        // 4xx Client errors - don't retry (except 429)
        429 => true, // Too Many Requests - should retry with backoff
        400..=499 => false,
        // 5xx Server errors - should retry
        500..=599 => true,
        // Unknown - don't retry
        _ => false,
    }
}

/// Check if we've exceeded max attempts
fn has_exceeded_max_attempts_internal(attempt: u32, max_attempts: u32) -> bool {
    attempt >= max_attempts
}

// =============================================================================
// WIT Interface Implementation
// =============================================================================

struct Component;

impl Guest for Component {
    fn get_default_config() -> RetryConfig {
        RetryConfig {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            initial_delay_ms: DEFAULT_INITIAL_DELAY_MS,
            max_delay_ms: DEFAULT_MAX_DELAY_MS,
            multiplier: DEFAULT_MULTIPLIER,
        }
    }

    fn calculate_backoff(attempt: u32, config: RetryConfig) -> u64 {
        calculate_backoff_internal(attempt, &config)
    }

    fn should_retry(error_message: String, attempt: u32, max_attempts: u32) -> bool {
        should_retry_internal(&error_message, attempt, max_attempts)
    }

    fn is_retryable_status(status_code: u32) -> bool {
        is_retryable_status_internal(status_code)
    }

    fn has_exceeded_max_attempts(attempt: u32, max_attempts: u32) -> bool {
        has_exceeded_max_attempts_internal(attempt, max_attempts)
    }
}

bindings::export!(Component with_types_in bindings);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config() -> RetryConfig {
        RetryConfig {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            initial_delay_ms: DEFAULT_INITIAL_DELAY_MS,
            max_delay_ms: DEFAULT_MAX_DELAY_MS,
            multiplier: DEFAULT_MULTIPLIER,
        }
    }

    #[test]
    fn test_calculate_backoff_default_config() {
        let config = get_test_config();

        // Attempt 0 should return 0
        assert_eq!(calculate_backoff_internal(0, &config), 0);

        // Attempt 1: 100ms * 2^0 = 100ms
        assert_eq!(calculate_backoff_internal(1, &config), 100);

        // Attempt 2: 100ms * 2^1 = 200ms
        assert_eq!(calculate_backoff_internal(2, &config), 200);

        // Attempt 3: 100ms * 2^2 = 400ms
        assert_eq!(calculate_backoff_internal(3, &config), 400);

        // Attempt 4: 100ms * 2^3 = 800ms
        assert_eq!(calculate_backoff_internal(4, &config), 800);

        // Attempt 5: 100ms * 2^4 = 1600ms
        assert_eq!(calculate_backoff_internal(5, &config), 1600);
    }

    #[test]
    fn test_calculate_backoff_with_max_delay() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay_ms: 100,
            max_delay_ms: 500, // Cap at 500ms
            multiplier: 2.0,
        };

        // Should cap at max_delay_ms
        assert_eq!(calculate_backoff_internal(1, &config), 100);
        assert_eq!(calculate_backoff_internal(2, &config), 200);
        assert_eq!(calculate_backoff_internal(3, &config), 400);
        assert_eq!(calculate_backoff_internal(4, &config), 500); // Capped
        assert_eq!(calculate_backoff_internal(5, &config), 500); // Capped
        assert_eq!(calculate_backoff_internal(10, &config), 500); // Capped
    }

    #[test]
    fn test_calculate_backoff_custom_multiplier() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            multiplier: 3.0, // Triple each time
        };

        assert_eq!(calculate_backoff_internal(1, &config), 100); // 100 * 3^0
        assert_eq!(calculate_backoff_internal(2, &config), 300); // 100 * 3^1
        assert_eq!(calculate_backoff_internal(3, &config), 900); // 100 * 3^2
        assert_eq!(calculate_backoff_internal(4, &config), 2700); // 100 * 3^3
    }

    #[test]
    fn test_should_retry_max_attempts_reached() {
        // Attempt 3 with max 3 - should NOT retry
        assert!(!should_retry_internal("HTTP 500", 3, 3));

        // Attempt 4 with max 3 - should NOT retry
        assert!(!should_retry_internal("HTTP 500", 4, 3));
    }

    #[test]
    fn test_should_retry_circuit_breaker_open() {
        assert!(!should_retry_internal("Circuit breaker is OPEN", 1, 3));
        assert!(!should_retry_internal("circuit breaker", 1, 3));
    }

    #[test]
    fn test_should_retry_client_errors() {
        // 4xx errors should not retry
        assert!(!should_retry_internal("HTTP 400", 1, 3));
        assert!(!should_retry_internal("HTTP 404", 1, 3));
        assert!(!should_retry_internal("HTTP 403", 1, 3));
    }

    #[test]
    fn test_should_retry_server_errors() {
        // 5xx errors should retry (if under max attempts)
        assert!(should_retry_internal("HTTP 500", 1, 3));
        assert!(should_retry_internal("HTTP 503", 1, 3));
        assert!(should_retry_internal("HTTP 502", 2, 3));
    }

    #[test]
    fn test_should_retry_network_errors() {
        // Network errors should retry
        assert!(should_retry_internal("Connection timeout", 1, 3));
        assert!(should_retry_internal("Network error", 1, 3));
        assert!(should_retry_internal("Failed to connect", 2, 3));
    }

    #[test]
    fn test_is_retryable_status_success() {
        // 2xx should not retry
        assert!(!is_retryable_status_internal(200));
        assert!(!is_retryable_status_internal(201));
        assert!(!is_retryable_status_internal(204));
    }

    #[test]
    fn test_is_retryable_status_redirect() {
        // 3xx should not retry
        assert!(!is_retryable_status_internal(301));
        assert!(!is_retryable_status_internal(302));
        assert!(!is_retryable_status_internal(304));
    }

    #[test]
    fn test_is_retryable_status_client_errors() {
        // Most 4xx should not retry
        assert!(!is_retryable_status_internal(400));
        assert!(!is_retryable_status_internal(401));
        assert!(!is_retryable_status_internal(403));
        assert!(!is_retryable_status_internal(404));

        // 429 is special - should retry with backoff
        assert!(is_retryable_status_internal(429));
    }

    #[test]
    fn test_is_retryable_status_server_errors() {
        // 5xx should retry
        assert!(is_retryable_status_internal(500));
        assert!(is_retryable_status_internal(502));
        assert!(is_retryable_status_internal(503));
        assert!(is_retryable_status_internal(504));
    }

    #[test]
    fn test_has_exceeded_max_attempts() {
        assert!(!has_exceeded_max_attempts_internal(1, 3));
        assert!(!has_exceeded_max_attempts_internal(2, 3));
        assert!(has_exceeded_max_attempts_internal(3, 3));
        assert!(has_exceeded_max_attempts_internal(4, 3));
    }

    #[test]
    fn test_default_config_values() {
        let config = get_test_config();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 10_000);
        assert_eq!(config.multiplier, 2.0);
    }
}
