# Toyota MyT2ABRP Testing Guide

## Overview

This guide covers the complete testing setup for the Toyota MyT2ABRP project, including component builds, Spin deployment, and end-to-end testing with Playwright.

## Prerequisites

- **Rust** 1.90.0+ with targets:
  - `wasm32-wasip1` (for cargo-component builds)
  - `wasm32-wasip2` (for Spin SDK builds)
- **cargo-component** v0.21.1+
- **Spin CLI** v3.0.0+
- **Node.js** v22+ (for Playwright tests)
- **wasm-tools**, **wasmtime**, **wac** (optional, for component inspection)

## Installation

### 1. Install Rust Targets

```bash
rustup target add wasm32-wasip1 wasm32-wasip2
```

### 2. Install cargo-component

```bash
cargo install cargo-component --locked
```

### 3. Install Spin CLI

```bash
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash -s -- --version v3.0.0
mv spin ~/bin/spin  # or add to your PATH
```

### 4. Install Node.js Dependencies

```bash
cd tests/e2e
npm install
```

## Building Components

### Component Model Components (7 components)

All 7 Toyota MyT2ABRP components use the Component Model and are built with `cargo-component`:

```bash
# Build all components
cargo component build --manifest-path components/validation/Cargo.toml --release
cargo component build --manifest-path components/retry-logic/Cargo.toml --release
cargo component build --manifest-path components/circuit-breaker/Cargo.toml --release
cargo component build --manifest-path components/metrics/Cargo.toml --release
cargo component build --manifest-path components/toyota-api-types/Cargo.toml --release
cargo component build --manifest-path components/data-transform/Cargo.toml --release
cargo component build --manifest-path components/business-logic/Cargo.toml --release
```

**Build outputs** (target/wasm32-wasip1/release/):
- `toyota_validation.wasm` (71K)
- `toyota_retry_logic.wasm` (84K)
- `toyota_circuit_breaker.wasm` (89K)
- `toyota_metrics.wasm` (101K)
- `toyota_api_types.wasm` (240K)
- `toyota_data_transform.wasm` (263K)
- `toyota_business_logic.wasm` (1.4M)

**Total size**: ~2.2MB

### Test HTTP Component (Spin SDK)

The test HTTP component uses Spin SDK for easier testing:

```bash
cd test-http
cargo build --target wasm32-wasip2 --release
```

**Build output**: `test-http/target/wasm32-wasip2/release/test_http.wasm` (375K)

### Native Builds (for development)

Both cargo-component and Spin SDK components can be built natively for faster development iteration:

```bash
# Native build (runs on host architecture)
cargo build --lib

# Much faster than WASM builds (21s vs 2.35s for test-http)
# Useful for: syntax checking, unit tests, development
```

## Running Tests

### 1. Start Spin Server

```bash
cd test-http
~/bin/spin up
```

Expected output:
```
Serving http://127.0.0.1:3000
Available Routes:
  test-http: http://127.0.0.1:3000 (wildcard)
```

### 2. Manual Testing with curl

```bash
# Health check
curl http://127.0.0.1:3000/health
# {"status":"healthy","timestamp":"2025-11-16T00:00:00Z"}

# Root endpoint
curl http://127.0.0.1:3000/
# {"message":"Toyota MyT2ABRP Test Component","version":"0.1.0","status":"ok"}

# Validation endpoint
curl -X POST http://127.0.0.1:3000/validate \
  -H "Content-Type: application/json" \
  -d '{"vin":"TEST123","data":{}}'
# {"valid":true,"message":"Validation successful"}

# Metrics
curl http://127.0.0.1:3000/metrics
# {"requests":1,"uptime":100}

# Protected endpoint (no auth)
curl http://127.0.0.1:3000/api/protected
# {"error":"Unauthorized"}

# Protected endpoint (with auth)
curl http://127.0.0.1:3000/api/protected \
  -H "Authorization: Bearer fake-token"
# {"error":"Invalid token"}
```

### 3. Run Playwright E2E Tests

```bash
cd tests/e2e
npm test
```

**Test suite includes**:
- Health & Status endpoints
- Component integration
- Error handling
- Circuit breaker & retry logic
- Data transformation
- Authentication & authorization
- Performance benchmarks

**Expected results** (16/17 passing):
```
✓ Health check (42ms)
✓ Application status (13ms)
✓ Validation component (13ms)
✓ Metrics component (13ms)
✓ 404 handling (9ms)
✓ Retry logic (47ms)
✓ Circuit breaker (35ms)
✓ Data transformation (13ms)
✓ Auth rejection (16ms)
✓ JWT validation (9ms)
✓ Response time < 1s (9ms)
✓ Concurrent requests (41ms)
✓ Response time test: avg 3.22ms (341ms)
✓ Throughput: 416.12 req/sec (5.0s)
✓ Memory stability: 5.78% degradation (3.2s)
✓ Composition overhead: 6.72% (251ms)
```

### 4. Run Specific Test Suites

```bash
# API tests only
npm run test:api

# Performance tests only
npm run test:perf

# Debug mode
npm run test:debug

# UI mode (interactive)
npm run test:ui

# View HTML report
npm run test:report
```

## Performance Metrics

### Response Time
- **Average**: 3.22ms
- **Min**: 2ms
- **Max**: 20ms
- **Target**: < 100ms average, < 500ms max ✅

### Throughput
- **Measured**: 416.12 requests/second
- **Target**: > 10 req/sec ✅

### Memory Stability
- **Degradation**: 5.78% over 1000 requests
- **Target**: < 50% degradation ✅

### Component Composition Overhead
- **Simple endpoint**: 2.38ms average
- **Composed endpoint**: 2.54ms average
- **Overhead**: 6.72%
- **Target**: < 500% overhead ✅

## Component Validation

Validate WASM components using `wasm-tools`:

```bash
# Validate single component
wasm-tools validate target/wasm32-wasip1/release/toyota_validation.wasm

# Validate all components
for component in target/wasm32-wasip1/release/toyota_*.wasm; do
  echo "Validating $component..."
  wasm-tools validate "$component"
done
```

## Troubleshooting

### Spin Not Found

```bash
# Verify Spin is in PATH
which spin

# If not, add to PATH or use full path
export PATH="$HOME/bin:$PATH"
# or
~/bin/spin up
```

### WASM Target Not Installed

```bash
# Error: the `wasm32-wasip2` target may not be installed
rustup target add wasm32-wasip2
```

### Port 3000 Already in Use

```bash
# Kill existing Spin process
pkill spin

# Or use different port
spin up --listen 127.0.0.1:3001
```

### Playwright Tests Fail

```bash
# Ensure Spin is running first
cd test-http && ~/bin/spin up &

# Wait for server to start
sleep 2

# Run tests
cd tests/e2e && npm test
```

### Build Errors

```bash
# Clean build artifacts
cargo clean

# Rebuild
cargo component build --release
```

## CI/CD Integration

See `.github/workflows/` for:
- Component builds
- WASM validation
- Spin deployment
- Playwright E2E tests
- Performance benchmarks

## Next Steps

1. **WAC Composition**: Use `wac` tool to compose all 7 components
2. **Gateway Integration**: Wire components via WAC plugs
3. **Bazel Build**: Once BCR proxy issue is resolved
4. **Production Deployment**: Deploy to Fermyon Cloud or Kubernetes
5. **Monitoring**: Add Prometheus metrics export

## References

- [Spin Documentation](https://developer.fermyon.com/spin)
- [Component Model](https://component-model.bytecodealliance.org/)
- [Playwright Documentation](https://playwright.dev/)
- [cargo-component](https://github.com/bytecodealliance/cargo-component)
- [WASI Preview 2](https://github.com/WebAssembly/WASI/tree/main/preview2)
