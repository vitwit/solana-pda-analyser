use crate::{Result, PdaInfo, SeedValue};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::str::FromStr;

/// Caches PDA analysis results for performance
type PdaCache = HashMap<(Pubkey, Vec<Vec<u8>>), Option<PdaInfo>>;

/// Pattern types detected by the analyzer
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PdaPattern {
    AssociatedTokenAccount,
    MetaplexMetadata,
    MetaplexMasterEdition,
    MetaplexEdition,
    StringSingleton,
    StringAuthority,
    StringPubkey,
    StringPubkeyString,
    PubkeyU64,
    PubkeyU8,
    Sequential,
    Complex,
    Unknown,
}

impl PdaPattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            PdaPattern::AssociatedTokenAccount => "WALLET_TOKEN_MINT",
            PdaPattern::MetaplexMetadata => "STRING_PROGRAM_MINT",
            PdaPattern::MetaplexMasterEdition => "STRING_PROGRAM_MINT_STRING",
            PdaPattern::MetaplexEdition => "STRING_PROGRAM_MINT_STRING_U64",
            PdaPattern::StringSingleton => "STRING_SINGLETON",
            PdaPattern::StringAuthority => "STRING_AUTHORITY",
            PdaPattern::StringPubkey => "STRING_PUBKEY",
            PdaPattern::StringPubkeyString => "STRING_PUBKEY_STRING",
            PdaPattern::PubkeyU64 => "PUBKEY_U64",
            PdaPattern::PubkeyU8 => "PUBKEY_U8",
            PdaPattern::Sequential => "SEQUENTIAL",
            PdaPattern::Complex => "COMPLEX",
            PdaPattern::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PdaAnalysisResult {
    pub pda_info: PdaInfo,
    pub pattern: PdaPattern,
    pub confidence: f64,
    pub analysis_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PdaAnalyzer {
    cache: PdaCache,
    known_programs: HashMap<Pubkey, String>,
    pattern_stats: HashMap<PdaPattern, u32>,
}

impl PdaAnalyzer {
    pub fn new() -> Self {
        let mut known_programs = HashMap::new();
        
        // System Programs
        known_programs.insert(solana_sdk::system_program::id(), "System Program".to_string());
        
        // SPL Programs
        if let Ok(spl_token_id) = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
            known_programs.insert(spl_token_id, "SPL Token".to_string());
        }
        if let Ok(ata_id) = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL") {
            known_programs.insert(ata_id, "SPL Associated Token Account".to_string());
        }
        
        // Metaplex Programs
        if let Ok(metadata_id) = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s") {
            known_programs.insert(metadata_id, "Metaplex Token Metadata".to_string());
        }
        if let Ok(candy_machine_id) = Pubkey::from_str("CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR") {
            known_programs.insert(candy_machine_id, "Metaplex Candy Machine".to_string());
        }
        if let Ok(auction_house_id) = Pubkey::from_str("hausS13jsjafwWwGqZTUQRmWyvyxn9EQpqMwV1PBBmk") {
            known_programs.insert(auction_house_id, "Metaplex Auction House".to_string());
        }
        
        // DeFi Programs
        if let Ok(serum_id) = Pubkey::from_str("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin") {
            known_programs.insert(serum_id, "Serum DEX".to_string());
        }
        if let Ok(raydium_id) = Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {
            known_programs.insert(raydium_id, "Raydium AMM".to_string());
        }
        if let Ok(marinade_id) = Pubkey::from_str("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD") {
            known_programs.insert(marinade_id, "Marinade Finance".to_string());
        }
        
        // Infrastructure Programs
        if let Ok(name_service_id) = Pubkey::from_str("namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX") {
            known_programs.insert(name_service_id, "Solana Name Service".to_string());
        }
        if let Ok(governance_id) = Pubkey::from_str("GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw") {
            known_programs.insert(governance_id, "SPL Governance".to_string());
        }

        Self {
            cache: HashMap::new(),
            known_programs,
            pattern_stats: HashMap::new(),
        }
    }

    /// Analyze a PDA to determine its seed derivation pattern with confidence scoring
    pub fn analyze_pda(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<PdaAnalysisResult>> {
        let start_time = std::time::Instant::now();
        
        // Try different PDA patterns in order of likelihood and specificity
        
        // 1. Try Associated Token Account pattern (most common on Solana)
        if let Some((pda_info, confidence)) = self.try_associated_token_account(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern: PdaPattern::AssociatedTokenAccount,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        // 2. Try Metaplex patterns (very common for NFTs)
        if let Some((pda_info, pattern, confidence)) = self.try_metaplex_patterns(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        // 3. Try common string singleton patterns
        if let Some((pda_info, confidence)) = self.try_string_singleton_patterns(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern: PdaPattern::StringSingleton,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        // 4. Try authority patterns
        if let Some((pda_info, pattern, confidence)) = self.try_authority_patterns(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        // 5. Try sequential patterns (numbered accounts)
        if let Some((pda_info, confidence)) = self.try_sequential_patterns(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern: PdaPattern::Sequential,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        // 6. Try complex multi-seed patterns
        if let Some((pda_info, confidence)) = self.try_complex_patterns(address, program_id)? {
            let result = PdaAnalysisResult {
                pda_info,
                pattern: PdaPattern::Complex,
                confidence,
                analysis_time_ms: start_time.elapsed().as_millis() as u64,
            };
            self.update_pattern_stats(&result.pattern);
            return Ok(Some(result));
        }

        Ok(None)
    }

    /// Try Associated Token Account pattern: [wallet, token_program, mint]
    fn try_associated_token_account(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, f64)>> {
        let ata_program_id = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")?;
        
        if *program_id != ata_program_id {
            return Ok(None);
        }

        // Common wallets and mints for testing
        let test_wallets = [
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh", 
            "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1",
            "8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC",
        ];
        
        let test_mints = [
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "So11111111111111111111111111111111111111112",   // SOL
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB",   // USDT
            "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh", // Example NFT
        ];

        let spl_token_program = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;

        for wallet_str in &test_wallets {
            if let Ok(wallet) = Pubkey::from_str(wallet_str) {
                for mint_str in &test_mints {
                    if let Ok(mint) = Pubkey::from_str(mint_str) {
                        let seeds = &[
                            wallet.as_ref(),
                            spl_token_program.as_ref(),
                            mint.as_ref(),
                        ];
                        
                        if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                            if derived_address == *address {
                                let pda_info = PdaInfo {
                                    address: *address,
                                    program_id: *program_id,
                                    seeds: vec![
                                        SeedValue::Pubkey(wallet),
                                        SeedValue::Pubkey(spl_token_program),
                                        SeedValue::Pubkey(mint),
                                    ],
                                    bump,
                                    first_seen_slot: None,
                                    first_seen_transaction: None,
                                };
                                return Ok(Some((pda_info, 0.98))); // High confidence for ATA pattern
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try Metaplex metadata patterns
    fn try_metaplex_patterns(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, PdaPattern, f64)>> {
        let metaplex_program_id = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")?;
        
        if *program_id != metaplex_program_id {
            return Ok(None);
        }

        let test_mints = [
            "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh",
            "8HYrKZBRZk9CgGfVv5u3r5G4W3dP2Qe2Y7rZRzMhQKkx",
            "So11111111111111111111111111111111111111112",
        ];

        for mint_str in &test_mints {
            if let Ok(mint) = Pubkey::from_str(mint_str) {
                // Try metadata pattern: ["metadata", program_id, mint]
                let metadata_seeds = &[
                    b"metadata",
                    program_id.as_ref(),
                    mint.as_ref(),
                ];
                
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(metadata_seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![
                                SeedValue::String("metadata".to_string()),
                                SeedValue::Pubkey(*program_id),
                                SeedValue::Pubkey(mint),
                            ],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, PdaPattern::MetaplexMetadata, 0.95)));
                    }
                }

                // Try master edition pattern: ["metadata", program_id, mint, "edition"]
                let edition_seeds = &[
                    b"metadata",
                    program_id.as_ref(),
                    mint.as_ref(),
                    b"edition",
                ];
                
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(edition_seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![
                                SeedValue::String("metadata".to_string()),
                                SeedValue::Pubkey(*program_id),
                                SeedValue::Pubkey(mint),
                                SeedValue::String("edition".to_string()),
                            ],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, PdaPattern::MetaplexMasterEdition, 0.93)));
                    }
                }

                // Try edition with number: ["metadata", program_id, master_mint, "edition", edition_number]
                for edition_num in 1..=10u64 {
                    let numbered_edition_seeds = &[
                        b"metadata",
                        program_id.as_ref(),
                        mint.as_ref(),
                        b"edition",
                        &edition_num.to_le_bytes(),
                    ];
                    
                    if let Some((derived_address, bump)) = Pubkey::try_find_program_address(numbered_edition_seeds, program_id) {
                        if derived_address == *address {
                            let pda_info = PdaInfo {
                                address: *address,
                                program_id: *program_id,
                                seeds: vec![
                                    SeedValue::String("metadata".to_string()),
                                    SeedValue::Pubkey(*program_id),
                                    SeedValue::Pubkey(mint),
                                    SeedValue::String("edition".to_string()),
                                    SeedValue::U64(edition_num),
                                ],
                                bump,
                                first_seen_slot: None,
                                first_seen_transaction: None,
                            };
                            return Ok(Some((pda_info, PdaPattern::MetaplexEdition, 0.90)));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try common string singleton patterns
    fn try_string_singleton_patterns(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, f64)>> {
        let common_strings = [
            "state", "config", "authority", "vault", "pool", "market",
            "escrow", "registry", "governance", "proposal", "metadata",
            "treasury", "rewards", "staking", "lending", "farming",
            "oracle", "price_feed", "liquidity", "swap", "mint_authority",
            "global", "settings", "admin", "owner", "controller",
        ];

        for string in &common_strings {
            let seeds = &[string.as_bytes()];
            if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                if derived_address == *address {
                    let confidence = match *string {
                        "state" | "config" | "authority" => 0.92,
                        "vault" | "pool" | "market" => 0.88,
                        _ => 0.85,
                    };
                    
                    let pda_info = PdaInfo {
                        address: *address,
                        program_id: *program_id,
                        seeds: vec![SeedValue::String(string.to_string())],
                        bump,
                        first_seen_slot: None,
                        first_seen_transaction: None,
                    };
                    return Ok(Some((pda_info, confidence)));
                }
            }
        }

        Ok(None)
    }

    /// Try authority patterns
    fn try_authority_patterns(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, PdaPattern, f64)>> {
        let test_authorities = [
            "11111111111111111111111111111112",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh",
        ];

        for auth_str in &test_authorities {
            if let Ok(authority) = Pubkey::from_str(auth_str) {
                // Try [authority] pattern
                let seeds = &[authority.as_ref()];
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![SeedValue::Pubkey(authority)],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, PdaPattern::StringAuthority, 0.87)));
                    }
                }

                // Try ["authority", authority] pattern
                let seeds = &[b"authority", authority.as_ref()];
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![
                                SeedValue::String("authority".to_string()),
                                SeedValue::Pubkey(authority),
                            ],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, PdaPattern::StringPubkey, 0.85)));
                    }
                }

                // Try [authority, nonce] patterns for DEX/AMM
                for nonce in 0..=10u64 {
                    let seeds = &[authority.as_ref(), &nonce.to_le_bytes()];
                    if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                        if derived_address == *address {
                            let pda_info = PdaInfo {
                                address: *address,
                                program_id: *program_id,
                                seeds: vec![
                                    SeedValue::Pubkey(authority),
                                    SeedValue::U64(nonce),
                                ],
                                bump,
                                first_seen_slot: None,
                                first_seen_transaction: None,
                            };
                            return Ok(Some((pda_info, PdaPattern::PubkeyU64, 0.83)));
                        }
                    }
                }

                // Try [authority, bump] patterns
                for bump_seed in 250..=255u8 {
                    let seeds = &[authority.as_ref(), &[bump_seed]];
                    if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                        if derived_address == *address {
                            let pda_info = PdaInfo {
                                address: *address,
                                program_id: *program_id,
                                seeds: vec![
                                    SeedValue::Pubkey(authority),
                                    SeedValue::U8(bump_seed),
                                ],
                                bump,
                                first_seen_slot: None,
                                first_seen_transaction: None,
                            };
                            return Ok(Some((pda_info, PdaPattern::PubkeyU8, 0.82)));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try sequential patterns (numbered accounts)
    fn try_sequential_patterns(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, f64)>> {
        let prefixes = ["account", "user", "pool", "vault", "market", "index", "item"];
        
        for prefix in &prefixes {
            for i in 0..=50u64 {
                // Try [prefix, number] as u64
                let seeds = &[prefix.as_bytes(), &i.to_le_bytes()];
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![
                                SeedValue::String(prefix.to_string()),
                                SeedValue::U64(i),
                            ],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, 0.80)));
                    }
                }

                // Try [prefix, number] as u32
                let seeds = &[prefix.as_bytes(), &(i as u32).to_le_bytes()];
                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                    if derived_address == *address {
                        let pda_info = PdaInfo {
                            address: *address,
                            program_id: *program_id,
                            seeds: vec![
                                SeedValue::String(prefix.to_string()),
                                SeedValue::U32(i as u32),
                            ],
                            bump,
                            first_seen_slot: None,
                            first_seen_transaction: None,
                        };
                        return Ok(Some((pda_info, 0.78)));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Try complex multi-seed patterns
    fn try_complex_patterns(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<(PdaInfo, f64)>> {
        let strings = ["governance", "proposal", "vote", "realm", "council"];
        let test_pubkeys = [
            "11111111111111111111111111111112",
            "DPiH3H3c7t47BMxqTxLsuPQpEC6Kne8GA9VXbxpnZxFE",
            "7gXKKGLQs2HpzrPTtBP7kkQ3LktDShQPE8VV9PYW9RSh",
        ];
        
        for s1 in &strings {
            for pubkey_str in &test_pubkeys {
                if let Ok(pubkey) = Pubkey::from_str(pubkey_str) {
                    for s2 in &strings {
                        if s1 != s2 {
                            for &num in &[0u32, 1u32, 2u32] {
                                // Try [string1, pubkey, string2, number]
                                let seeds = &[
                                    s1.as_bytes(),
                                    pubkey.as_ref(),
                                    s2.as_bytes(),
                                    &num.to_le_bytes(),
                                ];
                                if let Some((derived_address, bump)) = Pubkey::try_find_program_address(seeds, program_id) {
                                    if derived_address == *address {
                                        let pda_info = PdaInfo {
                                            address: *address,
                                            program_id: *program_id,
                                            seeds: vec![
                                                SeedValue::String(s1.to_string()),
                                                SeedValue::Pubkey(pubkey),
                                                SeedValue::String(s2.to_string()),
                                                SeedValue::U32(num),
                                            ],
                                            bump,
                                            first_seen_slot: None,
                                            first_seen_transaction: None,
                                        };
                                        return Ok(Some((pda_info, 0.75)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Derive a PDA with specific seeds
    pub fn derive_pda(&mut self, program_id: &Pubkey, seeds: &[SeedValue]) -> Result<PdaInfo> {
        let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
        let cache_key = (program_id.clone(), seed_bytes.clone());

        if let Some(cached_result) = self.cache.get(&cache_key) {
            if let Some(pda_info) = cached_result {
                return Ok(pda_info.clone());
            }
        }

        let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
        
        match Pubkey::try_find_program_address(&seed_refs, program_id) {
            Some((address, bump)) => {
                let pda_info = PdaInfo {
                    address,
                    program_id: *program_id,
                    seeds: seeds.to_vec(),
                    bump,
                    first_seen_slot: None,
                    first_seen_transaction: None,
                };
                
                self.cache.insert(cache_key, Some(pda_info.clone()));
                Ok(pda_info)
            }
            None => {
                self.cache.insert(cache_key, None);
                Err(crate::PdaAnalyzerError::PdaDerivationFailed("Invalid seeds".to_string()))
            }
        }
    }

    /// Get program name if known
    pub fn get_program_name(&self, program_id: &Pubkey) -> Option<&String> {
        self.known_programs.get(program_id)
    }

    /// Update pattern statistics
    fn update_pattern_stats(&mut self, pattern: &PdaPattern) {
        *self.pattern_stats.entry(pattern.clone()).or_insert(0) += 1;
    }

    /// Get pattern statistics
    pub fn get_pattern_stats(&self) -> &HashMap<PdaPattern, u32> {
        &self.pattern_stats
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        let hits = self.cache.values().filter(|v| v.is_some()).count();
        let total = self.cache.len();
        (hits, total)
    }

    /// Batch analyze multiple PDAs
    pub fn batch_analyze(&mut self, addresses: &[(Pubkey, Pubkey)]) -> Result<Vec<Option<PdaAnalysisResult>>> {
        let mut results = Vec::new();
        
        for (address, program_id) in addresses {
            results.push(self.analyze_pda(address, program_id)?);
        }
        
        Ok(results)
    }
}