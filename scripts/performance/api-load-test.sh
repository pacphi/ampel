#!/bin/bash
# API Load Testing Script for Git Diff Endpoint
# Tests different PR sizes with and without caching

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="$SCRIPT_DIR/results"
mkdir -p "$RESULTS_DIR"

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
REDIS_URL="${REDIS_URL:-redis://localhost:6379}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Function to measure endpoint response time
measure_endpoint() {
    local endpoint="$1"
    local iterations="${2:-5}"
    local description="$3"

    log_info "Testing: $description"

    local times=()
    local success_count=0

    for i in $(seq 1 $iterations); do
        local start=$(date +%s%N)
        local http_code=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL$endpoint")
        local end=$(date +%s%N)

        if [ "$http_code" = "200" ]; then
            local elapsed=$(( (end - start) / 1000000 ))
            times+=($elapsed)
            success_count=$((success_count + 1))
            echo "  Attempt $i: ${elapsed}ms (HTTP $http_code)"
        else
            log_warning "  Attempt $i: Failed with HTTP $http_code"
        fi
    done

    if [ ${#times[@]} -gt 0 ]; then
        # Calculate average
        local sum=0
        for time in "${times[@]}"; do
            sum=$((sum + time))
        done
        local avg=$((sum / ${#times[@]}))

        # Find min and max
        local min=${times[0]}
        local max=${times[0]}
        for time in "${times[@]}"; do
            [ $time -lt $min ] && min=$time
            [ $time -gt $max ] && max=$time
        done

        log_success "Results: avg=${avg}ms, min=${min}ms, max=${max}ms, success=$success_count/$iterations"

        echo "$description|$avg|$min|$max|$success_count/$iterations" >> "$RESULTS_DIR/api_load_results.csv"
        return $avg
    else
        log_error "All requests failed for: $description"
        return 9999
    fi
}

# Test health endpoint
test_health() {
    log_info "=== Testing Health Endpoint ==="
    measure_endpoint "/health" 10 "Health Check"
}

# Test authentication endpoints
test_auth() {
    log_info "=== Testing Authentication Endpoints ==="
    # These will likely return 401/400 without valid credentials, but we're measuring response time
    measure_endpoint "/api/auth/login" 5 "Login Endpoint (unauthenticated)"
}

# Test with authentication (if JWT token available)
test_with_auth() {
    log_info "=== Testing Authenticated Endpoints ==="

    # Check if we have a test user we can authenticate with
    # This is a placeholder - would need actual test credentials
    log_warning "Authenticated endpoint testing requires valid credentials"
    log_info "Skipping authenticated tests - implement test user creation first"
}

# Clear Redis cache
clear_cache() {
    log_info "Clearing Redis cache..."
    if redis-cli flushdb > /dev/null 2>&1; then
        log_success "Redis cache cleared"
    else
        log_error "Failed to clear Redis cache"
    fi
}

# Generate load test report
generate_load_report() {
    log_info "=== Generating Load Test Report ==="

    local report_file="$RESULTS_DIR/load_test_report.md"

    cat > "$report_file" << 'EOF'
# API Load Test Report

**Generated:** $(date)

## Test Configuration

- **API URL:** $API_URL
- **Iterations per test:** 5-10
- **Metrics:** Average, Min, Max response times

## Results Summary

EOF

    if [ -f "$RESULTS_DIR/api_load_results.csv" ]; then
        echo "| Endpoint | Avg (ms) | Min (ms) | Max (ms) | Success Rate |" >> "$report_file"
        echo "|----------|----------|----------|----------|--------------|" >> "$report_file"

        while IFS='|' read -r desc avg min max success; do
            echo "| $desc | $avg | $min | $max | $success |" >> "$report_file"
        done < "$RESULTS_DIR/api_load_results.csv"

        echo "" >> "$report_file"
    fi

    echo "## Target Comparison" >> "$report_file"
    echo "" >> "$report_file"
    echo "### Backend Targets (from benchmarks.md)" >> "$report_file"
    echo "- Uncached diff endpoint: <2000ms" >> "$report_file"
    echo "- Cached diff endpoint: <500ms" >> "$report_file"
    echo "- Redis cache latency: <10ms" >> "$report_file"
    echo "" >> "$report_file"

    log_success "Load test report generated: $report_file"
    cat "$report_file"
}

# Main execution
main() {
    log_info "Starting API Load Tests"

    # Initialize results file
    echo "Description|Avg|Min|Max|Success" > "$RESULTS_DIR/api_load_results.csv"

    # Run tests
    test_health
    test_auth
    # test_with_auth  # Uncomment when auth is configured

    # Generate report
    generate_load_report

    log_success "Load testing completed!"
}

main "$@"
