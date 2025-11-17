#!/bin/bash
# Compose all Toyota MyT2ABRP components with WAC

set -e

echo "🔧 Composing Toyota MyT2ABRP components..."
echo

START_TIME=$(date +%s)

# Build all components first
echo "Step 1: Building all components..."
./scripts/build-all-components.sh --release
echo

# Check if gateway is built
if [ ! -f "components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm" ]; then
    echo "Building gateway component..."
    cargo component build --manifest-path components/gateway/Cargo.toml --release
    echo
fi

# Check if wac is installed
if ! command -v wac &> /dev/null; then
    echo "❌ wac not found. Installing..."
    cargo install wac-cli --locked
    echo
fi

# Compose with WAC
echo "Step 2: Running WAC composition..."
wac plug \
  --plug toyota:validation/validator@0.1.0=target/wasm32-wasip1/release/toyota_validation.wasm \
  --plug toyota:retry/strategy@0.1.0=target/wasm32-wasip1/release/toyota_retry_logic.wasm \
  --plug toyota:circuit-breaker/breaker@0.1.0=target/wasm32-wasip1/release/toyota_circuit_breaker.wasm \
  --plug toyota:metrics/collector@0.1.0=target/wasm32-wasip1/release/toyota_metrics.wasm \
  --plug toyota:api-types/models@0.1.0=target/wasm32-wasip1/release/toyota_api_types.wasm \
  --plug toyota:data-transform/converter@0.1.0=target/wasm32-wasip1/release/toyota_data_transform.wasm \
  --plug toyota:business-logic/jwt@0.1.0=target/wasm32-wasip1/release/toyota_business_logic.wasm \
  components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm \
  -o composed-app.wasm

echo "✅ WAC composition complete"
echo

# Validate composed component
echo "Step 3: Validating composed component..."
if wasm-tools validate composed-app.wasm; then
    echo "✅ Validation passed"
else
    echo "❌ Validation failed"
    exit 1
fi
echo

# Print component info
echo "Step 4: Component information:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
wasm-tools component wit composed-app.wasm | head -30
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Print sizes
echo "Step 5: Size comparison:"
ORIG_TOTAL=0
for wasm in target/wasm32-wasip1/release/toyota_*.wasm components/gateway/target/wasm32-wasip1/release/toyota_gateway.wasm; do
    if [ -f "$wasm" ]; then
        SIZE=$(stat -f%z "$wasm" 2>/dev/null || stat -c%s "$wasm" 2>/dev/null)
        ORIG_TOTAL=$((ORIG_TOTAL + SIZE))
    fi
done

COMPOSED_SIZE=$(stat -f%z composed-app.wasm 2>/dev/null || stat -c%s composed-app.wasm 2>/dev/null)
COMPOSED_SIZE_H=$(du -h composed-app.wasm | cut -f1)

echo "  Individual components total: $(numfmt --to=iec $ORIG_TOTAL 2>/dev/null || echo "$ORIG_TOTAL bytes")"
echo "  Composed component:          $COMPOSED_SIZE_H"

if [ $COMPOSED_SIZE -lt $ORIG_TOTAL ]; then
    SAVINGS=$((ORIG_TOTAL - COMPOSED_SIZE))
    PERCENT=$((SAVINGS * 100 / ORIG_TOTAL))
    echo "  Space saved:                 $(numfmt --to=iec $SAVINGS 2>/dev/null || echo "$SAVINGS bytes") ($PERCENT%)"
fi
echo

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Composition complete: composed-app.wasm"
echo "   Total time: ${DURATION}s"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
