use crate::models::*;
use solana_pda_analyzer_core::{PdaAnalyzerError, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use tracing::{info, error, debug};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DatabaseRepository {
    pool: PgPool,
}

impl DatabaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn from_url(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;
        Ok(Self::new(pool))
    }

    // Program operations
    pub async fn create_program(&self, request: CreateProgramRequest) -> Result<ProgramRecord> {
        let record = sqlx::query_as::<_, ProgramRecord>(
            r#"
            INSERT INTO programs (program_id, name, description)
            VALUES ($1, $2, $3)
            ON CONFLICT (program_id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                updated_at = NOW()
            RETURNING id, program_id, name, description, created_at, updated_at
            "#,
        )
        .bind(request.program_id)
        .bind(request.name)
        .bind(request.description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_program_by_id(&self, program_id: &str) -> Result<Option<ProgramRecord>> {
        let record = sqlx::query_as::<_, ProgramRecord>(
            "SELECT id, program_id, name, description, created_at, updated_at FROM programs WHERE program_id = $1"
        )
        .bind(program_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn list_programs(&self, filter: ProgramFilter) -> Result<Vec<ProgramRecord>> {
        let mut query = "SELECT id, program_id, name, description, created_at, updated_at FROM programs WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(program_id) = &filter.program_id {
            query.push_str(&format!(" AND program_id = ${}", param_count));
            params.push(program_id.clone());
            param_count += 1;
        }

        if let Some(name) = &filter.name {
            query.push_str(&format!(" AND name ILIKE ${}", param_count));
            params.push(format!("%{}%", name));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(limit.to_string());
            param_count += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(offset.to_string());
        }

        let mut sql_query = sqlx::query_as::<_, ProgramRecord>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let records = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(records)
    }

    // Transaction operations
    pub async fn create_transaction(&self, request: CreateTransactionRequest) -> Result<TransactionRecord> {
        let record = sqlx::query_as::<_, TransactionRecord>(
            r#"
            INSERT INTO transactions (signature, slot, block_time, fee, success, error_message)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (signature) DO UPDATE SET
                slot = EXCLUDED.slot,
                block_time = EXCLUDED.block_time,
                fee = EXCLUDED.fee,
                success = EXCLUDED.success,
                error_message = EXCLUDED.error_message,
                updated_at = NOW()
            RETURNING id, signature, slot, block_time, fee, success, error_message, created_at, updated_at
            "#,
        )
        .bind(request.signature)
        .bind(request.slot)
        .bind(request.block_time)
        .bind(request.fee)
        .bind(request.success)
        .bind(request.error_message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_transaction_by_signature(&self, signature: &str) -> Result<Option<TransactionRecord>> {
        let record = sqlx::query_as::<_, TransactionRecord>(
            "SELECT id, signature, slot, block_time, fee, success, error_message, created_at, updated_at FROM transactions WHERE signature = $1"
        )
        .bind(signature)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn list_transactions(&self, filter: TransactionFilter) -> Result<Vec<TransactionRecord>> {
        let mut query = "SELECT id, signature, slot, block_time, fee, success, error_message, created_at, updated_at FROM transactions WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(signature) = &filter.signature {
            query.push_str(&format!(" AND signature = ${}", param_count));
            params.push(signature.clone());
            param_count += 1;
        }

        if let Some((min_slot, max_slot)) = filter.slot_range {
            query.push_str(&format!(" AND slot >= ${} AND slot <= ${}", param_count, param_count + 1));
            params.push(min_slot.to_string());
            params.push(max_slot.to_string());
            param_count += 2;
        }

        if let Some(success) = filter.success {
            query.push_str(&format!(" AND success = ${}", param_count));
            params.push(success.to_string());
            param_count += 1;
        }

        query.push_str(" ORDER BY slot DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(limit.to_string());
            param_count += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(offset.to_string());
        }

        let mut sql_query = sqlx::query_as::<_, TransactionRecord>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let records = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(records)
    }

    // PDA operations
    pub async fn create_pda(&self, request: CreatePdaRequest) -> Result<PdaRecord> {
        let record = sqlx::query_as::<_, PdaRecord>(
            r#"
            INSERT INTO pdas (address, program_id, seeds, bump, first_seen_transaction, data_hash)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (address) DO UPDATE SET
                seeds = EXCLUDED.seeds,
                bump = EXCLUDED.bump,
                first_seen_transaction = COALESCE(pdas.first_seen_transaction, EXCLUDED.first_seen_transaction),
                data_hash = EXCLUDED.data_hash,
                updated_at = NOW()
            RETURNING id, address, program_id, seeds, bump, first_seen_transaction, data_hash, created_at, updated_at
            "#,
        )
        .bind(request.address)
        .bind(request.program_id)
        .bind(request.seeds)
        .bind(request.bump)
        .bind(request.first_seen_transaction)
        .bind(request.data_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn get_pda_by_address(&self, address: &str) -> Result<Option<PdaRecord>> {
        let record = sqlx::query_as::<_, PdaRecord>(
            "SELECT id, address, program_id, seeds, bump, first_seen_transaction, data_hash, created_at, updated_at FROM pdas WHERE address = $1"
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn list_pdas(&self, filter: PdaFilter) -> Result<Vec<PdaRecord>> {
        let mut query = "SELECT id, address, program_id, seeds, bump, first_seen_transaction, data_hash, created_at, updated_at FROM pdas WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(address) = &filter.address {
            query.push_str(&format!(" AND address = ${}", param_count));
            params.push(address.clone());
            param_count += 1;
        }

        if let Some(program_id) = filter.program_id {
            query.push_str(&format!(" AND program_id = ${}", param_count));
            params.push(program_id.to_string());
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(limit.to_string());
            param_count += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(offset.to_string());
        }

        let mut sql_query = sqlx::query_as::<_, PdaRecord>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let records = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(records)
    }

    // Account interaction operations
    pub async fn create_account_interaction(&self, request: CreateAccountInteractionRequest) -> Result<AccountInteractionRecord> {
        let record = sqlx::query_as::<_, AccountInteractionRecord>(
            r#"
            INSERT INTO account_interactions (transaction_id, pda_id, instruction_index, interaction_type, data_before, data_after, lamports_before, lamports_after)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, transaction_id, pda_id, instruction_index, interaction_type, data_before, data_after, lamports_before, lamports_after, created_at
            "#,
        )
        .bind(request.transaction_id)
        .bind(request.pda_id)
        .bind(request.instruction_index)
        .bind(request.interaction_type)
        .bind(request.data_before)
        .bind(request.data_after)
        .bind(request.lamports_before)
        .bind(request.lamports_after)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(record)
    }

    pub async fn list_account_interactions(&self, filter: AccountInteractionFilter) -> Result<Vec<AccountInteractionRecord>> {
        let mut query = "SELECT id, transaction_id, pda_id, instruction_index, interaction_type, data_before, data_after, lamports_before, lamports_after, created_at FROM account_interactions WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_count = 1;

        if let Some(transaction_id) = filter.transaction_id {
            query.push_str(&format!(" AND transaction_id = ${}", param_count));
            params.push(transaction_id.to_string());
            param_count += 1;
        }

        if let Some(pda_id) = filter.pda_id {
            query.push_str(&format!(" AND pda_id = ${}", param_count));
            params.push(pda_id.to_string());
            param_count += 1;
        }

        if let Some(interaction_type) = &filter.interaction_type {
            query.push_str(&format!(" AND interaction_type = ${}", param_count));
            params.push(interaction_type.clone());
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            query.push_str(&format!(" LIMIT ${}", param_count));
            params.push(limit.to_string());
            param_count += 1;
        }

        if let Some(offset) = filter.offset {
            query.push_str(&format!(" OFFSET ${}", param_count));
            params.push(offset.to_string());
        }

        let mut sql_query = sqlx::query_as::<_, AccountInteractionRecord>(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let records = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(records)
    }

    // Statistics and analytics
    pub async fn get_program_stats(&self, program_id: Uuid) -> Result<ProgramStats> {
        let stats = sqlx::query_as::<_, ProgramStats>(
            r#"
            SELECT 
                p.id as program_id,
                COUNT(DISTINCT t.id) as total_transactions,
                COUNT(DISTINCT pd.id) as total_pdas,
                COUNT(DISTINCT ai.id) as total_interactions,
                (COUNT(CASE WHEN t.success THEN 1 END) * 100.0 / NULLIF(COUNT(t.id), 0)) as success_rate
            FROM programs p
            LEFT JOIN pdas pd ON p.id = pd.program_id
            LEFT JOIN account_interactions ai ON pd.id = ai.pda_id
            LEFT JOIN transactions t ON ai.transaction_id = t.id
            WHERE p.id = $1
            GROUP BY p.id
            "#,
        )
        .bind(program_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(stats)
    }

    pub async fn get_stats(&self) -> Result<DatabaseMetrics> {
        self.get_database_metrics().await
    }

    pub async fn get_database_metrics(&self) -> Result<DatabaseMetrics> {
        let row = sqlx::query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM programs) as total_programs,
                (SELECT COUNT(*) FROM transactions) as total_transactions,
                (SELECT COUNT(*) FROM pdas) as total_pdas,
                (SELECT COUNT(*) FROM account_interactions) as total_interactions,
                (SELECT pg_size_pretty(pg_database_size(current_database()))) as database_size
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        // Parse database size (simplified)
        let database_size_mb = 0.0; // In a real implementation, parse the pg_size_pretty output

        Ok(DatabaseMetrics {
            total_programs: row.get::<Option<i64>, _>("total_programs").unwrap_or(0),
            total_transactions: row.get::<Option<i64>, _>("total_transactions").unwrap_or(0),
            total_pdas: row.get::<Option<i64>, _>("total_pdas").unwrap_or(0),
            total_interactions: row.get::<Option<i64>, _>("total_interactions").unwrap_or(0),
            database_size_mb,
        })
    }

    pub async fn get_recent_pdas(&self, limit: i64) -> Result<Vec<PdaRecord>> {
        let filter = PdaFilter {
            address: None,
            program_id: None,
            limit: Some(limit),
            offset: None,
        };
        self.list_pdas(filter).await
    }

    pub async fn store_pda_analysis(&self, _analysis: &solana_pda_analyzer_core::PdaAnalysisResult) -> Result<()> {
        // TODO: Implement storing PDA analysis results
        Ok(())
    }

    pub async fn update_program_pda_count(&self, _program_id: &str) -> Result<()> {
        // TODO: Implement updating program PDA count
        Ok(())
    }

    pub async fn get_program(&self, program_id: &str) -> Result<Option<ProgramRecord>> {
        self.get_program_by_id(program_id).await
    }

    pub async fn get_programs(&self, filter: ProgramFilter) -> Result<Vec<ProgramRecord>> {
        self.list_programs(filter).await
    }

    pub async fn get_pdas_by_program(&self, program_id: &str, limit: i64) -> Result<Vec<PdaRecord>> {
        // TODO: Convert program_id string to UUID
        let filter = PdaFilter {
            address: None,
            program_id: None, // Should be converted from string to UUID
            limit: Some(limit),
            offset: None,
        };
        self.list_pdas(filter).await
    }

    pub async fn get_pdas_by_pattern(&self, _pattern: &str, limit: i64) -> Result<Vec<PdaRecord>> {
        // TODO: Implement pattern-based PDA search
        let filter = PdaFilter {
            address: None,
            program_id: None,
            limit: Some(limit),
            offset: None,
        };
        self.list_pdas(filter).await
    }

    pub async fn migrate(&self) -> Result<()> {
        // TODO: Implement database migrations
        Ok(())
    }

    // Batch operations
    pub async fn batch_create_pdas(&self, requests: Vec<CreatePdaRequest>) -> Result<Vec<PdaRecord>> {
        let mut results = Vec::new();
        
        for request in requests {
            match self.create_pda(request).await {
                Ok(record) => results.push(record),
                Err(e) => {
                    error!("Failed to create PDA: {}", e);
                    continue;
                }
            }
        }
        
        Ok(results)
    }

    pub async fn batch_create_interactions(&self, requests: Vec<CreateAccountInteractionRequest>) -> Result<Vec<AccountInteractionRecord>> {
        let mut results = Vec::new();
        
        for request in requests {
            match self.create_account_interaction(request).await {
                Ok(record) => results.push(record),
                Err(e) => {
                    error!("Failed to create interaction: {}", e);
                    continue;
                }
            }
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    
    // Note: These tests would require a test database setup
    // For now, they're just structural tests
    
    #[test]
    fn test_database_repository_creation() {
        // This would need a real PgPool for testing
        // let pool = PgPool::connect("postgresql://test").await.unwrap();
        // let repo = DatabaseRepository::new(pool);
        // assert!(repo.pool is not null);
    }
}