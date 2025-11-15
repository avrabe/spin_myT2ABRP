# Component Migration Plan

**Goal**: Migrate codebase to Bazel-first, component-based architecture with clear boundaries

---

## Current State Analysis

### Codebase Breakdown (4,315 lines total)

| Module | Lines | Spin Dependencies | Componentization |
|--------|-------|-------------------|------------------|
| **myt2abrp/lib.rs** | 2,548 | ✅ Yes (http, kv, vars) | Gateway layer only |
| **circuit_breaker.rs** | 184 | ❌ **ZERO** | ✅ Extract immediately |
| **metrics.rs** | 348 | ❌ **ZERO** | ✅ Extract immediately |
| **myt/lib.rs** | 603 | ❌ No (pure types) | ✅ Extract as toyota-api |
| **business-logic** | ~400 | ❌ **ZERO** | ✅ **Already done!** |

### Dependencies Found

```rust
// circuit_breaker.rs
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};
// ✅ Zero Spin SDK - Can extract!

// metrics.rs
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
// ✅ Zero Spin SDK - Can extract!

// myt/lib.rs
use serde::{Deserialize, Serialize};
use serde_path_to_error;
// ✅ Zero Spin SDK - Can extract!
```

---

## Proposed Component Architecture

```
spin_myT2ABRP/
├── components/
│   ├── business-logic/          ✅ DONE
│   │   ├── wit/jwt.wit
│   │   ├── src/lib.rs           (JWT operations)
│   │   ├── BUILD.bazel
│   │   └── Cargo.toml
│   │
│   ├── circuit-breaker/         🎯 NEXT (Phase 1)
│   │   ├── wit/breaker.wit
│   │   ├── src/lib.rs           (Circuit breaker logic)
│   │   ├── BUILD.bazel
│   │   └── Cargo.toml
│   │
│   ├── metrics/                 🎯 Phase 1
│   │   ├── wit/metrics.wit
│   │   ├── src/lib.rs           (Metrics collection)
│   │   ├── BUILD.bazel
│   │   └── Cargo.toml
│   │
│   ├── toyota-api/              🎯 Phase 2
│   │   ├── wit/client.wit
│   │   ├── src/lib.rs           (HTTP client + models)
│   │   ├── BUILD.bazel
│   │   └── Cargo.toml
│   │
│   └── gateway/                 🎯 Phase 3 (Thin Spin layer)
│       ├── src/lib.rs           (HTTP routing, KV, composition)
│       ├── BUILD.bazel
│       ├── Cargo.toml
│       └── spin.toml
│
├── BUILD.bazel                  (WAC composition)
├── MODULE.bazel                 (Bazel config)
└── compose.wac                  (Component wiring)
```

---

## Migration Phases

### **Phase 1: Extract circuit-breaker + metrics** (2-3 hours)

**Goal**: Prove component extraction works for multiple components

**Steps**:
1. ✅ Create `components/circuit-breaker/` directory
2. ✅ Define WIT interface for circuit breaker
3. ✅ Move code from `myt2abrp/src/circuit_breaker.rs`
4. ✅ Create Bazel BUILD file
5. ✅ Add wasmtime tests
6. ✅ Repeat for metrics component
7. ✅ Update root BUILD.bazel with both components

**Result**: 2 more independently testable components

### **Phase 2: Extract toyota-api client** (4-5 hours)

**Goal**: Separate Toyota API logic from Spin gateway

**Steps**:
1. Create `components/toyota-api/` directory
2. Define WIT interface for HTTP operations
3. Move `myt/src/lib.rs` models
4. Use `wasi:http/outgoing-handler` for HTTP calls
5. Create Bazel BUILD file
6. Add integration tests with mocked responses

**Result**: Toyota API logic testable without Spin

### **Phase 3: Thin gateway layer** (3-4 hours)

**Goal**: Spin gateway becomes pure composition orchestrator

**Remaining in gateway**:
- HTTP routing (`#[http_component]`)
- KV store operations (`spin_sdk::key_value`)
- Environment variables (`spin_sdk::variables`)
- Component imports (via WIT)

**Total gateway code**: ~500-800 lines (down from 2,548!)

### **Phase 4: Complete Bazel migration** (2-3 hours)

**Goal**: Bazel as primary build system

**Steps**:
1. Create `//gateway:BUILD.bazel`
2. Configure WAC composition for all components
3. Set up hermetic builds
4. Configure CI/CD with Bazel
5. Document build commands

---

## Component Boundaries (WIT Interfaces)

### circuit-breaker.wit

```wit
package toyota:circuit-breaker@0.1.0;

interface breaker {
    /// Circuit breaker states
    enum circuit-state {
        closed,      // Normal operation
        open,        // Failing fast
        half-open,   // Testing recovery
    }

    /// Check if request should be attempted
    can-attempt: func() -> result<_, string>;

    /// Record successful call
    record-success: func();

    /// Record failed call
    record-failure: func();

    /// Get current state (for monitoring)
    get-state: func() -> circuit-state;
}

world circuit-breaker {
    export breaker;
}
```

### metrics.wit

```wit
package toyota:metrics@0.1.0;

interface collector {
    /// Record request to endpoint
    record-request: func(endpoint: string);

    /// Record error for endpoint
    record-error: func(endpoint: string);

    /// Record cache hit
    record-cache-hit: func();

    /// Record cache miss
    record-cache-miss: func();

    /// Get Prometheus-formatted metrics
    export-prometheus: func() -> string;
}

world metrics {
    export collector;
}
```

### toyota-api.wit (Future)

```wit
package toyota:toyota-api@0.1.0;

interface client {
    record credentials {
        username: string,
        password: string,
    }

    record oauth-token {
        access-token: string,
        refresh-token: string,
        expires-in: s32,
    }

    record vehicle-status {
        battery-soc: f32,
        range-km: f32,
        charging: bool,
    }

    /// Authenticate with Toyota API
    authenticate: func(creds: credentials) -> result<oauth-token, string>;

    /// Get vehicle electric status
    get-electric-status: func(token: string, vin: string) -> result<vehicle-status, string>;
}

world toyota-api {
    export client;
    import wasi:http/outgoing-handler@0.2.0;
}
```

---

## Bazel Build Structure

### Root BUILD.bazel (Composition)

```python
load("@rules_wasm_component//wac:defs.bzl", "wac_plug")

# Compose all components into final application
wac_plug(
    name = "gateway_app",
    component = "//components/gateway:gateway",
    plug = {
        # Plug in business logic
        "toyota:business-logic/jwt": "//components/business-logic:business_logic",

        # Plug in circuit breaker
        "toyota:circuit-breaker/breaker": "//components/circuit-breaker:circuit_breaker",

        # Plug in metrics
        "toyota:metrics/collector": "//components/metrics:metrics",

        # Future: Toyota API client
        # "toyota:toyota-api/client": "//components/toyota-api:toyota_api",
    },
    visibility = ["//visibility:public"],
)

# Test all components
test_suite(
    name = "all_tests",
    tests = [
        "//components/business-logic:all",
        "//components/circuit-breaker:all",
        "//components/metrics:all",
    ],
)
```

### Build Commands

```bash
# Build all components
bazel build //...

# Build final composed application
bazel build //:gateway_app

# Run all tests
bazel test //...

# Build specific component
bazel build //components/circuit-breaker:circuit_breaker

# Test specific component
bazel test //components/circuit-breaker:...
```

---

## Benefits of This Architecture

### 1. **Independent Testing**
Each component can be tested with wasmtime on native target:
```bash
# Test circuit breaker independently
wasmtime components/circuit-breaker/circuit_breaker.wasm

# Test with coverage on native target
cargo test --target x86_64-unknown-linux-gnu --all-features
cargo llvm-cov --target x86_64-unknown-linux-gnu
```

### 2. **Parallel Builds**
Bazel builds all components in parallel:
```
Building: //components/business-logic    [============>      ] 60%
Building: //components/circuit-breaker   [=================> ] 90%
Building: //components/metrics           [============>      ] 65%
```

### 3. **Clear Boundaries**
WIT interfaces enforce contracts:
```rust
// Gateway can ONLY call exported functions
let can_attempt = toyota_circuit_breaker::can_attempt()?;

// Cannot access internal implementation
// toyota_circuit_breaker::CircuitBreakerState - COMPILE ERROR ✅
```

### 4. **Reusability**
Components can be used in other projects:
```toml
[dependencies]
toyota-circuit-breaker = { path = "../spin_myT2ABRP/components/circuit-breaker" }
```

### 5. **Incremental Migration**
- Phase 1: Extract 2 components (circuit-breaker, metrics)
- Phase 2: Extract toyota-api
- Phase 3: Refactor gateway
- **No big-bang rewrite!**

---

## Migration Timeline

| Phase | Duration | Complexity | Value |
|-------|----------|------------|-------|
| **Phase 1**: circuit-breaker + metrics | 2-3 hours | Low | High (proves architecture) |
| **Phase 2**: toyota-api extraction | 4-5 hours | Medium | High (major decoupling) |
| **Phase 3**: Thin gateway | 3-4 hours | Medium | High (final architecture) |
| **Phase 4**: Bazel CI/CD | 2-3 hours | Low | Medium (automation) |
| **Total** | **11-15 hours** | | |

With AI assistance: Likely **8-12 hours** actual work time.

---

## Next Steps

### Immediate (This Session):
1. ✅ Extract circuit-breaker component
2. ✅ Create WIT interface
3. ✅ Set up Bazel build
4. ✅ Add tests
5. ✅ Verify with wasmtime

### Follow-up (Next Session):
1. Extract metrics component
2. Update gateway to import both components
3. Test composition with WAC
4. Document the pattern

---

## Decision: Should We Proceed?

**Pros**:
- ✅ Circuit breaker is 184 lines, zero deps - **easy win**
- ✅ Proves the architecture works for multiple components
- ✅ Bazel builds are working perfectly
- ✅ Clear testability benefits
- ✅ Incremental migration path

**Cons**:
- ⚠️ Some upfront work (but only 2-3 hours for Phase 1)
- ⚠️ Need to learn WAC composition syntax
- ⚠️ Two build systems temporarily (Cargo + Bazel)

**Recommendation**: **YES - Let's start with circuit-breaker extraction!**

It's small (184 lines), well-tested, zero dependencies, and will prove the architecture works.
