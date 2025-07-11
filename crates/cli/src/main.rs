use clap::{Parser, Subcommand};
use solana_pda_analyzer_core::PdaAnalyzer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use anyhow::Result;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a PDA
    Analyze {
        /// PDA address to analyze
        #[clap(short, long)]
        address: String,
        /// Program ID
        #[clap(short, long)]
        program_id: String,
    },
    /// Run example analyses
    Examples,
    /// Show version information
    Version,
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
        Commands::Analyze { address, program_id } => {
            analyze_pda(&address, &program_id).await?;
        }
        Commands::Examples => {
            run_examples().await?;
        }
        Commands::Version => {
            println!("Solana PDA Analyzer v{}", env!("CARGO_PKG_VERSION"));
            println!("A comprehensive tool for analyzing Solana Program Derived Addresses");
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
            
            // Pattern analysis would be added here in future versions
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

async fn run_examples() -> Result<()> {
    println!("🚀 Running Solana PDA Analyzer Examples");
    println!("========================================");
    
    // Example 1: Associated Token Account
    println!("\n📊 Example 1: Associated Token Account");
    let ata_address = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr";
    let ata_program = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
    analyze_pda(ata_address, ata_program).await?;
    
    // Example 2: System Program Account
    println!("\n🔧 Example 2: System Program Account");
    let system_address = "11111111111111111111111111111111";
    let system_program = "11111111111111111111111111111112";
    analyze_pda(system_address, system_program).await?;
    
    println!("\n✅ Examples completed!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cli_parsing() {
        // Test that CLI commands parse correctly
        let cli = Cli::try_parse_from(["pda-analyzer", "examples"]);
        assert!(cli.is_ok());
        
        let cli = Cli::try_parse_from([
            "pda-analyzer", "analyze", 
            "--address", "11111111111111111111111111111111",
            "--program-id", "11111111111111111111111111111111"
        ]);
        assert!(cli.is_ok());
    }
}