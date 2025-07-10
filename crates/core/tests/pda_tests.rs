use solana_pda_analyzer_core::{PdaDeriver, PdaAnalyzer, SeedValue, PdaInfo};
use solana_sdk::pubkey::Pubkey;

#[test]
fn test_pda_derivation_with_string_seed() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![SeedValue::String("metadata".to_string())];
    
    let result = deriver.derive_pda(&program_id, &seeds);
    assert!(result.is_ok());
    
    let pda_info = result.unwrap();
    assert_eq!(pda_info.program_id, program_id);
    assert_eq!(pda_info.seeds, seeds);
    assert!(pda_info.bump < 256);
}

#[test]
fn test_pda_derivation_with_multiple_seeds() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![
        SeedValue::String("prefix".to_string()),
        SeedValue::Pubkey(Pubkey::new_unique()),
        SeedValue::U64(12345),
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
    
    // Derive the correct PDA
    let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
    let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
    let (correct_address, _bump) = Pubkey::find_program_address(&seed_refs, &program_id);
    
    // Verify it matches
    let result = deriver.verify_pda(&correct_address, &program_id, &seeds);
    assert!(result.is_ok());
    assert!(result.unwrap());
    
    // Test with wrong address
    let wrong_address = Pubkey::new_unique();
    let result = deriver.verify_pda(&wrong_address, &program_id, &seeds);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_seed_value_as_bytes() {
    // Test string seed
    let string_seed = SeedValue::String("hello".to_string());
    assert_eq!(string_seed.as_bytes(), b"hello".to_vec());
    
    // Test bytes seed
    let bytes_seed = SeedValue::Bytes(vec![1, 2, 3, 4]);
    assert_eq!(bytes_seed.as_bytes(), vec![1, 2, 3, 4]);
    
    // Test pubkey seed
    let pubkey = Pubkey::new_unique();
    let pubkey_seed = SeedValue::Pubkey(pubkey);
    assert_eq!(pubkey_seed.as_bytes(), pubkey.as_ref().to_vec());
    
    // Test u64 seed
    let u64_seed = SeedValue::U64(0x123456789ABCDEF0);
    assert_eq!(u64_seed.as_bytes(), 0x123456789ABCDEF0u64.to_le_bytes().to_vec());
    
    // Test u32 seed
    let u32_seed = SeedValue::U32(0x12345678);
    assert_eq!(u32_seed.as_bytes(), 0x12345678u32.to_le_bytes().to_vec());
    
    // Test u16 seed
    let u16_seed = SeedValue::U16(0x1234);
    assert_eq!(u16_seed.as_bytes(), 0x1234u16.to_le_bytes().to_vec());
    
    // Test u8 seed
    let u8_seed = SeedValue::U8(0x12);
    assert_eq!(u8_seed.as_bytes(), vec![0x12]);
}

#[test]
fn test_seed_value_types() {
    assert_eq!(SeedValue::String("test".to_string()).seed_type(), "string");
    assert_eq!(SeedValue::Bytes(vec![]).seed_type(), "bytes");
    assert_eq!(SeedValue::Pubkey(Pubkey::new_unique()).seed_type(), "pubkey");
    assert_eq!(SeedValue::U64(0).seed_type(), "u64");
    assert_eq!(SeedValue::U32(0).seed_type(), "u32");
    assert_eq!(SeedValue::U16(0).seed_type(), "u16");
    assert_eq!(SeedValue::U8(0).seed_type(), "u8");
}

#[test]
fn test_pda_analyzer() {
    let mut analyzer = PdaAnalyzer::new();
    let program_id = Pubkey::new_unique();
    let test_address = Pubkey::new_unique();
    
    // Test analysis of a random address (should not find seeds)
    let result = analyzer.analyze_pda(&test_address, &program_id);
    assert!(result.is_ok());
    // For a random address, we likely won't find a match
    // This test mainly ensures the function doesn't panic
}

#[test]
fn test_batch_analysis() {
    let mut analyzer = PdaAnalyzer::new();
    let program_id = Pubkey::new_unique();
    
    let addresses = vec![
        (Pubkey::new_unique(), program_id),
        (Pubkey::new_unique(), program_id),
        (Pubkey::new_unique(), program_id),
    ];
    
    let result = analyzer.batch_analyze(&addresses);
    assert!(result.is_ok());
    
    let results = result.unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_pda_caching() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![SeedValue::String("cache_test".to_string())];
    
    // First derivation
    let result1 = deriver.derive_pda(&program_id, &seeds);
    assert!(result1.is_ok());
    
    // Second derivation (should use cache)
    let result2 = deriver.derive_pda(&program_id, &seeds);
    assert!(result2.is_ok());
    
    // Results should be identical
    let pda1 = result1.unwrap();
    let pda2 = result2.unwrap();
    assert_eq!(pda1.address, pda2.address);
    assert_eq!(pda1.bump, pda2.bump);
}

#[test]
fn test_known_patterns() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    
    // Test with common seed patterns
    let test_patterns = vec![
        vec![SeedValue::String("metadata".to_string())],
        vec![SeedValue::String("vault".to_string())],
        vec![SeedValue::String("authority".to_string())],
        vec![SeedValue::String("config".to_string())],
    ];
    
    for seeds in test_patterns {
        let result = deriver.derive_pda(&program_id, &seeds);
        assert!(result.is_ok(), "Failed to derive PDA for seeds: {:?}", seeds);
        
        let pda_info = result.unwrap();
        assert_eq!(pda_info.program_id, program_id);
        assert_eq!(pda_info.seeds, seeds);
    }
}

#[test]
fn test_empty_seeds() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![];
    
    let result = deriver.derive_pda(&program_id, &seeds);
    assert!(result.is_ok());
    
    let pda_info = result.unwrap();
    assert_eq!(pda_info.program_id, program_id);
    assert_eq!(pda_info.seeds, seeds);
}

#[test]
fn test_large_seed_data() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    
    // Test with large byte array
    let large_data = vec![0u8; 1000];
    let seeds = vec![SeedValue::Bytes(large_data.clone())];
    
    let result = deriver.derive_pda(&program_id, &seeds);
    assert!(result.is_ok());
    
    let pda_info = result.unwrap();
    assert_eq!(pda_info.seeds[0].as_bytes(), large_data);
}

#[test] 
fn test_mixed_seed_types() {
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let user_pubkey = Pubkey::new_unique();
    
    let seeds = vec![
        SeedValue::String("user_account".to_string()),
        SeedValue::Pubkey(user_pubkey),
        SeedValue::U64(1234567890),
        SeedValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
        SeedValue::U32(42),
        SeedValue::U16(7),
        SeedValue::U8(255),
    ];
    
    let result = deriver.derive_pda(&program_id, &seeds);
    assert!(result.is_ok());
    
    let pda_info = result.unwrap();
    assert_eq!(pda_info.seeds.len(), 7);
    assert_eq!(pda_info.program_id, program_id);
}