# Testing Guide for Solana PDA Analyzer

This document provides comprehensive information about the testing infrastructure for the Solana PDA Analyzer project.

## Running Tests

### Quick Start

```bash
# Run all tests
make test-all

# Or use the test runner directly
scripts/run_tests.sh all
```

### Individual Test Types

```bash
# Unit tests only
make test-unit
# OR
scripts/run_tests.sh unit

# Integration tests only  
make test-integration
# OR
scripts/run_tests.sh integration

# API tests only
make test-api
# OR
scripts/run_tests.sh api

# Performance tests only
make test-performance
# OR
scripts/run_tests.sh performance

# End-to-end tests only
make test-e2e
# OR
scripts/run_tests.sh e2e

# Docker-based tests
make test-docker
# OR
scripts/run_tests.sh docker
```

### Benchmarks

```bash
# Run performance benchmarks
make bench
# OR
cargo bench

# View benchmark reports
open target/criterion/report/index.html
```

## Test Categories

### 1. Unit Tests

**Location**: `crates/*/tests/`

**What they test**:
- PDA derivation algorithms
- Seed value conversions
- Pattern detection logic
- Transaction analysis functions
- Database model operations

**How to run**:
```bash
cargo test --lib --bins
```

**Key test files**:
- `crates/core/tests/pda_tests.rs`: Tests PDA derivation, verification, and analysis
- `crates/core/tests/transaction_tests.rs`: Tests transaction parsing and analysis
- `crates/analyzer/tests/pattern_tests.rs`: Tests pattern detection algorithms

### 2. Integration Tests

**Location**: `crates/database/tests/`

**What they test**:
- Database operations (CRUD)
- Database migrations
- Multi-table operations
- Transaction consistency

**Prerequisites**:
- PostgreSQL running on localhost:5432
- `postgres` user with database creation privileges

**How to run**:
```bash
cargo test --test '*'
```

**Note**: These tests create temporary databases and clean up automatically.

### 3. API Tests

**Location**: `tests/integration_test.py`

**What they test**:
- REST API endpoints
- Request/response formats
- Error handling
- CORS headers
- Concurrent request handling

**Prerequisites**:
- API server running (automatically started if needed)
- Python 3.7+ with `aiohttp` and `asyncio`

**How to run**:
```bash
python3 tests/integration_test.py --url http://localhost:8080
```

### 4. Performance Tests

**Location**: `tests/performance_test.py`

**What they test**:
- API response times under load
- Concurrent request handling
- Memory usage with large payloads
- Sustained load performance
- Throughput metrics

**How to run**:
```bash
# Quick performance test
python3 tests/performance_test.py --quick

# Full performance test suite
python3 tests/performance_test.py --users 50 --requests 10

# Sustained load test
python3 tests/performance_test.py --sustained 60
```

### 5. End-to-End Tests

**Location**: `tests/e2e_test.sh`

**What they test**:
- Complete user workflows
- CLI functionality
- API integration
- Database initialization
- Error scenarios

**How to run**:
```bash
tests/e2e_test.sh

# CLI tests only
tests/e2e_test.sh --cli-only

# API tests only  
tests/e2e_test.sh --api-only
```

### 6. Docker Tests

**Location**: `docker-compose.test.yml`

**What they test**:
- Containerized deployment
- Multi-service integration
- Production-like environment
- Isolated test execution

**How to run**:
```bash
# Run all tests in Docker
docker-compose -f docker-compose.test.yml --profile test up --build

# Run performance tests in Docker
docker-compose -f docker-compose.test.yml --profile perf up --build
```

## Test Configuration

### Environment Variables

```bash
# Database configuration
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_USER=postgres
DATABASE_PASSWORD=""

# API configuration
API_HOST=localhost
API_PORT=8080

# Test configuration
RUST_LOG=debug
RUST_BACKTRACE=1
```

### Prerequisites

1. **Rust**: Version 1.70+
2. **PostgreSQL**: Version 12+
3. **Python**: Version 3.7+ with packages:
   ```bash
   pip3 install aiohttp asyncio numpy
   ```
4. **Docker**: For containerized tests (optional)

## Test Data

Tests use:
- Generated keypairs and addresses
- Temporary test databases
- Mock transaction data
- Synthetic PDA patterns

**Important**: No real Solana data is required for testing.

## Continuous Integration

The project is set up for CI/CD with the following test phases:

1. **Code Quality**: `cargo fmt`, `cargo clippy`
2. **Unit Tests**: All unit tests across crates
3. **Integration Tests**: Database and API integration
4. **Performance Tests**: Basic load testing
5. **Docker Tests**: Containerized environment validation

## Performance Benchmarks

### Expected Performance Metrics

| Operation | Target | Good | Acceptable |
|-----------|--------|------|------------|
| PDA Derivation | <1ms | <5ms | <10ms |
| API Response | <100ms | <500ms | <1s |
| Batch Analysis (10) | <50ms | <200ms | <500ms |
| Database Query | <10ms | <50ms | <100ms |

### Benchmark Categories

1. **PDA Operations**:
   - Single PDA derivation
   - Batch PDA analysis
   - PDA verification
   - Cache performance

2. **Pattern Detection**:
   - Pattern recognition
   - Large dataset analysis
   - Cache efficiency

3. **Database Operations**:
   - CRUD operations
   - Complex queries
   - Batch inserts

## Troubleshooting

### Common Issues

1. **Database Connection Failed**:
   ```bash
   # Start PostgreSQL
   brew services start postgresql
   # OR
   sudo systemctl start postgresql
   ```

2. **Port Already in Use**:
   ```bash
   # Kill existing processes
   lsof -ti:8080 | xargs kill -9
   ```

3. **Python Dependencies Missing**:
   ```bash
   pip3 install aiohttp asyncio numpy
   ```

4. **Docker Tests Failing**:
   ```bash
   # Clean up Docker environment
   docker-compose -f docker-compose.test.yml down -v
   docker system prune -f
   ```

### Test Debugging

1. **Enable Verbose Logging**:
   ```bash
   RUST_LOG=debug VERBOSE=true scripts/run_tests.sh
   ```

2. **Run Individual Tests**:
   ```bash
   cargo test test_pda_derivation -- --nocapture
   ```

3. **Check Test Database**:
   ```bash
   psql -h localhost -U postgres -l | grep test
   ```

## Contributing

When adding new features:

1. **Add Unit Tests**: For all new functions
2. **Update Integration Tests**: If database schema changes
3. **Add API Tests**: For new endpoints
4. **Update Benchmarks**: For performance-critical code
5. **Document Test Cases**: In this file

### Test Writing Guidelines

1. **Use descriptive test names**
2. **Test both success and error cases**
3. **Clean up test data**
4. **Use consistent test patterns**
5. **Add performance benchmarks for critical paths**

## Test Reports

Test results and reports are generated in:

- `target/criterion/`: Benchmark reports (HTML)
- `target/tarpaulin/`: Coverage reports (if using tarpaulin)
- Test logs: Console output with timestamps

Open benchmark reports:
```bash
open target/criterion/report/index.html
```

## Security Testing

The test suite includes:

- Input validation tests
- SQL injection prevention
- Error message sanitization
- Rate limiting validation
- CORS configuration testing

For security-focused testing, run:
```bash
scripts/run_tests.sh api  # Includes security tests
```