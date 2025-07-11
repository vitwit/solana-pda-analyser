use crate::{Result, PdaInfo, PdaAnalysisResult, PdaPattern, TransactionAnalysis, PdaAnalyzerError};
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbPdaInfo {
    pub id: Uuid,
    pub address: String,
    pub program_id: String,
    pub seeds: serde_json::Value,
    pub bump: i16,
    pub pattern: Option<String>,
    pub confidence: Option<f64>,
    pub analysis_time_ms: Option<i64>,
    pub first_seen_slot: Option<i64>,
    pub first_seen_transaction: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbProgram {
    pub id: Uuid,
    pub program_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub total_pdas: i64,
    pub last_analyzed: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTransaction {
    pub id: Uuid,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
    pub program_ids: Vec<String>,
    pub pda_interactions: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_pdas: i64,
    pub total_programs: i64,
    pub total_transactions: i64,
    pub patterns_distribution: Vec<PatternStat>,
    pub recent_analyses: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternStat {
    pub pattern: String,
    pub count: i64,
    pub percentage: f64,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// Store PDA analysis result
    pub async fn store_pda_analysis(&self, result: &PdaAnalysisResult) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let seeds_json = serde_json::to_value(&result.pda_info.seeds)
            .map_err(|e| PdaAnalyzerError::SerializationError(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO pdas (
                id, address, program_id, seeds, bump, pattern, confidence, 
                analysis_time_ms, first_seen_slot, first_seen_transaction,
                created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11
            )
            ON CONFLICT (address, program_id) 
            DO UPDATE SET
                pattern = EXCLUDED.pattern,
                confidence = EXCLUDED.confidence,
                analysis_time_ms = EXCLUDED.analysis_time_ms,
                updated_at = EXCLUDED.updated_at
            "#,
            id,
            result.pda_info.address.to_string(),
            result.pda_info.program_id.to_string(),
            seeds_json,
            result.pda_info.bump as i16,
            result.pattern.as_str(),
            result.confidence,
            result.analysis_time_ms as i64,
            result.pda_info.first_seen_slot,
            result.pda_info.first_seen_transaction,
            Utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// Get PDA info by address
    pub async fn get_pda(&self, address: &str, program_id: &str) -> Result<Option<DbPdaInfo>> {
        let result = sqlx::query_as!(
            DbPdaInfo,
            "SELECT * FROM pdas WHERE address = $1 AND program_id = $2",
            address,
            program_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Get all PDAs for a program
    pub async fn get_program_pdas(&self, program_id: &str) -> Result<Vec<DbPdaInfo>> {
        let results = sqlx::query_as!(
            DbPdaInfo,
            "SELECT * FROM pdas WHERE program_id = $1 ORDER BY created_at DESC",
            program_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Store program information
    pub async fn store_program(&self, program_id: &str, name: Option<&str>, description: Option<&str>) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO programs (id, program_id, name, description, total_pdas, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 0, $5, $5)
            ON CONFLICT (program_id) 
            DO UPDATE SET
                name = COALESCE(EXCLUDED.name, programs.name),
                description = COALESCE(EXCLUDED.description, programs.description),
                updated_at = EXCLUDED.updated_at
            "#,
            id,
            program_id,
            name,
            description,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// Get program information
    pub async fn get_program(&self, program_id: &str) -> Result<Option<DbProgram>> {
        let result = sqlx::query_as!(
            DbProgram,
            "SELECT * FROM programs WHERE program_id = $1",
            program_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(result)
    }

    /// Get all programs
    pub async fn get_all_programs(&self) -> Result<Vec<DbProgram>> {
        let results = sqlx::query_as!(
            DbProgram,
            "SELECT * FROM programs ORDER BY total_pdas DESC, name ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Store transaction analysis
    pub async fn store_transaction(&self, analysis: &TransactionAnalysis) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let program_ids: Vec<String> = analysis.program_interactions.keys()
            .map(|k| k.to_string())
            .collect();
        let interactions_json = serde_json::to_value(&analysis.pda_interactions)
            .map_err(|e| PdaAnalyzerError::SerializationError(e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO transactions (id, signature, slot, block_time, program_ids, pda_interactions, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (signature) DO NOTHING
            "#,
            id,
            analysis.signature,
            analysis.slot as i64,
            analysis.block_time,
            &program_ids,
            interactions_json,
            Utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        // Get total counts
        let total_pdas: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM pdas")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
            .unwrap_or(0);

        let total_programs: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM programs")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
            .unwrap_or(0);

        let total_transactions: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
            .unwrap_or(0);

        // Get pattern distribution
        let pattern_rows = sqlx::query!(
            r#"
            SELECT pattern, COUNT(*) as count
            FROM pdas 
            WHERE pattern IS NOT NULL 
            GROUP BY pattern 
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        let mut patterns_distribution = Vec::new();
        for row in pattern_rows {
            if let (Some(pattern), Some(count)) = (row.pattern, row.count) {
                let percentage = if total_pdas > 0 {
                    (count as f64 / total_pdas as f64) * 100.0
                } else {
                    0.0
                };
                patterns_distribution.push(PatternStat {
                    pattern,
                    count,
                    percentage,
                });
            }
        }

        // Get recent analyses (last 24 hours)
        let recent_analyses: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM pdas WHERE created_at > NOW() - INTERVAL '24 hours'"
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
        .unwrap_or(0);

        Ok(DatabaseStats {
            total_pdas,
            total_programs,
            total_transactions,
            patterns_distribution,
            recent_analyses,
        })
    }

    /// Update program PDA count
    pub async fn update_program_pda_count(&self, program_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE programs 
            SET total_pdas = (
                SELECT COUNT(*) FROM pdas WHERE program_id = $1
            ),
            last_analyzed = NOW(),
            updated_at = NOW()
            WHERE program_id = $1
            "#,
            program_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Search PDAs by pattern
    pub async fn search_pdas_by_pattern(&self, pattern: &str, limit: i64) -> Result<Vec<DbPdaInfo>> {
        let results = sqlx::query_as!(
            DbPdaInfo,
            "SELECT * FROM pdas WHERE pattern = $1 ORDER BY confidence DESC, created_at DESC LIMIT $2",
            pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Get recent PDAs
    pub async fn get_recent_pdas(&self, limit: i64) -> Result<Vec<DbPdaInfo>> {
        let results = sqlx::query_as!(
            DbPdaInfo,
            "SELECT * FROM pdas ORDER BY created_at DESC LIMIT $1",
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(results)
    }

    /// Close database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

impl From<DbPdaInfo> for PdaInfo {
    fn from(db_pda: DbPdaInfo) -> Self {
        let address = Pubkey::from_str(&db_pda.address).unwrap_or_default();
        let program_id = Pubkey::from_str(&db_pda.program_id).unwrap_or_default();
        let seeds = serde_json::from_value(db_pda.seeds).unwrap_or_default();
        
        PdaInfo {
            address,
            program_id,
            seeds,
            bump: db_pda.bump as u8,
            first_seen_slot: db_pda.first_seen_slot.map(|s| s as u64),
            first_seen_transaction: db_pda.first_seen_transaction,
        }
    }
}