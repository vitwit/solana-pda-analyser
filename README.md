# Solana PDA Analyzer

A comprehensive, production-ready tool for analyzing Solana Program Derived Addresses (PDAs), tracking transaction patterns, and understanding program behavior on the Solana blockchain. Features real-time analysis, pattern recognition, and beautiful visualization of PDA derivation patterns.

## Features

- ** Advanced PDA Analysis**: Derive seeds and analyze PDA patterns for any Solana program
- ** Real-World Examples**: Pre-loaded with examples from major protocols (SPL Token, Metaplex, Serum, Raydium)
- ** Pattern Recognition**: Automatically detect and classify common PDA seed patterns

## 📊 Analysis Output Sample

```
════════════════════════════════════════════════════════════════════════════════
🚀 SOLANA PDA ANALYZER - COMPREHENSIVE ANALYSIS REPORT
════════════════════════════════════════════════════════════════════════════════
📊 Analyzing Program Derived Addresses from Live Solana Programs
🔍 Reverse Engineering Seed Patterns and Derivation Logic
════════════════════════════════════════════════════════════════════════════════

📋 DETAILED ANALYSIS RESULTS
────────────────────────────────────────────────────────────────────────────────

1. ✅ USDC Associated Token Account
   🏷️  PDA Address: Gh9ZwEmd...hQKkx
   🔧 Program: SPL Associated Token (ATokenGP...nL)
   📝 Description: Stores USDC tokens for wallet 9WzDXwBbmkg8...
   🎯 Pattern: WALLET_TOKEN_MINT (98% confidence)
   ⏱️  Analysis Time: 12ms
   🌱 Seed Breakdown:
      1. 🔑 Pubkey (32 bytes): Wallet owner address
      2. 🔑 Pubkey (32 bytes): SPL Token Program ID
      3. 🔑 Pubkey (32 bytes): USDC mint address

📊 PATTERN ANALYSIS & STATISTICS
────────────────────────────────────────────────────────────────────────────────

🏆 Pattern Distribution:
   1. WALLET_TOKEN_MINT [█████████░░░░░░░░░░░] 45.0% (9 PDAs)
   2. STRING_PROGRAM_MINT [████░░░░░░░░░░░░░░░░] 20.0% (4 PDAs)
   3. STRING_AUTHORITY [███░░░░░░░░░░░░░░░░░░] 15.0% (3 PDAs)
   4. PUBKEY_U64 [██░░░░░░░░░░░░░░░░░░] 10.0% (2 PDAs)
   5. STRING_SINGLETON [██░░░░░░░░░░░░░░░░░░] 10.0% (2 PDAs)

📈 EXECUTIVE SUMMARY
────────────────────────────────────────────────────────────────────────────────
🎯 Analysis Overview:
   • Total PDAs Analyzed: 20
   • Unique Patterns Detected: 8
   • Overall Success Rate: 95.0%
   • Total Processing Time: 234ms
   • Average Time per PDA: 11.7ms
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Docker (optional)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/solana-pda-analyzer.git
   cd solana-pda-analyzer
   ```

2. **Set up the database**
   ```bash
   # Create PostgreSQL database
   createdb solana_pda_analyzer
   
   # Copy environment file
   cp .env.example .env
   
   # Edit .env with your database credentials
   nano .env
   ```

3. **Build the project**
   ```bash
   make build
   # or manually: cargo build --release
   ```

4. **Initialize the database**
   ```bash
   make db-init
   # or manually: ./target/release/pda-analyzer database init
   ```

5. **Run the tests**
   ```bash
   make test-all
   ```

6. **Start the server**
   ```bash
   make run
   # or manually: ./target/release/pda-analyzer serve
   ```

7. **Access the web interface**
   
   Open your browser to `http://localhost:8080`

### 🐳 Docker Setup (Alternative)

```bash
# Start with Docker Compose
docker-compose up -d

# Run tests in Docker
docker-compose exec app make test-all

# View logs
docker-compose logs -f app
```

## 💻 Usage

### Command Line Interface

#### Analyze Individual PDAs
```bash
# Analyze a specific PDA
./target/release/pda-analyzer analyze \
  --address "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr" \
  --program-id "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"

# Batch analyze multiple PDAs
./target/release/pda-analyzer batch-analyze \
  --input pdas.json \
  --output results.json
```

#### Run Real-World Examples
```bash
# Run all example analyses
./examples/run_examples.sh

# Run specific category
./examples/run_examples.sh spl-token
./examples/run_examples.sh metaplex
./examples/run_examples.sh real-world
```

#### Database Operations
```bash
# Initialize database with schema
./target/release/pda-analyzer database init

# Check database status and metrics
./target/release/pda-analyzer database status

# Reset database (caution!)
./target/release/pda-analyzer database reset

# Run pending migrations
./target/release/pda-analyzer database migrate
```

#### Fetch and Analyze Transactions
```bash
# Fetch recent transactions for a program
./target/release/pda-analyzer fetch \
  --program-id "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s" \
  --limit 1000

# Analyze transaction patterns
./target/release/pda-analyzer analyze-transactions \
  --program-id "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"
```

#### Statistics and Reports
```bash
# Show comprehensive statistics
./target/release/pda-analyzer stats

# Generate detailed analysis report
./target/release/pda-analyzer report \
  --output html \
  --file analysis_report.html
```

### 🔌 REST API

#### Analyze PDAs
```bash
# Analyze a single PDA
curl -X POST http://localhost:8080/api/v1/analyze/pda \
  -H "Content-Type: application/json" \
  -d '{
    "address": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
    "program_id": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
  }'

# Batch analyze multiple PDAs
curl -X POST http://localhost:8080/api/v1/analyze/pda/batch \
  -H "Content-Type: application/json" \
  -d '{
    "pdas": [
      {"address": "PDA1...", "program_id": "PROG1..."},
      {"address": "PDA2...", "program_id": "PROG2..."}
    ]
  }'
```

#### Programs and Patterns
```bash
# Get all analyzed programs
curl http://localhost:8080/api/v1/programs

# Get program details and statistics
curl http://localhost:8080/api/v1/programs/{program_id}

# Get PDA patterns for a program
curl http://localhost:8080/api/v1/programs/{program_id}/patterns
```

#### Analytics and Reports
```bash
# Get database metrics
curl http://localhost:8080/api/v1/analytics/database

# Get pattern distribution
curl http://localhost:8080/api/v1/analytics/patterns

# Get performance metrics
curl http://localhost:8080/api/v1/analytics/performance
```

## Real-World Examples

The tool includes comprehensive examples from major Solana protocols:

### 📊 SPL Token Examples
- **Associated Token Accounts**: Most common PDA pattern on Solana
- **Mint Authorities**: Program-controlled token minting
- **Vault Token Accounts**: DeFi protocol escrow patterns

### 🎨 Metaplex NFT Examples
- **NFT Metadata**: Standard NFT information storage
- **Master Editions**: NFT printing and edition control
- **Collection Metadata**: NFT collection organization
- **Auction House**: NFT marketplace structures

### 🌍 Protocol Examples
- **Serum DEX**: Market authorities and trading structures
- **Raydium AMM**: Liquidity pool management
- **Marinade Finance**: Liquid staking protocol state
- **Solana Name Service**: Domain name resolution
- **Governance**: DAO proposal and voting systems

Each example includes:
- ✅ **Real PDA addresses** from mainnet/testnet
- 🔍 **Expected seed patterns** with detailed breakdowns
- 📝 **Comprehensive descriptions** of functionality
- 🧪 **Automated tests** for validation

## ⚙️ Configuration

### Environment Variables

Create a `.env` file (copy from `.env.example`) and update as required

## 🧪 Testing

### Running Tests

```bash
# Run all tests
make test-all

# Run specific test categories
make test-unit        # Unit tests only
make test-integration # Integration tests only
make test-api        # API endpoint tests
make test-performance # Performance benchmarks

# Run tests with coverage
make test-coverage

# Run example validation
make test-examples
```

### Performance Testing

```bash
# Run benchmark tests
cargo bench

# Load test the API
./tests/load_test.py --concurrent 50 --requests 1000

# Profile memory usage
./tests/memory_profile.sh
```

## 📊 API Reference

### Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": {
    "pda_info": {
      "address": "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr",
      "program_id": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
      "seeds": [
        {
          "type": "Pubkey",
          "value": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
          "description": "Wallet owner address"
        }
      ],
      "pattern": "WALLET_TOKEN_MINT",
      "confidence": 0.98,
      "analysis_time_ms": 12
    }
  },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z",
  "request_id": "req_123456789"
}
```

### Complete Endpoint List

#### PDA Analysis
- `POST /api/v1/analyze/pda` - Analyze a single PDA
- `POST /api/v1/analyze/pda/batch` - Batch analyze multiple PDAs
- `GET /api/v1/pdas` - List analyzed PDAs with pagination
- `GET /api/v1/pdas/{address}` - Get detailed PDA information

#### Programs
- `GET /api/v1/programs` - List all programs
- `GET /api/v1/programs/{id}` - Get program details
- `GET /api/v1/programs/{id}/stats` - Get program statistics
- `GET /api/v1/programs/{id}/patterns` - Get program PDA patterns

#### Transactions
- `GET /api/v1/transactions` - List transactions with filters
- `GET /api/v1/transactions/{signature}` - Get transaction details
- `POST /api/v1/transactions/analyze` - Analyze transaction for PDAs

#### Analytics
- `GET /api/v1/analytics/database` - Database metrics and statistics
- `GET /api/v1/analytics/patterns` - Pattern distribution and trends
- `GET /api/v1/analytics/performance` - Performance metrics and timing

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Steps

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the test suite: `make test-all`
5. Commit your changes: `git commit -m "Add amazing feature"`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Submit a pull request

### Development Setup

```bash
# Install development dependencies
make dev-setup

# Run development server with hot reload
make dev

# Run linting and formatting
make lint
make fmt

# Generate documentation
make docs
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/your-username/solana-pda-analyzer/issues)
- **💡 Feature Requests**: [GitHub Discussions](https://github.com/your-username/solana-pda-analyzer/discussions)
- **📖 Documentation**: [Project Wiki](https://github.com/your-username/solana-pda-analyzer/wiki)
- **💬 Community**: [Discord Channel](https://discord.gg/your-server)
