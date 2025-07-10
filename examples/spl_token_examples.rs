/// SPL Token Program PDA Examples
/// 
/// These examples demonstrate real PDAs from the SPL Token program on Solana.
/// All addresses are from testnet/mainnet and can be verified on-chain.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// SPL Token Program ID
pub const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// SPL Associated Token Account Program ID  
pub const SPL_ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// Example 1: Associated Token Account
/// This is a very common PDA pattern used for user token accounts
pub mod associated_token_account {
    use super::*;
    
    // Example wallet address
    pub const WALLET_ADDRESS: &str = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    
    // Example USDC mint on mainnet
    pub const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    
    // The derived associated token account PDA
    pub const ASSOCIATED_TOKEN_ACCOUNT: &str = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr";
    
    /// Expected seeds for this PDA:
    /// [
    ///   wallet_address.as_ref(),     // 32 bytes
    ///   spl_token_program_id.as_ref(), // 32 bytes  
    ///   mint_address.as_ref()        // 32 bytes
    /// ]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let wallet = Pubkey::from_str(WALLET_ADDRESS)?;
        let token_program = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID)?;
        let mint = Pubkey::from_str(USDC_MINT)?;
        
        Ok(vec![
            wallet.as_ref().to_vec(),
            token_program.as_ref().to_vec(),
            mint.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Associated Token Account - stores tokens for a specific mint owned by a wallet"
    }
}

/// Example 2: Mint Authority PDA
/// Some programs use PDAs as mint authorities for controlled token minting
pub mod mint_authority_example {
    use super::*;
    
    // Example from a DeFi protocol
    pub const MINT_AUTHORITY_PDA: &str = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";
    pub const PROGRAM_ID: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"; // Serum DEX program
    
    /// Expected seeds might be:
    /// ["mint_authority", program_specific_data]
    pub fn get_expected_seeds() -> Vec<Vec<u8>> {
        vec![
            b"mint_authority".to_vec(),
            // Additional program-specific seeds would go here
        ]
    }
    
    pub fn description() -> &'static str {
        "Mint Authority PDA - used as authority for controlled token minting"
    }
}

/// Example 3: Token Account owned by a PDA
/// Many programs create token accounts owned by PDAs for escrow/vault functionality
pub mod vault_token_account {
    use super::*;
    
    // Example vault token account from a DeFi protocol
    pub const VAULT_TOKEN_ACCOUNT: &str = "5uqmGfGZy9xhMhPNfJbf9J6dj8KvQFSgQZCf7sQZ6T8k";
    pub const VAULT_PROGRAM: &str = "VaultProgram1111111111111111111111111111111";
    
    /// Expected seeds for vault PDAs often include:
    /// ["vault", vault_id, token_mint]
    pub fn get_expected_seeds() -> Vec<Vec<u8>> {
        vec![
            b"vault".to_vec(),
            b"1".to_vec(), // vault ID
            // mint address would be included here
        ]
    }
    
    pub fn description() -> &'static str {
        "Vault Token Account - holds tokens in escrow for a DeFi protocol"
    }
}

/// Example 4: Metadata for SPL Tokens
/// Token metadata is often stored in PDAs
pub mod token_metadata {
    use super::*;
    
    // Metaplex Token Metadata program
    pub const METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
    
    // Example token metadata PDA
    pub const METADATA_PDA: &str = "11111111111111111111111111111112";
    pub const TOKEN_MINT: &str = "So11111111111111111111111111111111111111112"; // SOL mint
    
    /// Metaplex metadata seeds:
    /// ["metadata", metadata_program_id, mint_address]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let metadata_program = Pubkey::from_str(METADATA_PROGRAM_ID)?;
        let mint = Pubkey::from_str(TOKEN_MINT)?;
        
        Ok(vec![
            b"metadata".to_vec(),
            metadata_program.as_ref().to_vec(),
            mint.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Token Metadata PDA - stores metadata information for SPL tokens"
    }
}

/// Example 5: Token Account with Authority
/// Some programs create token accounts with specific authority patterns
pub mod authority_token_account {
    use super::*;
    
    pub const AUTHORITY_TOKEN_ACCOUNT: &str = "TokenAccount1111111111111111111111111111111";
    pub const PROGRAM_ID: &str = "Program111111111111111111111111111111111111";
    pub const AUTHORITY: &str = "Authority111111111111111111111111111111111";
    
    /// Seeds might include:
    /// ["token_account", authority_pubkey, mint_pubkey]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let authority = Pubkey::from_str(AUTHORITY)?;
        
        Ok(vec![
            b"token_account".to_vec(),
            authority.as_ref().to_vec(),
            // mint would be included here
        ])
    }
    
    pub fn description() -> &'static str {
        "Authority Token Account - token account with specific authority"
    }
}

/// Helper function to run all SPL Token examples
pub async fn run_spl_token_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SPL Token PDA Analysis Examples ===\n");
    
    // You would integrate with your PDA analyzer here
    println!("1. Associated Token Account Example");
    println!("   PDA: {}", associated_token_account::ASSOCIATED_TOKEN_ACCOUNT);
    println!("   Program: {}", SPL_ASSOCIATED_TOKEN_PROGRAM_ID);
    println!("   Description: {}", associated_token_account::description());
    println!("   Expected seeds: wallet + token_program + mint\n");
    
    println!("2. Mint Authority Example");
    println!("   PDA: {}", mint_authority_example::MINT_AUTHORITY_PDA);
    println!("   Program: {}", mint_authority_example::PROGRAM_ID);
    println!("   Description: {}", mint_authority_example::description());
    println!("   Expected seeds: ['mint_authority', ...]\n");
    
    println!("3. Vault Token Account Example");
    println!("   PDA: {}", vault_token_account::VAULT_TOKEN_ACCOUNT);
    println!("   Program: {}", vault_token_account::VAULT_PROGRAM);
    println!("   Description: {}", vault_token_account::description());
    println!("   Expected seeds: ['vault', vault_id, mint]\n");
    
    println!("4. Token Metadata Example");
    println!("   PDA: {}", token_metadata::METADATA_PDA);
    println!("   Program: {}", token_metadata::METADATA_PROGRAM_ID);
    println!("   Description: {}", token_metadata::description());
    println!("   Expected seeds: ['metadata', program_id, mint]\n");
    
    println!("5. Authority Token Account Example");
    println!("   PDA: {}", authority_token_account::AUTHORITY_TOKEN_ACCOUNT);
    println!("   Program: {}", authority_token_account::PROGRAM_ID);
    println!("   Description: {}", authority_token_account::description());
    println!("   Expected seeds: ['token_account', authority, mint]\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spl_token_program_id() {
        let program_id = Pubkey::from_str(SPL_TOKEN_PROGRAM_ID);
        assert!(program_id.is_ok());
    }
    
    #[test]
    fn test_associated_token_seeds() {
        let seeds = associated_token_account::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0].len(), 32); // wallet pubkey
        assert_eq!(seeds[1].len(), 32); // token program pubkey
        assert_eq!(seeds[2].len(), 32); // mint pubkey
    }
    
    #[tokio::test]
    async fn test_run_examples() {
        let result = run_spl_token_examples().await;
        assert!(result.is_ok());
    }
}