# Circuit Breaker Component

Pure WASI component implementing the circuit breaker pattern for resilient API calls.

## Overview

This component protects against cascading failures when downstream services are unavailable by implementing the classic three-state circuit breaker pattern:

- **Closed**: Normal operation, requests are allowed
- **Open**: Too many failures, requests fail fast without trying
- **Half-Open**: Testing if service recovered, limited requests allowed

## Features

- **Zero Spin Dependencies**: Pure WASI component, no Spin SDK required
- **Thread-Safe**: Uses `Mutex` for safe concurrent access
- **Configurable**: Adjust failure thresholds, timeouts, and success thresholds
- **Well-Tested**: 3 comprehensive unit tests
- **Small**: ~200 lines of code

## WIT Interface

```wit
package toyota:circuit-breaker@0.1.0;

interface breaker {
    enum circuit-state {
        closed,
        open,
        half-open,
    }

    can-attempt: func() -> result<_, breaker-error>;
    record-success: func();
    record-failure: func();
    get-state: func() -> circuit-state;
    get-failure-count: func() -> u32;
}
```

## Configuration

Default configuration:
- Failure threshold: 5 consecutive failures
- Timeout: 60 seconds before retry
- Success threshold: 2 consecutive successes to close

## Usage

### From Gateway (via WIT import)

```rust
use toyota_circuit_breaker::*;

// Check if request should proceed
match can_attempt() {
    Ok(_) => {
        // Make API call
        match make_api_request() {
            Ok(result) => {
                record_success();
                Ok(result)
            }
            Err(e) => {
                record_failure();
                Err(e)
            }
        }
    }
    Err(error) => {
        // Circuit is open, fail fast
        Err(format!("Circuit breaker open: retry after {} seconds",
                    error.retry_after_seconds))
    }
}
```

### Monitoring

```rust
// Check current state
let state = get_state();
println!("Circuit state: {:?}", state);

// Check failure count
let failures = get_failure_count();
println!("Current failures: {}", failures);
```

## Testing

### Unit Tests

```bash
# Test with Cargo
cargo test

# Test with Bazel
bazel test //components/circuit-breaker:circuit_breaker_test
```

### Component Validation

```bash
# Build component
cargo component build --release

# Validate with wasm-tools
wasm-tools validate target/wasm32-wasip1/release/toyota_circuit_breaker.wasm

# Inspect interface
wasm-tools component wit target/wasm32-wasip1/release/toyota_circuit_breaker.wasm
```

## Build

### With Cargo Component

```bash
cd components/circuit-breaker
cargo component build --release
```

Output: `target/wasm32-wasip1/release/toyota_circuit_breaker.wasm`

### With Bazel

```bash
bazel build //components/circuit-breaker:circuit_breaker
```

## State Transitions

```
┌─────────┐
│ CLOSED  │ Normal operation
└────┬────┘
     │ failures >= threshold
     ↓
┌─────────┐
│  OPEN   │ Failing fast
└────┬────┘
     │ timeout elapsed
     ↓
┌───────────┐
│ HALF-OPEN │ Testing recovery
└─────┬─────┘
      │
      ├─ success >= threshold → CLOSED
      └─ failure → OPEN
```

## Architecture

This component is:
1. **Extracted** from `myt2abrp/src/circuit_breaker.rs`
2. **Pure WASI**: No Spin SDK dependencies
3. **Independently testable**: Can run with wasmtime
4. **Composable**: Plugs into gateway via WAC

## Dependencies

- `std` library only (no external crates for core logic)
- Component bindings auto-generated from WIT

## Component Model

Built with WebAssembly Component Model for:
- Strong interface contracts
- Language interoperability
- Composition via WAC
- Security isolation

## See Also

- [Component Migration Plan](../../component-migration-plan.md)
- [Business Logic Component](../business-logic/README.md)
- [Bazel Build Guide](../../bazel-build.md)
