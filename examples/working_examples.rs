/// Working PDA Examples with Real Derivations
/// 
/// These examples create actual working PDAs that can be analyzed by our tool.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Example 1: Create a working "state" PDA for any program
pub fn create_state_pda_example(program_id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let seeds = &[b"state"];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 2: Create a working "config" PDA
pub fn create_config_pda_example(program_id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let seeds = &[b"config"];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 3: Create a working "authority" PDA
pub fn create_authority_pda_example(program_id: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let seeds = &[b"authority"];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 4: Create a working sequential PDA
pub fn create_sequential_pda_example(program_id: &str, index: u64) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let seeds = &[b"pool", &index.to_le_bytes()];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 5: Create a working authority + pubkey PDA
pub fn create_authority_pubkey_pda_example(program_id: &str, authority: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let authority_pubkey = Pubkey::from_str(authority)?;
    let seeds = &[b"authority", authority_pubkey.as_ref()];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 6: Create a working governance-style complex PDA
pub fn create_governance_pda_example(program_id: &str, realm: &str, proposal_id: u32) -> Result<(String, String), Box<dyn std::error::Error>> {
    let program_pubkey = Pubkey::from_str(program_id)?;
    let realm_pubkey = Pubkey::from_str(realm)?;
    let seeds = &[
        b"governance",
        realm_pubkey.as_ref(),
        b"proposal",
        &proposal_id.to_le_bytes(),
    ];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &program_pubkey);
    
    Ok((pda_address.to_string(), program_id.to_string()))
}

/// Example 7: Create a working Associated Token Account
pub fn create_ata_example(wallet: &str, mint: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let wallet_pubkey = Pubkey::from_str(wallet)?;
    let mint_pubkey = Pubkey::from_str(mint)?;
    let token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
    let ata_program = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")?;
    
    let seeds = &[
        wallet_pubkey.as_ref(),
        token_program.as_ref(),
        mint_pubkey.as_ref(),
    ];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &ata_program);
    
    Ok((pda_address.to_string(), ata_program.to_string()))
}

/// Example 8: Create a working Metaplex metadata PDA
pub fn create_metaplex_metadata_example(mint: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mint_pubkey = Pubkey::from_str(mint)?;
    let metadata_program = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")?;
    
    let seeds = &[
        b"metadata",
        metadata_program.as_ref(),
        mint_pubkey.as_ref(),
    ];
    let (pda_address, _bump) = Pubkey::find_program_address(seeds, &metadata_program);
    
    Ok((pda_address.to_string(), metadata_program.to_string()))
}

/// Run all working examples
pub async fn run_working_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Running Working PDA Examples");
    println!("===============================");
    
    // Use a test program ID
    let test_program = "11111111111111111111111111111112";
    let test_wallet = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    let test_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let test_realm = "DPiH3H3c7t47BMxqTxLsuPQpEC6Kne8GA9VXbxpnZxFE";
    
    println!("\n📊 Example 1: State PDA");
    let (pda, prog) = create_state_pda_example(test_program)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: STRING_SINGLETON");
    
    println!("\n🔧 Example 2: Config PDA");
    let (pda, prog) = create_config_pda_example(test_program)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: STRING_SINGLETON");
    
    println!("\n👑 Example 3: Authority PDA");
    let (pda, prog) = create_authority_pda_example(test_program)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: STRING_SINGLETON");
    
    println!("\n🔢 Example 4: Sequential PDA (Pool #5)");
    let (pda, prog) = create_sequential_pda_example(test_program, 5)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: SEQUENTIAL");
    
    println!("\n🔐 Example 5: Authority + Pubkey PDA");
    let (pda, prog) = create_authority_pubkey_pda_example(test_program, test_wallet)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: STRING_PUBKEY");
    
    println!("\n🏛️  Example 6: Governance Proposal PDA");
    let (pda, prog) = create_governance_pda_example(test_program, test_realm, 1)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: COMPLEX");
    
    println!("\n💰 Example 7: Associated Token Account");
    let (pda, prog) = create_ata_example(test_wallet, test_mint)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: WALLET_TOKEN_MINT");
    
    println!("\n🎨 Example 8: Metaplex Metadata");
    let (pda, prog) = create_metaplex_metadata_example(test_mint)?;
    println!("   PDA: {}", pda);
    println!("   Program: {}", prog);
    println!("   Expected Pattern: STRING_PROGRAM_MINT");
    
    println!("\n✅ All working examples generated!");
    println!("\n💡 To test these, run:");
    println!("   ./target/release/pda-analyzer analyze --address <PDA> --program-id <PROGRAM>");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_pda_creation() {
        let test_program = "11111111111111111111111111111112";
        let result = create_state_pda_example(test_program);
        assert!(result.is_ok());
        
        let (pda, prog) = result.unwrap();
        assert_eq!(prog, test_program);
        assert!(!pda.is_empty());
    }
    
    #[test]
    fn test_ata_creation() {
        let wallet = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let result = create_ata_example(wallet, mint);
        assert!(result.is_ok());
        
        let (pda, prog) = result.unwrap();
        assert_eq!(prog, "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
        assert!(!pda.is_empty());
    }
    
    #[test]
    fn test_metaplex_creation() {
        let mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        let result = create_metaplex_metadata_example(mint);
        assert!(result.is_ok());
        
        let (pda, prog) = result.unwrap();
        assert_eq!(prog, "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
        assert!(!pda.is_empty());
    }
}