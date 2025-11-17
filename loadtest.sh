#!/bin/bash
#
# MyT2ABRP Load Testing Script
# Simulates concurrent users and measures performance under load
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
CONCURRENT_USERS="${CONCURRENT_USERS:-50}"
DURATION="${DURATION:-60}"
RAMP_UP="${RAMP_UP:-10}"

# Test scenarios
declare -a ENDPOINTS=(
    "/"
    "/styles.css"
    "/app.js"
    "/api/vehicle/status"
    "/api/charging/status"
    "/api/range"
    "/api/battery/health"
    "/api/charging/history"
    "/api/analytics/weekly"
    "/health"
)

# Check dependencies
check_deps() {
    for cmd in curl bc; do
        if ! command -v $cmd &> /dev/null; then
            echo -e "${RED}❌ Required command not found: $cmd${NC}"
            exit 1
        fi
    done
}

# Check server
check_server() {
    echo -e "${BLUE}Checking server availability...${NC}"
    if ! curl -sf "$BASE_URL/health" > /dev/null 2>&1; then
        echo -e "${RED}❌ Server is not running at $BASE_URL${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Server is ready${NC}"
    echo
}

# Single request test
single_request_test() {
    local endpoint=$1
    local start=$(date +%s%N)

    local status_code=$(curl -sf -o /dev/null -w "%{http_code}" "$BASE_URL$endpoint" 2>&1)

    local end=$(date +%s%N)
    local time_ms=$(echo "scale=2; ($end - $start) / 1000000" | bc)

    echo "$status_code,$time_ms"
}

# Concurrent load test
load_test_endpoint() {
    local endpoint=$1
    local users=$2
    local duration=$3

    echo -e "${YELLOW}Load Testing: $endpoint${NC}"
    echo "Concurrent Users: $users"
    echo "Duration: ${duration}s"

    local success=0
    local errors=0
    local total_time=0
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))

    # Create temp file for results
    local temp_file=$(mktemp)

    # Launch concurrent workers
    for i in $(seq 1 $users); do
        (
            while [ $(date +%s) -lt $end_time ]; do
                result=$(single_request_test "$endpoint")
                echo "$result" >> "$temp_file"
                sleep 0.1  # Small delay between requests
            done
        ) &
    done

    # Wait for all workers
    wait

    # Analyze results
    while IFS=',' read -r status time; do
        if [ "$status" = "200" ]; then
            ((success++))
            total_time=$(echo "$total_time + $time" | bc)
        else
            ((errors++))
        fi
    done < "$temp_file"

    rm -f "$temp_file"

    # Calculate metrics
    local total=$((success + errors))
    local avg_time=0
    local rps=0

    if [ $success -gt 0 ]; then
        avg_time=$(echo "scale=2; $total_time / $success" | bc)
        rps=$(echo "scale=2; $success / $duration" | bc)
    fi

    # Print results
    echo -e "${GREEN}Results:${NC}"
    echo "  Total Requests: $total"
    echo "  Successful: $success"
    if [ $errors -gt 0 ]; then
        echo -e "  ${RED}Errors: $errors${NC}"
    else
        echo "  Errors: 0"
    fi
    echo "  Average Response Time: ${avg_time} ms"
    echo "  Requests/Second: ${rps}"

    # Performance assessment
    local avg_int=$(printf "%.0f" "$avg_time")
    if [ $avg_int -lt 100 ]; then
        echo -e "  ${GREEN}⚡ Excellent performance under load!${NC}"
    elif [ $avg_int -lt 200 ]; then
        echo -e "  ${GREEN}✅ Good performance under load${NC}"
    elif [ $avg_int -lt 500 ]; then
        echo -e "  ${YELLOW}⚠️  Acceptable performance${NC}"
    else
        echo -e "  ${RED}❌ Poor performance under load${NC}"
    fi

    echo
}

# Ramp-up test
ramp_up_test() {
    local endpoint=$1
    local max_users=$2
    local ramp_duration=$3

    echo -e "${YELLOW}Ramp-Up Test: $endpoint${NC}"
    echo "Ramping from 1 to $max_users users over ${ramp_duration}s"

    local step=$((ramp_duration / 10))
    local user_increment=$((max_users / 10))

    for users in $(seq $user_increment $user_increment $max_users); do
        echo -e "${BLUE}Testing with $users concurrent users...${NC}"
        load_test_endpoint "$endpoint" "$users" "$step"
    done
}

# Stress test - find breaking point
stress_test() {
    local endpoint=$1
    local start_users=10
    local max_users=500
    local duration=30

    echo -e "${RED}Stress Test: Finding Breaking Point${NC}"
    echo "Endpoint: $endpoint"
    echo "Testing from $start_users to $max_users users"

    local current_users=$start_users
    local error_threshold=5  # 5% error rate

    while [ $current_users -le $max_users ]; do
        echo -e "${BLUE}Stress testing with $current_users users...${NC}"

        # Run load test
        local success=0
        local errors=0
        local start_time=$(date +%s)
        local end_time=$((start_time + duration))
        local temp_file=$(mktemp)

        # Launch concurrent workers
        for i in $(seq 1 $current_users); do
            (
                while [ $(date +%s) -lt $end_time ]; do
                    result=$(single_request_test "$endpoint")
                    echo "$result" >> "$temp_file"
                done
            ) &
        done

        wait

        # Analyze results
        while IFS=',' read -r status time; do
            if [ "$status" = "200" ]; then
                ((success++))
            else
                ((errors++))
            fi
        done < "$temp_file"

        rm -f "$temp_file"

        # Check error rate
        local total=$((success + errors))
        local error_rate=0
        if [ $total -gt 0 ]; then
            error_rate=$(echo "scale=2; ($errors * 100) / $total" | bc)
        fi

        echo "  Success: $success, Errors: $errors, Error Rate: ${error_rate}%"

        # Check if breaking point reached
        local er_int=$(printf "%.0f" "$error_rate")
        if [ $er_int -ge $error_threshold ]; then
            echo -e "${RED}❌ Breaking point reached at $current_users concurrent users${NC}"
            echo -e "${RED}   Error rate: ${error_rate}%${NC}"
            break
        fi

        # Increase load
        current_users=$((current_users * 2))
    done

    if [ $current_users -gt $max_users ]; then
        echo -e "${GREEN}✅ Server handled up to $max_users concurrent users successfully!${NC}"
    fi

    echo
}

# Endurance test
endurance_test() {
    local endpoint=$1
    local users=$2
    local duration=$3

    echo -e "${YELLOW}Endurance Test: $endpoint${NC}"
    echo "Sustained load: $users users for ${duration}s"

    local interval=10
    local iterations=$((duration / interval))
    local memory_start=$(ps aux | grep spin | awk '{sum+=$6} END {print sum}')

    for i in $(seq 1 $iterations); do
        local elapsed=$((i * interval))
        echo -e "${BLUE}[$elapsed/${duration}s] Testing...${NC}"

        # Run short load test
        local temp_file=$(mktemp)
        local start_time=$(date +%s)
        local end_time=$((start_time + interval))

        for j in $(seq 1 $users); do
            (
                while [ $(date +%s) -lt $end_time ]; do
                    single_request_test "$endpoint" >> "$temp_file" 2>/dev/null
                done
            ) &
        done

        wait

        # Check success rate
        local success=$(grep -c "^200," "$temp_file" || echo 0)
        local total=$(wc -l < "$temp_file")
        local success_rate=100
        if [ $total -gt 0 ]; then
            success_rate=$(echo "scale=1; ($success * 100) / $total" | bc)
        fi

        # Check memory usage
        local memory_current=$(ps aux | grep spin | awk '{sum+=$6} END {print sum}')
        local memory_growth=0
        if [ -n "$memory_start" ] && [ -n "$memory_current" ]; then
            memory_growth=$(echo "scale=1; (($memory_current - $memory_start) * 100) / $memory_start" | bc)
        fi

        echo "  Success Rate: ${success_rate}% | Memory Growth: ${memory_growth}%"

        rm -f "$temp_file"
    done

    echo -e "${GREEN}✅ Endurance test complete${NC}"
    echo
}

# Full test suite
run_full_suite() {
    echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  MyT2ABRP Load Testing Suite          ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
    echo

    check_deps
    check_server

    # Quick load test on key endpoints
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Quick Load Tests${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    load_test_endpoint "/api/vehicle/status" 20 30
    load_test_endpoint "/api/charging/status" 20 30
    load_test_endpoint "/health" 50 30

    # Ramp-up test
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Ramp-Up Test${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    ramp_up_test "/api/vehicle/status" "$CONCURRENT_USERS" "$RAMP_UP"

    # Stress test (optional - commented out by default)
    # echo -e "${BLUE}══════════════════════════════════════════${NC}"
    # echo -e "${BLUE}  Stress Test${NC}"
    # echo -e "${BLUE}══════════════════════════════════════════${NC}"
    # echo
    # stress_test "/api/vehicle/status"

    # Endurance test
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Endurance Test${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo

    endurance_test "/api/vehicle/status" 10 60

    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  Load Testing Complete!                ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
}

# Command-line interface
case "${1:-full}" in
    load)
        check_deps
        check_server
        load_test_endpoint "${2:-/api/vehicle/status}" "${CONCURRENT_USERS}" "${DURATION}"
        ;;
    ramp)
        check_deps
        check_server
        ramp_up_test "${2:-/api/vehicle/status}" "${CONCURRENT_USERS}" "${RAMP_UP}"
        ;;
    stress)
        check_deps
        check_server
        stress_test "${2:-/api/vehicle/status}"
        ;;
    endurance)
        check_deps
        check_server
        endurance_test "${2:-/api/vehicle/status}" "${CONCURRENT_USERS}" "${DURATION}"
        ;;
    full)
        run_full_suite
        ;;
    help|--help|-h)
        echo "MyT2ABRP Load Testing Tool"
        echo ""
        echo "Usage: $0 [command] [options]"
        echo ""
        echo "Commands:"
        echo "  full                    Run full test suite (default)"
        echo "  load [endpoint]         Quick load test on endpoint"
        echo "  ramp [endpoint]         Ramp-up test from 1 to N users"
        echo "  stress [endpoint]       Stress test to find breaking point"
        echo "  endurance [endpoint]    Sustained load over time"
        echo "  help                    Show this help"
        echo ""
        echo "Environment Variables:"
        echo "  BASE_URL               Server URL (default: http://localhost:3000)"
        echo "  CONCURRENT_USERS       Number of concurrent users (default: 50)"
        echo "  DURATION               Test duration in seconds (default: 60)"
        echo "  RAMP_UP                Ramp-up duration in seconds (default: 10)"
        echo ""
        echo "Examples:"
        echo "  $0                                   # Run full test suite"
        echo "  $0 load /api/vehicle/status          # Quick load test"
        echo "  CONCURRENT_USERS=100 $0 ramp         # Ramp test with 100 users"
        echo "  $0 stress /api/charging/status       # Stress test"
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac
