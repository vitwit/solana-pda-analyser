# Solana PDA Analyzer Examples

This directory contains real-world examples of PDA analysis using the Solana PDA Analyzer tool. All examples use actual data from Solana testnet and mainnet.

## Example Categories

- [SPL Token Examples](#spl-token-examples)
- [Metaplex NFT Examples](#metaplex-nft-examples)
- [Solana Program Examples](#solana-program-examples)
- [Custom Program Examples](#custom-program-examples)

## Quick Start

```bash
# Build the analyzer
cargo build --release

# Run an example
./target/release/pda-analyzer analyze \
  --address "EXAMPLE_PDA_ADDRESS" \
  --program-id "EXAMPLE_PROGRAM_ID"

# Or use the API
curl -X POST http://localhost:8080/api/v1/analyze/pda \
  -H "Content-Type: application/json" \
  -d '{"address": "EXAMPLE_PDA_ADDRESS", "program_id": "EXAMPLE_PROGRAM_ID"}'
```

## Understanding the Examples

Each example includes:
- **PDA Address**: The actual program-derived address
- **Program ID**: The program that owns this PDA
- **Expected Seeds**: The seeds used to derive this PDA
- **Context**: What this PDA represents in the application
- **Analysis Output**: What the analyzer should detect

## Example Format

```rust
// Example Name: Description
pub const EXAMPLE_PDA: &str = "PDA_ADDRESS_HERE";
pub const EXAMPLE_PROGRAM: &str = "PROGRAM_ID_HERE";
// Expected seeds: ["seed1", "seed2", ...]
// Context: Description of what this PDA does
```

## Running Examples

Use the provided scripts to run all examples:

```bash
# Run all examples
./examples/run_examples.sh

# Run specific category
./examples/run_examples.sh spl-token

# Run single example
./examples/run_examples.sh metaplex-metadata
```