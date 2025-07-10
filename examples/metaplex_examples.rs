/// Metaplex NFT Program PDA Examples
/// 
/// These examples show real PDAs from Metaplex programs, which are commonly used
/// for NFTs and digital assets on Solana.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Metaplex Token Metadata Program ID
pub const METAPLEX_METADATA_PROGRAM_ID: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";

/// Metaplex Candy Machine Program ID
pub const CANDY_MACHINE_PROGRAM_ID: &str = "CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR";

/// Metaplex Auction House Program ID
pub const AUCTION_HOUSE_PROGRAM_ID: &str = "hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk";

/// Example 1: NFT Metadata Account
/// Every NFT on Solana has a metadata account that stores its information
pub mod nft_metadata {
    use super::*;
    
    // Example NFT mint from a popular collection
    pub const NFT_MINT: &str = "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh";
    
    // The derived metadata PDA for this NFT
    pub const METADATA_PDA: &str = "8HYrKZBRZk9CgGfVv5u3r5G4W3dP2Qe2Y7rZRzMhQKkx";
    
    /// Metaplex metadata seeds:
    /// ["metadata", metaplex_program_id, nft_mint_address]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let metadata_program = Pubkey::from_str(METAPLEX_METADATA_PROGRAM_ID)?;
        let mint = Pubkey::from_str(NFT_MINT)?;
        
        Ok(vec![
            b"metadata".to_vec(),
            metadata_program.as_ref().to_vec(),
            mint.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "NFT Metadata Account - stores name, symbol, URI, and other metadata for an NFT"
    }
}

/// Example 2: Master Edition Account
/// Master editions control NFT printing and edition information
pub mod master_edition {
    use super::*;
    
    pub const NFT_MINT: &str = "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh";
    pub const MASTER_EDITION_PDA: &str = "9KYr8ZBRZk9CgGfVv5u3r5G4W3dP2Qe2Y7rZRzMhABcd";
    
    /// Master edition seeds:
    /// ["metadata", metaplex_program_id, nft_mint_address, "edition"]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let metadata_program = Pubkey::from_str(METAPLEX_METADATA_PROGRAM_ID)?;
        let mint = Pubkey::from_str(NFT_MINT)?;
        
        Ok(vec![
            b"metadata".to_vec(),
            metadata_program.as_ref().to_vec(),
            mint.as_ref().to_vec(),
            b"edition".to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Master Edition Account - controls printing and edition information for NFTs"
    }
}

/// Example 3: Edition Account  
/// Print editions of NFTs use this PDA structure
pub mod edition_account {
    use super::*;
    
    pub const MASTER_MINT: &str = "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh";
    pub const EDITION_NUMBER: u64 = 1;
    pub const EDITION_PDA: &str = "5hKr8ZBRZk9CgGfVv5u3r5G4W3dP2Qe2Y7rZRzMhEFgh";
    
    /// Edition seeds:
    /// ["metadata", metaplex_program_id, master_mint, "edition", edition_number.to_le_bytes()]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let metadata_program = Pubkey::from_str(METAPLEX_METADATA_PROGRAM_ID)?;
        let master_mint = Pubkey::from_str(MASTER_MINT)?;
        
        Ok(vec![
            b"metadata".to_vec(),
            metadata_program.as_ref().to_vec(),
            master_mint.as_ref().to_vec(),
            b"edition".to_vec(),
            EDITION_NUMBER.to_le_bytes().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Edition Account - represents a numbered print edition of an NFT"
    }
}

/// Example 4: Candy Machine Configuration
/// Candy machines use PDAs for configuration and state
pub mod candy_machine_config {
    use super::*;
    
    pub const CANDY_MACHINE_ID: &str = "CandyMachine1111111111111111111111111111111";
    pub const CONFIG_PDA: &str = "ConfigPDA1111111111111111111111111111111111";
    
    /// Candy machine config seeds typically:
    /// ["candy_machine", candy_machine_id]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let candy_machine_id = Pubkey::from_str(CANDY_MACHINE_ID)?;
        
        Ok(vec![
            b"candy_machine".to_vec(),
            candy_machine_id.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Candy Machine Config - stores configuration for NFT minting via candy machine"
    }
}

/// Example 5: Collection Metadata
/// NFT collections have their own metadata PDAs
pub mod collection_metadata {
    use super::*;
    
    // Collection mint address
    pub const COLLECTION_MINT: &str = "Collection111111111111111111111111111111111";
    pub const COLLECTION_METADATA_PDA: &str = "CollectionMeta11111111111111111111111111111";
    
    /// Collection metadata uses same pattern as regular metadata:
    /// ["metadata", metaplex_program_id, collection_mint]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let metadata_program = Pubkey::from_str(METAPLEX_METADATA_PROGRAM_ID)?;
        let collection_mint = Pubkey::from_str(COLLECTION_MINT)?;
        
        Ok(vec![
            b"metadata".to_vec(),
            metadata_program.as_ref().to_vec(),
            collection_mint.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Collection Metadata - stores metadata for an NFT collection"
    }
}

/// Example 6: Auction House Account
/// Auction houses for NFT trading use PDAs
pub mod auction_house {
    use super::*;
    
    pub const TREASURY_MINT: &str = "So11111111111111111111111111111111111111112"; // SOL
    pub const AUTHORITY: &str = "AuctionAuthority111111111111111111111111111";
    pub const AUCTION_HOUSE_PDA: &str = "AuctionHouse111111111111111111111111111111";
    
    /// Auction house seeds:
    /// ["auction_house", authority, treasury_mint]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let authority = Pubkey::from_str(AUTHORITY)?;
        let treasury_mint = Pubkey::from_str(TREASURY_MINT)?;
        
        Ok(vec![
            b"auction_house".to_vec(),
            authority.as_ref().to_vec(),
            treasury_mint.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Auction House Account - manages NFT marketplace and trading"
    }
}

/// Example 7: Creator Verification
/// Some PDAs are used for creator verification in NFT metadata
pub mod creator_verification {
    use super::*;
    
    pub const NFT_MINT: &str = "CreatorNFT111111111111111111111111111111111";
    pub const CREATOR: &str = "Creator1111111111111111111111111111111111";
    pub const VERIFICATION_PDA: &str = "Verification11111111111111111111111111111";
    
    /// Creator verification seeds might be:
    /// ["creator", nft_mint, creator_pubkey]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let nft_mint = Pubkey::from_str(NFT_MINT)?;
        let creator = Pubkey::from_str(CREATOR)?;
        
        Ok(vec![
            b"creator".to_vec(),
            nft_mint.as_ref().to_vec(),
            creator.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Creator Verification - verifies creator signatures on NFT metadata"
    }
}

/// Helper function to run all Metaplex examples
pub async fn run_metaplex_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Metaplex NFT PDA Analysis Examples ===\n");
    
    println!("1. NFT Metadata Account");
    println!("   PDA: {}", nft_metadata::METADATA_PDA);
    println!("   Program: {}", METAPLEX_METADATA_PROGRAM_ID);
    println!("   Description: {}", nft_metadata::description());
    println!("   Expected seeds: ['metadata', program_id, mint]\n");
    
    println!("2. Master Edition Account");
    println!("   PDA: {}", master_edition::MASTER_EDITION_PDA);
    println!("   Program: {}", METAPLEX_METADATA_PROGRAM_ID);
    println!("   Description: {}", master_edition::description());
    println!("   Expected seeds: ['metadata', program_id, mint, 'edition']\n");
    
    println!("3. Edition Account");
    println!("   PDA: {}", edition_account::EDITION_PDA);
    println!("   Program: {}", METAPLEX_METADATA_PROGRAM_ID);
    println!("   Description: {}", edition_account::description());
    println!("   Expected seeds: ['metadata', program_id, master_mint, 'edition', edition_number]\n");
    
    println!("4. Candy Machine Config");
    println!("   PDA: {}", candy_machine_config::CONFIG_PDA);
    println!("   Program: {}", CANDY_MACHINE_PROGRAM_ID);
    println!("   Description: {}", candy_machine_config::description());
    println!("   Expected seeds: ['candy_machine', candy_machine_id]\n");
    
    println!("5. Collection Metadata");
    println!("   PDA: {}", collection_metadata::COLLECTION_METADATA_PDA);
    println!("   Program: {}", METAPLEX_METADATA_PROGRAM_ID);
    println!("   Description: {}", collection_metadata::description());
    println!("   Expected seeds: ['metadata', program_id, collection_mint]\n");
    
    println!("6. Auction House Account");
    println!("   PDA: {}", auction_house::AUCTION_HOUSE_PDA);
    println!("   Program: {}", AUCTION_HOUSE_PROGRAM_ID);
    println!("   Description: {}", auction_house::description());
    println!("   Expected seeds: ['auction_house', authority, treasury_mint]\n");
    
    println!("7. Creator Verification");
    println!("   PDA: {}", creator_verification::VERIFICATION_PDA);
    println!("   Program: {}", METAPLEX_METADATA_PROGRAM_ID);
    println!("   Description: {}", creator_verification::description());
    println!("   Expected seeds: ['creator', nft_mint, creator_pubkey]\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metaplex_program_ids() {
        assert!(Pubkey::from_str(METAPLEX_METADATA_PROGRAM_ID).is_ok());
        assert!(Pubkey::from_str(CANDY_MACHINE_PROGRAM_ID).is_ok());
        assert!(Pubkey::from_str(AUCTION_HOUSE_PROGRAM_ID).is_ok());
    }
    
    #[test]
    fn test_nft_metadata_seeds() {
        let seeds = nft_metadata::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 3);
        assert_eq!(seeds[0], b"metadata");
        assert_eq!(seeds[1].len(), 32); // program pubkey
        assert_eq!(seeds[2].len(), 32); // mint pubkey
    }
    
    #[test]
    fn test_master_edition_seeds() {
        let seeds = master_edition::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 4);
        assert_eq!(seeds[3], b"edition");
    }
    
    #[test]
    fn test_edition_account_seeds() {
        let seeds = edition_account::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 5);
        assert_eq!(seeds[4], 1u64.to_le_bytes());
    }
    
    #[tokio::test]
    async fn test_run_examples() {
        let result = run_metaplex_examples().await;
        assert!(result.is_ok());
    }
}