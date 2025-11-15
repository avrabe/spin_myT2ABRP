#!/bin/bash
# Build script for Toyota MyT2ABRP
# Uses Bazel if available, falls back to cargo-component

set -e

# Try Bazel first (preferred for WAC composition)
if command -v bazel &> /dev/null; then
    echo "🏗️  Building with Bazel (WAC composition)..."
    bazel build //:myt2abrp_app --compilation_mode=opt

    # Verify output exists
    if [ -f "bazel-bin/myt2abrp_app.wasm" ]; then
        echo "✅ Built: bazel-bin/myt2abrp_app.wasm"
        ls -lh bazel-bin/myt2abrp_app.wasm
        exit 0
    else
        echo "❌ Bazel build succeeded but output not found"
        exit 1
    fi
fi

# Fallback to cargo-component
echo "⚠️  Bazel not found, using cargo-component fallback..."
echo "   (Note: This builds gateway only, not full WAC composition)"
echo

# Build gateway component
cargo component build --manifest-path components/gateway/Cargo.toml --release

# Create bazel-bin directory structure if needed
mkdir -p bazel-bin

# Copy to expected location
cp target/wasm32-wasip1/release/toyota_gateway.wasm bazel-bin/myt2abrp_app.wasm

echo "✅ Built: bazel-bin/myt2abrp_app.wasm (cargo-component fallback)"
ls -lh bazel-bin/myt2abrp_app.wasm
echo
echo "⚠️  WARNING: This is the gateway component only."
echo "   For full WAC composition with all components, install Bazel:"
echo "   brew install bazelisk  # macOS"
echo "   npm install -g @bazel/bazelisk  # npm"
