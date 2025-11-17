#!/bin/bash
#
# MyT2ABRP Performance Benchmark Script
# Measures API response times and throughput
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
WARMUP_REQUESTS=10
BENCHMARK_REQUESTS=100
CONCURRENT_REQUESTS=10

# Check if server is running
check_server() {
    echo -e "${BLUE}Checking if server is running...${NC}"
    if ! curl -sf "$BASE_URL/health" > /dev/null 2>&1; then
        echo -e "${RED}❌ Server is not running at $BASE_URL${NC}"
        echo "Start the server with: spin up"
        exit 1
    fi
    echo -e "${GREEN}✅ Server is running${NC}"
    echo
}

# Check dependencies
check_deps() {
    for cmd in curl jq bc; do
        if ! command -v $cmd &> /dev/null; then
            echo -e "${RED}❌ Required command not found: $cmd${NC}"
            echo "Install with: sudo apt-get install $cmd (Ubuntu/Debian) or brew install $cmd (macOS)"
            exit 1
        fi
    done
}

# Warmup function
warmup() {
    echo -e "${BLUE}Warming up with $WARMUP_REQUESTS requests...${NC}"
    for i in $(seq 1 $WARMUP_REQUESTS); do
        curl -sf "$BASE_URL/api/vehicle/status" > /dev/null 2>&1
    done
    echo -e "${GREEN}✅ Warmup complete${NC}"
    echo
}

# Benchmark a single endpoint
benchmark_endpoint() {
    local endpoint=$1
    local description=$2
    local times=()
    local errors=0

    echo -e "${YELLOW}Benchmarking: $description${NC}"
    echo "Endpoint: $BASE_URL$endpoint"
    echo "Requests: $BENCHMARK_REQUESTS"

    # Run requests and measure time
    for i in $(seq 1 $BENCHMARK_REQUESTS); do
        start=$(date +%s%N)
        if curl -sf "$BASE_URL$endpoint" > /dev/null 2>&1; then
            end=$(date +%s%N)
            time_ms=$(echo "scale=2; ($end - $start) / 1000000" | bc)
            times+=($time_ms)
        else
            ((errors++))
        fi

        # Progress indicator
        if [ $((i % 10)) -eq 0 ]; then
            echo -n "."
        fi
    done
    echo

    # Calculate statistics
    if [ ${#times[@]} -eq 0 ]; then
        echo -e "${RED}❌ All requests failed${NC}"
        return
    fi

    # Sort times
    IFS=$'\n' sorted_times=($(sort -n <<<"${times[*]}"))
    unset IFS

    # Calculate percentiles
    count=${#sorted_times[@]}
    p50_index=$((count * 50 / 100))
    p95_index=$((count * 95 / 100))
    p99_index=$((count * 99 / 100))

    min=${sorted_times[0]}
    p50=${sorted_times[$p50_index]}
    p95=${sorted_times[$p95_index]}
    p99=${sorted_times[$p99_index]}
    max=${sorted_times[-1]}

    # Calculate average
    sum=0
    for time in "${times[@]}"; do
        sum=$(echo "$sum + $time" | bc)
    done
    avg=$(echo "scale=2; $sum / ${#times[@]}" | bc)

    # Calculate throughput
    throughput=$(echo "scale=2; 1000 / $avg" | bc)

    # Print results
    echo -e "${GREEN}Results:${NC}"
    echo "  Success: $((BENCHMARK_REQUESTS - errors))/$BENCHMARK_REQUESTS"
    if [ $errors -gt 0 ]; then
        echo -e "  ${RED}Errors: $errors${NC}"
    fi
    echo "  Min:     ${min} ms"
    echo "  Average: ${avg} ms"
    echo "  p50:     ${p50} ms"
    echo "  p95:     ${p95} ms"
    echo "  p99:     ${p99} ms"
    echo "  Max:     ${max} ms"
    echo "  Throughput: ${throughput} req/s"

    # Performance assessment
    avg_int=$(printf "%.0f" $avg)
    if [ $avg_int -lt 50 ]; then
        echo -e "  ${GREEN}⚡ Excellent performance!${NC}"
    elif [ $avg_int -lt 100 ]; then
        echo -e "  ${GREEN}✅ Good performance${NC}"
    elif [ $avg_int -lt 200 ]; then
        echo -e "  ${YELLOW}⚠️  Acceptable performance${NC}"
    else
        echo -e "  ${RED}❌ Slow performance${NC}"
    fi

    echo
}

# Concurrent requests benchmark
benchmark_concurrent() {
    local endpoint=$1
    local description=$2

    echo -e "${YELLOW}Concurrent Requests Test: $description${NC}"
    echo "Endpoint: $BASE_URL$endpoint"
    echo "Concurrent: $CONCURRENT_REQUESTS requests"

    start=$(date +%s%N)
    for i in $(seq 1 $CONCURRENT_REQUESTS); do
        curl -sf "$BASE_URL$endpoint" > /dev/null 2>&1 &
    done
    wait
    end=$(date +%s%N)

    total_time=$(echo "scale=2; ($end - $start) / 1000000" | bc)
    avg_time=$(echo "scale=2; $total_time / $CONCURRENT_REQUESTS" | bc)
    throughput=$(echo "scale=2; $CONCURRENT_REQUESTS * 1000 / $total_time" | bc)

    echo -e "${GREEN}Results:${NC}"
    echo "  Total time: ${total_time} ms"
    echo "  Average:    ${avg_time} ms"
    echo "  Throughput: ${throughput} req/s"
    echo
}

# Static file benchmark
benchmark_static() {
    echo -e "${YELLOW}Static File Performance${NC}"

    # CSS
    start=$(date +%s%N)
    curl -sf "$BASE_URL/styles.css" > /dev/null 2>&1
    end=$(date +%s%N)
    css_time=$(echo "scale=2; ($end - $start) / 1000000" | bc)

    # JS
    start=$(date +%s%N)
    curl -sf "$BASE_URL/app.js" > /dev/null 2>&1
    end=$(date +%s%N)
    js_time=$(echo "scale=2; ($end - $start) / 1000000" | bc)

    # HTML
    start=$(date +%s%N)
    curl -sf "$BASE_URL/" > /dev/null 2>&1
    end=$(date +%s%N)
    html_time=$(echo "scale=2; ($end - $start) / 1000000" | bc)

    echo "  CSS:  ${css_time} ms"
    echo "  JS:   ${js_time} ms"
    echo "  HTML: ${html_time} ms"
    echo
}

# Main benchmark suite
main() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  MyT2ABRP Performance Benchmark       ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo

    check_deps
    check_server
    warmup

    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  API Endpoint Benchmarks${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    # Health/Monitoring
    benchmark_endpoint "/health" "Health Check"
    benchmark_endpoint "/api/metrics" "Metrics Endpoint"

    # Vehicle endpoints
    benchmark_endpoint "/api/vehicle/status" "Vehicle Status"
    benchmark_endpoint "/api/range" "Range Information"
    benchmark_endpoint "/api/battery/health" "Battery Health"

    # Charging endpoints
    benchmark_endpoint "/api/charging/status" "Charging Status"
    benchmark_endpoint "/api/charging/history" "Charging History"

    # Analytics endpoints
    benchmark_endpoint "/api/analytics/weekly" "Weekly Analytics"
    benchmark_endpoint "/api/analytics/costs" "Cost Analytics"
    benchmark_endpoint "/api/analytics/efficiency" "Efficiency Metrics"

    # Alerts
    benchmark_endpoint "/api/alerts/active" "Active Alerts"

    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Static File Performance${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    benchmark_static

    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Concurrent Request Performance${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    benchmark_concurrent "/api/vehicle/status" "Vehicle Status (Concurrent)"

    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Benchmark Complete!                   ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
}

# Run if executed directly
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi
