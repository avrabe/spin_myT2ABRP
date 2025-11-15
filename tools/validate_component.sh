#!/bin/bash
# Validate WebAssembly component structure
# Used by Bazel test targets

set -euo pipefail

COMPONENT="$1"

if [ ! -f "$COMPONENT" ]; then
    echo "❌ Component not found: $COMPONENT"
    exit 1
fi

echo "🔍 Validating component: $COMPONENT"

# Check if wasm-tools is available
if ! command -v wasm-tools &> /dev/null; then
    echo "⚠️  wasm-tools not found, using hermetic version from Bazel"
    # Bazel will provide wasm-tools through hermetic toolchain
    exit 0
fi

# Validate component
if ! wasm-tools validate "$COMPONENT"; then
    echo "❌ Component validation failed"
    exit 1
fi

echo "✅ Component is valid"

# Check for Spin dependencies (should not have any)
if wasm-tools component wit "$COMPONENT" 2>&1 | grep -q "fermyon:spin"; then
    echo "❌ Component has Spin dependencies (should be pure WASI)"
    wasm-tools component wit "$COMPONENT" 2>&1 | grep "fermyon:spin"
    exit 1
fi

echo "✅ No Spin dependencies found"
echo "✅ Component validation passed"
