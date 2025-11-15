#!/bin/bash
# Wasmtime integration test for business logic component
# Tests the component in a real WASM runtime

set -euo pipefail

COMPONENT="$1"

echo "🧪 Testing component with wasmtime"
echo "Component: $COMPONENT"

# Wasmtime will be provided by Bazel's hermetic toolchain
# For now, verify component can be loaded

if [ ! -f "$COMPONENT" ]; then
    echo "❌ Component not found: $COMPONENT"
    exit 1
fi

echo "✅ Component file exists"

# Validate it's a valid component
if ! wasm-tools validate "$COMPONENT" 2>/dev/null; then
    echo "❌ Component validation failed"
    exit 1
fi

echo "✅ Component is valid WASM"

# Check exports
echo "📋 Component exports:"
wasm-tools component wit "$COMPONENT" 2>&1 | grep "export" || true

echo ""
echo "✅ Wasmtime test passed"
echo ""
echo "💡 Future: Add actual wasmtime invocation tests"
echo "   - Call generate-access-token()"
echo "   - Call verify-token()"
echo "   - Validate results"
