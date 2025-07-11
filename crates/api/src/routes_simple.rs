use crate::handlers_simple::*;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use solana_pda_analyzer_core::PdaAnalyzer;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub pda_analyzer: Arc<RwLock<PdaAnalyzer>>,
}

pub fn create_simple_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // API documentation
        .route("/docs", get(api_docs))
        
        // PDA analysis routes
        .route("/api/v1/analyze/pda", post(analyze_pda))
        .route("/api/v1/analyze/pda/batch", post(batch_analyze_pda))
        
        // Analytics routes
        .route("/api/v1/analytics/performance", get(get_performance_metrics))
        
        // Stub routes for future implementation
        .route("/api/v1/programs", get(list_programs))
        .route("/api/v1/programs/:program_id", get(get_program))
        .route("/api/v1/pdas", get(list_pdas))
        .route("/api/v1/analytics/database", get(get_database_metrics))
        
        // Add CORS middleware
        .layer(CorsLayer::permissive())
        
        // Add state
        .with_state(state)
}