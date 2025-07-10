#!/bin/bash

# Comprehensive test runner script for Solana PDA Analyzer

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
TEST_TYPE=${1:-"all"}
VERBOSE=${VERBOSE:-false}

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

show_help() {
    echo "Solana PDA Analyzer Test Runner"
    echo
    echo "Usage: $0 [TEST_TYPE]"
    echo
    echo "Test Types:"
    echo "  unit         Run unit tests only"
    echo "  integration  Run integration tests only"
    echo "  api          Run API tests only"
    echo "  performance  Run performance tests only"
    echo "  e2e          Run end-to-end tests only"
    echo "  docker       Run Docker-based tests"
    echo "  all          Run all tests (default)"
    echo
    echo "Environment Variables:"
    echo "  VERBOSE=true     Enable verbose output"
    echo "  NO_DOCKER=true   Skip Docker tests"
    echo
}

run_unit_tests() {
    log_info "Running unit tests..."
    
    if cargo test --lib --bins; then
        log_success "Unit tests passed"
        return 0
    else
        log_error "Unit tests failed"
        return 1
    fi
}

run_integration_tests() {
    log_info "Running integration tests..."
    
    # Check if database is available
    if ! command -v psql &> /dev/null; then
        log_warning "PostgreSQL not available, skipping database integration tests"
        return 0
    fi
    
    # Set test database environment
    export DATABASE_NAME="solana_pda_analyzer_test_$(date +%s)"
    export DATABASE_HOST="localhost"
    export DATABASE_PORT="5432"
    export DATABASE_USER="postgres"
    export DATABASE_PASSWORD=""
    
    log_info "Using test database: ${DATABASE_NAME}"
    
    if cargo test --test '*'; then
        log_success "Integration tests passed"
        # Cleanup test database
        psql -h localhost -U postgres -c "DROP DATABASE IF EXISTS ${DATABASE_NAME};" postgres 2>/dev/null || true
        return 0
    else
        log_error "Integration tests failed"
        # Cleanup test database
        psql -h localhost -U postgres -c "DROP DATABASE IF EXISTS ${DATABASE_NAME};" postgres 2>/dev/null || true
        return 1
    fi
}

run_api_tests() {
    log_info "Running API tests..."
    
    # Check if server is running
    if ! curl -s http://localhost:8080/health > /dev/null; then
        log_warning "API server not running at localhost:8080"
        log_info "Starting test server..."
        
        # Start server in background for testing
        export DATABASE_NAME="solana_pda_analyzer_api_test"
        export PORT="8082"
        
        cargo run --bin pda-analyzer serve --host 127.0.0.1 --port 8082 &
        SERVER_PID=$!
        
        # Wait for server to start
        for i in {1..30}; do
            if curl -s http://localhost:8082/health > /dev/null; then
                break
            fi
            sleep 1
        done
        
        # Run Python integration tests
        if python3 tests/integration_test.py --url http://localhost:8082; then
            log_success "API tests passed"
            kill $SERVER_PID 2>/dev/null || true
            return 0
        else
            log_error "API tests failed"
            kill $SERVER_PID 2>/dev/null || true
            return 1
        fi
    else
        # Use existing server
        if python3 tests/integration_test.py --url http://localhost:8080; then
            log_success "API tests passed"
            return 0
        else
            log_error "API tests failed"
            return 1
        fi
    fi
}

run_performance_tests() {
    log_info "Running performance tests..."
    
    # Check if server is running
    if ! curl -s http://localhost:8080/health > /dev/null; then
        log_warning "API server not running, skipping performance tests"
        return 0
    fi
    
    if python3 tests/performance_test.py --url http://localhost:8080 --quick; then
        log_success "Performance tests passed"
        return 0
    else
        log_error "Performance tests failed"
        return 1
    fi
}

run_e2e_tests() {
    log_info "Running end-to-end tests..."
    
    if tests/e2e_test.sh; then
        log_success "End-to-end tests passed"
        return 0
    else
        log_error "End-to-end tests failed"
        return 1
    fi
}

run_docker_tests() {
    if [ "${NO_DOCKER}" = "true" ]; then
        log_warning "Docker tests skipped (NO_DOCKER=true)"
        return 0
    fi
    
    log_info "Running Docker-based tests..."
    
    if ! command -v docker-compose &> /dev/null; then
        log_warning "Docker Compose not available, skipping Docker tests"
        return 0
    fi
    
    # Clean up any existing test containers
    docker-compose -f docker-compose.test.yml down -v 2>/dev/null || true
    
    # Run tests in Docker
    if docker-compose -f docker-compose.test.yml --profile test up --build --abort-on-container-exit; then
        log_success "Docker tests passed"
        docker-compose -f docker-compose.test.yml down -v
        return 0
    else
        log_error "Docker tests failed"
        docker-compose -f docker-compose.test.yml down -v
        return 1
    fi
}

run_benchmarks() {
    log_info "Running benchmarks..."
    
    if cargo bench; then
        log_success "Benchmarks completed"
        return 0
    else
        log_error "Benchmarks failed"
        return 1
    fi
}

check_code_quality() {
    log_info "Checking code quality..."
    
    # Run formatter check
    if ! cargo fmt --check; then
        log_error "Code formatting issues found. Run 'cargo fmt' to fix."
        return 1
    fi
    
    # Run clippy
    if ! cargo clippy -- -D warnings; then
        log_error "Clippy found issues"
        return 1
    fi
    
    log_success "Code quality checks passed"
    return 0
}

main() {
    log_info "Starting Solana PDA Analyzer Test Suite"
    
    case $TEST_TYPE in
        "help"|"-h"|"--help")
            show_help
            exit 0
            ;;
        "unit")
            run_unit_tests
            exit $?
            ;;
        "integration")
            run_integration_tests
            exit $?
            ;;
        "api")
            run_api_tests
            exit $?
            ;;
        "performance"|"perf")
            run_performance_tests
            exit $?
            ;;
        "e2e")
            run_e2e_tests
            exit $?
            ;;
        "docker")
            run_docker_tests
            exit $?
            ;;
        "bench"|"benchmark")
            run_benchmarks
            exit $?
            ;;
        "quality"|"lint")
            check_code_quality
            exit $?
            ;;
        "all")
            # Run all tests
            FAILED_TESTS=()
            
            # Code quality first
            if ! check_code_quality; then
                FAILED_TESTS+=("code_quality")
            fi
            
            # Unit tests
            if ! run_unit_tests; then
                FAILED_TESTS+=("unit")
            fi
            
            # Integration tests
            if ! run_integration_tests; then
                FAILED_TESTS+=("integration")
            fi
            
            # API tests
            if ! run_api_tests; then
                FAILED_TESTS+=("api")
            fi
            
            # Performance tests
            if ! run_performance_tests; then
                FAILED_TESTS+=("performance")
            fi
            
            # E2E tests
            if ! run_e2e_tests; then
                FAILED_TESTS+=("e2e")
            fi
            
            # Docker tests (optional)
            if ! run_docker_tests; then
                FAILED_TESTS+=("docker")
            fi
            
            # Summary
            echo
            log_info "Test Summary:"
            if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
                log_success "All tests passed!"
                exit 0
            else
                log_error "Failed tests: ${FAILED_TESTS[*]}"
                exit 1
            fi
            ;;
        *)
            log_error "Unknown test type: $TEST_TYPE"
            show_help
            exit 1
            ;;
    esac
}

# Run main function
main "$@"