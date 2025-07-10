use solana_pda_analyzer_core::{
    PdaAnalyzerError, Result, SeedValue, PdaInfo, PdaPattern, SeedTemplate,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PatternDetector {
    known_patterns: HashMap<Pubkey, Vec<PdaPattern>>,
    detected_patterns: HashMap<Pubkey, Vec<DetectedPattern>>,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            known_patterns: HashMap::new(),
            detected_patterns: HashMap::new(),
        }
    }

    pub fn add_known_pattern(&mut self, pattern: PdaPattern) {
        self.known_patterns
            .entry(pattern.program_id)
            .or_insert_with(Vec::new)
            .push(pattern);
    }

    pub fn detect_patterns(&mut self, program_id: &Pubkey, pdas: &[PdaInfo]) -> Result<Vec<DetectedPattern>> {
        let mut pattern_frequency = HashMap::new();
        let mut seed_combinations = HashMap::new();
        
        // Analyze seed patterns
        for pda in pdas {
            if pda.program_id == *program_id {
                let pattern_signature = self.create_pattern_signature(&pda.seeds);
                let entry = pattern_frequency.entry(pattern_signature.clone()).or_insert(0);
                *entry += 1;
                
                seed_combinations
                    .entry(pattern_signature)
                    .or_insert_with(Vec::new)
                    .push(pda.clone());
            }
        }
        
        let mut detected_patterns = Vec::new();
        
        // Convert frequent patterns to DetectedPattern
        for (pattern_sig, frequency) in pattern_frequency {
            if frequency >= 2 { // Only consider patterns that appear at least twice
                let examples = seed_combinations.get(&pattern_sig).unwrap();
                let seed_template = self.create_seed_template(&examples[0].seeds);
                
                let pattern = DetectedPattern {
                    id: Uuid::new_v4(),
                    program_id: *program_id,
                    pattern_signature: pattern_sig.clone(),
                    seed_template,
                    frequency,
                    confidence: self.calculate_confidence(frequency, pdas.len()),
                    examples: examples.iter().take(5).map(|p| p.address).collect(),
                };
                
                detected_patterns.push(pattern);
            }
        }
        
        // Sort by frequency and confidence
        detected_patterns.sort_by(|a, b| {
            b.frequency.cmp(&a.frequency)
                .then_with(|| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal))
        });
        
        // Cache detected patterns
        self.detected_patterns.insert(*program_id, detected_patterns.clone());
        
        Ok(detected_patterns)
    }

    pub fn match_against_known_patterns(&self, pda: &PdaInfo) -> Vec<PatternMatch> {
        let mut matches = Vec::new();
        
        if let Some(patterns) = self.known_patterns.get(&pda.program_id) {
            for pattern in patterns {
                if let Some(match_score) = self.calculate_pattern_match(&pda.seeds, &pattern.seeds_template) {
                    matches.push(PatternMatch {
                        pattern_id: pattern.id,
                        pattern_name: pattern.pattern_name.clone(),
                        match_score,
                        matched_seeds: pda.seeds.clone(),
                    });
                }
            }
        }
        
        matches.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap_or(std::cmp::Ordering::Equal));
        matches
    }

    pub fn generate_pattern_suggestions(&self, program_id: &Pubkey, pdas: &[PdaInfo]) -> Vec<PatternSuggestion> {
        let mut suggestions = Vec::new();
        
        // Analyze common seed types
        let mut seed_type_frequency = HashMap::new();
        for pda in pdas {
            if pda.program_id == *program_id {
                for (index, seed) in pda.seeds.iter().enumerate() {
                    let key = (index, seed.seed_type());
                    *seed_type_frequency.entry(key).or_insert(0) += 1;
                }
            }
        }
        
        // Generate suggestions based on common patterns
        if seed_type_frequency.len() > 0 {
            let max_seed_count = seed_type_frequency.keys().map(|(index, _)| index).max().unwrap_or(&0) + 1;
            
            for seed_index in 0..max_seed_count {
                let mut type_counts = HashMap::new();
                for ((index, seed_type), count) in &seed_type_frequency {
                    if *index == seed_index {
                        *type_counts.entry(seed_type.clone()).or_insert(0) += count;
                    }
                }
                
                if let Some((most_common_type, _)) = type_counts.iter().max_by_key(|(_, count)| *count) {
                    suggestions.push(PatternSuggestion {
                        seed_index,
                        suggested_type: most_common_type.clone(),
                        frequency: *type_counts.get(*most_common_type).unwrap_or(&0),
                        confidence: self.calculate_type_confidence(&type_counts),
                    });
                }
            }
        }
        
        suggestions
    }

    fn create_pattern_signature(&self, seeds: &[SeedValue]) -> String {
        if seeds.is_empty() {
            return "empty".to_string();
        }
        
        seeds.iter()
            .map(|seed| seed.seed_type())
            .collect::<Vec<_>>()
            .join(":")
    }

    fn create_seed_template(&self, seeds: &[SeedValue]) -> Vec<SeedTemplate> {
        seeds.iter()
            .enumerate()
            .map(|(index, seed)| SeedTemplate {
                name: format!("seed_{}", index),
                seed_type: seed.seed_type().to_string(),
                description: Some(format!("Seed parameter {}", index)),
                is_variable: true,
            })
            .collect()
    }

    fn calculate_confidence(&self, frequency: usize, total_pdas: usize) -> f64 {
        if total_pdas == 0 {
            return 0.0;
        }
        
        let ratio = frequency as f64 / total_pdas as f64;
        // Confidence increases with frequency ratio, but caps at reasonable levels
        (ratio * 100.0).min(95.0)
    }

    fn calculate_pattern_match(&self, seeds: &[SeedValue], template: &[SeedTemplate]) -> Option<f64> {
        if seeds.len() != template.len() {
            return None;
        }
        
        let mut matches = 0;
        for (seed, template_seed) in seeds.iter().zip(template.iter()) {
            if seed.seed_type() == template_seed.seed_type {
                matches += 1;
            }
        }
        
        Some(matches as f64 / seeds.len() as f64 * 100.0)
    }

    fn calculate_type_confidence(&self, type_counts: &HashMap<String, usize>) -> f64 {
        let total: usize = type_counts.values().sum();
        let max_count = type_counts.values().max().unwrap_or(&0);
        
        if total == 0 {
            return 0.0;
        }
        
        (*max_count as f64 / total as f64) * 100.0
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub id: Uuid,
    pub program_id: Pubkey,
    pub pattern_signature: String,
    pub seed_template: Vec<SeedTemplate>,
    pub frequency: usize,
    pub confidence: f64,
    pub examples: Vec<Pubkey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: Uuid,
    pub pattern_name: String,
    pub match_score: f64,
    pub matched_seeds: Vec<SeedValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSuggestion {
    pub seed_index: usize,
    pub suggested_type: String,
    pub frequency: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct PatternRegistry {
    detector: PatternDetector,
    builtin_patterns: HashMap<Pubkey, Vec<PdaPattern>>,
}

impl PatternRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            detector: PatternDetector::new(),
            builtin_patterns: HashMap::new(),
        };
        
        registry.register_builtin_patterns();
        registry
    }

    pub fn register_builtin_patterns(&mut self) {
        // Register common known patterns for popular programs
        
        // SPL Token patterns
        if let Ok(spl_token_id) = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>() {
            self.add_pattern(PdaPattern {
                id: Uuid::new_v4(),
                program_id: spl_token_id,
                pattern_name: "Token Account".to_string(),
                seeds_template: vec![
                    SeedTemplate {
                        name: "owner".to_string(),
                        seed_type: "pubkey".to_string(),
                        description: Some("Token account owner".to_string()),
                        is_variable: true,
                    },
                    SeedTemplate {
                        name: "mint".to_string(),
                        seed_type: "pubkey".to_string(),
                        description: Some("Token mint".to_string()),
                        is_variable: true,
                    },
                ],
                description: Some("Standard SPL token associated account".to_string()),
            });
        }
        
        // Metaplex patterns
        if let Ok(metaplex_id) = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".parse::<Pubkey>() {
            self.add_pattern(PdaPattern {
                id: Uuid::new_v4(),
                program_id: metaplex_id,
                pattern_name: "Metadata Account".to_string(),
                seeds_template: vec![
                    SeedTemplate {
                        name: "prefix".to_string(),
                        seed_type: "string".to_string(),
                        description: Some("Metadata prefix".to_string()),
                        is_variable: false,
                    },
                    SeedTemplate {
                        name: "program_id".to_string(),
                        seed_type: "pubkey".to_string(),
                        description: Some("Metadata program ID".to_string()),
                        is_variable: false,
                    },
                    SeedTemplate {
                        name: "mint".to_string(),
                        seed_type: "pubkey".to_string(),
                        description: Some("NFT mint".to_string()),
                        is_variable: true,
                    },
                ],
                description: Some("NFT metadata account".to_string()),
            });
        }
    }

    pub fn add_pattern(&mut self, pattern: PdaPattern) {
        self.detector.add_known_pattern(pattern.clone());
        self.builtin_patterns
            .entry(pattern.program_id)
            .or_insert_with(Vec::new)
            .push(pattern);
    }

    pub fn detect_patterns(&mut self, program_id: &Pubkey, pdas: &[PdaInfo]) -> Result<Vec<DetectedPattern>> {
        self.detector.detect_patterns(program_id, pdas)
    }

    pub fn match_pda(&self, pda: &PdaInfo) -> Vec<PatternMatch> {
        self.detector.match_against_known_patterns(pda)
    }

    pub fn get_suggestions(&self, program_id: &Pubkey, pdas: &[PdaInfo]) -> Vec<PatternSuggestion> {
        self.detector.generate_pattern_suggestions(program_id, pdas)
    }
}

impl Default for PatternRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_detector_creation() {
        let detector = PatternDetector::new();
        assert_eq!(detector.known_patterns.len(), 0);
    }
    
    #[test]
    fn test_pattern_signature() {
        let detector = PatternDetector::new();
        let seeds = vec![
            SeedValue::String("test".to_string()),
            SeedValue::U64(123),
        ];
        let signature = detector.create_pattern_signature(&seeds);
        assert_eq!(signature, "string:u64");
    }
    
    #[test]
    fn test_pattern_registry_creation() {
        let registry = PatternRegistry::new();
        assert!(registry.builtin_patterns.len() > 0);
    }
    
    #[test]
    fn test_confidence_calculation() {
        let detector = PatternDetector::new();
        let confidence = detector.calculate_confidence(50, 100);
        assert_eq!(confidence, 50.0);
        
        let confidence = detector.calculate_confidence(200, 100);
        assert_eq!(confidence, 95.0); // Capped at 95%
    }
}