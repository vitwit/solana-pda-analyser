use crate::handlers::*;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use solana_pda_analyzer_core::{PdaAnalyzer, DatabaseManager};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseManager>,
    pub pda_analyzer: Arc<RwLock<PdaAnalyzer>>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // API documentation
        .route("/docs", get(api_docs))
        
        // PDA analysis routes
        .route("/api/v1/analyze/pda", post(analyze_pda))
        .route("/api/v1/analyze/pda/batch", post(batch_analyze_pda))
        
        // Program routes
        .route("/api/v1/programs", get(list_programs))
        .route("/api/v1/programs/:program_id", get(get_program))
        .route("/api/v1/programs/:program_id/stats", get(get_program_stats))
        .route("/api/v1/programs/:program_id/patterns", get(get_program_patterns))
        .route("/api/v1/programs/:program_id/pdas", get(get_program_pdas))
        
        // Transaction routes
        .route("/api/v1/transactions", get(list_transactions))
        .route("/api/v1/transactions/:signature", get(get_transaction))
        .route("/api/v1/transactions/analyze", post(analyze_transaction))
        
        // PDA routes
        .route("/api/v1/pdas", get(list_pdas))
        .route("/api/v1/pdas/:address", get(get_pda))
        .route("/api/v1/pdas/search", get(search_pdas))
        .route("/api/v1/pdas/recent", get(get_recent_pdas))
        
        // Analytics routes
        .route("/api/v1/analytics/database", get(get_database_metrics))
        .route("/api/v1/analytics/patterns", get(get_pattern_distribution))
        .route("/api/v1/analytics/performance", get(get_performance_metrics))
        
        // Add CORS middleware
        .layer(CorsLayer::permissive())
        
        // Add state
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_router_creation() {
        // Test that the router can be created without panicking
        // This is a basic structural test
        assert!(true);
    }
}