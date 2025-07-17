# Solana PDA Analyzer - Test Suite

This directory contains comprehensive Rust-based tests for the Solana PDA Analyzer API and client functionality.

## Test Categories

### 1. Integration Tests (`integration_tests.rs`)
Tests the API endpoints end-to-end to ensure they work correctly:
- Health check endpoint
- PDA analysis (single and batch)
- Program listing and details
- Analytics endpoints
- Error handling and edge cases
- Concurrent request handling

**Usage:**
```bash
# Run integration tests
cargo run --bin integration_tests

# Run with custom API URL
API_URL=http://localhost:3000 cargo run --bin integration_tests
```

### 2. API Client Tests (`api_client_tests.rs`)
Tests the API client library functionality:
- Client configuration and initialization
- Request/response handling
- Error handling and retry logic
- Statistics tracking
- Concurrent request handling
- Client performance metrics

**Usage:**
```bash
# Run API client tests
cargo run --bin api_client_tests

# Run with custom API URL
API_URL=http://localhost:3000 cargo run --bin api_client_tests
```

### 3. Performance Tests (`performance_tests.rs`)
Comprehensive performance and load testing:
- Concurrent request handling
- Response time measurement
- Throughput analysis
- Sustained load testing
- Memory usage under load
- Performance assessment

**Usage:**
```bash
# Run full performance test suite
cargo run --bin performance_tests

# Run quick performance tests
QUICK_TEST=1 cargo run --bin performance_tests

# Run sustained load test for 60 seconds
SUSTAINED_DURATION=60 cargo run --bin performance_tests

# Run with custom API URL
API_URL=http://localhost:3000 cargo run --bin performance_tests
```

## Test Requirements

### Server Setup
Before running tests, ensure the API server is running:

```bash
# Start server in simple mode (no database)
cargo run --bin pda-analyzer serve

# Start server with database support
cargo run --bin pda-analyzer serve --database --database-url postgresql://user:pass@localhost/db
```

### Environment Variables
- `API_URL`: Base URL for the API server (default: `http://localhost:8080`)
- `QUICK_TEST`: Run quick performance tests only
- `SUSTAINED_DURATION`: Duration in seconds for sustained load test

## Test Features

### Comprehensive Coverage
- **Health Checks**: Verify server availability and health
- **PDA Analysis**: Test single and batch PDA analysis
- **Database Operations**: Test database queries and metrics
- **Error Handling**: Validate error responses and edge cases
- **Concurrency**: Test concurrent request handling
- **Performance**: Measure response times and throughput

### Metrics and Analysis
- **Response Times**: Average, min, max, 95th/99th percentiles
- **Throughput**: Requests per second
- **Success Rates**: Percentage of successful requests
- **Error Analysis**: Detailed error reporting
- **Performance Assessment**: Automated performance evaluation

### Colored Output
All tests provide colorized output for better readability:
- 🟢 Green: Success/Pass
- 🟡 Yellow: Warnings/Needs Improvement
- 🔴 Red: Errors/Failures
- 🔵 Blue: Information

## Example Test Runs

### Integration Tests
```bash
$ cargo run --bin integration_tests

Solana PDA Analyzer - Integration Tests
Target URL: http://localhost:8080

[INFO] Starting integration tests...
[INFO] Running test: Health Check
[SUCCESS] Health Check passed (0.045s)
[INFO] Running test: PDA Analysis
[SUCCESS] PDA Analysis passed (0.123s)
...

============================================================
TEST SUMMARY
============================================================
PASS Health Check (0.045s)
      Details: Status: healthy, DB Connected: true
PASS PDA Analysis (0.123s)
      Details: PDA analyzed: 11111111111111111111111111111111, pattern: Some("system")
...

------------------------------------------------------------
Overall: 9/9 tests passed
All tests passed!
```

### Performance Tests
```bash
$ cargo run --bin performance_tests

Solana PDA Analyzer - Performance Tests
Target URL: http://localhost:8080

[INFO] Starting performance tests...
[INFO] Running Health Endpoint Load Test - 500 requests (50 concurrent users, 10 requests each)
[SUCCESS] Health Endpoint Load Test completed

============================================================
Test: Health Endpoint Load Test
============================================================
Total Requests:      500
Duration:            2.34s
Successful:          500
Failed:              0
Success Rate:        100.0%
Requests/Second:     213.68

Response Times:
  Average:           45.23ms
  Minimum:           12.45ms
  Maximum:           156.78ms
  95th Percentile:   89.12ms
  99th Percentile:   134.56ms

Performance Assessment:
✓ Excellent success rate (100.0%)
✓ Excellent average response time (45ms)
✓ Excellent throughput (213.7 req/s)
✓ Good 99th percentile (134ms)
```

## Building and Running

### Prerequisites
- Rust 1.70+
- Running Solana PDA Analyzer API server

### Build Tests
```bash
cd tests
cargo build
```

### Run All Tests
```bash
# Run each test suite
cargo run --bin integration_tests
cargo run --bin api_client_tests
cargo run --bin performance_tests
```

### Run Tests with Docker
```bash
# Build and run tests in container
docker build -t pda-analyzer-tests .
docker run --network host pda-analyzer-tests
```

## Test Data

The tests use predefined test data:
- **PDA Addresses**: Well-known test addresses for consistent results
- **Program IDs**: Standard Solana program IDs
- **Test Patterns**: Common PDA patterns (system, config, authority, etc.)

## Troubleshooting

### Common Issues

1. **Server Not Running**
   ```
   [ERROR] Cannot connect to server at http://localhost:8080
   ```
   Solution: Start the API server before running tests

2. **Database Connection Issues**
   ```
   [ERROR] Database health check failed
   ```
   Solution: Ensure database is running and accessible

3. **Performance Issues**
   ```
   [WARNING] Poor average response time (500ms)
   ```
   Solution: Check server load and system resources

### Debug Mode
Add debug logging to tests:
```bash
RUST_LOG=debug cargo run --bin integration_tests
```

## Contributing

When adding new tests:
1. Follow the existing patterns and structure
2. Add comprehensive error handling
3. Include performance metrics where applicable
4. Update this README with new test descriptions
5. Ensure tests are idempotent and can run concurrently

## License

This test suite is part of the Solana PDA Analyzer project and follows the same license terms.