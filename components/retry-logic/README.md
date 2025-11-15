# retry-logic

Pure retry strategy utilities for exponential backoff and retry decisions with **zero dependencies** on Spin SDK.

## Overview

This component provides stateless retry logic functions including exponential backoff calculation and retry decision making. All functions are pure - no async, no I/O, no state, fully deterministic.

## Features

- **Zero Spin SDK dependencies** - Pure WASI component
- **Pure functions** - No async, no HTTP, no state
- **Exponential backoff** - Configurable backoff with cap
- **Smart retry decisions** - Based on error type and attempt count
- **HTTP status awareness** - Knows which codes to retry
- **Well-tested** - 14 unit tests, 100% passing
- **Minimal size** - 84KB release binary

## WIT Interface

```wit
interface strategy {
    /// Retry configuration for exponential backoff
    record retry-config {
        max-attempts: u32,
        initial-delay-ms: u64,
        max-delay-ms: u64,
        multiplier: f64,
    }

    /// Get default retry configuration
    get-default-config: func() -> retry-config;

    /// Calculate exponential backoff delay
    calculate-backoff: func(attempt: u32, config: retry-config) -> u64;

    /// Determine if a request should be retried
    should-retry: func(error-message: string, attempt: u32, max-attempts: u32) -> bool;

    /// Check if an HTTP status code is retryable
    is-retryable-status: func(status-code: u32) -> bool;

    /// Check if we've exceeded max attempts
    has-exceeded-max-attempts: func(attempt: u32, max-attempts: u32) -> bool;
}
```

## Retry Strategy

### Default Configuration

```rust
RetryConfig {
    max_attempts: 3,        // Try up to 3 times
    initial_delay_ms: 100,  // Start with 100ms delay
    max_delay_ms: 10_000,   // Cap at 10 seconds
    multiplier: 2.0,        // Double the delay each time
}
```

### Exponential Backoff

Formula: `min(initial_delay * multiplier^(attempt-1), max_delay)`

With default config:
- Attempt 1: 100ms
- Attempt 2: 200ms
- Attempt 3: 400ms
- Attempt 4: 800ms
- Attempt 5: 1,600ms
- Attempt 6: 3,200ms
- Attempt 7: 6,400ms
- Attempt 8: 10,000ms (capped)

### Retry Decision Logic

The `should-retry` function returns `false` if:

1. **Max attempts exceeded**: `attempt >= max_attempts`
2. **Circuit breaker open**: Error message contains "Circuit breaker is OPEN"
3. **Client errors**: Error message contains "HTTP 4" (4xx status codes)

The function returns `true` for:
- Server errors (5xx)
- Network errors
- Timeouts
- Any other transient failures

### HTTP Status Code Decisions

| Status Code | Retryable | Reason |
|-------------|-----------|--------|
| 2xx (Success) | ❌ No | Request succeeded |
| 3xx (Redirect) | ❌ No | Not a failure |
| 4xx (Client Error) | ❌ No | Client's fault, won't change on retry |
| 429 (Too Many Requests) | ✅ Yes | Rate limited, should retry with backoff |
| 5xx (Server Error) | ✅ Yes | Server's fault, may recover |

## Usage Example

```rust
use toyota_retry_logic::*;

// Get default configuration
let config = get_default_config();

// Calculate backoff for attempt 3
let delay_ms = calculate_backoff(3, config);
// delay_ms == 400 (100 * 2^2)

// Check if should retry a server error
let should = should_retry("HTTP 503", 2, 3);
// should == true (server error, under max attempts)

// Check if should retry a client error
let should = should_retry("HTTP 404", 1, 3);
// should == false (client error)

// Check if should retry when circuit is open
let should = should_retry("Circuit breaker is OPEN", 1, 3);
// should == false (circuit breaker prevents retry)

// Check if HTTP status is retryable
assert!(is_retryable_status(503));  // Server error - retry
assert!(!is_retryable_status(404)); // Not found - don't retry
assert!(is_retryable_status(429));  // Rate limited - retry

// Check if exceeded max attempts
assert!(!has_exceeded_max_attempts(2, 3)); // Still ok
assert!(has_exceeded_max_attempts(3, 3));  // Reached limit
```

## Integration Example

This component is designed to work with async retry loops:

```rust
async fn fetch_with_retry(url: &str) -> Result<Response> {
    let config = get_default_config();
    let mut attempt = 0;

    loop {
        attempt += 1;

        match fetch(url).await {
            Ok(response) => {
                // Check status
                if is_retryable_status(response.status) {
                    let error = format!("HTTP {}", response.status);
                    if !should_retry(&error, attempt, config.max_attempts) {
                        return Err(anyhow!("Failed after retries: {}", error));
                    }

                    // Calculate backoff and wait
                    let delay = calculate_backoff(attempt, config);
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }

                return Ok(response);
            }
            Err(e) => {
                let error = e.to_string();
                if !should_retry(&error, attempt, config.max_attempts) {
                    return Err(e);
                }

                // Calculate backoff and wait
                let delay = calculate_backoff(attempt, config);
                sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}
```

## Building

```bash
# Build component
cargo component build --release

# Run tests (native target)
cargo test --package toyota-retry-logic --target x86_64-unknown-linux-gnu

# Size
ls -lh target/wasm32-wasip1/release/toyota_retry_logic.wasm
# 84K
```

## Dependencies

- `wit-bindgen-rt` - WIT bindings (only dependency!)

**No other dependencies** - Pure Rust standard library code.

## Test Coverage

All 14 tests passing:
- ✅ `test_calculate_backoff_default_config` - Default backoff sequence
- ✅ `test_calculate_backoff_with_max_delay` - Respects max delay cap
- ✅ `test_calculate_backoff_custom_multiplier` - Custom multiplier works
- ✅ `test_should_retry_max_attempts_reached` - Stops at max attempts
- ✅ `test_should_retry_circuit_breaker_open` - Respects circuit breaker
- ✅ `test_should_retry_client_errors` - Doesn't retry 4xx
- ✅ `test_should_retry_server_errors` - Retries 5xx
- ✅ `test_should_retry_network_errors` - Retries network errors
- ✅ `test_is_retryable_status_success` - Doesn't retry 2xx
- ✅ `test_is_retryable_status_redirect` - Doesn't retry 3xx
- ✅ `test_is_retryable_status_client_errors` - Handles 4xx (except 429)
- ✅ `test_is_retryable_status_server_errors` - Retries 5xx
- ✅ `test_has_exceeded_max_attempts` - Attempt counting
- ✅ `test_default_config_values` - Validates defaults

## Architecture

Part of the component-based Toyota integration system:

```
┌────────────────────────────────────┐
│  Gateway (imports via WAC)         │
│    toyota:retry/strategy           │
└───────────────┬────────────────────┘
                │
                ↓
┌────────────────────────────────────┐
│  retry-logic (84KB)                │
│  - Exponential backoff calc        │
│  - Retry decision logic            │
│  - HTTP status awareness           │
│  - Zero Spin dependencies          │
│  - Pure functions                  │
└────────────────────────────────────┘
```

## Design Decisions

### Why pure functions?

No async, no HTTP, no state:
- **Testable** - Deterministic, no mocking needed
- **Composable** - Works with any async runtime
- **Portable** - Runs anywhere (WASI, native, browser)
- **Fast** - Pure CPU, no I/O overhead

### Why separate backoff and decision?

Two orthogonal concerns:
- **Backoff calculation** - Mathematical formula
- **Retry decision** - Business logic (what to retry)

Separating them allows:
- Different backoff strategies (linear, exponential, fibonacci)
- Different retry policies (retry all, retry selectively)
- Easy testing of each concern

### Why cap exponential backoff?

Without a cap, delays grow unbounded:
- Attempt 10: 51,200ms (51 seconds!)
- Attempt 20: 52,428,800ms (14.5 hours!)

The cap prevents:
- Unreasonably long waits
- Integer overflow
- Poor user experience

### Why default to 3 attempts?

Balances:
- **Resilience** - Handles transient failures
- **Performance** - Doesn't wait too long
- **Cost** - Limits redundant requests

Industry standard:
- AWS SDK: 3 attempts
- Google Cloud: 3 attempts
- Azure SDK: 3 attempts

## Advanced Configuration

### Aggressive Retry (faster, more attempts)

```rust
RetryConfig {
    max_attempts: 5,
    initial_delay_ms: 50,
    max_delay_ms: 2_000,
    multiplier: 1.5,
}
```

### Conservative Retry (slower, fewer attempts)

```rust
RetryConfig {
    max_attempts: 2,
    initial_delay_ms: 500,
    max_delay_ms: 30_000,
    multiplier: 3.0,
}
```

### Linear Backoff

```rust
RetryConfig {
    max_attempts: 5,
    initial_delay_ms: 1_000,
    max_delay_ms: 1_000,
    multiplier: 1.0,  // No growth
}
```

## License

Apache 2.0
