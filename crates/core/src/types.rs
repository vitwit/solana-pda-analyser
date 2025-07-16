use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaInfo {
    pub address: Pubkey,
    pub program_id: Pubkey,
    pub seeds: Vec<SeedValue>,
    pub bump: u8,
    pub first_seen_slot: Option<u64>,
    pub first_seen_transaction: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeedValue {
    String(String),
    Bytes(Vec<u8>),
    Pubkey(Pubkey),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
}

impl SeedValue {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            SeedValue::String(s) => s.as_bytes().to_vec(),
            SeedValue::Bytes(b) => b.clone(),
            SeedValue::Pubkey(pk) => pk.as_ref().to_vec(),
            SeedValue::U64(n) => n.to_le_bytes().to_vec(),
            SeedValue::U32(n) => n.to_le_bytes().to_vec(),
            SeedValue::U16(n) => n.to_le_bytes().to_vec(),
            SeedValue::U8(n) => vec![*n],
        }
    }
    
    pub fn seed_type(&self) -> &'static str {
        match self {
            SeedValue::String(_) => "string",
            SeedValue::Bytes(_) => "bytes",
            SeedValue::Pubkey(_) => "pubkey",
            SeedValue::U64(_) => "u64",
            SeedValue::U32(_) => "u32",
            SeedValue::U16(_) => "u16",
            SeedValue::U8(_) => "u8",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaPatternTemplate {
    pub id: Uuid,
    pub program_id: Pubkey,
    pub pattern_name: String,
    pub seeds_template: Vec<SeedTemplate>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedTemplate {
    pub name: String,
    pub seed_type: String,
    pub description: Option<String>,
    pub is_variable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<DateTime<Utc>>,
    pub success: bool,
    pub error_message: Option<String>,
    pub pda_interactions: Vec<PdaInteraction>,
    pub discovered_pdas: Vec<PdaInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdaInteraction {
    pub pda_address: Pubkey,
    pub instruction_index: u32,
    pub interaction_type: InteractionType,
    pub data_before: Option<Vec<u8>>,
    pub data_after: Option<Vec<u8>>,
    pub lamports_before: Option<u64>,
    pub lamports_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionType {
    Read,
    Write,
    Create,
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub id: Uuid,
    pub program_id: Pubkey,
    pub name: Option<String>,
    pub description: Option<String>,
    pub total_pdas: u64,
    pub total_transactions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedDerivationAttempt {
    pub pda_address: Pubkey,
    pub program_id: Pubkey,
    pub attempted_seeds: Vec<SeedValue>,
    pub success: bool,
    pub attempted_at: DateTime<Utc>,
}