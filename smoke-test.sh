#!/usr/bin/env bash
#
# Smoke Test Script for MyT2ABRP
#
# Runs quick smoke tests to verify basic functionality after deployment.
# Useful for CI/CD pipelines and post-deployment verification.
#
# Usage:
#   ./smoke-test.sh [--url URL] [--verbose]

set -e

# Settings
BASE_URL="${BASE_URL:-http://localhost:3000}"
VERBOSE=false
FAILED=0

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --url)
            BASE_URL="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

pass() {
    echo -e "${GREEN}✓${NC} $1"
}

fail() {
    echo -e "${RED}✗${NC} $1"
    FAILED=$((FAILED + 1))
}

test_endpoint() {
    local name="$1"
    local url="$2"
    local expected_code="${3:-200}"

    if [ "$VERBOSE" = true ]; then
        log "Testing: $name"
    fi

    local code=$(curl -s -o /dev/null -w "%{http_code}" "$url" 2>/dev/null || echo "000")

    if [ "$code" = "$expected_code" ]; then
        pass "$name (HTTP $code)"
    else
        fail "$name (expected $expected_code, got $code)"
    fi
}

echo -e "${BLUE}MyT2ABRP Smoke Tests${NC}"
echo "Target: $BASE_URL"
echo ""

# Test 1: Health endpoint
log "Health Checks"
test_endpoint "Health endpoint responds" "$BASE_URL/health" 200
test_endpoint "API health endpoint responds" "$BASE_URL/api/health" 200

# Test 2: Metrics
log "Metrics"
test_endpoint "Metrics endpoint responds" "$BASE_URL/api/metrics" 200

# Test 3: Static files
log "Static Files"
test_endpoint "Main page loads" "$BASE_URL/" 200
test_endpoint "CSS loads" "$BASE_URL/styles.css" 200
test_endpoint "JavaScript loads" "$BASE_URL/app.js" 200

# Test 4: API endpoints
log "API Endpoints"
test_endpoint "Vehicle status API" "$BASE_URL/api/vehicle/status" 200
test_endpoint "Charging status API" "$BASE_URL/api/charging/status" 200
test_endpoint "Battery health API" "$BASE_URL/api/battery/health" 200
test_endpoint "Range API" "$BASE_URL/api/range" 200

# Test 5: 404 handling
log "Error Handling"
test_endpoint "404 for non-existent page" "$BASE_URL/this-does-not-exist" 404

# Test 6: Response content
log "Content Validation"
HEALTH_RESPONSE=$(curl -s "$BASE_URL/health")
if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    pass "Health response contains 'healthy'"
else
    fail "Health response missing 'healthy' status"
fi

METRICS_RESPONSE=$(curl -s "$BASE_URL/api/metrics")
if echo "$METRICS_RESPONSE" | grep -q "uptime_seconds"; then
    pass "Metrics contain uptime data"
else
    fail "Metrics missing uptime data"
fi

# Test 7: Performance
log "Performance"
START=$(date +%s%3N)
curl -s "$BASE_URL/health" > /dev/null
END=$(date +%s%3N)
RESPONSE_TIME=$((END - START))

if [ $RESPONSE_TIME -lt 100 ]; then
    pass "Response time: ${RESPONSE_TIME}ms (excellent)"
elif [ $RESPONSE_TIME -lt 500 ]; then
    pass "Response time: ${RESPONSE_TIME}ms (good)"
else
    echo -e "${YELLOW}⚠${NC} Response time: ${RESPONSE_TIME}ms (slow but acceptable)"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All smoke tests passed!${NC}"
    echo "Deployment verified successfully."
    exit 0
else
    echo -e "${RED}✗ $FAILED test(s) failed!${NC}"
    echo "Deployment may have issues - investigate before production use."
    exit 1
fi
