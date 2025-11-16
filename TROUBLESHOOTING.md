# Troubleshooting Guide - Toyota MyT2ABRP

This guide covers common issues and their solutions when building, testing, and deploying the Toyota MyT2ABRP project.

## Table of Contents

- [Build Issues](#build-issues)
- [Runtime Issues](#runtime-issues)
- [Testing Issues](#testing-issues)
- [Deployment Issues](#deployment-issues)
- [Performance Issues](#performance-issues)
- [Tools Issues](#tools-issues)

## Build Issues

### cargo-component Not Found

**Symptom:**
```
bash: cargo-component: command not found
```

**Solution:**
```bash
# Install cargo-component
cargo install cargo-component --locked

# Verify installation
cargo component --version
```

### WASM Target Not Installed

**Symptom:**
```
error[E0463]: can't find crate for `core`
= note: the `wasm32-wasip2` target may not be installed
```

**Solution:**
```bash
# Install required WASM targets
rustup target add wasm32-wasip1  # For cargo-component
rustup target add wasm32-wasip2  # For Spin SDK

# Verify installation
rustup target list --installed | grep wasm32
```

### Workspace Conflicts

**Symptom:**
```
error: current package believes it's in a workspace when it's not
```

**Solution:**
Add to root `Cargo.toml`:
```toml
[workspace]
exclude = [
    "test-http",
    # other standalone packages
]
```

### WIT Interface Not Found

**Symptom:**
```
package 'wasi:http@0.2.0' not found
```

**Solution:**

**Option 1:** Use Spin SDK instead of raw WASI bindings:
```toml
# Cargo.toml
[dependencies]
spin-sdk = "5.1.1"
```

**Option 2:** Add WASI WIT files to `deps/` directory:
```bash
mkdir -p wit/deps/wasi
cd wit/deps/wasi
wget https://github.com/WebAssembly/wasi-http/raw/main/wit/handler.wit
wget https://github.com/WebAssembly/wasi-http/raw/main/wit/types.wit
```

### BCR Proxy Authentication Error

**Symptom:**
```
ERROR: HTTP/1.1 401 Unauthorized
Failed to fetch registry file from https://bcr.bazel.build/
```

**Root Cause:** Corporate proxy blocks Bazel Central Registry access

**Solutions:**

1. **Use git_override** (Recommended for now):
   - Already implemented in MODULE.bazel
   - Bypasses BCR for direct dependencies

2. **Configure proxy bypass**:
   ```bash
   export NO_PROXY="bcr.bazel.build,$NO_PROXY"
   ```

3. **Use local Bazel registry** (see BCR_PROXY_WORKAROUNDS.md)

4. **Use cargo-component** as alternative build path

## Runtime Issues

### Spin Not Found

**Symptom:**
```
bash: spin: command not found
```

**Solution:**
```bash
# Install Spin CLI
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash -s -- --version v3.0.0

# Move to PATH
sudo mv spin /usr/local/bin/
# or
mv spin ~/bin/spin && export PATH="$HOME/bin:$PATH"

# Verify
spin --version
```

### Port Already in Use

**Symptom:**
```
Error: Address already in use (os error 98)
```

**Solution:**
```bash
# Find process using port 3000
lsof -i :3000
# or
netstat -tlnp | grep 3000

# Kill the process
kill <PID>

# Or use different port
spin up --listen 127.0.0.1:3001
```

### Component Failed to Load

**Symptom:**
```
Error: failed to load component from `target/wasm32-wasip2/release/component.wasm`
```

**Solution:**

1. **Verify WASM file exists**:
   ```bash
   ls -lh target/wasm32-wasip2/release/*.wasm
   ```

2. **Rebuild component**:
   ```bash
   cargo clean
   cargo build --target wasm32-wasip2 --release
   ```

3. **Validate WASM**:
   ```bash
   wasm-tools validate target/wasm32-wasip2/release/component.wasm
   ```

4. **Check spin.toml**:
   ```toml
   [component.component-name]
   source = "target/wasm32-wasip2/release/component.wasm"  # Correct path?
   ```

### WASI Preview Mismatch

**Symptom:**
```
Error: component is not compatible with this version of Spin
```

**Solution:**

Components built with cargo-component use WASI P1, Spin SDK uses WASI P2:

```bash
# For Spin deployment, use Spin SDK:
cargo build --target wasm32-wasip2 --release

# For Component Model/WAC composition:
cargo component build --release
```

## Testing Issues

### Playwright Not Found

**Symptom:**
```
bash: playwright: command not found
```

**Solution:**
```bash
cd tests/e2e
npm install
npx playwright install  # Install browser binaries
```

### Tests Fail - Server Not Running

**Symptom:**
```
Error: connect ECONNREFUSED 127.0.0.1:3000
```

**Solution:**

1. **Start Spin server first**:
   ```bash
   cd test-http
   ~/bin/spin up &
   sleep 2  # Wait for server to start
   ```

2. **Or use Playwright's webServer** (already configured in playwright.config.js)

3. **Check server logs**:
   ```bash
   cat test-http/.spin/logs/*
   ```

### Test Timeouts

**Symptom:**
```
Test timeout of 30000ms exceeded
```

**Solution:**

1. **Increase timeout** in `playwright.config.js`:
   ```js
   timeout: 60 * 1000,  // 60 seconds
   ```

2. **Check server performance**:
   ```bash
   curl -w "@curl-format.txt" http://127.0.0.1:3000/health
   ```

3. **Reduce test concurrency**:
   ```js
   workers: 1,  // Run tests sequentially
   ```

### Node.js Version Mismatch

**Symptom:**
```
error: package requires Node.js version >=18
```

**Solution:**
```bash
# Install nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash

# Install Node.js 22
nvm install 22
nvm use 22

# Verify
node --version  # Should show v22.x.x
```

## Deployment Issues

### Fermyon Cloud Authentication

**Symptom:**
```
Error: Not logged in to Fermyon Cloud
```

**Solution:**
```bash
# Login to Fermyon Cloud
spin login

# Verify authentication
spin cloud apps list
```

### Missing Environment Variables

**Symptom:**
```
Error: environment variable 'VARIABLE_NAME' not found
```

**Solution:**

Add to `spin.toml`:
```toml
[component.component-name.variables]
VARIABLE_NAME = "{{ variable_name }}"

[[variables]]
name = "variable_name"
default = "default_value"
```

Or set via CLI:
```bash
spin up --env VARIABLE_NAME=value
```

### Outbound HTTP Blocked

**Symptom:**
```
Error: Outbound HTTP requests not allowed
```

**Solution:**

Update `spin.toml`:
```toml
[component.component-name]
allowed_outbound_hosts = [
    "https://api.example.com",
    "https://*.example.com",  # Wildcard supported
]
```

## Performance Issues

### Slow Build Times

**Symptoms:**
- Builds take > 5 minutes
- Incremental builds are slow

**Solutions:**

1. **Use faster linker** (mold):
   ```toml
   # .cargo/config.toml
   [target.x86_64-unknown-linux-gnu]
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

2. **Enable incremental compilation**:
   ```toml
   [build]
   incremental = true
   ```

3. **Use sccache**:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

4. **Adjust codegen units**:
   ```toml
   [profile.dev]
   codegen-units = 256  # Faster builds, larger binaries
   ```

### Slow Response Times

**Symptoms:**
- API responses > 100ms
- Timeouts occur

**Solutions:**

1. **Enable release mode**:
   ```bash
   cargo build --target wasm32-wasip2 --release
   ```

2. **Profile the code**:
   ```bash
   # Add to Cargo.toml
   [profile.release]
   debug = true  # Enable symbols for profiling

   # Profile with perf
   perf record spin up
   perf report
   ```

3. **Check resource limits**:
   ```toml
   [component.component-name]
   allowed_outbound_hosts = ["*"]  # Too permissive?
   ```

4. **Monitor with metrics**:
   ```bash
   # Add prometheus metrics endpoint
   curl http://127.0.0.1:3000/metrics
   ```

### High Memory Usage

**Symptoms:**
- OOM errors
- Process killed by system

**Solutions:**

1. **Optimize WASM size**:
   ```toml
   [profile.release]
   opt-level = "z"  # Optimize for size
   lto = true
   ```

2. **Use wasm-opt**:
   ```bash
   wasm-opt -Oz input.wasm -o output.wasm
   ```

3. **Check for memory leaks**:
   - Review string allocations
   - Check Vec::with_capacity usage
   - Profile with valgrind (native builds)

## Tools Issues

### wasm-tools Not Found

**Solution:**
```bash
cargo install wasm-tools --locked
```

### wasmtime Not Found

**Solution:**
```bash
cargo install wasmtime-cli --locked
```

### wac Not Found

**Solution:**
```bash
cargo install wac-cli --locked
```

### Bazelisk Installation Failed

**Symptom:**
```
sudo: /etc/sudo.conf is owned by uid 999, should be 0
```

**Solution:**
```bash
# Install to user directory instead
mkdir -p ~/bin
cd ~/bin
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.27.0/bazelisk-linux-amd64
mv bazelisk-linux-amd64 bazelisk
chmod +x bazelisk
ln -s bazelisk bazel

# Add to PATH
export PATH="$HOME/bin:$PATH"
```

## Common Error Messages

### "No such file or directory"

**Typical Causes:**
1. File path is wrong (absolute vs relative)
2. File hasn't been built yet
3. Wrong working directory

**Debug Steps:**
```bash
# Show current directory
pwd

# List files
ls -la

# Find files
find . -name "*.wasm"
```

### "Permission denied"

**Typical Causes:**
1. Script not executable
2. Port requires sudo (< 1024)

**Solution:**
```bash
# Make script executable
chmod +x script.sh

# Use port > 1024 (no sudo needed)
spin up --listen 127.0.0.1:3000
```

### "Address already in use"

**Solution:**
```bash
# Find and kill process
lsof -ti:3000 | xargs kill -9

# Or use different port
spin up --listen 127.0.0.1:8080
```

## Getting More Help

### Debug Logging

Enable verbose logging:
```bash
# Rust builds
RUST_LOG=debug cargo build

# Spin runtime
RUST_LOG=spin=trace spin up

# Tests
DEBUG=pw:api npm test
```

### Component Inspection

```bash
# Show component info
wasm-tools component wit component.wasm

# Print exports
wasm-tools component wit component.wasm | grep export

# Validate
wasm-tools validate component.wasm
```

### Check Versions

```bash
# Rust toolchain
rustc --version
cargo --version

# WASM tools
cargo-component --version
wasm-tools --version
wasmtime --version
wac --version

# Spin
spin --version

# Node.js
node --version
npm --version
npx playwright --version
```

### Community Resources

- **Spin Discord**: https://discord.gg/AAFNfS7NGf
- **Component Model**: https://component-model.bytecodealliance.org/
- **Fermyon Docs**: https://developer.fermyon.com/
- **Bazel Slack**: https://slack.bazel.build/

### Filing Issues

When reporting issues, include:

1. **Environment**:
   ```bash
   uname -a
   rustc --version
   spin --version
   ```

2. **Error output**:
   ```bash
   command 2>&1 | tee error.log
   ```

3. **Reproduction steps**:
   - Minimal code example
   - Build commands used
   - Expected vs actual behavior

4. **Configurations**:
   - Cargo.toml
   - spin.toml
   - MODULE.bazel (if using Bazel)

---

**Last Updated**: 2025-11-16
**Maintained By**: Toyota MyT2ABRP Team
