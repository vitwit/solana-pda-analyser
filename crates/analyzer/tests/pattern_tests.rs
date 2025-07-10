use solana_pda_analyzer_analyzer::{PatternDetector, PatternRegistry, DetectedPattern};
use solana_pda_analyzer_core::{PdaInfo, SeedValue, PdaPattern, SeedTemplate};
use solana_sdk::pubkey::Pubkey;
use uuid::Uuid;

fn create_test_pda(program_id: Pubkey, seeds: Vec<SeedValue>, address: Option<Pubkey>) -> PdaInfo {
    PdaInfo {
        address: address.unwrap_or_else(|| Pubkey::new_unique()),
        program_id,
        seeds,
        bump: 254,
        first_seen_slot: Some(12345),
        first_seen_transaction: Some("test_signature".to_string()),
    }
}

#[test]
fn test_pattern_detector_creation() {
    let detector = PatternDetector::new();
    assert_eq!(detector.known_patterns.len(), 0);
    assert_eq!(detector.detected_patterns.len(), 0);
}

#[test]
fn test_add_known_pattern() {
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    let pattern = PdaPattern {
        id: Uuid::new_v4(),
        program_id,
        pattern_name: "Test Pattern".to_string(),
        seeds_template: vec![
            SeedTemplate {
                name: "prefix".to_string(),
                seed_type: "string".to_string(),
                description: Some("Prefix seed".to_string()),
                is_variable: false,
            },
        ],
        description: Some("A test pattern".to_string()),
    };
    
    detector.add_known_pattern(pattern.clone());
    
    assert_eq!(detector.known_patterns.len(), 1);
    assert!(detector.known_patterns.contains_key(&program_id));
    assert_eq!(detector.known_patterns[&program_id].len(), 1);
}

#[test]
fn test_pattern_detection_with_single_pattern() {
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create PDAs with the same pattern
    let pdas = vec![
        create_test_pda(program_id, vec![SeedValue::String("metadata".to_string())], None),
        create_test_pda(program_id, vec![SeedValue::String("config".to_string())], None),
        create_test_pda(program_id, vec![SeedValue::String("vault".to_string())], None),
    ];
    
    let patterns = detector.detect_patterns(&program_id, &pdas).unwrap();
    
    // Should detect one pattern (all string seeds)
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0].pattern_signature, "string");
    assert_eq!(patterns[0].frequency, 3);
    assert_eq!(patterns[0].program_id, program_id);
}

#[test]
fn test_pattern_detection_with_multiple_patterns() {
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create PDAs with different patterns
    let pdas = vec![
        // String pattern (appears 3 times)
        create_test_pda(program_id, vec![SeedValue::String("metadata".to_string())], None),
        create_test_pda(program_id, vec![SeedValue::String("config".to_string())], None),
        create_test_pda(program_id, vec![SeedValue::String("vault".to_string())], None),
        
        // String + U64 pattern (appears 2 times)
        create_test_pda(program_id, vec![
            SeedValue::String("user".to_string()),
            SeedValue::U64(123)
        ], None),
        create_test_pda(program_id, vec![
            SeedValue::String("account".to_string()),
            SeedValue::U64(456)
        ], None),
    ];
    
    let patterns = detector.detect_patterns(&program_id, &pdas).unwrap();
    
    // Should detect two patterns
    assert_eq!(patterns.len(), 2);
    
    // Patterns should be sorted by frequency (descending)
    assert!(patterns[0].frequency >= patterns[1].frequency);
    
    // Check pattern signatures
    let signatures: Vec<String> = patterns.iter().map(|p| p.pattern_signature.clone()).collect();
    assert!(signatures.contains(&"string".to_string()));
    assert!(signatures.contains(&"string:u64".to_string()));
}

#[test]
fn test_pattern_confidence_calculation() {
    let detector = PatternDetector::new();
    
    // Test with 50% frequency
    let confidence = detector.calculate_confidence(5, 10);
    assert_eq!(confidence, 50.0);
    
    // Test with 100% frequency (should cap at 95%)
    let confidence = detector.calculate_confidence(10, 10);
    assert_eq!(confidence, 95.0);
    
    // Test with >100% frequency (should cap at 95%)
    let confidence = detector.calculate_confidence(15, 10);
    assert_eq!(confidence, 95.0);
    
    // Test with 0 total
    let confidence = detector.calculate_confidence(5, 0);
    assert_eq!(confidence, 0.0);
}

#[test]
fn test_pattern_matching() {
    let detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create a pattern template
    let template = vec![
        SeedTemplate {
            name: "prefix".to_string(),
            seed_type: "string".to_string(),
            description: None,
            is_variable: false,
        },
        SeedTemplate {
            name: "id".to_string(),
            seed_type: "u64".to_string(),
            description: None,
            is_variable: true,
        },
    ];
    
    // Test matching seeds
    let matching_seeds = vec![
        SeedValue::String("test".to_string()),
        SeedValue::U64(12345),
    ];
    
    let match_score = detector.calculate_pattern_match(&matching_seeds, &template);
    assert_eq!(match_score, Some(100.0));
    
    // Test non-matching seeds
    let non_matching_seeds = vec![
        SeedValue::String("test".to_string()),
        SeedValue::String("wrong_type".to_string()),
    ];
    
    let match_score = detector.calculate_pattern_match(&non_matching_seeds, &template);
    assert_eq!(match_score, Some(50.0));
    
    // Test different length
    let wrong_length_seeds = vec![SeedValue::String("test".to_string())];
    let match_score = detector.calculate_pattern_match(&wrong_length_seeds, &template);
    assert_eq!(match_score, None);
}

#[test]
fn test_pattern_suggestions() {
    let detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create PDAs with consistent patterns
    let pdas = vec![
        create_test_pda(program_id, vec![
            SeedValue::String("metadata".to_string()),
            SeedValue::U64(1),
        ], None),
        create_test_pda(program_id, vec![
            SeedValue::String("config".to_string()),
            SeedValue::U64(2),
        ], None),
        create_test_pda(program_id, vec![
            SeedValue::String("vault".to_string()),
            SeedValue::U64(3),
        ], None),
    ];
    
    let suggestions = detector.generate_pattern_suggestions(&program_id, &pdas);
    
    // Should have suggestions for both seed positions
    assert_eq!(suggestions.len(), 2);
    
    // First position should suggest "string"
    let first_suggestion = suggestions.iter().find(|s| s.seed_index == 0).unwrap();
    assert_eq!(first_suggestion.suggested_type, "string");
    assert_eq!(first_suggestion.frequency, 3);
    assert_eq!(first_suggestion.confidence, 100.0);
    
    // Second position should suggest "u64"
    let second_suggestion = suggestions.iter().find(|s| s.seed_index == 1).unwrap();
    assert_eq!(second_suggestion.suggested_type, "u64");
    assert_eq!(second_suggestion.frequency, 3);
    assert_eq!(second_suggestion.confidence, 100.0);
}

#[test]
fn test_pattern_registry() {
    let mut registry = PatternRegistry::new();
    
    // Should have some builtin patterns
    assert!(registry.builtin_patterns.len() > 0);
    
    // Test adding a custom pattern
    let program_id = Pubkey::new_unique();
    let custom_pattern = PdaPattern {
        id: Uuid::new_v4(),
        program_id,
        pattern_name: "Custom Pattern".to_string(),
        seeds_template: vec![
            SeedTemplate {
                name: "custom".to_string(),
                seed_type: "string".to_string(),
                description: Some("Custom seed".to_string()),
                is_variable: true,
            },
        ],
        description: Some("A custom pattern".to_string()),
    };
    
    registry.add_pattern(custom_pattern.clone());
    
    // Test pattern matching
    let test_pda = create_test_pda(program_id, vec![
        SeedValue::String("custom_value".to_string())
    ], None);
    
    let matches = registry.match_pda(&test_pda);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].pattern_name, "Custom Pattern");
    assert_eq!(matches[0].match_score, 100.0);
}

#[test]
fn test_builtin_patterns() {
    let registry = PatternRegistry::new();
    
    // Test that builtin patterns are loaded
    assert!(registry.builtin_patterns.len() >= 2); // Should have at least SPL Token and Metaplex patterns
    
    // Find SPL Token pattern
    let spl_token_patterns = registry.builtin_patterns.values()
        .flatten()
        .find(|p| p.pattern_name.contains("Token"));
    
    assert!(spl_token_patterns.is_some());
    
    // Find Metaplex pattern
    let metaplex_patterns = registry.builtin_patterns.values()
        .flatten()
        .find(|p| p.pattern_name.contains("Metadata"));
    
    assert!(metaplex_patterns.is_some());
}

#[test]
fn test_complex_seed_patterns() {
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create PDAs with complex seed patterns
    let user_pubkey = Pubkey::new_unique();
    let mint_pubkey = Pubkey::new_unique();
    
    let pdas = vec![
        // Authority pattern
        create_test_pda(program_id, vec![
            SeedValue::String("authority".to_string()),
            SeedValue::Pubkey(user_pubkey),
            SeedValue::U64(1),
        ], None),
        
        // Authority pattern (repeated)
        create_test_pda(program_id, vec![
            SeedValue::String("authority".to_string()),
            SeedValue::Pubkey(mint_pubkey),
            SeedValue::U64(2),
        ], None),
        
        // Vault pattern
        create_test_pda(program_id, vec![
            SeedValue::String("vault".to_string()),
            SeedValue::Bytes(vec![1, 2, 3, 4]),
        ], None),
    ];
    
    let patterns = detector.detect_patterns(&program_id, &pdas).unwrap();
    
    // Should detect two patterns
    assert_eq!(patterns.len(), 2);
    
    // Find the authority pattern
    let authority_pattern = patterns.iter()
        .find(|p| p.pattern_signature == "string:pubkey:u64")
        .unwrap();
    assert_eq!(authority_pattern.frequency, 2);
    
    // Find the vault pattern
    let vault_pattern = patterns.iter()
        .find(|p| p.pattern_signature == "string:bytes")
        .unwrap();
    assert_eq!(vault_pattern.frequency, 1);
}

#[test]
fn test_empty_patterns() {
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Test with no PDAs
    let patterns = detector.detect_patterns(&program_id, &[]).unwrap();
    assert_eq!(patterns.len(), 0);
    
    // Test with single PDA (below threshold)
    let single_pda = vec![
        create_test_pda(program_id, vec![SeedValue::String("test".to_string())], None),
    ];
    
    let patterns = detector.detect_patterns(&program_id, &single_pda).unwrap();
    assert_eq!(patterns.len(), 0); // Below minimum frequency threshold
}

#[test]
fn test_pattern_signature_creation() {
    let detector = PatternDetector::new();
    
    // Test empty seeds
    let empty_signature = detector.create_pattern_signature(&[]);
    assert_eq!(empty_signature, "empty");
    
    // Test single seed
    let single_signature = detector.create_pattern_signature(&[
        SeedValue::String("test".to_string())
    ]);
    assert_eq!(single_signature, "string");
    
    // Test multiple seeds
    let multi_signature = detector.create_pattern_signature(&[
        SeedValue::String("prefix".to_string()),
        SeedValue::Pubkey(Pubkey::new_unique()),
        SeedValue::U64(123),
        SeedValue::Bytes(vec![1, 2, 3]),
    ]);
    assert_eq!(multi_signature, "string:pubkey:u64:bytes");
}

#[test]
fn test_seed_template_creation() {
    let detector = PatternDetector::new();
    
    let seeds = vec![
        SeedValue::String("test".to_string()),
        SeedValue::U64(123),
        SeedValue::Pubkey(Pubkey::new_unique()),
    ];
    
    let template = detector.create_seed_template(&seeds);
    
    assert_eq!(template.len(), 3);
    assert_eq!(template[0].name, "seed_0");
    assert_eq!(template[0].seed_type, "string");
    assert_eq!(template[1].name, "seed_1");
    assert_eq!(template[1].seed_type, "u64");
    assert_eq!(template[2].name, "seed_2");
    assert_eq!(template[2].seed_type, "pubkey");
    
    // All should be variable by default
    assert!(template.iter().all(|t| t.is_variable));
}