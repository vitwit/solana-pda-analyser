use crate::{ApiError, ApiResponse, AppState};
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::collections::HashMap;
use tracing::info;

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzePdaRequest {
    pub address: String,
    pub program_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAnalyzePdaRequest {
    pub pdas: Vec<AnalyzePdaRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiDocsResponse {
    pub title: String,
    pub version: String,
    pub description: String,
    pub endpoints: Vec<EndpointDoc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointDoc {
    pub method: String,
    pub path: String,
    pub description: String,
    pub example: Option<String>,
}

// Health check handler
pub async fn health_check(_state: State<AppState>) -> impl IntoResponse {
    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    ApiResponse::success(response)
}

/// API documentation endpoint
pub async fn api_docs() -> impl IntoResponse {
    let endpoints = vec![
        EndpointDoc {
            method: "GET".to_string(),
            path: "/health".to_string(),
            description: "Health check endpoint".to_string(),
            example: None,
        },
        EndpointDoc {
            method: "POST".to_string(),
            path: "/api/v1/analyze/pda".to_string(),
            description: "Analyze a single PDA".to_string(),
            example: Some(r#"{"address": "...", "program_id": "..."}"#.to_string()),
        },
        EndpointDoc {
            method: "POST".to_string(),
            path: "/api/v1/analyze/pda/batch".to_string(),
            description: "Batch analyze multiple PDAs".to_string(),
            example: Some(r#"{"pdas": [{"address": "...", "program_id": "..."}]}"#.to_string()),
        },
    ];

    let response = ApiDocsResponse {
        title: "Solana PDA Analyzer API".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "Comprehensive API for analyzing Solana Program Derived Addresses".to_string(),
        endpoints,
    };

    ApiResponse::success(response)
}

// PDA analysis handlers
pub async fn analyze_pda(
    State(state): State<AppState>,
    Json(request): Json<AnalyzePdaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Analyzing PDA: {} for program: {}", request.address, request.program_id);

    let address = Pubkey::from_str(&request.address)
        .map_err(|e| ApiError::bad_request(format!("Invalid PDA address: {}", e)))?;
    
    let program_id = Pubkey::from_str(&request.program_id)
        .map_err(|e| ApiError::bad_request(format!("Invalid program ID: {}", e)))?;

    let mut analyzer = state.pda_analyzer.write().await;
    let result = analyzer.analyze_pda(&address, &program_id)
        .map_err(|e| ApiError::internal_server_error(format!("Analysis failed: {}", e)))?;

    match result {
        Some(analysis_result) => {
            info!("PDA analysis successful for {}", request.address);
            Ok(ApiResponse::success(analysis_result))
        }
        None => Err(ApiError::not_found("Could not analyze PDA - pattern not recognized".to_string())),
    }
}

pub async fn batch_analyze_pda(
    State(state): State<AppState>,
    Json(request): Json<BatchAnalyzePdaRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Batch analyzing {} PDAs", request.pdas.len());

    let mut results = Vec::new();
    let mut analyzer = state.pda_analyzer.write().await;

    for pda_request in request.pdas {
        let address = Pubkey::from_str(&pda_request.address)
            .map_err(|e| ApiError::bad_request(format!("Invalid PDA address: {}", e)))?;
        
        let program_id = Pubkey::from_str(&pda_request.program_id)
            .map_err(|e| ApiError::bad_request(format!("Invalid program ID: {}", e)))?;

        let result = analyzer.analyze_pda(&address, &program_id)
            .map_err(|e| ApiError::internal_server_error(format!("Analysis failed: {}", e)))?;

        results.push(result);
    }

    Ok(ApiResponse::success(results))
}

// Get performance metrics
pub async fn get_performance_metrics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let analyzer = state.pda_analyzer.read().await;
    let (cache_hits, cache_total) = analyzer.cache_stats();
    let pattern_stats = analyzer.get_pattern_stats();

    let mut metrics = HashMap::new();
    metrics.insert("cache_hits".to_string(), serde_json::Value::Number(cache_hits.into()));
    metrics.insert("cache_total".to_string(), serde_json::Value::Number(cache_total.into()));
    metrics.insert("cache_hit_rate".to_string(), serde_json::Value::Number(
        if cache_total > 0 { 
            serde_json::Number::from_f64(cache_hits as f64 / cache_total as f64).unwrap_or(serde_json::Number::from(0))
        } else { 
            serde_json::Number::from(0) 
        }
    ));
    metrics.insert("pattern_stats".to_string(), serde_json::to_value(pattern_stats).unwrap());

    Ok(ApiResponse::success(metrics))
}

// Stub handlers for future implementation
pub async fn list_programs() -> Result<ApiResponse<Vec<String>>, ApiError> {
    Err(ApiError::not_implemented("Programs listing not implemented yet - database required".to_string()))
}

pub async fn get_program(_program_id: Path<String>) -> Result<ApiResponse<serde_json::Value>, ApiError> {
    Err(ApiError::not_implemented("Program details not implemented yet - database required".to_string()))
}

pub async fn list_pdas() -> Result<ApiResponse<Vec<serde_json::Value>>, ApiError> {
    Err(ApiError::not_implemented("PDAs listing not implemented yet - database required".to_string()))
}

pub async fn get_database_metrics() -> Result<ApiResponse<serde_json::Value>, ApiError> {
    Err(ApiError::not_implemented("Database metrics not implemented yet - database required".to_string()))
}