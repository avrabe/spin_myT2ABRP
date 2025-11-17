#!/usr/bin/env bash
#
# Health Check Script for MyT2ABRP
#
# Comprehensive health checking for all components.
# Can be used in production for monitoring, CI/CD, or manual verification.
#
# Usage:
#   ./health-check.sh [--timeout 5] [--verbose] [--json]
#
# Exit Codes:
#   0 - All healthy
#   1 - Some components unhealthy
#   2 - Critical failure

set -e

# Default settings
TIMEOUT=5
VERBOSE=false
JSON_OUTPUT=false
BASE_URL="${BASE_URL:-http://localhost:3000}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --url)
            BASE_URL="$2"
            shift 2
            ;;
        --help|-h)
            cat << EOF
MyT2ABRP Health Check

Usage: ./health-check.sh [options]

Options:
    --timeout N     Timeout in seconds (default: 5)
    --verbose, -v   Verbose output
    --json          JSON output format
    --url URL       Base URL (default: http://localhost:3000)
    --help, -h      Show this help

Examples:
    ./health-check.sh
    ./health-check.sh --verbose --timeout 10
    ./health-check.sh --json > health.json
    ./health-check.sh --url https://myt2abrp.example.com

Exit Codes:
    0 - All checks passed
    1 - Some checks failed
    2 - Critical failure
EOF
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 2
            ;;
    esac
done

# Colors (only if not JSON output)
if [ "$JSON_OUTPUT" = false ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Health check results
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0
WARNINGS=0

declare -a RESULTS=()

# Helper functions
log_verbose() {
    if [ "$VERBOSE" = true ] && [ "$JSON_OUTPUT" = false ]; then
        echo -e "${BLUE}[INFO]${NC} $1"
    fi
}

check_endpoint() {
    local name="$1"
    local url="$2"
    local expected_code="${3:-200}"
    local check_body="${4:-}"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    log_verbose "Checking $name at $url"

    # Make request
    local response=$(curl -s -w "\n%{http_code}" --max-time "$TIMEOUT" "$url" 2>/dev/null || echo -e "\n000")
    local body=$(echo "$response" | head -n -1)
    local code=$(echo "$response" | tail -n 1)

    local status="PASS"
    local message="OK"

    if [ "$code" != "$expected_code" ]; then
        status="FAIL"
        message="Expected HTTP $expected_code, got $code"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    elif [ -n "$check_body" ] && ! echo "$body" | grep -q "$check_body"; then
        status="FAIL"
        message="Response body doesn't contain expected text"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    else
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    fi

    RESULTS+=("{\"check\":\"$name\",\"status\":\"$status\",\"code\":$code,\"message\":\"$message\"}")

    if [ "$JSON_OUTPUT" = false ]; then
        if [ "$status" = "PASS" ]; then
            echo -e "${GREEN}✓${NC} $name"
        else
            echo -e "${RED}✗${NC} $name - $message"
        fi
    fi
}

check_service() {
    local name="$1"
    local command="$2"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    log_verbose "Checking service: $name"

    if eval "$command" &>/dev/null; then
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        RESULTS+=("{\"check\":\"$name\",\"status\":\"PASS\",\"message\":\"Running\"}")
        if [ "$JSON_OUTPUT" = false ]; then
            echo -e "${GREEN}✓${NC} $name is running"
        fi
    else
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
        RESULTS+=("{\"check\":\"$name\",\"status\":\"FAIL\",\"message\":\"Not running\"}")
        if [ "$JSON_OUTPUT" = false ]; then
            echo -e "${YELLOW}⚠${NC} $name is not running"
        fi
        WARNINGS=$((WARNINGS + 1))
    fi
}

# Start health checks
if [ "$JSON_OUTPUT" = false ]; then
    echo -e "${BLUE}MyT2ABRP Health Check${NC}"
    echo "Target: $BASE_URL"
    echo "Timeout: ${TIMEOUT}s"
    echo ""
fi

# Core application checks
if [ "$JSON_OUTPUT" = false ]; then
    echo -e "${BLUE}Core Application:${NC}"
fi

check_endpoint "Health endpoint" "$BASE_URL/health" 200 "healthy"
check_endpoint "API health endpoint" "$BASE_URL/api/health" 200 "healthy"
check_endpoint "Metrics endpoint" "$BASE_URL/api/metrics" 200
check_endpoint "Main page" "$BASE_URL/" 200
check_endpoint "Static CSS" "$BASE_URL/styles.css" 200
check_endpoint "Static JS" "$BASE_URL/app.js" 200

# API endpoints
if [ "$JSON_OUTPUT" = false ]; then
    echo ""
    echo -e "${BLUE}API Endpoints:${NC}"
fi

check_endpoint "Vehicle status" "$BASE_URL/api/vehicle/status" 200
check_endpoint "Charging status" "$BASE_URL/api/charging/status" 200
check_endpoint "Battery health" "$BASE_URL/api/battery/health" 200
check_endpoint "Range info" "$BASE_URL/api/range" 200

# Optional services (warnings only, don't fail)
if [ "$JSON_OUTPUT" = false ]; then
    echo ""
    echo -e "${BLUE}Optional Services:${NC}"
fi

check_service "Docker" "docker ps"
check_service "Prometheus" "curl -sf http://localhost:9090/-/healthy"
check_service "Grafana" "curl -sf http://localhost:3001/api/health"
check_service "Loki" "curl -sf http://localhost:3100/ready"

# Performance check
if [ "$JSON_OUTPUT" = false ]; then
    echo ""
    echo -e "${BLUE}Performance:${NC}"
fi

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
START_TIME=$(date +%s%3N)
curl -sf "$BASE_URL/health" > /dev/null 2>&1
END_TIME=$(date +%s%3N)
RESPONSE_TIME=$((END_TIME - START_TIME))

if [ $RESPONSE_TIME -lt 100 ]; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    RESULTS+=("{\"check\":\"Response time\",\"status\":\"PASS\",\"value\":$RESPONSE_TIME,\"unit\":\"ms\"}")
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${GREEN}✓${NC} Response time: ${RESPONSE_TIME}ms (excellent)"
    fi
elif [ $RESPONSE_TIME -lt 500 ]; then
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
    RESULTS+=("{\"check\":\"Response time\",\"status\":\"PASS\",\"value\":$RESPONSE_TIME,\"unit\":\"ms\"}")
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${GREEN}✓${NC} Response time: ${RESPONSE_TIME}ms (good)"
    fi
else
    WARNINGS=$((WARNINGS + 1))
    RESULTS+=("{\"check\":\"Response time\",\"status\":\"WARN\",\"value\":$RESPONSE_TIME,\"unit\":\"ms\"}")
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "${YELLOW}⚠${NC} Response time: ${RESPONSE_TIME}ms (slow)"
    fi
fi

# Summary
if [ "$JSON_OUTPUT" = true ]; then
    # JSON output
    echo "{"
    echo "  \"timestamp\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\","
    echo "  \"target\": \"$BASE_URL\","
    echo "  \"total_checks\": $TOTAL_CHECKS,"
    echo "  \"passed\": $PASSED_CHECKS,"
    echo "  \"failed\": $FAILED_CHECKS,"
    echo "  \"warnings\": $WARNINGS,"
    echo "  \"health_score\": $(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc),"
    echo "  \"results\": ["

    for i in "${!RESULTS[@]}"; do
        echo "    ${RESULTS[$i]}"
        if [ $i -lt $((${#RESULTS[@]} - 1)) ]; then
            echo ","
        fi
    done

    echo "  ]"
    echo "}"
else
    # Human-readable output
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}Summary:${NC}"
    echo "  Total checks: $TOTAL_CHECKS"
    echo -e "  ${GREEN}Passed: $PASSED_CHECKS${NC}"
    echo -e "  ${RED}Failed: $FAILED_CHECKS${NC}"
    echo -e "  ${YELLOW}Warnings: $WARNINGS${NC}"

    HEALTH_SCORE=$(echo "scale=2; $PASSED_CHECKS * 100 / $TOTAL_CHECKS" | bc)
    echo "  Health score: ${HEALTH_SCORE}%"

    echo ""
    if [ $FAILED_CHECKS -eq 0 ]; then
        echo -e "${GREEN}✓ All critical checks passed!${NC}"
        EXIT_CODE=0
    else
        echo -e "${RED}✗ Some critical checks failed!${NC}"
        EXIT_CODE=1
    fi

    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}⚠ $WARNINGS warning(s) - optional services not available${NC}"
    fi
fi

# Exit with appropriate code
if [ $FAILED_CHECKS -eq 0 ]; then
    exit 0
else
    exit 1
fi
