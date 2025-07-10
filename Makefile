# Makefile for Solana PDA Analyzer

.PHONY: help build test run clean dev docker-build docker-run docker-dev setup

# Default target
help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  test         - Run all tests"
	@echo "  run          - Run the application locally"
	@echo "  clean        - Clean build artifacts"
	@echo "  dev          - Run in development mode"
	@echo "  docker-build - Build Docker image"
	@echo "  docker-run   - Run with Docker Compose"
	@echo "  docker-dev   - Run development environment with Docker"
	@echo "  setup        - Initial project setup"
	@echo "  db-init      - Initialize database"
	@echo "  db-migrate   - Run database migrations"
	@echo "  db-reset     - Reset database"

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test

# Run the application locally
run:
	cargo run --bin pda-analyzer -- serve

# Clean build artifacts
clean:
	cargo clean

# Run in development mode
dev:
	cargo run --bin pda-analyzer -- serve --host 127.0.0.1 --port 8080

# Build Docker image
docker-build:
	docker build -t solana-pda-analyzer:latest .

# Run with Docker Compose
docker-run:
	docker-compose up -d

# Run development environment with Docker
docker-dev:
	docker-compose -f docker-compose.dev.yml up

# Initial project setup
setup:
	@echo "Setting up Solana PDA Analyzer..."
	@if [ ! -f .env ]; then cp .env.example .env; echo "Created .env file"; fi
	@echo "Please edit .env with your configuration"
	@echo "Run 'make db-init' to initialize the database"

# Database operations
db-init:
	cargo run --bin pda-analyzer -- database init

db-migrate:
	cargo run --bin pda-analyzer -- database migrate

db-reset:
	cargo run --bin pda-analyzer -- database reset

db-status:
	cargo run --bin pda-analyzer -- database status

# Development helpers
format:
	cargo fmt

lint:
	cargo clippy -- -D warnings

check:
	cargo check

# Docker development shortcuts
docker-logs:
	docker-compose logs -f pda-analyzer

docker-stop:
	docker-compose down

docker-clean:
	docker-compose down -v
	docker system prune -f

# Testing shortcuts
test-unit:
	cargo test --lib

test-integration:
	cargo test --test '*'

test-api:
	python3 tests/integration_test.py

test-performance:
	python3 tests/performance_test.py --quick

test-e2e:
	tests/e2e_test.sh

test-docker:
	docker-compose -f docker-compose.test.yml --profile test up --build --abort-on-container-exit

test-all:
	scripts/run_tests.sh all

test-coverage:
	cargo tarpaulin --out html

# Database shortcuts for Docker
docker-db-shell:
	docker-compose exec postgres psql -U postgres -d solana_pda_analyzer

docker-db-backup:
	docker-compose exec postgres pg_dump -U postgres solana_pda_analyzer > backup.sql

docker-db-restore:
	docker-compose exec -T postgres psql -U postgres -d solana_pda_analyzer < backup.sql

# Monitoring
logs:
	tail -f logs/pda-analyzer.log

stats:
	cargo run --bin pda-analyzer -- stats

# Release builds
release:
	cargo build --release
	strip target/release/pda-analyzer

# Install from source
install:
	cargo install --path crates/cli

# Benchmark
bench:
	cargo bench

# Documentation
docs:
	cargo doc --open

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update

# Cross-compilation targets
build-linux:
	cargo build --release --target x86_64-unknown-linux-gnu

build-windows:
	cargo build --release --target x86_64-pc-windows-gnu

build-macos:
	cargo build --release --target x86_64-apple-darwin