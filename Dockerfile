# Multi-stage Docker build for Solana PDA Analyzer

# Build stage
FROM rust:1.75-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (this layer will be cached)
RUN cargo build --release --bin pda-analyzer

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin pda-analyzer

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN groupadd -r app && useradd -r -g app app

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/pda-analyzer /usr/local/bin/pda-analyzer

# Copy web assets
COPY web/ ./web/

# Copy configuration files
COPY config.toml ./
COPY .env.example ./.env

# Create directories for logs and data
RUN mkdir -p /app/logs /app/data && \
    chown -R app:app /app

# Switch to non-root user
USER app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["pda-analyzer", "serve", "--host", "0.0.0.0", "--port", "8080"]