#!/bin/bash
# Validate all WASM components
# Usage: ./scripts/validate-wasm.sh

set -e

echo "🔍 Validating WASM components..."
echo

# Check if wasm-tools is installed
if ! command -v wasm-tools &> /dev/null; then
    echo "❌ wasm-tools not found. Install with:"
    echo "   cargo install wasm-tools"
    exit 1
fi

TOTAL=0
SUCCESS=0
FAILED=0
FAILED_FILES=()

# Find all WASM files
WASM_FILES=$(find target -name "*.wasm" -type f 2>/dev/null || true)

if [ -z "$WASM_FILES" ]; then
    echo "❌ No WASM files found in target/"
    echo "   Run build scripts first"
    exit 1
fi

for wasm_file in $WASM_FILES; do
    TOTAL=$((TOTAL + 1))
    filename=$(basename "$wasm_file")

    echo -n "[$TOTAL] Validating $filename... "

    if wasm-tools validate "$wasm_file" 2>&1 | grep -q "^$"; then
        SUCCESS=$((SUCCESS + 1))
        echo "✅ PASS"

        # Show component info
        echo "    Size: $(du -h "$wasm_file" | cut -f1)"

        # Extract exports
        exports=$(wasm-tools component wit "$wasm_file" 2>/dev/null | grep "export" | head -3 || echo "    (none)")
        if [ "$exports" != "    (none)" ]; then
            echo "    Exports: $(echo "$exports" | tr '\n' ' ' | sed 's/export//g')"
        fi
    else
        FAILED=$((FAILED + 1))
        FAILED_FILES+=("$filename")
        echo "❌ FAIL"
        wasm-tools validate "$wasm_file" 2>&1 | head -5 | sed 's/^/    /'
    fi
    echo
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Validation Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Total:    $TOTAL files"
echo "Passed:   $SUCCESS files"
echo "Failed:   $FAILED files"
echo

if [ $FAILED -gt 0 ]; then
    echo "❌ Failed files:"
    for file in "${FAILED_FILES[@]}"; do
        echo "   - $file"
    done
    echo
    exit 1
else
    echo "✅ All WASM files are valid!"
    exit 0
fi
