use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use solana_pda_analyzer_core::{PdaDeriver, PdaAnalyzer, SeedValue};
use solana_sdk::pubkey::Pubkey;
use std::time::Duration;

fn benchmark_pda_derivation(c: &mut Criterion) {
    let mut group = c.benchmark_group("pda_derivation");
    
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    
    // Test single seed derivation
    group.bench_function("single_string_seed", |b| {
        let seeds = vec![SeedValue::String("metadata".to_string())];
        b.iter(|| {
            deriver.derive_pda(&program_id, &seeds).unwrap()
        });
    });
    
    // Test multiple seed derivation
    group.bench_function("multiple_seeds", |b| {
        let seeds = vec![
            SeedValue::String("metadata".to_string()),
            SeedValue::Pubkey(Pubkey::new_unique()),
            SeedValue::U64(12345),
        ];
        b.iter(|| {
            deriver.derive_pda(&program_id, &seeds).unwrap()
        });
    });
    
    // Test complex seed derivation
    group.bench_function("complex_seeds", |b| {
        let seeds = vec![
            SeedValue::String("complex_metadata".to_string()),
            SeedValue::Pubkey(Pubkey::new_unique()),
            SeedValue::U64(1234567890),
            SeedValue::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]),
            SeedValue::U32(42),
            SeedValue::U16(7),
            SeedValue::U8(255),
        ];
        b.iter(|| {
            deriver.derive_pda(&program_id, &seeds).unwrap()
        });
    });
    
    group.finish();
}

fn benchmark_pda_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("pda_verification");
    
    let deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![
        SeedValue::String("test".to_string()),
        SeedValue::U64(123),
    ];
    
    // Derive a valid PDA for testing
    let seed_bytes: Vec<Vec<u8>> = seeds.iter().map(|s| s.as_bytes()).collect();
    let seed_refs: Vec<&[u8]> = seed_bytes.iter().map(|s| s.as_slice()).collect();
    let (valid_address, _bump) = Pubkey::find_program_address(&seed_refs, &program_id);
    
    group.bench_function("valid_pda", |b| {
        b.iter(|| {
            deriver.verify_pda(&valid_address, &program_id, &seeds).unwrap()
        });
    });
    
    group.bench_function("invalid_pda", |b| {
        let invalid_address = Pubkey::new_unique();
        b.iter(|| {
            deriver.verify_pda(&invalid_address, &program_id, &seeds).unwrap()
        });
    });
    
    group.finish();
}

fn benchmark_pda_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("pda_analysis");
    
    let mut analyzer = PdaAnalyzer::new();
    let program_id = Pubkey::new_unique();
    
    group.bench_function("single_pda_analysis", |b| {
        let address = Pubkey::new_unique();
        b.iter(|| {
            analyzer.analyze_pda(&address, &program_id).unwrap()
        });
    });
    
    // Test batch analysis with varying sizes
    for batch_size in [1, 5, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch_analysis", batch_size),
            batch_size,
            |b, &size| {
                let addresses: Vec<(Pubkey, Pubkey)> = (0..size)
                    .map(|_| (Pubkey::new_unique(), program_id))
                    .collect();
                
                b.iter(|| {
                    analyzer.batch_analyze(&addresses).unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_seed_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("seed_value_operations");
    
    // Test different seed value conversions
    group.bench_function("string_to_bytes", |b| {
        let seed = SeedValue::String("test_string_seed_value".to_string());
        b.iter(|| seed.as_bytes());
    });
    
    group.bench_function("pubkey_to_bytes", |b| {
        let seed = SeedValue::Pubkey(Pubkey::new_unique());
        b.iter(|| seed.as_bytes());
    });
    
    group.bench_function("u64_to_bytes", |b| {
        let seed = SeedValue::U64(0x123456789ABCDEF0);
        b.iter(|| seed.as_bytes());
    });
    
    group.bench_function("bytes_to_bytes", |b| {
        let seed = SeedValue::Bytes(vec![0; 1000]); // Large byte array
        b.iter(|| seed.as_bytes());
    });
    
    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    let mut deriver = PdaDeriver::new();
    let program_id = Pubkey::new_unique();
    let seeds = vec![SeedValue::String("cached_seed".to_string())];
    
    // First derivation (should populate cache)
    deriver.derive_pda(&program_id, &seeds).unwrap();
    
    group.bench_function("cached_derivation", |b| {
        b.iter(|| {
            deriver.derive_pda(&program_id, &seeds).unwrap()
        });
    });
    
    group.bench_function("uncached_derivation", |b| {
        b.iter(|| {
            let unique_seeds = vec![SeedValue::String(format!("unique_{}", rand::random::<u64>()))];
            deriver.derive_pda(&program_id, &unique_seeds).unwrap()
        });
    });
    
    group.finish();
}

fn benchmark_pattern_detection(c: &mut Criterion) {
    use solana_pda_analyzer_analyzer::PatternDetector;
    use solana_pda_analyzer_core::PdaInfo;
    
    let mut group = c.benchmark_group("pattern_detection");
    
    let mut detector = PatternDetector::new();
    let program_id = Pubkey::new_unique();
    
    // Create test PDAs with various patterns
    let create_test_pdas = |count: usize| -> Vec<PdaInfo> {
        (0..count)
            .map(|i| PdaInfo {
                address: Pubkey::new_unique(),
                program_id,
                seeds: vec![
                    SeedValue::String(format!("seed_{}", i % 3)), // Create patterns
                    SeedValue::U64(i as u64),
                ],
                bump: 254,
                first_seen_slot: Some(12345),
                first_seen_transaction: Some("test".to_string()),
            })
            .collect()
    };
    
    // Test pattern detection with varying PDA counts
    for pda_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("detect_patterns", pda_count),
            pda_count,
            |b, &count| {
                let pdas = create_test_pdas(count);
                b.iter(|| {
                    detector.detect_patterns(&program_id, &pdas).unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_large_scale_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale");
    group.sample_size(10); // Fewer samples for large scale tests
    group.measurement_time(Duration::from_secs(10));
    
    let mut analyzer = PdaAnalyzer::new();
    let program_id = Pubkey::new_unique();
    
    // Test very large batch analysis
    group.bench_function("large_batch_1000", |b| {
        let addresses: Vec<(Pubkey, Pubkey)> = (0..1000)
            .map(|_| (Pubkey::new_unique(), program_id))
            .collect();
        
        b.iter(|| {
            analyzer.batch_analyze(&addresses).unwrap()
        });
    });
    
    // Test with many different programs
    group.bench_function("multi_program_analysis", |b| {
        let addresses: Vec<(Pubkey, Pubkey)> = (0..100)
            .map(|_| (Pubkey::new_unique(), Pubkey::new_unique())) // Different program each time
            .collect();
        
        b.iter(|| {
            analyzer.batch_analyze(&addresses).unwrap()
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_pda_derivation,
    benchmark_pda_verification,
    benchmark_pda_analysis,
    benchmark_seed_value_operations,
    benchmark_cache_performance,
    benchmark_pattern_detection,
    benchmark_large_scale_operations
);

criterion_main!(benches);