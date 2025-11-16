# Bazel Central Registry (BCR) Proxy Workarounds

## Problem Statement

The build environment's HTTP proxy returns `401 Unauthorized` when accessing the Bazel Central Registry (bcr.bazel.build), preventing Bazel from downloading module dependencies.

```
ERROR: Error accessing registry https://bcr.bazel.build/:
Failed to fetch registry file: Unable to tunnel through proxy.
Proxy returns "HTTP/1.1 401 Unauthorized"
```

## Root Cause

The environment's proxy requires authentication, but Bazel cannot authenticate properly with bcr.bazel.build through the proxy. Even direct dependencies with `git_override` work, but their **transitive dependencies** still require BCR access.

## Solution 1: Fix Proxy Configuration (RECOMMENDED)

### Option A: Allow BCR in Proxy Whitelist
Contact your infrastructure team to add `bcr.bazel.build` and `*.bcr.bazel.build` to the proxy whitelist.

### Option B: Configure Proxy Authentication
If the proxy supports authentication, configure Bazel to use it:

```bash
# In .bazelrc
common --repo_env=HTTP_PROXY=http://user:pass@proxy:port
common --repo_env=HTTPS_PROXY=http://user:pass@proxy:port
```

## Solution 2: Bazel Vendor Mode (Requires Initial BCR Access)

Vendor mode downloads all dependencies locally, enabling offline builds.

### Step 1: Enable Vendor Mode

Add to `.bazelrc`:
```
common --vendor_dir=vendor_src
```

### Step 2: Vendor Dependencies (Requires Working BCR Access Once)

```bash
# From a machine with working BCR access
bazel vendor --vendor_dir=vendor_src //...
```

This downloads all dependencies to `vendor_src/`.

### Step 3: Commit Vendored Dependencies

```bash
git add vendor_src/
git commit -m "vendor: Add all Bazel dependencies for offline builds"
```

### Step 4: Build Offline

```bash
bazel build --vendor_dir=vendor_src //...
```

**Limitation**: Requires initial BCR access from at least one machine (e.g., developer laptop, CI runner without proxy).

## Solution 3: Local BCR Mirror

Set up a local mirror of the Bazel Central Registry.

### Step 1: Clone BCR

```bash
git clone https://github.com/bazelbuild/bazel-central-registry.git
cd bazel-central-registry
```

### Step 2: Serve Locally (Option A: HTTP Server)

```bash
python3 -m http.server 8080
```

### Step 2: Serve Locally (Option B: Nginx)

Configure nginx to serve the BCR directory:
```nginx
server {
    listen 8080;
    root /path/to/bazel-central-registry;
    autoindex on;
}
```

### Step 3: Configure Bazel

In `.bazelrc`:
```
common --registry=http://localhost:8080
common --registry=https://bcr.bazel.build  # Fallback (optional)
```

**Limitation**: Requires maintaining the local mirror (pulling updates from GitHub).

## Solution 4: Complete Git Override Strategy

Override ALL dependencies (including transitive) with `git_override` to bypass BCR entirely.

### Current Status

We've already added git_overrides for:
- ✅ rules_rust
- ✅ bazel_skylib
- ✅ platforms
- ✅ rules_cc
- ✅ rules_go
- ✅ rules_shell
- ✅ rules_oci
- ✅ rules_nodejs
- ✅ bazel_features
- ✅ rules_pkg
- ✅ rules_proto
- ✅ rules_python

### Remaining Transitive Dependencies

To identify remaining BCR dependencies:
```bash
bazel mod graph 2>&1 | grep "bcr.bazel.build"
```

Common transitive dependencies that need git_override:
- rules_license
- buildozer
- zlib (C library - may need special handling)
- protobuf
- abseil-cpp
- And others discovered during build

### Adding More Overrides

For each BCR dependency, add to MODULE.bazel:

```python
# Find version in BCR
# https://github.com/bazelbuild/bazel-central-registry/tree/main/modules/<module_name>

git_override(
    module_name = "module_name",
    remote = "https://github.com/org/repo.git",
    tag = "vX.Y.Z",  # or commit = "abc123..."
)
```

## Solution 5: Hybrid Approach with cargo-component

Since the project has a `build.sh` fallback, use cargo-component for development until BCR access is resolved.

### Current Implementation

The `build.sh` script already implements this:
1. Try Bazel first (preferred)
2. Fall back to `cargo component build` if Bazel fails
3. Produces gateway component only (no WAC composition)

### Limitations of Fallback

- ❌ Only builds gateway component
- ❌ No WAC composition with other components
- ❌ Uses stub WIT file (without imports)
- ✅ Good for development/testing
- ✅ Fast iteration

### When to Use

Use this approach to:
- Develop and test individual components
- Verify Rust code compiles
- Test basic functionality
- Iterate quickly without full system build

## Solution 6: Request Pre-Vendored Dependencies

If you have access to another build environment with working BCR access:

1. **On working machine**: Run `bazel vendor`
2. **Create tarball**: `tar -czf bazel-vendor.tar.gz vendor_src/`
3. **Transfer**: Copy tarball to restricted environment
4. **Extract**: `tar -xzf bazel-vendor.tar.gz`
5. **Configure**: Add `common --vendor_dir=vendor_src` to `.bazelrc`
6. **Build offline**: `bazel build //...`

## Solution 7: Use archive_override Instead of git_override

For modules that don't have GitHub repos or have complex setups, use `archive_override`:

```python
# Download archives from GitHub releases or other sources
archive_override(
    module_name = "module_name",
    urls = ["https://github.com/org/repo/archive/refs/tags/vX.Y.Z.tar.gz"],
    strip_prefix = "repo-X.Y.Z",
    integrity = "sha256-...",  # Use `bazel mod download` to get
)
```

## Recommended Action Plan

Given the current blocker, here's the recommended approach:

### Phase 1: Immediate Workaround (Now)
1. ✅ Use cargo-component fallback for development
2. ✅ Continue adding git_overrides as needed
3. ⏳ Document all remaining BCR dependencies

### Phase 2: Proxy Resolution (ASAP)
1. Contact infrastructure team about proxy whitelist
2. Test BCR access from build environment
3. Validate Bazel builds work end-to-end

### Phase 3: Long-term Solution (After Proxy Fix)
1. Run `bazel vendor //...` to vendor all dependencies
2. Commit vendored dependencies to repo
3. Configure CI/CD to use vendored mode
4. Update documentation with build instructions

### Phase 4: Complete Migration to Bazel
1. Remove cargo-component fallback (optional)
2. Full WAC composition with all 8 components
3. Integration tests with composed application
4. Deploy to Spin

## Verification Steps

After implementing any solution:

```bash
# Test clean build
bazel clean --expunge
bazel build //...

# Verify outputs
ls -lh bazel-bin/myt2abrp_app.wasm

# Test with Spin (when ready)
spin up
```

## References

- [Bazel Vendor Mode](https://bazel.build/external/vendor)
- [Bazel External Dependencies](https://bazel.build/external/overview)
- [Bazel Central Registry](https://github.com/bazelbuild/bazel-central-registry)
- [Module Extensions](https://bazel.build/external/extension)
- [Git Override Documentation](https://bazel.build/external/module#git_override)

## Status

**Current**: Using git_override for 12+ direct dependencies, but blocked by transitive dependencies.

**Next Step**: Either fix proxy configuration OR use cargo-component fallback for development until proxy is resolved.
