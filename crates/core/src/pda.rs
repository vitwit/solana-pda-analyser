use crate::{PdaAnalyzerError, Result, PdaInfo, SeedValue};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PdaDeriver {
    cache: HashMap<(Pubkey, Vec<Vec<u8>>), (Pubkey, u8)>,
}

impl PdaDeriver {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn derive_pda(&mut self, program_id: &Pubkey, seeds: &[SeedValue]) -> Result<PdaInfo> {
        let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
        
        let cache_key = (program_id.clone(), seed_bytes.clone());
        
        if let Some((address, bump)) = self.cache.get(&cache_key) {
            return Ok(PdaInfo {
                address: *address,
                program_id: *program_id,
                seeds: seeds.to_vec(),
                bump: *bump,
                first_seen_slot: None,
                first_seen_transaction: None,
            });
        }

        let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
        
        let (address, bump) = Pubkey::find_program_address(&seed_refs, program_id);
        
        self.cache.insert(cache_key, (address, bump));
        
        Ok(PdaInfo {
            address,
            program_id: *program_id,
            seeds: seeds.to_vec(),
            bump,
            first_seen_slot: None,
            first_seen_transaction: None,
        })
    }

    pub fn verify_pda(&self, address: &Pubkey, program_id: &Pubkey, seeds: &[SeedValue]) -> Result<bool> {
        let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
        let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
        
        let (derived_address, _bump) = Pubkey::find_program_address(&seed_refs, program_id);
        
        Ok(derived_address == *address)
    }

    pub fn generate_seed_combinations(&self, program_id: &Pubkey, target_address: &Pubkey) -> Vec<Vec<SeedValue>> {
        let mut combinations = Vec::new();
        
        // Common seed patterns to try
        let common_seeds = vec![
            vec![SeedValue::String("metadata".to_string())],
            vec![SeedValue::String("vault".to_string())],
            vec![SeedValue::String("authority".to_string())],
            vec![SeedValue::String("config".to_string())],
            vec![SeedValue::String("state".to_string())],
            vec![SeedValue::String("pool".to_string())],
            vec![SeedValue::String("mint".to_string())],
            vec![SeedValue::String("escrow".to_string())],
            vec![SeedValue::String("treasury".to_string())],
        ];
        
        // Try single seed patterns
        for seeds in common_seeds {
            if self.test_seed_combination(program_id, target_address, &seeds) {
                combinations.push(seeds);
            }
        }
        
        // Try with common pubkey combinations
        let system_program = solana_sdk::system_program::id();
        let token_program = spl_token::id();
        
        for base_seed in &["metadata", "vault", "authority", "config"] {
            let seeds = vec![
                SeedValue::String(base_seed.to_string()),
                SeedValue::Pubkey(system_program),
            ];
            if self.test_seed_combination(program_id, target_address, &seeds) {
                combinations.push(seeds);
            }
            
            let seeds = vec![
                SeedValue::String(base_seed.to_string()),
                SeedValue::Pubkey(token_program),
            ];
            if self.test_seed_combination(program_id, target_address, &seeds) {
                combinations.push(seeds);
            }
        }
        
        // Try with numeric patterns
        for i in 0..10u64 {
            let seeds = vec![
                SeedValue::String("index".to_string()),
                SeedValue::U64(i),
            ];
            if self.test_seed_combination(program_id, target_address, &seeds) {
                combinations.push(seeds);
            }
        }
        
        combinations
    }
    
    fn test_seed_combination(&self, program_id: &Pubkey, target_address: &Pubkey, seeds: &[SeedValue]) -> bool {
        match self.verify_pda(target_address, program_id, seeds) {
            Ok(valid) => valid,
            Err(_) => false,
        }
    }
}

impl Default for PdaDeriver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PdaAnalyzer {
    deriver: PdaDeriver,
}

impl PdaAnalyzer {
    pub fn new() -> Self {
        Self {
            deriver: PdaDeriver::new(),
        }
    }

    pub fn analyze_pda(&mut self, address: &Pubkey, program_id: &Pubkey) -> Result<Option<PdaInfo>> {
        let combinations = self.deriver.generate_seed_combinations(program_id, address);
        
        for seeds in combinations {
            if let Ok(pda_info) = self.deriver.derive_pda(program_id, &seeds) {
                if pda_info.address == *address {
                    return Ok(Some(pda_info));
                }
            }
        }
        
        Ok(None)
    }

    pub fn batch_analyze(&mut self, addresses: &[(Pubkey, Pubkey)]) -> Result<Vec<Option<PdaInfo>>> {
        let mut results = Vec::new();
        
        for (address, program_id) in addresses {
            results.push(self.analyze_pda(address, program_id)?);
        }
        
        Ok(results)
    }
}

impl Default for PdaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    
    #[test]
    fn test_pda_derivation() {
        let mut deriver = PdaDeriver::new();
        let program_id = Pubkey::new_unique();
        let seeds = vec![
            SeedValue::String("test".to_string()),
            SeedValue::U64(123),
        ];
        
        let result = deriver.derive_pda(&program_id, &seeds);
        assert!(result.is_ok());
        
        let pda_info = result.unwrap();
        assert_eq!(pda_info.program_id, program_id);
        assert_eq!(pda_info.seeds, seeds);
    }
    
    #[test]
    fn test_pda_verification() {
        let deriver = PdaDeriver::new();
        let program_id = Pubkey::new_unique();
        let seeds = vec![SeedValue::String("test".to_string())];
        
        let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
        let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
        let (address, _bump) = Pubkey::find_program_address(&seed_refs, &program_id);
        
        let result = deriver.verify_pda(&address, &program_id, &seeds);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}