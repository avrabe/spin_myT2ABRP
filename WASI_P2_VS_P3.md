# WASI Preview 2 vs Preview 3

## Quick Reference

| Feature | WASI Preview 2 (0.2.0) | WASI Preview 3 (Expected 2025) |
|---------|------------------------|--------------------------------|
| **Status** | ✅ Stable (Jan 2024) | 🚧 In Development |
| **Component Model** | Full support | Enhanced support |
| **Concurrency** | Single-task per instance | Composable concurrency |
| **Async I/O** | Via `poll` function | Native async/await ABI |
| **Stream Types** | Custom implementations | Built-in generic streams |
| **Future Types** | Custom implementations | Built-in generic futures |
| **Spin Support** | ✅ Full (Spin 2.0+) | 🔜 Experimental (Mid-2025) |

## WASI Preview 2 (Current - Stable)

### Released
- January 25, 2024 (WASI 0.2.0)
- Supported by Spin 2.0+ (November 2023)

### Key Features
- ✅ WebAssembly Component Model support
- ✅ Polyglot component composition
- ✅ wasi:http interface for HTTP handlers
- ✅ wasi:cli, wasi:clocks, wasi:filesystem, wasi:sockets
- ✅ Asynchronous I/O via `poll` function
- ✅ Production-ready runtimes (Wasmtime, Spin)

### Limitations
- ❌ Limited concurrency: one task at a time per instance
- ❌ Components cannot perform I/O operations simultaneously
- ❌ No native async/await ABI
- ❌ Requires custom stream/future implementations

## WASI Preview 3 (Upcoming)

### Timeline
- **RC Snapshots**: Expected Q1 2025
- **Experimental Runtime Support**: Q2 2025
- **Final Release**: Expected Mid-Late 2025
- **Fermyon Status**: Already building with snapshots

### Main Theme: Composable Concurrency

WASIp3's central focus is enabling multiple components to perform I/O operations concurrently within the same instance.

### New Features

#### 1. Asynchronous Function ABI
- Components can export/import functions with sync OR async calling conventions
- Seamless connection: async imports ↔ sync exports (and vice versa)
- Solves "function coloring" problem
- Arbitrary numbers of guest tasks run concurrently

#### 2. Built-in Stream and Future Types
- Generic stream types for cross-component communication
- Generic future types
- No need for custom implementations
- Efficient inter-component data transfer

#### 3. Simplified Specifications
- Reduces complexity in WebAssembly Interface Types
- More elegant async patterns
- Better ergonomics for developers

### Impact on Applications

**Current (P2):**
```
Component A → poll() → wait → Component B
(Sequential, blocks entire instance)
```

**Future (P3):**
```
Component A ⇄ async ⇄ concurrent tasks ⇄ Component B
(Concurrent, multiple tasks per instance)
```

### Backward Compatibility

- P2 components will continue to work
- Spin maintains backward compatibility with snapshots
- Gradual migration path expected
- No major breaking changes anticipated

## Recommendations for This Project

### Current Strategy (WASI P2)
✅ **Stick with WASI Preview 2** for now:
- Stable and production-ready
- Full Spin 2.0+ support
- Component Model fully functional
- All required features available

### Future Considerations (WASI P3)
When P3 is released:
- Evaluate concurrency benefits for this application
- Consider upgrade if:
  - Multiple components need concurrent I/O
  - Stream processing becomes a bottleneck
  - Better async patterns would help
- Migration likely straightforward (no major breaking changes expected)

## Project Status

### Current Build Target
```rust
// Cargo.toml - Gateway component
[dependencies]
wit-bindgen = "0.41.0"  // P2 compatible
```

### Rust Toolchain
```bash
rustc 1.91.1 (2025-11-07)
# Supports wasm32-wasip1 and wasm32-wasip2 targets
```

### Spin Configuration
```toml
# spin.toml
spin_manifest_version = 2  # Spin 2.0 = WASI P2 support
```

## Key Takeaway

**For Toyota MyT2ABRP project**: WASI Preview 2 is the right choice. It's stable, production-ready, and provides everything needed for the current architecture. P3 can be evaluated when released in 2025, but there's no urgency to wait for it.

## References

- [WASI 0.2.0 Release](https://github.com/WebAssembly/WASI/releases/tag/v0.2.0)
- [Looking Ahead to WASIp3](https://www.fermyon.com/blog/looking-ahead-to-wasip3)
- [Spin 2.0 Release](https://www.fermyon.com/blog/introducing-spin-v2)
- [Component Model Specification](https://component-model.bytecodealliance.org/)
