# Build Instructions - Component Model P2

## Quick Start

This project now uses **WebAssembly Component Model P2** for all components. Here's how to build and test.

## Prerequisites

### Install Tools

```bash
# Install Rust toolchain with wasm32-wasip2 target
rustup target add wasm32-wasip2

# Install cargo-component for building WebAssembly components
cargo install cargo-component

# Install wac for component composition
cargo install wac-cli

# Install wasm-tools for validation and inspection
cargo install wasm-tools

# Optional: Install Bazelisk via npm (requires Node.js)
npm install

# Optional: Install Spin CLI
cargo install spin-cli
```

## Building Components

### Option 1: Build Individual Components (Development)

Build a single component:

```bash
cd components/gateway
cargo component build --release

# Output: target/wasm32-wasip2/release/toyota_gateway.wasm
```

Build all components:

```bash
# Validation component
cd components/validation && cargo component build --release

# Retry logic component
cd components/retry-logic && cargo component build --release

# Business logic (JWT) component
cd components/business-logic && cargo component build --release

# Circuit breaker component
cd components/circuit-breaker && cargo component build --release

# Data transform component
cd components/data-transform && cargo component build --release

# API types component
cd components/toyota-api-types && cargo component build --release

# Metrics component
cd components/metrics && cargo component build --release

# Gateway component
cd components/gateway && cargo component build --release
```

### Option 2: Build with Bazel (Production - Recommended)

Bazel provides hermetic builds with automatic WAC composition:

```bash
# Build all components
npx bazelisk build //components/...

# Build composed gateway application
npx bazelisk build //:myt2abrp_app

# Output: bazel-bin/myt2abrp_app.wasm
```

**Note**: Due to network restrictions in some environments, Bazel downloads may fail. If this happens, use cargo-component instead.

## Component Composition with WAC

### Manual Composition

If you build components individually, you can compose them manually using `wac`:

```bash
# Make sure all components are built first
cd /home/user/spin_myT2ABRP

# Example: Compose gateway with all dependencies
wac plug \
  --plug 'toyota:validation/validator@0.1.0'=components/validation/target/wasm32-wasip2/release/toyota_validation.wasm \
  --plug 'toyota:retry/strategy@0.1.0'=components/retry-logic/target/wasm32-wasip2/release/toyota_retry_logic.wasm \
  --plug 'toyota:business-logic/jwt@0.1.0'=components/business-logic/target/wasm32-wasip2/release/business_logic.wasm \
  --plug 'toyota:circuit-breaker/breaker@0.1.0'=components/circuit-breaker/target/wasm32-wasip2/release/toyota_circuit_breaker.wasm \
  --plug 'toyota:data-transform/converter@0.1.0'=components/data-transform/target/wasm32-wasip2/release/toyota_data_transform.wasm \
  --plug 'toyota:api-types/models@0.1.0'=components/toyota-api-types/target/wasm32-wasip2/release/toyota_api_types.wasm \
  --plug 'toyota:metrics/collector@0.1.0'=components/metrics/target/wasm32-wasip2/release/toyota_metrics.wasm \
  components/gateway/target/wasm32-wasip2/release/toyota_gateway.wasm \
  -o myt2abrp_composed.wasm
```

### Using WAC Composition File

```bash
# Use the compose.wac file for composition
wac compose compose.wac -o myt2abrp_app.wasm
```

## Validation

Verify that your component is valid and exports the correct interfaces:

```bash
# Validate the component
wasm-tools validate myt2abrp_app.wasm

# Inspect the component's WIT interfaces
wasm-tools component wit myt2abrp_app.wasm

# Should show:
# export wasi:http/incoming-handler@0.2.0
```

Check the exports:

```bash
# List all exports
wasm-tools component new --encoding utf8 myt2abrp_app.wasm | grep "export"
```

## Testing

### Test with Spin

```bash
# Update spin.toml if needed to point to your compiled component
# Default: source = "bazel-bin/myt2abrp_app.wasm"

# Run with Spin (requires Spin 2.0+)
spin up

# Test in another terminal:
curl http://localhost:3000/health
curl http://localhost:3000/validate
curl http://localhost:3000/jwt/generate
curl http://localhost:3000/transform
curl http://localhost:3000/metrics
```

Expected output from `/health`:

```json
{
  "status": "healthy",
  "message": "Component Model P2 Gateway",
  "components": {
    "validation": "ready",
    "retry": "ready",
    "business-logic": "ready",
    "circuit-breaker": "ready",
    "data-transform": "ready",
    "api-types": "ready",
    "metrics": "ready"
  },
  "wasi_version": "0.2.0"
}
```

### Test with Wasmtime

You can also run the component directly with Wasmtime (WASI runtime):

```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash

# Run the component
wasmtime serve myt2abrp_app.wasm

# Test
curl http://localhost:8080/health
```

## Testing Individual Components

Each component can be tested independently:

```bash
# Run unit tests for a component
cd components/business-logic
cargo test

# Run all tests
cd /home/user/spin_myT2ABRP
cargo test --workspace
```

## Troubleshooting

### "Component not found" errors

Make sure you've built all dependencies:

```bash
# Quick check - build all components
find components -name Cargo.toml -exec dirname {} \; | while read dir; do
  echo "Building $dir..."
  (cd "$dir" && cargo component build --release)
done
```

### WAC composition fails

Verify all component paths in your `compose.wac` file match the actual build outputs:

```bash
# Check if components exist
ls -lh components/*/target/wasm32-wasip2/release/*.wasm
```

### Spin doesn't recognize the component

Ensure you're using Spin 2.0 or later, which supports Component Model:

```bash
spin --version
# Should show version >= 2.0
```

### "Unknown import" errors

This means WAC composition hasn't wired all imports. Verify:

1. All imported components are specified in WAC
2. Interface names match exactly (case-sensitive)
3. Version numbers match

Example fix in `compose.wac`:

```wac
let gateway = new toyota:gateway@0.1.0 {
    // Make sure version matches: @0.1.0
    "toyota:validation/validator@0.1.0": validation."toyota:validation/validator@0.1.0",
    ...
};
```

## Performance Optimization

For production builds:

```bash
# Build with optimizations
cd components/gateway
cargo component build --release

# Or with Bazel (automatically optimized)
npx bazelisk build -c opt //:myt2abrp_app
```

Component sizes after optimization:

```
business-logic:    ~150KB
validation:        ~71KB
data-transform:    ~263KB
retry-logic:       ~84KB
circuit-breaker:   ~89KB
metrics:           ~101KB
toyota-api-types:  ~240KB
gateway:           ~227KB
--------------------------------
Total composed:    ~1.2MB
```

## Development Workflow

1. **Make changes** to a component's source code
2. **Build** the component: `cargo component build --release`
3. **Validate**: `wasm-tools validate target/wasm32-wasip2/release/*.wasm`
4. **Compose** (if needed): `wac plug ...` or `bazel build`
5. **Test** with Spin: `spin up`
6. **Verify**: `curl http://localhost:3000/health`

## CI/CD Integration

For continuous integration:

```yaml
# .github/workflows/build.yml
name: Build Components

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasip2

      - name: Install cargo-component
        run: cargo install cargo-component

      - name: Build all components
        run: |
          for dir in components/*/; do
            (cd "$dir" && cargo component build --release)
          done

      - name: Install wac
        run: cargo install wac-cli

      - name: Compose application
        run: wac compose compose.wac -o myt2abrp_app.wasm

      - name: Validate
        run: |
          cargo install wasm-tools
          wasm-tools validate myt2abrp_app.wasm

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: myt2abrp-wasm
          path: myt2abrp_app.wasm
```

## Next Steps

- Read [COMPONENT_MODEL_P2_MIGRATION.md](./COMPONENT_MODEL_P2_MIGRATION.md) for technical details
- See [README.md](./README.md) for project overview
- Check [bazel-build.md](./bazel-build.md) for Bazel-specific instructions

## Support

For issues related to:
- **Component Model**: https://component-model.bytecodealliance.org/
- **WAC**: https://github.com/bytecodealliance/wac
- **Spin**: https://developer.fermyon.com/spin
- **cargo-component**: https://github.com/bytecodealliance/cargo-component
