# WebAssembly Component Model P2 Migration Guide

## Executive Summary

This document explains how the Toyota MyT2ABRP gateway has been migrated from Spin SDK's proprietary abstractions to **pure WebAssembly Component Model P2 (WASI 0.2.0)** for full compatibility with WAC (WebAssembly Composition) tooling.

## Problem Statement

### Original Issue

The project was using Spin SDK's `#[http_component]` macro which:
- Generated Spin-specific bindings (P1/module-based approach)
- Did **not** export the standard `wasi:http/incoming-handler@0.2.0` interface
- Could **not** be properly composed with WAC tooling
- Was incompatible with Component Model P2 ecosystem tools

### Key Difference: P1 vs P2

| Aspect | WASM P1 (Module) | WASM P2 (Component Model) |
|--------|------------------|---------------------------|
| **Type System** | Limited (i32, i64, f32, f64) | Rich (strings, records, variants, lists, options) |
| **Interface Definition** | No standard | WIT (WebAssembly Interface Types) |
| **Composition** | Link-time only | Build-time + runtime via WAC |
| **WASI Version** | Preview 1 (2019) | Preview 2 / WASI 0.2.0 (2024) |
| **Networking** | None | HTTP, sockets included |
| **Interoperability** | C ABI only | Cross-language via Component Model ABI |

## Solution Architecture

### Changes Made

1. **Gateway WIT Interface** (`components/gateway/wit/gateway.wit`)
   - Added `export wasi:http/incoming-handler@0.2.0` to the world
   - Now properly exports the standard HTTP handler interface

2. **Gateway Implementation** (`components/gateway/src/lib.rs`)
   - **Removed**: `spin_sdk::http_component` macro
   - **Added**: Direct `wit_bindgen::generate!` usage
   - **Implemented**: `wasi:http/incoming-handler` Guest trait
   - Uses pure Component Model types (no Spin SDK dependencies)

3. **Dependencies** (`components/gateway/Cargo.toml`)
   - **Removed**: `spin-sdk = "5.1.1"`
   - **Added**: `wit-bindgen = "0.41.0"` (direct bindings generation)
   - Pure Component Model - portable across all P2 hosts

4. **WAC Composition** (`compose.wac`)
   - Full composition of all 8 components
   - Wires all 7 business logic components into the gateway
   - Exports the composed `wasi:http/incoming-handler@0.2.0`

5. **Build Configuration** (`components/gateway/BUILD.bazel`)
   - Updated to use `wit-bindgen` instead of `spin-sdk`
   - Builds a pure Component Model component

## Technical Deep Dive

### Component Model HTTP Handler

The gateway now implements the standard WASI HTTP interface:

```rust
// Generate bindings from WIT files
wit_bindgen::generate!({
    world: "gateway",
    path: "wit",
    exports: {
        "wasi:http/incoming-handler@0.2.0": GatewayComponent,
    },
});

// Implement the Guest trait
impl Guest for GatewayComponent {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        // Pure Component Model implementation
    }
}
```

### WAC Composition Flow

```
┌─────────────────┐
│   validation    │───┐
└─────────────────┘   │
┌─────────────────┐   │
│  retry-logic    │───┤
└─────────────────┘   │
┌─────────────────┐   │
│ business-logic  │───┤
└─────────────────┘   │
┌─────────────────┐   │    ┌──────────────┐      ┌──────────────┐
│ circuit-breaker │───┼───→│   gateway    │─────→│  Spin / WASI │
└─────────────────┘   │    │  (exports    │      │  HTTP Host   │
┌─────────────────┐   │    │   handler)   │      └──────────────┘
│ data-transform  │───┤    └──────────────┘
└─────────────────┘   │
┌─────────────────┐   │
│   api-types     │───┤
└─────────────────┘   │
┌─────────────────┐   │
│    metrics      │───┘
└─────────────────┘
```

All components are wired together at build-time using WAC, producing a single composed component that:
- Exports `wasi:http/incoming-handler@0.2.0`
- Works with Spin, Wasmtime, or any WASI HTTP host
- Can be further composed with other components

## Compatibility Matrix

| Runtime | P1 (spin-sdk) | P2 (Component Model) |
|---------|---------------|----------------------|
| **Spin 2.0+** | ✅ Yes | ✅ Yes |
| **Spin 1.x** | ✅ Yes | ❌ No |
| **Wasmtime** | ❌ No | ✅ Yes |
| **WasmEdge** | ❌ No | ✅ Yes (with wasi-http) |
| **WAC Composition** | ❌ No | ✅ Yes |
| **cargo-component** | ❌ No | ✅ Yes |

## Build Process

### With Bazel (Recommended)

```bash
# Build all components
bazel build //components/...

# Build composed gateway
bazel build //:myt2abrp_app

# Output: bazel-bin/myt2abrp_app.wasm
```

### With cargo-component (Development)

```bash
# Build gateway component
cd components/gateway
cargo component build --release

# Output: target/wasm32-wasip2/release/toyota_gateway.wasm
```

### With WAC (Manual Composition)

```bash
# Build all individual components
cd components/business-logic && cargo component build --release
cd components/validation && cargo component build --release
# ... repeat for all components

# Compose with WAC
wac plug \
  --plug validation=target/wasm32-wasip2/release/toyota_validation.wasm \
  --plug retry-logic=target/wasm32-wasip2/release/toyota_retry_logic.wasm \
  --plug business-logic=target/wasm32-wasip2/release/business_logic.wasm \
  # ... all other components
  gateway.wasm \
  -o composed_gateway.wasm
```

## Testing

### Run with Spin

```bash
# Build the composed component
bazel build //:myt2abrp_app

# Run with Spin
spin up

# Test endpoints
curl http://localhost:3000/health
curl http://localhost:3000/validate
curl http://localhost:3000/jwt/generate
curl http://localhost:3000/transform
curl http://localhost:3000/metrics
```

### Verify Component Model Compliance

```bash
# Install wasm-tools
cargo install wasm-tools

# Validate the component
wasm-tools validate bazel-bin/myt2abrp_app.wasm

# Inspect exports
wasm-tools component wit bazel-bin/myt2abrp_app.wasm

# Should show:
# export wasi:http/incoming-handler@0.2.0
```

## Benefits of P2 Migration

### 1. **Portability**
- Runs on any WASI 0.2.0 compatible runtime
- Not locked into Spin ecosystem
- Can be deployed to WasmEdge, Wasmtime, Fermyon Platform, etc.

### 2. **Composability**
- WAC tooling works correctly
- Can compose with other Component Model components
- Build complex applications from simple components

### 3. **Standardization**
- Uses W3C WebAssembly Component Model standard
- Standard WASI HTTP interface
- Future-proof as ecosystem evolves

### 4. **Type Safety**
- Rich WIT types (records, variants, options)
- Compile-time interface validation
- Cross-language type compatibility

### 5. **Performance**
- Component Model runtime optimizations
- Efficient cross-component calls
- Zero-copy data passing where possible

## Migration Checklist for Other Projects

If you're migrating a Spin SDK project to pure Component Model:

- [ ] Add `export wasi:http/incoming-handler@0.2.0` to your WIT world
- [ ] Replace `#[http_component]` with `wit_bindgen::generate!`
- [ ] Implement the `Guest` trait for the HTTP handler
- [ ] Remove `spin-sdk` dependency
- [ ] Add `wit-bindgen` dependency
- [ ] Update request/response handling to use WASI types
- [ ] Test with `cargo component build`
- [ ] Validate with `wasm-tools validate`
- [ ] Update WAC composition files if using composition
- [ ] Test with Spin and verify compatibility

## Known Limitations

### Spin-Specific Features

The following Spin SDK features are **not available** in pure Component Model:

- `spin_sdk::key_value::Store` - Use WASI KV proposal when stable
- `spin_sdk::variables` - Use environment variables or WASI config
- `spin_sdk::sqlite` - Use WASI SQL proposal when available
- `spin_sdk::llm` - Spin-specific, no standard interface yet
- `spin_sdk::redis` - Use WASI Redis proposal when available

### Workarounds

For production applications that need Spin-specific features:

1. **Option A**: Create a Spin-specific wrapper component
   - Keep business logic in pure Component Model components
   - Create a thin Spin SDK adapter that imports your components
   - Adapter handles Spin-specific APIs

2. **Option B**: Use WASI proposals
   - WASI KV is in development
   - WASI SQL is in development
   - Use proposed interfaces with polyfills

3. **Option C**: HTTP-based integration
   - Call external services via HTTP (already WASI 0.2.0)
   - Use Redis/SQL via HTTP APIs
   - More overhead but fully portable

## Future Work

### WASI 0.3 (Preview 3)

Expected in 2025, will add:
- Native async support in Component Model
- Threads and parallel execution
- Additional I/O interfaces
- Enhanced performance

### Spin 3.0+

Expected improvements:
- Better Component Model integration
- Spin SDK as pure Component Model components
- Transparent composition support
- Migration tooling

## Resources

- [Component Model Specification](https://component-model.bytecodealliance.org/)
- [WASI 0.2.0 Documentation](https://bytecodealliance.org/articles/WASI-0.2)
- [WAC Tool GitHub](https://github.com/bytecodealliance/wac)
- [Spin Documentation](https://developer.fermyon.com/spin)
- [wit-bindgen Guide](https://docs.rs/wit-bindgen/latest/wit_bindgen/)

## Conclusion

This migration demonstrates that **Spin SDK components can be fully migrated to pure WebAssembly Component Model P2** while maintaining compatibility with Spin runtime. The result is:

✅ Portable across all P2 hosts
✅ Composable with WAC tooling
✅ Standards-compliant
✅ Future-proof
✅ Still works with Spin 2.0+

The gateway now exports a standard `wasi:http/incoming-handler@0.2.0` interface and can be composed with any other Component Model components using WAC.
