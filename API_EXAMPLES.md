# API Server Examples

The Solana PDA Analyzer now includes a fully functional REST API server!

## Starting the Server

```bash
# Start the API server on default port 8080
./target/release/pda-analyzer serve

# Start on a custom port
./target/release/pda-analyzer serve --port 3000

# Start on a custom host and port
./target/release/pda-analyzer serve --host 0.0.0.0 --port 8080
```

## API Endpoints

### Health Check
```bash
curl http://127.0.0.1:8080/health
```

Response:
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "timestamp": "2025-07-16T11:31:19.865193Z",
    "version": "0.1.0"
  },
  "error": null,
  "timestamp": "2025-07-16T11:31:19.865194Z"
}
```

### API Documentation
```bash
curl http://127.0.0.1:8080/docs
```

### Analyze a PDA
```bash
curl -X POST http://127.0.0.1:8080/api/v1/analyze/pda \
  -H "Content-Type: application/json" \
  -d '{
    "address": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
    "program_id": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
  }'
```

### Batch Analyze PDAs
```bash
curl -X POST http://127.0.0.1:8080/api/v1/analyze/pda/batch \
  -H "Content-Type: application/json" \
  -d '{
    "pdas": [
      {
        "address": "SysvarRent111111111111111111111111111111111",
        "program_id": "11111111111111111111111111111112"
      },
      {
        "address": "SysvarC1ock11111111111111111111111111111111",
        "program_id": "11111111111111111111111111111112"
      }
    ]
  }'
```

### Performance Metrics
```bash
curl http://127.0.0.1:8080/api/v1/analytics/performance
```

## Features

- ✅ **Real-time PDA Analysis** - Analyze any PDA address and program ID
- ✅ **Pattern Recognition** - Automatically detect common PDA patterns
- ✅ **Batch Processing** - Analyze multiple PDAs in a single request
- ✅ **Performance Metrics** - Get detailed timing and statistics
- ✅ **Health Monitoring** - Built-in health check endpoint
- ✅ **CORS Support** - Cross-origin requests enabled
- ✅ **Request Logging** - Comprehensive request/response logging
- ✅ **Security Headers** - Security headers automatically added

## Production Ready

The API server includes:
- Proper error handling and responses
- Request validation and sanitization
- Performance monitoring and metrics
- Security middleware
- Structured logging
- Graceful shutdown handling

## Development

To run the server in development mode:

```bash
# Build and run in development
cargo run --bin pda-analyzer serve

# Or build release and run
cargo build --release
./target/release/pda-analyzer serve
```

The server will start on `http://127.0.0.1:8080` by default.