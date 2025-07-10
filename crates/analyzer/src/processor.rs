use solana_pda_analyzer_core::{
    PdaAnalyzerError, Result, TransactionAnalysis, PdaAnalyzer, TransactionAnalyzer,
    PdaInfo, AccountState,
};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone)]
pub struct BatchProcessor {
    pda_analyzer: Arc<RwLock<PdaAnalyzer>>,
    transaction_analyzer: Arc<RwLock<TransactionAnalyzer>>,
    stats: Arc<RwLock<ProcessingStats>>,
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            pda_analyzer: Arc::new(RwLock::new(PdaAnalyzer::new())),
            transaction_analyzer: Arc::new(RwLock::new(TransactionAnalyzer::new())),
            stats: Arc::new(RwLock::new(ProcessingStats::new())),
        }
    }

    pub async fn process_transaction(
        &self,
        encoded_transaction: EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<TransactionAnalysis> {
        let signature = encoded_transaction.transaction.signatures[0].clone();
        debug!("Processing transaction: {}", signature);
        
        // Parse the transaction and extract account states
        let (transaction, pre_account_states, post_account_states) = 
            self.parse_encoded_transaction(&encoded_transaction)?;
        
        let slot = encoded_transaction.slot;
        let block_time = encoded_transaction.block_time.map(|t| {
            DateTime::from_timestamp(t, 0).unwrap_or_else(|| Utc::now())
        });
        
        let success = encoded_transaction.transaction.meta
            .as_ref()
            .map(|meta| meta.err.is_none())
            .unwrap_or(false);
        
        let error_message = encoded_transaction.transaction.meta
            .as_ref()
            .and_then(|meta| meta.err.as_ref())
            .map(|err| format!("{:?}", err));
        
        // Analyze the transaction
        let transaction_analyzer = self.transaction_analyzer.read().await;
        let analysis = transaction_analyzer.analyze_transaction(
            &signature,
            &transaction,
            slot,
            block_time,
            success,
            error_message,
            &pre_account_states,
            &post_account_states,
        )?;
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.transactions_processed += 1;
            stats.pdas_discovered += analysis.discovered_pdas.len();
            stats.interactions_recorded += analysis.pda_interactions.len();
        }
        
        info!("Processed transaction {} with {} PDA interactions", 
              signature, analysis.pda_interactions.len());
        
        Ok(analysis)
    }

    pub async fn process_batch(
        &self,
        transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Result<Vec<TransactionAnalysis>> {
        let mut results = Vec::new();
        
        for transaction in transactions {
            match self.process_transaction(transaction).await {
                Ok(analysis) => results.push(analysis),
                Err(e) => {
                    error!("Failed to process transaction: {}", e);
                    continue;
                }
            }
        }
        
        Ok(results)
    }

    pub async fn analyze_pda_patterns(
        &self,
        program_id: &Pubkey,
        pdas: &[PdaInfo],
    ) -> Result<Vec<PdaPatternAnalysis>> {
        let mut patterns = Vec::new();
        let mut seed_frequency = HashMap::new();
        
        // Analyze seed patterns
        for pda in pdas {
            if pda.program_id == *program_id {
                let seed_pattern = self.extract_seed_pattern(&pda.seeds);
                *seed_frequency.entry(seed_pattern.clone()).or_insert(0) += 1;
            }
        }
        
        // Convert to pattern analysis
        for (pattern, count) in seed_frequency {
            patterns.push(PdaPatternAnalysis {
                pattern,
                frequency: count,
                program_id: *program_id,
                examples: pdas.iter()
                    .filter(|pda| pda.program_id == *program_id)
                    .take(5)
                    .map(|pda| pda.address)
                    .collect(),
            });
        }
        
        // Sort by frequency
        patterns.sort_by_key(|p| std::cmp::Reverse(p.frequency));
        
        Ok(patterns)
    }

    pub async fn get_stats(&self) -> ProcessingStats {
        self.stats.read().await.clone()
    }

    fn parse_encoded_transaction(
        &self,
        encoded_transaction: &EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<(Transaction, Vec<AccountState>, Vec<AccountState>)> {
        // This is a simplified parser
        // In a full implementation, you'd need to properly decode the transaction
        // and extract pre/post account states from the meta
        
        // For now, return placeholder values
        let transaction = Transaction::default();
        let pre_account_states = Vec::new();
        let post_account_states = Vec::new();
        
        Ok((transaction, pre_account_states, post_account_states))
    }

    fn extract_seed_pattern(&self, seeds: &[solana_pda_analyzer_core::SeedValue]) -> String {
        if seeds.is_empty() {
            return "empty".to_string();
        }
        
        seeds.iter()
            .map(|seed| seed.seed_type())
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingStats {
    pub transactions_processed: usize,
    pub pdas_discovered: usize,
    pub interactions_recorded: usize,
    pub errors_encountered: usize,
    pub processing_start_time: DateTime<Utc>,
}

impl ProcessingStats {
    pub fn new() -> Self {
        Self {
            transactions_processed: 0,
            pdas_discovered: 0,
            interactions_recorded: 0,
            errors_encountered: 0,
            processing_start_time: Utc::now(),
        }
    }
    
    pub fn processing_duration(&self) -> chrono::Duration {
        Utc::now() - self.processing_start_time
    }
    
    pub fn transactions_per_second(&self) -> f64 {
        let duration = self.processing_duration().num_seconds() as f64;
        if duration > 0.0 {
            self.transactions_processed as f64 / duration
        } else {
            0.0
        }
    }
}

impl Default for ProcessingStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PdaPatternAnalysis {
    pub pattern: String,
    pub frequency: usize,
    pub program_id: Pubkey,
    pub examples: Vec<Pubkey>,
}

#[derive(Debug, Clone)]
pub struct ProgramAnalyzer {
    processor: BatchProcessor,
    program_id: Pubkey,
}

impl ProgramAnalyzer {
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            processor: BatchProcessor::new(),
            program_id,
        }
    }

    pub async fn analyze_program_activity(
        &self,
        transactions: Vec<EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Result<ProgramAnalysis> {
        let analyses = self.processor.process_batch(transactions).await?;
        
        let mut all_pdas = Vec::new();
        let mut total_interactions = 0;
        let mut successful_transactions = 0;
        let mut failed_transactions = 0;
        
        for analysis in &analyses {
            all_pdas.extend(analysis.discovered_pdas.clone());
            total_interactions += analysis.pda_interactions.len();
            
            if analysis.success {
                successful_transactions += 1;
            } else {
                failed_transactions += 1;
            }
        }
        
        // Remove duplicate PDAs
        all_pdas.sort_by_key(|pda| pda.address);
        all_pdas.dedup_by_key(|pda| pda.address);
        
        let patterns = self.processor.analyze_pda_patterns(&self.program_id, &all_pdas).await?;
        
        Ok(ProgramAnalysis {
            program_id: self.program_id,
            total_transactions: analyses.len(),
            successful_transactions,
            failed_transactions,
            unique_pdas: all_pdas.len(),
            total_interactions,
            patterns,
            transaction_analyses: analyses,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProgramAnalysis {
    pub program_id: Pubkey,
    pub total_transactions: usize,
    pub successful_transactions: usize,
    pub failed_transactions: usize,
    pub unique_pdas: usize,
    pub total_interactions: usize,
    pub patterns: Vec<PdaPatternAnalysis>,
    pub transaction_analyses: Vec<TransactionAnalysis>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_batch_processor_creation() {
        let processor = BatchProcessor::new();
        let stats = processor.get_stats().await;
        assert_eq!(stats.transactions_processed, 0);
    }
    
    #[tokio::test]
    async fn test_processing_stats() {
        let mut stats = ProcessingStats::new();
        stats.transactions_processed = 100;
        
        // Sleep a bit to ensure duration is > 0
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let tps = stats.transactions_per_second();
        assert!(tps > 0.0);
    }
    
    #[test]
    fn test_program_analyzer_creation() {
        let program_id = Pubkey::new_unique();
        let analyzer = ProgramAnalyzer::new(program_id);
        assert_eq!(analyzer.program_id, program_id);
    }
}