# Solana PDA Analyzer

A comprehensive tool for analyzing Solana Program Derived Addresses (PDAs), tracking transaction patterns, and understanding program behavior on the Solana blockchain.

## Features

- **PDA Analysis**: Derive seeds and analyze PDA patterns for any Solana program
- **Transaction Monitoring**: Track and analyze transactions involving PDAs
- **Pattern Recognition**: Automatically detect common PDA seed patterns
- **Web Interface**: Modern web dashboard for visualization and interaction
- **REST API**: Complete API for programmatic access
- **Database Storage**: Persistent storage for analysis results and patterns
- **CLI Tool**: Command-line interface for batch operations

## Quick Start

### Prerequisites

- Rust 1.70+
- PostgreSQL 12+
- Node.js 16+ (for web interface)

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
   cargo build --release
   ```

4. **Initialize the database**
   ```bash
   ./target/release/pda-analyzer database init
   ```

5. **Start the server**
   ```bash
   ./target/release/pda-analyzer serve
   ```

6. **Access the web interface**
   
   Open your browser to `http://localhost:8080`

## Usage

### Command Line Interface

#### Analyze a PDA
```bash
./target/release/pda-analyzer analyze \
  --address "YourPDAAddressHere" \
  --program-id "YourProgramIdHere"
```

#### Fetch and analyze transactions
```bash
./target/release/pda-analyzer fetch \
  --program-id "YourProgramIdHere" \
  --limit 1000
```

#### Database operations
```bash
# Initialize database
./target/release/pda-analyzer database init

# Check database status
./target/release/pda-analyzer database status

# Reset database
./target/release/pda-analyzer database reset

# Run migrations
./target/release/pda-analyzer database migrate
```

#### Show statistics
```bash
./target/release/pda-analyzer stats
```

### Web Interface

The web interface provides:

- **PDA Analyzer**: Interactive form to analyze individual PDAs
- **Programs Dashboard**: View all analyzed programs and their statistics
- **Transactions**: Monitor recent transactions involving PDAs
- **Analytics**: Charts and visualizations of PDA patterns and usage

### REST API

#### Analyze a PDA
```bash
curl -X POST http://localhost:8080/api/v1/analyze/pda \
  -H "Content-Type: application/json" \
  -d '{
    "address": "YourPDAAddressHere",
    "program_id": "YourProgramIdHere"
  }'
```

#### Get programs
```bash
curl http://localhost:8080/api/v1/programs
```

#### Get transactions
```bash
curl http://localhost:8080/api/v1/transactions?limit=10
```

#### Get database metrics
```bash
curl http://localhost:8080/api/v1/analytics/database
```

## Configuration

### Environment Variables

See `.env.example` for all available configuration options.

### Configuration File

You can also use a TOML configuration file (`config.toml`):

```toml
[database]
host = "localhost"
port = 5432
name = "solana_pda_analyzer"
user = "postgres"
password = "your_password"

[server]
host = "127.0.0.1"
port = 8080
log_level = "info"

[solana]
rpc_url = "https://api.mainnet-beta.solana.com"
```

## Architecture

The project is structured as a Rust workspace with the following crates:

- **`core`**: Core PDA analysis logic and types
- **`analyzer`**: Transaction analysis and pattern detection
- **`database`**: Database models and repository layer
- **`api`**: REST API server implementation
- **`cli`**: Command-line interface

### Database Schema

The database stores:
- Programs and their metadata
- PDA patterns and seed templates
- Transaction records
- Account interactions
- Seed derivation attempts

## Development

### Running Tests

```bash
cargo test
```

### Running with Docker

```bash
docker-compose up
```

### Building for Production

```bash
cargo build --release
```

## API Reference

### Endpoints

#### PDA Analysis
- `POST /api/v1/analyze/pda` - Analyze a single PDA
- `POST /api/v1/analyze/pda/batch` - Batch analyze multiple PDAs

#### Programs
- `GET /api/v1/programs` - List all programs
- `GET /api/v1/programs/{id}` - Get program details
- `GET /api/v1/programs/{id}/stats` - Get program statistics
- `GET /api/v1/programs/{id}/patterns` - Get program PDA patterns

#### Transactions
- `GET /api/v1/transactions` - List transactions
- `GET /api/v1/transactions/{signature}` - Get transaction details

#### PDAs
- `GET /api/v1/pdas` - List PDAs
- `GET /api/v1/pdas/{address}` - Get PDA details

#### Analytics
- `GET /api/v1/analytics/database` - Get database metrics

### Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [Solana SDK](https://docs.rs/solana-sdk/) for blockchain interaction
- Database layer powered by [SQLx](https://github.com/launchbadge/sqlx)
- Web server built with [Axum](https://github.com/tokio-rs/axum)
- Frontend uses [Bootstrap](https://getbootstrap.com/) and [Chart.js](https://www.chartjs.org/)

## Support

For questions and support, please open an issue on GitHub or contact the maintainers.