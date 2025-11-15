# WebAssembly Components

This directory contains 8 standalone WebAssembly components built using the Component Model. Each component is independently testable, composable, and reusable.

## Component Architecture

```
┌──────────────────────────────────────────┐
│  Gateway (Spin SDK)                      │
│  - HTTP routing                          │
│  - KV store orchestration                │
│  - Imports: All components via WAC       │
└──────────────┬───────────────────────────┘
               │
               ↓
┌──────────────────────────────────────────┐
│  Pure WASI Components (7 components)     │
│  - Zero Spin SDK dependencies            │
│  - Independently testable                │
│  - Reusable across projects              │
└──────────────────────────────────────────┘
```

## Components (8 total)

### 1. business-logic (150KB, 7 tests)
**Package**: `toyota:business-logic@0.1.0`
**Purpose**: JWT operations (generate, verify, hash)
**Dependencies**: Zero Spin SDK

### 2. circuit-breaker (89KB, 3 tests)
**Package**: `toyota:circuit-breaker@0.1.0`
**Purpose**: Circuit breaker pattern (Open/Closed/Half-Open states)
**Dependencies**: Zero Spin SDK

### 3. data-transform (263KB, 8 tests)
**Package**: `toyota:data-transform@0.1.0`
**Purpose**: Toyota API → ABRP telemetry transformation
**Dependencies**: Zero Spin SDK (chrono, serde, serde_json)

### 4. gateway (227KB)
**Package**: `toyota:gateway@0.1.0`
**Purpose**: HTTP routing and orchestration
**Dependencies**: Spin SDK (HTTP, KV store, variables)

### 5. metrics (101KB)
**Package**: `toyota:metrics@0.1.0`
**Purpose**: Prometheus-compatible metrics collection
**Dependencies**: Zero Spin SDK

### 6. retry-logic (84KB, 14 tests)
**Package**: `toyota:retry@0.1.0`
**Purpose**: Exponential backoff and retry decisions
**Dependencies**: Zero Spin SDK

### 7. toyota-api-types (240KB, 9 tests)
**Package**: `toyota:api-types@0.1.0`
**Purpose**: Toyota Connected Services API data models
**Dependencies**: Zero Spin SDK (serde, serde_json)

### 8. validation (71KB, 13 tests)
**Package**: `toyota:validation@0.1.0`
**Purpose**: Input validation (credentials, email, length)
**Dependencies**: Zero Spin SDK

## Statistics

- **Total Components**: 8
- **Pure WASI Components**: 7 (88%)
- **Total Tests**: 54/54 passing (100%)
- **Total Size**: 1,225KB (1.2 MB)
- **Total Lines**: 1,892 lines of code

## Building Components

### Using Cargo Component
```bash
# Build single component
cargo component build --release --package toyota-validation

# Build all components
for pkg in business-logic circuit-breaker data-transform metrics retry-logic toyota-api-types validation; do
  cargo component build --release --package toyota-$pkg
done
```

### Using Bazel
```bash
# Build single component
bazel build //components/validation:toyota_validation

# Build all components
bazel build //components/...

# Run all tests
bazel test //components/...
```

See [../bazel-build.md](../bazel-build.md) for detailed Bazel instructions.

## Testing Components

### Native Target (faster)
```bash
# Test single component
cargo test --package toyota-validation --target x86_64-unknown-linux-gnu

# Test all components
cargo test --workspace --target x86_64-unknown-linux-gnu
```

### WASM Target (production)
```bash
# Build and validate
cargo component build --release --package toyota-validation
wasm-tools validate target/wasm32-wasip1/release/toyota_validation.wasm
```

## Component Documentation

Each component has a detailed README:
- [business-logic/](business-logic/) - JWT operations
- [circuit-breaker/README.md](circuit-breaker/README.md) - Circuit breaker pattern
- [data-transform/README.md](data-transform/README.md) - Data transformation
- [gateway/README.md](gateway/README.md) - HTTP gateway
- [metrics/](metrics/) - Metrics collection
- [retry-logic/README.md](retry-logic/README.md) - Retry strategy
- [toyota-api-types/README.md](toyota-api-types/README.md) - API data models
- [validation/README.md](validation/README.md) - Input validation

## Component Composition (Future)

Components will be composed using WAC (WebAssembly Composition):

```wit
// Example: Gateway imports all components
import toyota:business-logic/jwt@0.1.0;
import toyota:circuit-breaker/breaker@0.1.0;
import toyota:data-transform/converter@0.1.0;
import toyota:metrics/collector@0.1.0;
import toyota:retry/strategy@0.1.0;
import toyota:api-types/models@0.1.0;
import toyota:validation/validator@0.1.0;
```

## License

Apache 2.0
