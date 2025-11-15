# WebAssembly Components

This directory contains standalone WebAssembly components that can be tested, composed, and reused independently.

## Components

### business-logic

**Status**: ✅ Proof of Concept Complete

A standalone component containing JWT operations and business logic with ZERO Spin dependencies.

**Exports**:
- `toyota:business-logic/jwt@0.1.0`
  - `generate-access-token(username, secret) -> result<string>`
  - `generate-refresh-token(username, secret) -> result<string>`
  - `verify-token(token, secret) -> result<claims>`
  - `hash-username(username, key) -> string`

**Features**:
- ✅ Pure WASI (no Spin SDK)
- ✅ Testable with wasmtime
- ✅ 7 unit tests passing
- ✅ 1.4 MB component size
- ✅ Component Model native

**Build**:
```bash
cargo component build --release -p toyota-business-logic
```

**Test**:
```bash
cargo test -p toyota-business-logic --target x86_64-unknown-linux-gnu
```

**Output**: `../target/wasm32-wasip1/release/toyota_business_logic.wasm`

## Future Components

### validation (Planned)
- Input validation
- Credential checking
- Rate limiting logic

### data-transform (Planned)
- ABRP telemetry formatting
- Timestamp parsing
- Response mapping

### circuit-breaker (Planned)
- Failure tracking
- State management
- Retry logic

## Architecture

Each component:
1. Defines a WIT interface (`wit/*.wit`)
2. Implements the interface in Rust
3. Has NO Spin dependencies
4. Can be tested standalone
5. Can be composed with other components

## Documentation

See [../poc-component-composition.md](../poc-component-composition.md) for the complete proof of concept documentation.
