#!/bin/bash
# Comprehensive Performance Benchmark Suite for Ampel
# Based on docs/performance/benchmarks.md

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/scripts/performance/results"
mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if services are running
check_services() {
    log_info "Checking if required services are running..."

    # Check API
    if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
        log_error "API server is not running on port 8080"
        log_info "Please start API with: make dev-api"
        exit 1
    fi
    log_success "API server is running"

    # Check Redis
    if ! redis-cli ping > /dev/null 2>&1; then
        log_error "Redis is not running"
        log_info "Please start Redis with: redis-server"
        exit 1
    fi
    log_success "Redis is running"

    # Check Frontend
    if ! curl -s http://localhost:5173 > /dev/null 2>&1; then
        log_warning "Frontend is not running on port 5173 (optional for backend tests)"
    else
        log_success "Frontend is running"
    fi
}

# Phase 1: Backend Performance Tests
run_backend_tests() {
    log_info "=== Phase 1: Backend Performance Tests ==="

    # Test if we have any PRs in the database
    log_info "Checking for test data..."

    # Simple API response time test
    log_info "Testing API health endpoint response time..."
    local health_times=()
    for i in {1..10}; do
        local start=$(date +%s%N)
        curl -s http://localhost:8080/health > /dev/null
        local end=$(date +%s%N)
        local elapsed=$(( (end - start) / 1000000 ))
        health_times+=($elapsed)
    done

    local avg_health=$(IFS=+; echo "scale=2; (${health_times[*]}) / ${#health_times[@]}" | bc)
    log_success "Average health endpoint response time: ${avg_health}ms"

    echo "health_endpoint_avg_ms=$avg_health" >> "$RESULTS_DIR/backend_metrics.txt"
}

# Phase 2: Redis Cache Performance
run_redis_tests() {
    log_info "=== Phase 2: Redis Cache Performance Tests ==="

    # Redis latency test
    log_info "Testing Redis latency..."
    redis-cli --latency-history -i 1 2>&1 | head -n 5 | tee "$RESULTS_DIR/redis_latency.txt"

    # Redis info
    log_info "Getting Redis cache statistics..."
    redis-cli info stats | grep -E "keyspace_hits|keyspace_misses" | tee "$RESULTS_DIR/redis_stats.txt"

    # Calculate hit rate if we have data
    local hits=$(redis-cli info stats | grep keyspace_hits | cut -d: -f2 | tr -d '\r')
    local misses=$(redis-cli info stats | grep keyspace_misses | cut -d: -f2 | tr -d '\r')

    if [ -n "$hits" ] && [ -n "$misses" ] && [ "$hits" != "0" ] || [ "$misses" != "0" ]; then
        local total=$((hits + misses))
        if [ "$total" -gt 0 ]; then
            local hit_rate=$(echo "scale=2; ($hits * 100) / $total" | bc)
            log_info "Redis cache hit rate: ${hit_rate}%"
            echo "redis_hit_rate_percent=$hit_rate" >> "$RESULTS_DIR/backend_metrics.txt"

            if (( $(echo "$hit_rate >= 87" | bc -l) )); then
                log_success "Redis hit rate meets target (>87%)"
            else
                log_warning "Redis hit rate below target: ${hit_rate}% < 87%"
            fi
        else
            log_warning "No cache activity yet"
        fi
    else
        log_warning "No cache statistics available yet"
    fi
}

# Phase 3: Frontend Bundle Size Analysis
run_bundle_analysis() {
    log_info "=== Phase 3: Frontend Bundle Size Analysis ==="

    cd "$PROJECT_ROOT/frontend"

    # Build frontend
    log_info "Building frontend for production..."
    if pnpm run build > "$RESULTS_DIR/frontend_build.log" 2>&1; then
        log_success "Frontend build completed"
    else
        log_error "Frontend build failed"
        cat "$RESULTS_DIR/frontend_build.log"
        return 1
    fi

    # Analyze bundle sizes
    log_info "Analyzing bundle sizes..."
    if [ -d "dist/assets" ]; then
        log_info "Bundle size breakdown:"
        du -sh dist/assets/* | tee "$RESULTS_DIR/bundle_sizes.txt"

        # Calculate total size
        local total_kb=$(du -sk dist/assets | cut -f1)
        log_info "Total bundle size: ${total_kb}KB"
        echo "total_bundle_size_kb=$total_kb" >> "$RESULTS_DIR/frontend_metrics.txt"

        # Check if diff-related chunks are code-split
        if ls dist/assets/*diff*.js 2>/dev/null; then
            log_success "Diff view appears to be code-split (found separate chunk)"
            ls -lh dist/assets/*diff*.js | tee -a "$RESULTS_DIR/bundle_sizes.txt"
        else
            log_warning "No separate diff chunk found - may not be code-split"
        fi
    else
        log_error "dist/assets directory not found"
        return 1
    fi

    cd "$PROJECT_ROOT"
}

# Phase 4: Lighthouse CI (if available)
run_lighthouse() {
    log_info "=== Phase 4: Lighthouse Performance Audit ==="

    if ! command -v lighthouse &> /dev/null; then
        log_warning "Lighthouse not installed. Install with: npm install -g lighthouse"
        log_info "Skipping Lighthouse tests..."
        return 0
    fi

    if ! curl -s http://localhost:5173 > /dev/null 2>&1; then
        log_warning "Frontend not running. Skipping Lighthouse tests..."
        return 0
    fi

    log_info "Running Lighthouse audit..."
    lighthouse http://localhost:5173 \
        --output=json \
        --output-path="$RESULTS_DIR/lighthouse-report.json" \
        --only-categories=performance \
        --quiet

    if [ -f "$RESULTS_DIR/lighthouse-report.json" ]; then
        log_success "Lighthouse report generated"

        # Extract key metrics using jq if available
        if command -v jq &> /dev/null; then
            local perf_score=$(jq -r '.categories.performance.score * 100' "$RESULTS_DIR/lighthouse-report.json")
            local lcp=$(jq -r '.audits["largest-contentful-paint"].numericValue / 1000' "$RESULTS_DIR/lighthouse-report.json")
            local fid=$(jq -r '.audits["max-potential-fid"].numericValue' "$RESULTS_DIR/lighthouse-report.json")
            local cls=$(jq -r '.audits["cumulative-layout-shift"].numericValue' "$RESULTS_DIR/lighthouse-report.json")

            log_info "Performance Score: $perf_score/100"
            log_info "Largest Contentful Paint: ${lcp}s"
            log_info "First Input Delay: ${fid}ms"
            log_info "Cumulative Layout Shift: $cls"

            echo "lighthouse_performance_score=$perf_score" >> "$RESULTS_DIR/frontend_metrics.txt"
            echo "lighthouse_lcp_seconds=$lcp" >> "$RESULTS_DIR/frontend_metrics.txt"
            echo "lighthouse_fid_ms=$fid" >> "$RESULTS_DIR/frontend_metrics.txt"
            echo "lighthouse_cls=$cls" >> "$RESULTS_DIR/frontend_metrics.txt"
        fi
    fi
}

# Generate final report
generate_report() {
    log_info "=== Generating Performance Report ==="

    local report_file="$RESULTS_DIR/performance_report.md"

    cat > "$report_file" << 'EOF'
# Ampel Performance Benchmark Report

**Generated:** $(date)
**Project:** Ampel PR Management Dashboard

## Executive Summary

This report contains performance benchmark results for the Git Diff integration feature.

---

## Backend Performance

EOF

    if [ -f "$RESULTS_DIR/backend_metrics.txt" ]; then
        echo "### API Response Times" >> "$report_file"
        echo '```' >> "$report_file"
        cat "$RESULTS_DIR/backend_metrics.txt" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi

    echo "## Redis Cache Performance" >> "$report_file"
    if [ -f "$RESULTS_DIR/redis_stats.txt" ]; then
        echo '```' >> "$report_file"
        cat "$RESULTS_DIR/redis_stats.txt" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi

    echo "## Frontend Bundle Analysis" >> "$report_file"
    if [ -f "$RESULTS_DIR/bundle_sizes.txt" ]; then
        echo '```' >> "$report_file"
        cat "$RESULTS_DIR/bundle_sizes.txt" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi

    if [ -f "$RESULTS_DIR/frontend_metrics.txt" ]; then
        echo "### Metrics Summary" >> "$report_file"
        echo '```' >> "$report_file"
        cat "$RESULTS_DIR/frontend_metrics.txt" >> "$report_file"
        echo '```' >> "$report_file"
        echo "" >> "$report_file"
    fi

    echo "## Lighthouse Audit" >> "$report_file"
    if [ -f "$RESULTS_DIR/lighthouse-report.json" ]; then
        echo "See detailed report: \`results/lighthouse-report.json\`" >> "$report_file"
    else
        echo "Lighthouse audit not run." >> "$report_file"
    fi

    echo "" >> "$report_file"
    echo "---" >> "$report_file"
    echo "**Report Location:** $report_file" >> "$report_file"

    log_success "Performance report generated: $report_file"

    # Display report
    cat "$report_file"
}

# Main execution
main() {
    log_info "Starting Ampel Performance Benchmark Suite"
    log_info "Results will be saved to: $RESULTS_DIR"

    # Clean previous results
    rm -rf "$RESULTS_DIR"
    mkdir -p "$RESULTS_DIR"

    # Run all test phases
    check_services
    run_backend_tests
    run_redis_tests
    run_bundle_analysis
    run_lighthouse

    # Generate final report
    generate_report

    log_success "All benchmarks completed!"
    log_info "Review the report at: $RESULTS_DIR/performance_report.md"
}

# Run main function
main "$@"
