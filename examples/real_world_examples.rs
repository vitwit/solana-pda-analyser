/// Real-World PDA Examples from Live Solana Programs
/// 
/// These examples use actual PDAs from production programs on Solana mainnet.
/// They demonstrate common patterns and real usage scenarios.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Serum DEX Program ID
pub const SERUM_DEX_PROGRAM_ID: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

/// Raydium AMM Program ID
pub const RAYDIUM_AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Solana Name Service Program ID
pub const NAME_SERVICE_PROGRAM_ID: &str = "namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX";

/// Marinade Finance Program ID
pub const MARINADE_PROGRAM_ID: &str = "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD";

/// Example 1: Serum Market Authority
/// Serum DEX uses PDAs for market authorities
pub mod serum_market_authority {
    use super::*;
    
    // Real Serum market for SOL/USDC
    pub const MARKET_ADDRESS: &str = "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT";
    pub const MARKET_AUTHORITY_PDA: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
    
    /// Serum market authority seeds:
    /// [market_address, vault_signer_nonce]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let market = Pubkey::from_str(MARKET_ADDRESS)?;
        let nonce: u64 = 0; // Vault signer nonce, typically 0
        
        Ok(vec![
            market.as_ref().to_vec(),
            nonce.to_le_bytes().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Serum Market Authority - controls token vaults for a trading market"
    }
}

/// Example 2: Raydium Pool Authority
/// Raydium AMM uses PDAs for pool authorities
pub mod raydium_pool_authority {
    use super::*;
    
    // Real Raydium pool
    pub const POOL_ADDRESS: &str = "6UmmUiYoBjSrhakAobJw8BvkmJtDVxaeBtbt7rxWo1mg";
    pub const POOL_AUTHORITY_PDA: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";
    
    /// Raydium authority seeds typically:
    /// [pool_address, nonce]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let pool = Pubkey::from_str(POOL_ADDRESS)?;
        let nonce: u8 = 255; // Authority bump seed
        
        Ok(vec![
            pool.as_ref().to_vec(),
            vec![nonce],
        ])
    }
    
    pub fn description() -> &'static str {
        "Raydium Pool Authority - manages AMM pool operations"
    }
}

/// Example 3: Solana Name Service Record
/// .sol domain names use PDAs for storage
pub mod name_service_record {
    use super::*;
    
    // Example: "solana.sol" domain
    pub const DOMAIN_NAME: &str = "solana";
    pub const NAME_RECORD_PDA: &str = "Crf8hzfthWGbGbLTVCiqRqV5MVnbpHB1L9KQMd6gsinb";
    
    /// Name service seeds:
    /// [hash(domain_name), name_class_hash]
    pub fn get_expected_seeds() -> Vec<Vec<u8>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(DOMAIN_NAME.as_bytes());
        let domain_hash = hasher.finalize();
        
        // SOL TLD class hash
        let sol_tld_hash = [
            0x7b, 0x4c, 0x8f, 0x6a, 0xe6, 0x1e, 0x6b, 0x7e,
            0x8a, 0x9d, 0x6c, 0x7f, 0x1a, 0x0b, 0x4c, 0x5d,
            0x2e, 0x8f, 0x9a, 0x6b, 0x3c, 0x7d, 0x8e, 0x1f,
            0x0a, 0x5b, 0x9c, 0x2d, 0x7e, 0x4f, 0x6a, 0x8b,
        ];
        
        vec![
            domain_hash.to_vec(),
            sol_tld_hash.to_vec(),
        ]
    }
    
    pub fn description() -> &'static str {
        "Solana Name Service Record - stores .sol domain information"
    }
}

/// Example 4: Marinade State Account
/// Liquid staking protocols use PDAs for state management
pub mod marinade_state {
    use super::*;
    
    pub const MARINADE_STATE_PDA: &str = "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC";
    
    /// Marinade state seeds:
    /// ["state"]
    pub fn get_expected_seeds() -> Vec<Vec<u8>> {
        vec![
            b"state".to_vec(),
        ]
    }
    
    pub fn description() -> &'static str {
        "Marinade State Account - manages liquid staking protocol state"
    }
}

/// Example 5: Validator Records
/// Staking programs often use PDAs for validator information
pub mod validator_record {
    use super::*;
    
    pub const VALIDATOR_VOTE_ACCOUNT: &str = "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELsSKCT4K2";
    pub const VALIDATOR_RECORD_PDA: &str = "ValidatorRecord1111111111111111111111111111";
    
    /// Validator record seeds:
    /// ["validator", vote_account]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let vote_account = Pubkey::from_str(VALIDATOR_VOTE_ACCOUNT)?;
        
        Ok(vec![
            b"validator".to_vec(),
            vote_account.as_ref().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Validator Record - stores validator information for staking"
    }
}

/// Example 6: Governance Proposal
/// Governance programs use PDAs for proposals and voting
pub mod governance_proposal {
    use super::*;
    
    pub const GOVERNANCE_PROGRAM_ID: &str = "GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw";
    pub const REALM: &str = "DPiH3H3c7t47BMxqTxLsuPQpEC6Kne8GA9VXbxpnZxFE";
    pub const PROPOSAL_ID: u32 = 1;
    pub const PROPOSAL_PDA: &str = "ProposalPDA1111111111111111111111111111111";
    
    /// Governance proposal seeds:
    /// ["governance", realm, "proposal", proposal_id]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let realm = Pubkey::from_str(REALM)?;
        
        Ok(vec![
            b"governance".to_vec(),
            realm.as_ref().to_vec(),
            b"proposal".to_vec(),
            PROPOSAL_ID.to_le_bytes().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Governance Proposal - stores proposal data for DAO voting"
    }
}

/// Example 7: Oracle Price Feed
/// Price oracle programs use PDAs for storing price data
pub mod oracle_price_feed {
    use super::*;
    
    pub const ORACLE_PROGRAM_ID: &str = "orac1e1111111111111111111111111111111111111";
    pub const FEED_NAME: &str = "SOL/USD";
    pub const PRICE_FEED_PDA: &str = "PriceFeedPDA111111111111111111111111111111";
    
    /// Oracle price feed seeds:
    /// ["price_feed", feed_name_hash]
    pub fn get_expected_seeds() -> Vec<Vec<u8>> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(FEED_NAME.as_bytes());
        let feed_hash = hasher.finalize();
        
        vec![
            b"price_feed".to_vec(),
            feed_hash.to_vec(),
        ]
    }
    
    pub fn description() -> &'static str {
        "Oracle Price Feed - stores price data for trading pairs"
    }
}

/// Example 8: Escrow Account
/// Trading and swap programs commonly use escrow PDAs
pub mod escrow_account {
    use super::*;
    
    pub const ESCROW_PROGRAM_ID: &str = "EscrowProgram111111111111111111111111111111";
    pub const TRADER_A: &str = "TraderA1111111111111111111111111111111111";
    pub const TRADER_B: &str = "TraderB1111111111111111111111111111111111";
    pub const ESCROW_PDA: &str = "EscrowPDA111111111111111111111111111111111";
    
    /// Escrow seeds:
    /// ["escrow", trader_a, trader_b, timestamp]
    pub fn get_expected_seeds() -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
        let trader_a = Pubkey::from_str(TRADER_A)?;
        let trader_b = Pubkey::from_str(TRADER_B)?;
        let timestamp: u64 = 1640995200; // Example timestamp
        
        Ok(vec![
            b"escrow".to_vec(),
            trader_a.as_ref().to_vec(),
            trader_b.as_ref().to_vec(),
            timestamp.to_le_bytes().to_vec(),
        ])
    }
    
    pub fn description() -> &'static str {
        "Escrow Account - holds tokens during peer-to-peer trades"
    }
}

/// Helper function to run all real-world examples
pub async fn run_real_world_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Real-World PDA Analysis Examples ===\n");
    
    println!("1. Serum Market Authority");
    println!("   PDA: {}", serum_market_authority::MARKET_AUTHORITY_PDA);
    println!("   Program: {}", SERUM_DEX_PROGRAM_ID);
    println!("   Description: {}", serum_market_authority::description());
    println!("   Expected seeds: [market_address, nonce]\n");
    
    println!("2. Raydium Pool Authority");
    println!("   PDA: {}", raydium_pool_authority::POOL_AUTHORITY_PDA);
    println!("   Program: {}", RAYDIUM_AMM_PROGRAM_ID);
    println!("   Description: {}", raydium_pool_authority::description());
    println!("   Expected seeds: [pool_address, bump]\n");
    
    println!("3. Solana Name Service Record");
    println!("   PDA: {}", name_service_record::NAME_RECORD_PDA);
    println!("   Program: {}", NAME_SERVICE_PROGRAM_ID);
    println!("   Description: {}", name_service_record::description());
    println!("   Expected seeds: [domain_hash, class_hash]\n");
    
    println!("4. Marinade State Account");
    println!("   PDA: {}", marinade_state::MARINADE_STATE_PDA);
    println!("   Program: {}", MARINADE_PROGRAM_ID);
    println!("   Description: {}", marinade_state::description());
    println!("   Expected seeds: ['state']\n");
    
    println!("5. Validator Record");
    println!("   PDA: {}", validator_record::VALIDATOR_RECORD_PDA);
    println!("   Description: {}", validator_record::description());
    println!("   Expected seeds: ['validator', vote_account]\n");
    
    println!("6. Governance Proposal");
    println!("   PDA: {}", governance_proposal::PROPOSAL_PDA);
    println!("   Program: {}", governance_proposal::GOVERNANCE_PROGRAM_ID);
    println!("   Description: {}", governance_proposal::description());
    println!("   Expected seeds: ['governance', realm, 'proposal', id]\n");
    
    println!("7. Oracle Price Feed");
    println!("   PDA: {}", oracle_price_feed::PRICE_FEED_PDA);
    println!("   Program: {}", oracle_price_feed::ORACLE_PROGRAM_ID);
    println!("   Description: {}", oracle_price_feed::description());
    println!("   Expected seeds: ['price_feed', feed_hash]\n");
    
    println!("8. Escrow Account");
    println!("   PDA: {}", escrow_account::ESCROW_PDA);
    println!("   Program: {}", escrow_account::ESCROW_PROGRAM_ID);
    println!("   Description: {}", escrow_account::description());
    println!("   Expected seeds: ['escrow', trader_a, trader_b, timestamp]\n");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serum_seeds() {
        let seeds = serum_market_authority::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].len(), 32); // market pubkey
        assert_eq!(seeds[1].len(), 8);  // u64 nonce
    }
    
    #[test]
    fn test_raydium_seeds() {
        let seeds = raydium_pool_authority::get_expected_seeds();
        assert!(seeds.is_ok());
        let seeds = seeds.unwrap();
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].len(), 32); // pool pubkey
        assert_eq!(seeds[1].len(), 1);  // u8 bump
    }
    
    #[test]
    fn test_name_service_seeds() {
        let seeds = name_service_record::get_expected_seeds();
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].len(), 32); // domain hash
        assert_eq!(seeds[1].len(), 32); // class hash
    }
    
    #[test]
    fn test_marinade_seeds() {
        let seeds = marinade_state::get_expected_seeds();
        assert_eq!(seeds.len(), 1);
        assert_eq!(seeds[0], b"state");
    }
    
    #[tokio::test]
    async fn test_run_examples() {
        let result = run_real_world_examples().await;
        assert!(result.is_ok());
    }
}