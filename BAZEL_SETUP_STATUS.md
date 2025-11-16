# Bazel Setup Status

## Overview
This document describes the Bazel build system setup for the Toyota MyT2ABRP WebAssembly component project using `rules_wasm_component` from the pulseengine repository.

## Completed Setup

### ✅ 1. Bazelisk Installation
- **Version**: v1.27.0
- **Location**: `~/bin/bazelisk`
- **Bazel Version**: 8.0.0 (auto-downloaded per `.bazelversion`)

### ✅ 2. MODULE.bazel Configuration
The project uses Bazel's modern module system (bzlmod) with the following configuration:

#### Core Dependencies
- `rules_wasm_component` (pulseengine) - commit `55a6a0a3`
- `rules_rust` v0.65.0 with git_override to avrabe/rules_rust
  - Fixes wasm-component-ld.exe issues on Windows
  - Branch: `fix/add-missing-rustc-binaries`

#### Git Overrides (to workaround BCR proxy issues)
The following dependencies use git_override instead of BCR:
- bazel_skylib v1.8.1
- platforms v0.0.10
- rules_cc v0.2.4
- rules_go v0.57.0
- rules_shell v0.2.0
- rules_oci v1.8.0
- rules_nodejs v6.5.0
- bazel_features v1.32.0
- rules_pkg 1.1.1
- rules_proto 7.1.0
- rules_python 1.2.0

#### Rust Toolchain
- **Edition**: 2021
- **Version**: 1.90.0
- **WASM Targets**:
  - wasm32-unknown-unknown
  - wasm32-wasip1
  - wasm32-wasip2 (Component Model P2)

#### Crate Universe
Configured to manage Rust dependencies from all component Cargo.toml files with cross-compilation support for WASM targets.

### ✅ 3. BUILD Files Verification
All BUILD.bazel files correctly use rules_wasm_component:

| Component | Rule Used | Load Path |
|-----------|-----------|-----------|
| Root (composition) | `wac_plug` | `@rules_wasm_component//wac:defs.bzl` |
| Gateway | `rust_wasm_component` | `@rules_wasm_component//rust:defs.bzl` |
| Business Logic | `rust_wasm_component` | `@rules_wasm_component//rust:defs.bzl` |
| Other components | `wasm_component` | `@rules_wasm_component//wasm_component:defs.bzl` |

## ❌ Current Blocker: BCR Proxy Authentication

### Problem
The build environment's HTTP proxy returns `401 Unauthorized` for all requests to the Bazel Central Registry (bcr.bazel.build):

```
ERROR: Error accessing registry https://bcr.bazel.build/:
Failed to fetch registry file https://bcr.bazel.build/modules/.../MODULE.bazel:
Unable to tunnel through proxy. Proxy returns "HTTP/1.1 401 Unauthorized"
```

### Impact
Even with git_overrides for direct dependencies, transitive dependencies still require BCR access. Examples of blocked transitive dependencies:
- buildozer (from rules_python)
- rules_license (from various packages)
- zlib, protobuf, abseil-cpp (C++ libraries)
- And many others...

### Why NO_PROXY Doesn't Work
Adding `bcr.bazel.build` to NO_PROXY bypasses the proxy entirely, but the container environment cannot reach external hosts without the proxy:
```
ERROR: Error accessing registry https://bcr.bazel.build/:
Failed to fetch registry file: Unknown host: bcr.bazel.build
```

## Potential Solutions

### 1. Fix Proxy Authentication (Recommended)
Configure the proxy to allow authenticated access to bcr.bazel.build. This is the cleanest solution as it allows Bazel to work normally.

### 2. Local BCR Mirror
Set up a local mirror of the Bazel Central Registry:
```starlark
common --registry=http://local-mirror/bazel-central-registry
common --registry=https://bcr.bazel.build  # Fallback
```

### 3. Vendored Dependencies
Use Bazel's vendor mode to download all dependencies once and commit them:
```bash
bazel vendor --vendor_dir=bazel_vendor
```
Then configure:
```starlark
common --vendor_dir=bazel_vendor
```

### 4. WORKSPACE Mode (Not Recommended)
Fall back to legacy WORKSPACE mode instead of bzlmod, but this loses the benefits of the modern module system and rules_wasm_component may not support it.

## Build Structure

The project is configured for Component Model P2 builds with the following structure:

```
spin_myT2ABRP/
├── MODULE.bazel              # Bazel module configuration
├── .bazelrc                  # Bazel build settings
├── .bazelversion            # Bazel version (8.0.0)
├── BUILD.bazel              # Root build file (WAC composition)
└── components/
    ├── gateway/             # HTTP handler (rust_wasm_component)
    ├── business-logic/      # JWT operations (rust_wasm_component)
    ├── validation/          # Input validation (wasm_component)
    ├── retry-logic/         # Retry strategies (wasm_component)
    ├── circuit-breaker/     # Resilient API calls (wasm_component)
    ├── data-transform/      # Data transformations (wasm_component)
    ├── toyota-api-types/    # Common types (wasm_component)
    └── metrics/             # Observability (wasm_component)
```

## Next Steps

1. **Resolve proxy authentication** for bcr.bazel.build access
2. Once BCR is accessible:
   - Run `bazel build //...` to build all components
   - Run `bazel test //...` to run all tests
   - Build the final composed app: `bazel build //:myt2abrp_app`
3. Verify WASM component outputs and WAC composition

## Resources

- [rules_wasm_component Documentation](https://github.com/pulseengine/rules_wasm_component)
- [Bazel Module System (bzlmod)](https://bazel.build/external/module)
- [Component Model Specification](https://component-model.bytecodealliance.org/)
