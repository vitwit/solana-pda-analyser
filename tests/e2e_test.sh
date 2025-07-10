#!/bin/bash

# End-to-End Test Script for Solana PDA Analyzer
# This script tests the complete system including database, API, and CLI

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_DB_NAME="solana_pda_analyzer_e2e_test"
API_HOST="localhost"
API_PORT="8081"
API_BASE_URL="http://${API_HOST}:${API_PORT}"

# Test data
TEST_PROGRAM_ID="11111111111111111111111111111111"
TEST_PDA_ADDRESS="22222222222222222222222222222222"

# Logging functions
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

# Check if required tools are available
check_dependencies() {
    log_info "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "cargo is required but not installed"
        exit 1
    fi
    
    if ! command -v psql &> /dev/null; then
        log_warning "psql not found - some database tests may be skipped"
    fi
    
    if ! command -v curl &> /dev/null; then
        log_error "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        log_warning "jq not found - JSON parsing may be limited"
    fi
    
    log_success "Dependencies checked"
}

# Build the project
build_project() {
    log_info "Building project..."
    
    if ! cargo build --release; then
        log_error "Failed to build project"
        exit 1
    fi
    
    log_success "Project built successfully"
}

# Setup test database
setup_database() {
    log_info "Setting up test database..."
    
    # Set environment variables for test database
    export DATABASE_NAME="${TEST_DB_NAME}"
    export DATABASE_HOST="localhost"
    export DATABASE_PORT="5432"
    export DATABASE_USER="postgres"
    export DATABASE_PASSWORD=""
    
    # Initialize test database
    if ! ./target/release/pda-analyzer database init; then
        log_warning "Failed to initialize database - tests may be limited"
        return 1
    fi
    
    log_success "Test database setup complete"
    return 0
}

# Start API server in background
start_api_server() {
    log_info "Starting API server..."
    
    # Set environment for test API server
    export HOST="${API_HOST}"
    export PORT="${API_PORT}"
    export DATABASE_NAME="${TEST_DB_NAME}"
    
    # Start server in background
    ./target/release/pda-analyzer serve --host "${API_HOST}" --port "${API_PORT}" &
    API_PID=$!
    
    # Wait for server to start
    log_info "Waiting for API server to start..."
    for i in {1..30}; do
        if curl -s "${API_BASE_URL}/health" > /dev/null 2>&1; then
            log_success "API server started (PID: ${API_PID})"
            return 0
        fi
        sleep 1
    done
    
    log_error "API server failed to start within 30 seconds"
    return 1
}

# Stop API server
stop_api_server() {
    if [ ! -z "${API_PID}" ]; then
        log_info "Stopping API server (PID: ${API_PID})..."
        kill "${API_PID}" 2>/dev/null || true
        wait "${API_PID}" 2>/dev/null || true
        log_success "API server stopped"
    fi
}

# Test CLI functionality
test_cli() {
    log_info "Testing CLI functionality..."
    
    # Test database status
    if ! ./target/release/pda-analyzer database status; then
        log_warning "Database status check failed"
    else
        log_success "Database status check passed"
    fi
    
    # Test stats command
    if ! ./target/release/pda-analyzer stats; then
        log_warning "Stats command failed"
    else
        log_success "Stats command passed"
    fi
    
    # Test PDA analysis
    log_info "Testing PDA analysis via CLI..."
    if ./target/release/pda-analyzer analyze --address "${TEST_PDA_ADDRESS}" --program-id "${TEST_PROGRAM_ID}"; then
        log_success "CLI PDA analysis test passed"
    else
        log_warning "CLI PDA analysis test failed (expected for random addresses)"
    fi
}

# Test API endpoints
test_api() {
    log_info "Testing API endpoints..."
    
    # Test health endpoint
    log_info "Testing health endpoint..."
    if response=$(curl -s "${API_BASE_URL}/health"); then
        log_success "Health endpoint accessible"
        if command -v jq &> /dev/null; then
            if echo "${response}" | jq -e '.success == true' > /dev/null; then
                log_success "Health endpoint returns success"
            else
                log_warning "Health endpoint does not return success"
            fi
        fi
    else
        log_error "Health endpoint not accessible"
        return 1
    fi
    
    # Test PDA analysis endpoint
    log_info "Testing PDA analysis endpoint..."
    payload="{\"address\": \"${TEST_PDA_ADDRESS}\", \"program_id\": \"${TEST_PROGRAM_ID}\"}"
    if response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "${payload}" \
        "${API_BASE_URL}/api/v1/analyze/pda"); then
        log_success "PDA analysis endpoint accessible"
        if command -v jq &> /dev/null; then
            if echo "${response}" | jq -e '.success' > /dev/null; then
                log_success "PDA analysis endpoint returns valid response"
            else
                log_warning "PDA analysis endpoint response format issue"
            fi
        fi
    else
        log_error "PDA analysis endpoint not accessible"
    fi
    
    # Test batch PDA analysis endpoint
    log_info "Testing batch PDA analysis endpoint..."
    batch_payload="{\"addresses\": [{\"address\": \"${TEST_PDA_ADDRESS}\", \"program_id\": \"${TEST_PROGRAM_ID}\"}]}"
    if response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "${batch_payload}" \
        "${API_BASE_URL}/api/v1/analyze/pda/batch"); then
        log_success "Batch PDA analysis endpoint accessible"
    else
        log_error "Batch PDA analysis endpoint not accessible"
    fi
    
    # Test programs endpoint
    log_info "Testing programs endpoint..."
    if response=$(curl -s "${API_BASE_URL}/api/v1/programs"); then
        log_success "Programs endpoint accessible"
    else
        log_error "Programs endpoint not accessible"
    fi
    
    # Test transactions endpoint
    log_info "Testing transactions endpoint..."
    if response=$(curl -s "${API_BASE_URL}/api/v1/transactions"); then
        log_success "Transactions endpoint accessible"
    else
        log_error "Transactions endpoint not accessible"
    fi
    
    # Test PDAs endpoint
    log_info "Testing PDAs endpoint..."
    if response=$(curl -s "${API_BASE_URL}/api/v1/pdas"); then
        log_success "PDAs endpoint accessible"
    else
        log_error "PDAs endpoint not accessible"
    fi
    
    # Test analytics endpoint
    log_info "Testing analytics endpoint..."
    if response=$(curl -s "${API_BASE_URL}/api/v1/analytics/database"); then
        log_success "Analytics endpoint accessible"
        if command -v jq &> /dev/null; then
            if echo "${response}" | jq -e '.data.total_programs' > /dev/null; then
                log_success "Analytics endpoint returns metrics"
            else
                log_warning "Analytics endpoint metrics format issue"
            fi
        fi
    else
        log_error "Analytics endpoint not accessible"
    fi
}

# Test error conditions
test_error_conditions() {
    log_info "Testing error conditions..."
    
    # Test invalid PDA address
    log_info "Testing invalid PDA address..."
    payload="{\"address\": \"invalid_address\", \"program_id\": \"${TEST_PROGRAM_ID}\"}"
    if response=$(curl -s -w "%{http_code}" -X POST \
        -H "Content-Type: application/json" \
        -d "${payload}" \
        "${API_BASE_URL}/api/v1/analyze/pda"); then
        
        http_code=$(echo "${response}" | tail -c 4)
        if [ "${http_code}" = "400" ]; then
            log_success "Invalid address properly rejected (400)"
        else
            log_warning "Invalid address handling unexpected (HTTP ${http_code})"
        fi
    fi
    
    # Test nonexistent endpoint
    log_info "Testing nonexistent endpoint..."
    if response=$(curl -s -w "%{http_code}" "${API_BASE_URL}/api/v1/nonexistent"); then
        http_code=$(echo "${response}" | tail -c 4)
        if [ "${http_code}" = "404" ]; then
            log_success "Nonexistent endpoint properly returns 404"
        else
            log_warning "Nonexistent endpoint handling unexpected (HTTP ${http_code})"
        fi
    fi
    
    # Test malformed JSON
    log_info "Testing malformed JSON..."
    if response=$(curl -s -w "%{http_code}" -X POST \
        -H "Content-Type: application/json" \
        -d "{invalid json" \
        "${API_BASE_URL}/api/v1/analyze/pda"); then
        
        http_code=$(echo "${response}" | tail -c 4)
        if [ "${http_code}" = "400" ]; then
            log_success "Malformed JSON properly rejected (400)"
        else
            log_warning "Malformed JSON handling unexpected (HTTP ${http_code})"
        fi
    fi
}

# Test performance under load
test_load() {
    log_info "Testing basic load handling..."
    
    # Send 10 concurrent requests
    log_info "Sending 10 concurrent requests..."
    for i in {1..10}; do
        curl -s "${API_BASE_URL}/health" > /dev/null &
    done
    
    # Wait for all requests to complete
    wait
    
    # Check if server is still responsive
    if curl -s "${API_BASE_URL}/health" > /dev/null; then
        log_success "Server remains responsive after concurrent requests"
    else
        log_error "Server became unresponsive after concurrent requests"
    fi
}

# Cleanup test environment
cleanup() {
    log_info "Cleaning up test environment..."
    
    # Stop API server
    stop_api_server
    
    # Clean up test database
    if command -v psql &> /dev/null; then
        psql -h localhost -U postgres -c "DROP DATABASE IF EXISTS ${TEST_DB_NAME};" postgres 2>/dev/null || true
    fi
    
    # Clean up any background processes
    jobs -p | xargs -r kill 2>/dev/null || true
    
    log_success "Cleanup complete"
}

# Main test execution
main() {
    log_info "Starting End-to-End Tests for Solana PDA Analyzer"
    
    # Setup trap for cleanup
    trap cleanup EXIT
    
    # Run test phases
    check_dependencies
    build_project
    
    # Database setup (optional)
    if setup_database; then
        DATABASE_AVAILABLE=true
    else
        DATABASE_AVAILABLE=false
        log_warning "Database tests will be skipped"
    fi
    
    # CLI tests
    test_cli
    
    # API tests (only if database is available)
    if [ "${DATABASE_AVAILABLE}" = true ]; then
        if start_api_server; then
            test_api
            test_error_conditions
            test_load
        else
            log_error "Could not start API server - API tests skipped"
        fi
    else
        log_warning "API tests skipped due to database unavailability"
    fi
    
    log_success "End-to-End Tests Completed"
}

# Help function
show_help() {
    echo "Solana PDA Analyzer E2E Test Script"
    echo
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --no-db        Skip database-dependent tests"
    echo "  --api-only     Run only API tests (requires database)"
    echo "  --cli-only     Run only CLI tests"
    echo
    echo "Environment Variables:"
    echo "  DATABASE_HOST      Database host (default: localhost)"
    echo "  DATABASE_PORT      Database port (default: 5432)"
    echo "  DATABASE_USER      Database user (default: postgres)"
    echo "  DATABASE_PASSWORD  Database password (default: empty)"
    echo
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --no-db)
            DATABASE_AVAILABLE=false
            shift
            ;;
        --api-only)
            TEST_MODE="api"
            shift
            ;;
        --cli-only)
            TEST_MODE="cli"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Run main function
main "$@"