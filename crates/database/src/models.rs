use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProgramRecord {
    pub id: Uuid,
    pub program_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PdaPatternRecord {
    pub id: Uuid,
    pub program_id: Uuid,
    pub pattern_name: String,
    pub seeds_template: serde_json::Value,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
    pub fee: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PdaRecord {
    pub id: Uuid,
    pub address: String,
    pub program_id: Uuid,
    pub seeds: serde_json::Value,
    pub bump: i16,
    pub first_seen_transaction: Option<Uuid>,
    pub data_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountInteractionRecord {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub pda_id: Uuid,
    pub instruction_index: i32,
    pub interaction_type: String,
    pub data_before: Option<Vec<u8>>,
    pub data_after: Option<Vec<u8>>,
    pub lamports_before: Option<i64>,
    pub lamports_after: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SeedDerivationAttemptRecord {
    pub id: Uuid,
    pub pda_address: String,
    pub program_id: Uuid,
    pub attempted_seeds: serde_json::Value,
    pub success: bool,
    pub attempted_at: DateTime<Utc>,
}

// New transaction structs for database operations
#[derive(Debug, Clone)]
pub struct CreateProgramRequest {
    pub program_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePdaPatternRequest {
    pub program_id: Uuid,
    pub pattern_name: String,
    pub seeds_template: serde_json::Value,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateTransactionRequest {
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
    pub fee: Option<i64>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePdaRequest {
    pub address: String,
    pub program_id: Uuid,
    pub seeds: serde_json::Value,
    pub bump: i16,
    pub first_seen_transaction: Option<Uuid>,
    pub data_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateAccountInteractionRequest {
    pub transaction_id: Uuid,
    pub pda_id: Uuid,
    pub instruction_index: i32,
    pub interaction_type: String,
    pub data_before: Option<Vec<u8>>,
    pub data_after: Option<Vec<u8>>,
    pub lamports_before: Option<i64>,
    pub lamports_after: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CreateSeedDerivationAttemptRequest {
    pub pda_address: String,
    pub program_id: Uuid,
    pub attempted_seeds: serde_json::Value,
    pub success: bool,
}

// Query filters
#[derive(Debug, Clone, Default)]
pub struct ProgramFilter {
    pub program_id: Option<String>,
    pub name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct TransactionFilter {
    pub signature: Option<String>,
    pub slot_range: Option<(i64, i64)>,
    pub success: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct PdaFilter {
    pub address: Option<String>,
    pub program_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct AccountInteractionFilter {
    pub transaction_id: Option<Uuid>,
    pub pda_id: Option<Uuid>,
    pub interaction_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Statistics and aggregation structs
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProgramStats {
    pub program_id: Uuid,
    pub total_transactions: i64,
    pub total_pdas: i64,
    pub total_interactions: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PdaStats {
    pub pda_id: Uuid,
    pub total_interactions: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub interaction_types: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMetrics {
    pub total_programs: i64,
    pub total_transactions: i64,
    pub total_pdas: i64,
    pub total_interactions: i64,
    pub database_size_mb: f64,
}

// Helper functions to convert between InteractionType and String
pub fn interaction_type_to_string(interaction_type: solana_pda_analyzer_core::InteractionType) -> String {
    match interaction_type {
        solana_pda_analyzer_core::InteractionType::Read => "read".to_string(),
        solana_pda_analyzer_core::InteractionType::Write => "write".to_string(),
        solana_pda_analyzer_core::InteractionType::Create => "create".to_string(),
        solana_pda_analyzer_core::InteractionType::Close => "close".to_string(),
    }
}

pub fn string_to_interaction_type(s: String) -> solana_pda_analyzer_core::InteractionType {
    match s.as_str() {
        "read" => solana_pda_analyzer_core::InteractionType::Read,
        "write" => solana_pda_analyzer_core::InteractionType::Write,
        "create" => solana_pda_analyzer_core::InteractionType::Create,
        "close" => solana_pda_analyzer_core::InteractionType::Close,
        _ => solana_pda_analyzer_core::InteractionType::Read,
    }
}