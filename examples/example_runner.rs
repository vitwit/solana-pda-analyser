use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

mod spl_token_examples;
mod metaplex_examples;
mod real_world_examples;

use spl_token_examples::*;
use metaplex_examples::*;
use real_world_examples::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Solana PDA Analyzer - Example Analysis Runner\n");
    println!("=" .repeat(60));
    
    // Run SPL Token examples
    println!("\n📊 Running SPL Token Examples...");
    run_spl_token_examples().await?;
    
    println!("\n" + &"=".repeat(60));
    
    // Run Metaplex examples  
    println!("\n🎨 Running Metaplex NFT Examples...");
    run_metaplex_examples().await?;
    
    println!("\n" + &"=".repeat(60));
    
    // Run real-world protocol examples
    println!("\n🌍 Running Real-World Protocol Examples...");
    run_real_world_examples().await?;
    
    println!("\n" + &"=".repeat(60));
    println!("\n✅ All PDA analysis examples completed successfully!");
    println!("\n💡 These examples demonstrate common PDA patterns found on Solana:");
    println!("   • Associated Token Accounts (most common)");
    println!("   • NFT Metadata and Master Editions");  
    println!("   • DeFi Protocol Authorities and Vaults");
    println!("   • Governance and DAO Structures");
    println!("   • Oracle Price Feeds");
    println!("   • Escrow and Trading Accounts");
    
    Ok(())
}