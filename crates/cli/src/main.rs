use clap::{Parser, Subcommand};
use solana_pda_analyzer_core::{PdaAnalyzer, SeedValue};
use solana_pda_analyzer_database::{DatabaseConfig, initialize_database, DatabaseRepository};
use solana_pda_analyzer_analyzer::{SolanaClient, TransactionFetcher, BatchProcessor};
use solana_pda_analyzer_api::{run_server, ServerConfig};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Analyze a PDA
    Analyze {
        /// PDA address to analyze
        #[arg(short, long)]
        address: String,
        /// Program ID
        #[arg(short, long)]
        program_id: String,
    },
    /// Fetch and analyze transactions for a program
    Fetch {
        /// Program ID to fetch transactions for
        #[arg(short, long)]
        program_id: String,
        /// Solana RPC URL
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
        /// Number of transactions to fetch
        #[arg(long, default_value = "100")]
        limit: usize,
    },
    /// Database operations
    Database {
        #[command(subcommand)]
        command: DatabaseCommands,
    },
    /// Show statistics
    Stats,
}

#[derive(Subcommand)]
enum DatabaseCommands {
    /// Initialize the database
    Init,
    /// Reset the database
    Reset,
    /// Show database status
    Status,
    /// Run migrations
    Migrate,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port } => {
            info!("Starting Solana PDA Analyzer server...");
            
            let mut config = ServerConfig::from_env()?;
            config.host = host;
            config.port = port;
            
            run_server(config).await?;
        }
        Commands::Analyze { address, program_id } => {
            analyze_pda(&address, &program_id).await?;
        }
        Commands::Fetch { program_id, rpc_url, limit } => {
            fetch_transactions(&program_id, &rpc_url, limit).await?;
        }
        Commands::Database { command } => {
            handle_database_command(command).await?;
        }
        Commands::Stats => {
            show_stats().await?;
        }
    }

    Ok(())
}

async fn analyze_pda(address: &str, program_id: &str) -> Result<()> {
    info!("Analyzing PDA: {} for program: {}", address, program_id);
    
    let pda_address = Pubkey::from_str(address)?;
    let program_pubkey = Pubkey::from_str(program_id)?;
    
    let mut analyzer = PdaAnalyzer::new();
    
    match analyzer.analyze_pda(&pda_address, &program_pubkey)? {
        Some(pda_info) => {
            println!("✅ PDA Analysis Successful!");
            println!("Address: {}", pda_info.address);
            println!("Program ID: {}", pda_info.program_id);
            println!("Bump: {}", pda_info.bump);
            println!("Seeds:");
            for (i, seed) in pda_info.seeds.iter().enumerate() {
                println!("  {}: {:?}", i, seed);
            }
        }
        None => {
            println!("❌ Could not derive seeds for the given PDA");
            println!("This could mean:");
            println!("  - The address is not a valid PDA for this program");
            println!("  - The seed derivation pattern is not recognized");
        }
    }
    
    Ok(())
}

async fn fetch_transactions(program_id: &str, rpc_url: &str, limit: usize) -> Result<()> {
    info!("Fetching transactions for program: {}", program_id);
    
    let program_pubkey = Pubkey::from_str(program_id)?;
    let client = SolanaClient::new(rpc_url);
    let fetcher = TransactionFetcher::new(client, 50);
    
    let signatures = fetcher.fetch_transactions_for_program(&program_pubkey, Some(limit)).await?;
    
    println!("📊 Found {} transactions for program {}", signatures.len(), program_id);
    
    // Initialize database if needed
    let db_config = DatabaseConfig::from_env()?;
    let pool = initialize_database(&db_config).await?;
    let repository = DatabaseRepository::new(pool);
    
    // Process transactions
    let processor = BatchProcessor::new();
    let mut processed_count = 0;
    
    for signature in signatures.iter().take(10) { // Process first 10 for demo
        println!("Processing transaction: {}", signature);
        // In a real implementation, we'd fetch and process the full transaction
        processed_count += 1;
    }
    
    println!("✅ Processed {} transactions", processed_count);
    
    Ok(())
}

async fn handle_database_command(command: DatabaseCommands) -> Result<()> {
    let db_config = DatabaseConfig::from_env()?;
    
    match command {
        DatabaseCommands::Init => {
            info!("Initializing database...");
            let pool = initialize_database(&db_config).await?;
            println!("✅ Database initialized successfully");
        }
        DatabaseCommands::Reset => {
            info!("Resetting database...");
            let migrator = solana_pda_analyzer_database::DatabaseMigrator::new(db_config.database_url());
            migrator.reset_database().await?;
            println!("✅ Database reset successfully");
        }
        DatabaseCommands::Status => {
            info!("Checking database status...");
            match db_config.create_pool().await {
                Ok(pool) => {
                    match solana_pda_analyzer_database::health_check(&pool).await {
                        Ok(_) => println!("✅ Database is healthy"),
                        Err(e) => println!("❌ Database health check failed: {}", e),
                    }
                }
                Err(e) => println!("❌ Cannot connect to database: {}", e),
            }
        }
        DatabaseCommands::Migrate => {
            info!("Running database migrations...");
            let pool = db_config.create_pool().await?;
            let migrator = solana_pda_analyzer_database::DatabaseMigrator::new(db_config.database_url());
            migrator.run_migrations(&pool).await?;
            println!("✅ Database migrations completed");
        }
    }
    
    Ok(())
}

async fn show_stats() -> Result<()> {
    info!("Fetching statistics...");
    
    let db_config = DatabaseConfig::from_env()?;
    let pool = initialize_database(&db_config).await?;
    let repository = DatabaseRepository::new(pool);
    
    match repository.get_database_metrics().await {
        Ok(metrics) => {
            println!("📊 Database Statistics:");
            println!("  Programs: {}", metrics.total_programs);
            println!("  Transactions: {}", metrics.total_transactions);
            println!("  PDAs: {}", metrics.total_pdas);
            println!("  Interactions: {}", metrics.total_interactions);
            println!("  Database Size: {:.2} MB", metrics.database_size_mb);
        }
        Err(e) => {
            error!("Failed to fetch statistics: {}", e);
            println!("❌ Failed to fetch statistics");
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_parsing() {
        // Test that CLI commands parse correctly
        let cli = Cli::try_parse_from(["pda-analyzer", "stats"]);
        assert!(cli.is_ok());
        
        let cli = Cli::try_parse_from([
            "pda-analyzer", "analyze", 
            "--address", "11111111111111111111111111111111",
            "--program-id", "11111111111111111111111111111111"
        ]);
        assert!(cli.is_ok());
    }
}