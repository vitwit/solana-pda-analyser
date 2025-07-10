use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdaAnalyzerError {
    #[error("Invalid seed data: {0}")]
    InvalidSeedData(String),
    
    #[error("PDA derivation failed: {0}")]
    PdaDerivationFailed(String),
    
    #[error("Invalid program ID: {0}")]
    InvalidProgramId(String),
    
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    #[error("Transaction parsing error: {0}")]
    TransactionParsingError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

pub type Result<T> = std::result::Result<T, PdaAnalyzerError>;

impl From<serde_json::Error> for PdaAnalyzerError {
    fn from(err: serde_json::Error) -> Self {
        PdaAnalyzerError::SerializationError(err.to_string())
    }
}

impl From<solana_sdk::pubkey::ParsePubkeyError> for PdaAnalyzerError {
    fn from(err: solana_sdk::pubkey::ParsePubkeyError) -> Self {
        PdaAnalyzerError::InvalidPublicKey(err.to_string())
    }
}