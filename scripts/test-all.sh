#!/bin/bash
# Complete test suite for Toyota MyT2ABRP
# Usage: ./scripts/test-all.sh

set -e

echo "🧪 Toyota MyT2ABRP Complete Test Suite"
echo "======================================="
echo

# Track results
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# 1. Build all components
echo "Step 1: Building all components..."
if ./scripts/build-all-components.sh --release; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ Component builds passed"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ Component builds failed"
    exit 1
fi
echo

# 2. Validate WASM
echo "Step 2: Validating WASM components..."
if ./scripts/validate-wasm.sh; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ WASM validation passed"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ WASM validation failed"
    exit 1
fi
echo

# 3. Build test component
echo "Step 3: Building test HTTP component..."
cd test-http
if cargo build --target wasm32-wasip2 --release; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ Test component build passed"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ Test component build failed"
    exit 1
fi
cd ..
echo

# 4. Start Spin server
echo "Step 4: Starting Spin server..."
cd test-http
~/bin/spin up &
SPIN_PID=$!
cd ..

# Wait for server to start
sleep 3

# Check if server is running
if kill -0 $SPIN_PID 2>/dev/null; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ Spin server started (PID: $SPIN_PID)"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ Spin server failed to start"
    exit 1
fi
echo

# 5. Quick smoke test
echo "Step 5: Smoke testing endpoints..."
if curl -s http://127.0.0.1:3000/health | grep -q "healthy"; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ Smoke test passed"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ Smoke test failed"
    kill $SPIN_PID 2>/dev/null || true
    exit 1
fi
echo

# 6. Run Playwright tests
echo "Step 6: Running Playwright E2E tests..."
cd tests/e2e
if npm test; then
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_PASSED=$((TESTS_PASSED + 1))
    echo "✅ Playwright tests passed"
else
    TESTS_RUN=$((TESTS_RUN + 1))
    TESTS_FAILED=$((TESTS_FAILED + 1))
    echo "❌ Playwright tests failed"
    kill $SPIN_PID 2>/dev/null || true
    exit 1
fi
cd ../..
echo

# Cleanup
echo "Cleaning up..."
kill $SPIN_PID 2>/dev/null || true
sleep 1

# Summary
echo
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Test Suite Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Tests Run:    $TESTS_RUN"
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $TESTS_FAILED"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo "✅ All tests passed!"
    exit 0
else
    echo "❌ Some tests failed"
    exit 1
fi
