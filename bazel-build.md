# Bazel Build System for WebAssembly Components

This project uses Bazel with [rules_wasm_component](https://github.com/pulseengine/rules_wasm_component.git) for building, testing, and composing WebAssembly components.

## Why Bazel?

- **Hermetic builds**: All toolchains auto-downloaded, no manual setup
- **Reproducible**: Same inputs → same outputs, always
- **Fast**: Incremental builds, remote caching support
- **Integrated**: Component building, WAC composition, and wasmtime testing in one system
- **Multi-language**: Can mix Rust, Go, C++ components

## Prerequisites

**Only Bazel is required!** All other tools (Rust, wasmtime, wasm-tools, etc.) are automatically downloaded.

Install Bazel:
```bash
# macOS
brew install bazelisk

# Linux
npm install -g @bazel/bazelisk
# or download from https://github.com/bazelbuild/bazelisk
```

## Quick Start

### Build the Business Logic Component

```bash
# Build release version
bazel build //components/business-logic:business_logic

# Output: bazel-bin/components/business-logic/business_logic.wasm
```

### Run Tests

```bash
# Run all tests
bazel test //...

# Run component validation
bazel test //components/business-logic:validate_component

# Run unit tests
bazel test //components/business-logic:business_logic_test
```

### Build Composed Application

```bash
# Build WAC composition
bazel build //:composed_app

# Output: bazel-bin/composed_app.wasm
```

## Build Targets

### Component Builds

| Target | Description | Output |
|--------|-------------|--------|
| `//components/business-logic:business_logic` | JWT business logic component | `business_logic.wasm` |
| `//components/business-logic:business_logic_test` | Unit tests (native) | Test results |
| `//components/business-logic:validate_component` | Component structure validation | Test results |

### Composition

| Target | Description | Output |
|--------|-------------|--------|
| `//:composed_app` | WAC composition | `composed_app.wasm` |

## Configuration

### Build Modes

```bash
# Debug build (faster, larger)
bazel build //components/business-logic:business_logic

# Release build (optimized, smaller)
bazel build --config=release //components/business-logic:business_logic

# Custom config
bazel build --config=myconfig //...
```

### Profiles

Components support multiple build profiles (defined in BUILD files):

```python
rust_wasm_component(
    profiles = ["debug", "release"],
)
```

## File Structure

```
.
├── MODULE.bazel                    # Bazel module dependencies
├── .bazelrc                        # Bazel configuration
├── BUILD.bazel                     # Root build file (composition)
├── compose.wac                     # WAC composition spec
│
├── components/
│   └── business-logic/
│       ├── BUILD.bazel             # Component build rules
│       ├── wit/
│       │   └── jwt.wit             # WIT interface
│       └── src/
│           └── lib.rs              # Implementation
│
└── tools/
    ├── BUILD.bazel
    └── validate_component.sh       # Validation script
```

## Hermetic Toolchains

rules_wasm_component automatically downloads:

- **Rust**: 1.90.0+ (configurable)
- **wasm-tools**: Latest (for component manipulation)
- **wasmtime**: Latest (for testing/AOT compilation)
- **WAC**: For component composition

No manual installation needed!

## Advanced Usage

### Remote Caching

Enable remote cache for faster CI builds:

```bash
# .bazelrc
build --remote_cache=https://your-cache-server
```

### Parallel Builds

```bash
# Use all CPU cores
bazel build --jobs=auto //...

# Limit to 4 jobs
bazel build --jobs=4 //...
```

### Query Targets

```bash
# List all targets
bazel query //...

# Find dependencies
bazel query 'deps(//components/business-logic:business_logic)'

# Reverse dependencies
bazel query 'rdeps(//..., //components/business-logic:business_logic)'
```

### Build Information

```bash
# Show what would be built
bazel build --explain=explain.log //...

# Profile build
bazel build --profile=profile.json //...
bazel analyze-profile profile.json
```

## Comparison: Bazel vs Cargo

| Aspect | Cargo | Bazel |
|--------|-------|-------|
| **Setup** | Rust only | All tools hermetic |
| **Reproducibility** | Good | Excellent (hash-based) |
| **Incremental** | Good | Excellent (fine-grained) |
| **Multi-language** | Rust only | Rust, Go, C++, etc. |
| **Caching** | Local only | Local + remote |
| **Component composition** | Manual | Integrated (WAC) |
| **Testing** | `cargo test` | `bazel test` (hermetic) |

## CI/CD Integration

### GitHub Actions

```yaml
name: Bazel Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Bazel
        uses: bazel-contrib/setup-bazel@0.9.0
        with:
          bazelisk-cache: true

      - name: Build components
        run: bazel build //...

      - name: Run tests
        run: bazel test //...

      - name: Build composition
        run: bazel build //:composed_app
```

## Troubleshooting

### Clean Build

```bash
# Clean all build outputs
bazel clean

# Clean everything including downloaded toolchains
bazel clean --expunge
```

### Debug Build Issues

```bash
# Verbose output
bazel build --verbose_failures //...

# Show all commands
bazel build --subcommands //...
```

### Check Toolchain

```bash
# Show resolved toolchains
bazel cquery --output=build //components/business-logic:business_logic
```

## Migration from Cargo

Current setup maintains both build systems:

- **Cargo**: For IDE support, `rust-analyzer`, local development
- **Bazel**: For CI/CD, hermetic builds, component composition

Both systems build the same source files independently.

### IDE Support

For VS Code + rust-analyzer, keep using Cargo workspace:
- `Cargo.toml` files remain for IDE
- Bazel uses `Cargo.Bazel.toml` for dependency resolution
- No conflicts!

## Next Steps

1. ✅ Build business logic component with Bazel
2. 🎯 Extract more components (validation, transforms)
3. 🎯 Create Spin gateway component
4. 🎯 Compose all components with `wac_plug`
5. 🎯 Set up remote caching for CI
6. 🎯 Add wasmtime integration tests

## Resources

- [Bazel Documentation](https://bazel.build/)
- [rules_wasm_component](https://github.com/pulseengine/rules_wasm_component)
- [Bazel Best Practices](https://bazel.build/basics/best-practices)
- [rules_rust](https://github.com/bazelbuild/rules_rust)
