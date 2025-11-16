#!/bin/bash
# Build all Toyota MyT2ABRP components
# Usage: ./scripts/build-all-components.sh [--release]

set -e

PROFILE="dev"
if [ "$1" == "--release" ]; then
    PROFILE="release"
    RELEASE_FLAG="--release"
fi

echo "🏗️  Building all Toyota MyT2ABRP components ($PROFILE)"
echo

START_TIME=$(date +%s)

# Component directories
COMPONENTS=(
    "validation"
    "retry-logic"
    "circuit-breaker"
    "metrics"
    "toyota-api-types"
    "data-transform"
    "business-logic"
)

# Track build results
TOTAL=0
SUCCESS=0
FAILED=0
SIZES=()

for component in "${COMPONENTS[@]}"; do
    TOTAL=$((TOTAL + 1))
    echo "[$TOTAL/${#COMPONENTS[@]}] Building $component..."

    if cargo component build --manifest-path "components/$component/Cargo.toml" $RELEASE_FLAG 2>&1 | tail -3; then
        SUCCESS=$((SUCCESS + 1))

        # Get WASM file size
        if [ "$PROFILE" == "release" ]; then
            WASM_FILE="target/wasm32-wasip1/release/toyota_${component/-/_}.wasm"
        else
            WASM_FILE="target/wasm32-wasip1/debug/toyota_${component/-/_}.wasm"
        fi

        if [ -f "$WASM_FILE" ]; then
            SIZE=$(du -h "$WASM_FILE" | cut -f1)
            SIZES+=("$component: $SIZE")
            echo "✅ Built: $SIZE"
        else
            echo "✅ Built (size unknown)"
        fi
    else
        FAILED=$((FAILED + 1))
        echo "❌ Failed to build $component"
    fi
    echo
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Build Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Total:    $TOTAL components"
echo "Success:  $SUCCESS components"
echo "Failed:   $FAILED components"
echo "Duration: ${DURATION}s"
echo

if [ ${#SIZES[@]} -gt 0 ]; then
    echo "📦 Component Sizes:"
    for size in "${SIZES[@]}"; do
        echo "  $size"
    done
    echo
fi

if [ $FAILED -eq 0 ]; then
    echo "✅ All components built successfully!"
    exit 0
else
    echo "❌ Some components failed to build"
    exit 1
fi
