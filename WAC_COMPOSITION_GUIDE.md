# WAC (WebAssembly Composition) Guide

## Overview

WAC is the WebAssembly Composition tool that allows you to compose multiple Component Model components together by wiring their imports and exports.

For Toyota MyT2ABRP, we have 7 components that need to be composed:
1. validation
2. retry-logic
3. circuit-breaker
4. metrics
5. toyota-api-types
6. data-transform
7. business-logic

These are all wired into the gateway component.

## Installation

```bash
cargo install wac-cli --locked
wac --version
```

## Component Model Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Gateway                               │
│  (HTTP Handler - exports wasi:http/incoming-handler)        │
└───────┬──────────┬──────────┬─────────┬──────────┬──────────┘
        │          │          │         │          │
        ▼          ▼          ▼         ▼          ▼
    ┌───────┐ ┌────────┐ ┌─────────┐ ┌──────┐ ┌──────────┐
    │Validat││ Retry  │ │ Circuit │ │Metric││  Data    │
    │ ion   ││ Logic  │ │ Breaker │ │  s   ││Transform │
    └───────┘ └────────┘ └─────────┘ └──────┘ └──────────┘
                                                     │
                                                     ▼
                                              ┌─────────────┐
                                              │ API Types   │
                                              └─────────────┘
                                                     │
                                                     ▼
                                              ┌─────────────┐
                                              │ Business    │
                                              │ Logic       │
                                              └─────────────┘
```

## Component Dependencies

Create `compose.wac`:

```wac
// Toyota MyT2ABRP Component Composition
package toyota:myt2abrp@0.1.0;

// Define component imports
let validation = new toyota:validation { };
let retry = new toyota:retry-logic { };
let circuit-breaker = new toyota:circuit-breaker { };
let metrics = new toyota:metrics { };
let api-types = new toyota:api-types { };
let data-transform = new toyota:data-transform {
    types: api-types
};
let business-logic = new toyota:business-logic {
    types: api-types,
    transform: data-transform
};

// Create gateway with all dependencies wired
let gateway = new toyota:gateway {
    validator: validation,
    retry-strategy: retry,
    breaker: circuit-breaker,
    collector: metrics,
    models: api-types,
    converter: data-transform,
    jwt: business-logic
};

// Export the gateway's HTTP handler
export gateway;
```

## Composition Commands

### Method 1: Using WAC CLI

```bash
# Compose all components
wac plug \
  --plug toyota:validation/validator@0.1.0=target/wasm32-wasip1/release/toyota_validation.wasm \
  --plug toyota:retry/strategy@0.1.0=target/wasm32-wasip1/release/toyota_retry_logic.wasm \
  --plug toyota:circuit-breaker/breaker@0.1.0=target/wasm32-wasip1/release/toyota_circuit_breaker.wasm \
  --plug toyota:metrics/collector@0.1.0=target/wasm32-wasip1/release/toyota_metrics.wasm \
  --plug toyota:api-types/models@0.1.0=target/wasm32-wasip1/release/toyota_api_types.wasm \
  --plug toyota:data-transform/converter@0.1.0=target/wasm32-wasip1/release/toyota_data_transform.wasm \
  --plug toyota:business-logic/jwt@0.1.0=target/wasm32-wasip1/release/toyota_business_logic.wasm \
  components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm \
  -o composed-app.wasm
```

### Method 2: Using Bazel (when BCR proxy is fixed)

```bash
bazel build //:myt2abrp_app
```

The Bazel BUILD.bazel file already defines the composition:

```python
load("@rules_wasm_component//wac:defs.bzl", "wac_plug")

wac_plug(
    name = "myt2abrp_app",
    component = "//components/gateway:gateway",
    plug = {
        "toyota:validation/validator@0.1.0": "//components/validation:toyota_validation",
        "toyota:retry/strategy@0.1.0": "//components/retry-logic:toyota_retry_logic",
        "toyota:circuit-breaker/breaker@0.1.0": "//components/circuit-breaker:toyota_circuit_breaker",
        "toyota:metrics/collector@0.1.0": "//components/metrics:toyota_metrics",
        "toyota:api-types/models@0.1.0": "//components/toyota-api-types:toyota_api_types",
        "toyota:data-transform/converter@0.1.0": "//components/data-transform:toyota_data_transform",
        "toyota:business-logic/jwt@0.1.0": "//components/business-logic:toyota_business_logic",
    },
    visibility = ["//visibility:public"],
)
```

## Composition Script

Create `scripts/compose.sh`:

```bash
#!/bin/bash
# Compose all Toyota MyT2ABRP components

set -e

echo "🔧 Composing Toyota MyT2ABRP components..."

# Build all components first
./scripts/build-all-components.sh --release

# Check if gateway is built
if [ ! -f "components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm" ]; then
    echo "Building gateway component..."
    cargo component build --manifest-path components/gateway/Cargo.toml --release
fi

# Compose with WAC
echo "Running WAC composition..."
wac plug \
  --plug toyota:validation/validator@0.1.0=target/wasm32-wasip1/release/toyota_validation.wasm \
  --plug toyota:retry/strategy@0.1.0=target/wasm32-wasip1/release/toyota_retry_logic.wasm \
  --plug toyota:circuit-breaker/breaker@0.1.0=target/wasm32-wasip1/release/toyota_circuit_breaker.wasm \
  --plug toyota:metrics/collector@0.1.0=target/wasm32-wasip1/release/toyota_metrics.wasm \
  --plug toyota:api-types/models@0.1.0=target/wasm32-wasip1/release/toyota_api_types.wasm \
  --plug toyota:data-transform/converter@0.1.0=target/wasm32-wasip1/release/toyota_data_transform.wasm \
  --plug toyota:business-logic/jwt@0.1.0=target/wasm32-wasip1/release/toyota_business_logic.wasm \
  components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm \
  -o composed-app.wasm

# Validate composed component
echo "Validating composed component..."
wasm-tools validate composed-app.wasm

# Print component info
echo "Component info:"
wasm-tools component wit composed-app.wasm | head -20

# Print size
SIZE=$(du -h composed-app.wasm | cut -f1)
echo "✅ Composed component size: $SIZE"

echo "✅ Composition complete: composed-app.wasm"
```

## Testing Composed Component

### With Wasmtime

```bash
# Run with wasmtime
wasmtime serve composed-app.wasm
```

### With Spin

Create a spin.toml for the composed component:

```toml
spin_manifest_version = 2

[application]
name = "toyota-myt2abrp-composed"
version = "0.1.0"

[[trigger.http]]
route = "/..."
component = "gateway"

[component.gateway]
source = "composed-app.wasm"
allowed_outbound_hosts = [
    "https://b2c-login.toyota-europe.com",
    "https://ctpa-oneapi.tceu-ctp-prd.toyotaconnectedeurope.io"
]
```

Then run:

```bash
spin up
```

## Inspecting Composition

### View Component Structure

```bash
# Show WIT interfaces
wasm-tools component wit composed-app.wasm

# Show exports
wasm-tools component wit composed-app.wasm | grep export

# Show imports (should be minimal after composition)
wasm-tools component wit composed-app.wasm | grep import
```

### Verify Wiring

```bash
# Check that imports are satisfied
wasm-tools validate composed-app.wasm

# Detailed validation
wasm-tools component new composed-app.wasm --validate
```

## Troubleshooting

### Missing WIT Dependencies

**Error:**
```
package 'wasi:http@0.2.0' not found
```

**Solution:**
Download WASI WIT files:
```bash
mkdir -p wit/deps/wasi
cd wit/deps/wasi
wget https://github.com/WebAssembly/wasi-http/raw/main/wit/handler.wit
wget https://github.com/WebAssembly/wasi-http/raw/main/wit/types.wit
```

### Component Version Mismatch

**Error:**
```
cannot satisfy import 'toyota:validation/validator@0.1.0'
```

**Solution:**
Ensure all components use matching versions in their WIT files:

```wit
package toyota:validation@0.1.0;  // Must match!
```

### Composition Size Too Large

**Solution:**
Optimize components before composition:

```bash
# Optimize each component
for wasm in target/wasm32-wasip1/release/toyota_*.wasm; do
    wasm-opt -Oz "$wasm" -o "${wasm}.opt"
    mv "${wasm}.opt" "$wasm"
done

# Then compose
./scripts/compose.sh
```

## Best Practices

1. **Version Alignment**: Ensure all component versions match
2. **Minimize Imports**: Only import what you need
3. **Test Components Independently**: Before composing
4. **Validate After Composition**: Always validate the result
5. **Track Composition Size**: Monitor for bloat
6. **Document Dependencies**: Keep dependency graph up to date
7. **Use CI/CD**: Automate composition in pipeline

## Advanced: Dynamic Composition

For runtime component swapping:

```rust
// Load components dynamically
let validator = load_component("toyota_validation.wasm");
let retry = load_component("toyota_retry_logic.wasm");

// Compose at runtime
let app = compose_components(vec![validator, retry]);
```

## Integration with Build Systems

### Cargo Integration

Add to `Cargo.toml`:

```toml
[package.metadata.component.dependencies]
"toyota:validation" = { path = "../validation" }
"toyota:retry-logic" = { path = "../retry-logic" }
```

### Bazel Integration

Already configured in `BUILD.bazel` - use when BCR proxy is resolved.

## Resources

- [WAC Documentation](https://github.com/bytecodealliance/wac)
- [Component Model Spec](https://component-model.bytecodealliance.org/)
- [WIT Format](https://component-model.bytecodealliance.org/design/wit.html)
- [Bazel rules_wasm_component](https://github.com/pulseengine/rules_wasm_component)

---

**Last Updated**: 2025-11-16
**WAC Version**: 0.8.1+
**Component Model**: 0.2.0
