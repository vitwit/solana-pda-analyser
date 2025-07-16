#!/bin/bash
# Test script for the PDA Analyzer API server

set -e

echo "🧪 Testing PDA Analyzer API Server"
echo "=================================="

# Build the project
echo "📦 Building project..."
cargo build --release

# Start the server in the background
echo "🚀 Starting API server..."
./target/release/pda-analyzer serve --port 8081 &
SERVER_PID=$!

# Wait for server to start
echo "⏱️ Waiting for server to start..."
sleep 3

# Function to cleanup on exit
cleanup() {
    echo "🧹 Cleaning up..."
    kill $SERVER_PID 2>/dev/null || true
}
trap cleanup EXIT

# Test health check
echo "❤️ Testing health check..."
curl -s http://127.0.0.1:8081/health | jq .

# Test API documentation
echo "📖 Testing API documentation..."
curl -s http://127.0.0.1:8081/docs | jq .

# Test PDA analysis
echo "🔍 Testing PDA analysis..."
curl -s -X POST http://127.0.0.1:8081/api/v1/analyze/pda \
  -H "Content-Type: application/json" \
  -d '{
    "address": "11111111111111111111111111111111",
    "program_id": "11111111111111111111111111111112"
  }' | jq .

# Test performance metrics
echo "📊 Testing performance metrics..."
curl -s http://127.0.0.1:8081/api/v1/analytics/performance | jq .

echo "✅ All API tests completed successfully!"