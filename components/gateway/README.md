# Gateway Component

Thin Spin HTTP gateway that orchestrates business logic and circuit breaker components via WAC composition.

## Overview

This is a **proof-of-concept** gateway showing component composition in action. It imports:

- `toyota:business-logic/jwt@0.1.0` - JWT token operations
- `toyota:circuit-breaker/breaker@0.1.0` - Circuit breaker pattern

## Architecture

```
┌─────────────────────────────────────┐
│  Gateway (Spin HTTP Handler)        │
│  - HTTP routing                     │
│  - Test endpoints                   │
│  - Component orchestration          │
│                                     │
│  imports (via WAC):                 │
│    toyota:business-logic/jwt        │
│    toyota:circuit-breaker/breaker   │
└─────────────────────────────────────┘
         ↓                    ↓
┌─────────────────┐  ┌─────────────────┐
│ business-logic  │  │ circuit-breaker │
│   component     │  │   component     │
└─────────────────┘  └─────────────────┘
```

## Endpoints

### `GET /health`

Health check showing available components.

**Response**:
```json
{
  "status": "healthy",
  "components": ["business-logic", "circuit-breaker"]
}
```

### `GET /jwt/test`

Tests JWT component by generating and verifying a token.

**Response** (success):
```json
{
  "status": "ok",
  "component": "business-logic",
  "test": "jwt",
  "claims": {
    "sub": "test@example.com",
    "exp": 1234567890,
    "token_type": "access"
  }
}
```

### `GET /breaker/test`

Tests circuit breaker component state.

**Response** (circuit closed):
```json
{
  "status": "ok",
  "component": "circuit-breaker",
  "state": "closed",
  "failure_count": 0,
  "can_attempt": "yes"
}
```

**Response** (circuit open):
```json
{
  "status": "circuit_open",
  "component": "circuit-breaker",
  "state": "open",
  "failure_count": 5,
  "retry_after_seconds": 42
}
```

HTTP Status: `503 Service Unavailable`
Header: `Retry-After: 42`

## Building

### With Cargo Component

```bash
cd components/gateway
cargo component build --release
```

Output: `target/wasm32-wasip1/release/toyota_gateway.wasm`

### With Bazel + WAC Composition

```bash
# Build gateway with components composed in
bazel build //:gateway_app
```

This uses `wac_plug` to compose:
- Gateway component (this)
- Business logic component
- Circuit breaker component

## WAC Composition

In root `BUILD.bazel`:

```python
wac_plug(
    name = "gateway_app",
    component = "//components/gateway:gateway",
    plug = {
        "toyota:business-logic/jwt":
            "//components/business-logic:business_logic",
        "toyota:circuit-breaker/breaker":
            "//components/circuit-breaker:circuit_breaker",
    },
)
```

This creates a single composed WASM component with all dependencies linked.

## Running

### With Spin

```bash
cd components/gateway
spin build
spin up
```

Then test:

```bash
# Health check
curl http://localhost:3000/health

# Test JWT component
curl http://localhost:3000/jwt/test

# Test circuit breaker
curl http://localhost:3000/breaker/test
```

## Component Imports

The gateway imports components via WIT:

```rust
use bindings::toyota::business_logic::jwt as jwt_component;
use bindings::toyota::circuit_breaker::breaker as breaker_component;

// Generate JWT using imported component
let token = jwt_component::generate_access_token(username, secret)?;

// Check circuit breaker state
let state = breaker_component::get_state();
```

## Future Enhancements

This is a minimal PoC. Future gateway would:

1. Import metrics component for observability
2. Import toyota-api component for API calls
3. Use circuit breaker around toyota-api calls
4. Add KV store for caching
5. Add full authentication flow
6. Add ABRP data transformation endpoints

## See Also

- [Component Migration Plan](../../component-migration-plan.md)
- [Business Logic Component](../business-logic/README.md)
- [Circuit Breaker Component](../circuit-breaker/README.md)
