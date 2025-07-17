pub mod pda;
pub mod transaction;
pub mod error;
pub mod types;
// pub mod database;

pub use pda::{PdaPattern, PdaAnalysisResult, PdaAnalyzer};
pub use transaction::*;
pub use error::*;
pub use types::{PdaInfo, SeedValue, PdaPatternTemplate, SeedTemplate, TransactionAnalysis, PdaInteraction, InteractionType, ProgramInfo, SeedDerivationAttempt};
// pub use database::*;

// Export database types for API compatibility
// pub use solana_pda_analyzer_database::{DatabaseMetrics as DatabaseStats, ProgramRecord as DbProgram, PdaRecord as DbPdaInfo, DatabaseRepository as DatabaseManager};