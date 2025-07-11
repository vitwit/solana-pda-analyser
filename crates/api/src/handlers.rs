use crate::{ApiError, ApiResponse, AppState};
use axum::{
    extract::{Path, Query, State},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use solana_pda_analyzer_core::{PdaAnalysisResult, DatabaseStats, DbPdaInfo, DbProgram};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::collections::HashMap;
use tracing::{info, error};

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzePdaRequest {
    pub address: String,
    pub program_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeTransactionRequest {
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAnalyzePdaRequest {
    pub pdas: Vec<AnalyzePdaRequest>,
}

#[derive(Debug, Deserialize)]
pub struct ProgramQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub success: Option<bool>,
    pub min_slot: Option<i64>,
    pub max_slot: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PdaQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub program_id: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub database_connected: bool,
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
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let database_connected = match state.database.get_stats().await {
        Ok(_) => true,
        Err(e) => {
            error!("Database health check failed: {}", e);
            false
        }
    };

    let response = HealthCheckResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        database_connected,
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
        EndpointDoc {
            method: "GET".to_string(),
            path: "/api/v1/programs".to_string(),
            description: "List all programs".to_string(),
            example: None,
        },
        EndpointDoc {
            method: "GET".to_string(),
            path: "/api/v1/programs/:program_id".to_string(),
            description: "Get program details".to_string(),
            example: None,
        },
        EndpointDoc {
            method: "GET".to_string(),
            path: "/api/v1/analytics/database".to_string(),
            description: "Get database metrics".to_string(),
            example: None,
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
        .map_err(|e| ApiError::BadRequest(format!("Invalid PDA address: {}", e)))?;
    
    let program_id = Pubkey::from_str(&request.program_id)
        .map_err(|e| ApiError::BadRequest(format!("Invalid program ID: {}", e)))?;

    let mut analyzer = state.pda_analyzer.write().await;
    let result = analyzer.analyze_pda(&address, &program_id)
        .map_err(|e| ApiError::InternalServerError(format!("Analysis failed: {}", e)))?;

    match result {
        Some(analysis_result) => {
            // Store the result in the database
            if let Err(e) = state.database.store_pda_analysis(&analysis_result).await {
                error!("Failed to store PDA analysis: {}", e);
            }

            // Update program stats
            if let Err(e) = state.database.update_program_pda_count(&request.program_id).await {
                error!("Failed to update program stats: {}", e);
            }

            Ok(ApiResponse::success(analysis_result))
        }
        None => Err(ApiError::NotFound("Could not analyze PDA - pattern not recognized".to_string())),
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
            .map_err(|e| ApiError::BadRequest(format!("Invalid PDA address: {}", e)))?;
        
        let program_id = Pubkey::from_str(&pda_request.program_id)
            .map_err(|e| ApiError::BadRequest(format!("Invalid program ID: {}", e)))?;

        let result = analyzer.analyze_pda(&address, &program_id)
            .map_err(|e| ApiError::InternalServerError(format!("Analysis failed: {}", e)))?;

        if let Some(ref analysis_result) = result {
            // Store the result in the database
            if let Err(e) = state.database.store_pda_analysis(analysis_result).await {
                error!("Failed to store PDA analysis: {}", e);
            }
        }

        results.push(result);
    }

    Ok(ApiResponse::success(results))
}

// Program handlers
pub async fn list_programs(
    State(state): State<AppState>,
    Query(query): Query<ProgramQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let programs = state.database.get_all_programs().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch programs: {}", e)))?;

    let limit = query.limit.unwrap_or(50).min(500) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let paginated_programs = programs.into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    Ok(ApiResponse::success(paginated_programs))
}

pub async fn get_program(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let program = state.database.get_program(&program_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch program: {}", e)))?;

    match program {
        Some(program) => Ok(ApiResponse::success(program)),
        None => Err(ApiError::NotFound("Program not found".to_string())),
    }
}

pub async fn get_program_stats(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let program = state.database.get_program(&program_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch program: {}", e)))?;

    let pdas = state.database.get_program_pdas(&program_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch PDAs: {}", e)))?;

    let mut stats = HashMap::new();
    stats.insert("total_pdas".to_string(), serde_json::Value::Number(pdas.len().into()));
    
    if let Some(program) = program {
        stats.insert("program_name".to_string(), serde_json::Value::String(program.name.unwrap_or("Unknown".to_string())));
        stats.insert("last_analyzed".to_string(), serde_json::to_value(program.last_analyzed).unwrap_or(serde_json::Value::Null));
    }

    // Pattern distribution
    let mut pattern_counts = HashMap::new();
    for pda in pdas {
        if let Some(pattern) = pda.pattern {
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }
    }
    stats.insert("pattern_distribution".to_string(), serde_json::to_value(pattern_counts).unwrap());

    Ok(ApiResponse::success(stats))
}

pub async fn get_program_patterns(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let pdas = state.database.get_program_pdas(&program_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch PDAs: {}", e)))?;

    let patterns: Vec<String> = pdas.into_iter()
        .filter_map(|pda| pda.pattern)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    Ok(ApiResponse::success(patterns))
}

pub async fn get_program_pdas(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
    Query(query): Query<PdaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let pdas = state.database.get_program_pdas(&program_id).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch PDAs: {}", e)))?;

    let limit = query.limit.unwrap_or(50).min(500) as usize;
    let offset = query.offset.unwrap_or(0) as usize;

    let paginated_pdas = pdas.into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();

    Ok(ApiResponse::success(paginated_pdas))
}

// Transaction handlers
pub async fn list_transactions(
    State(_state): State<AppState>,
    Query(_query): Query<TransactionQuery>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement transaction listing
    Ok(ApiResponse::success(Vec::<serde_json::Value>::new()))
}

pub async fn get_transaction(
    State(_state): State<AppState>,
    Path(_signature): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement transaction details
    Err(ApiError::NotImplemented("Transaction details not implemented yet".to_string()))
}

pub async fn analyze_transaction(
    State(_state): State<AppState>,
    Json(_request): Json<AnalyzeTransactionRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: Implement transaction analysis
    Err(ApiError::NotImplemented("Transaction analysis not implemented yet".to_string()))
}

// PDA handlers
pub async fn list_pdas(
    State(state): State<AppState>,
    Query(query): Query<PdaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(50).min(500) as i64;
    let pdas = state.database.get_recent_pdas(limit).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch PDAs: {}", e)))?;

    Ok(ApiResponse::success(pdas))
}

pub async fn get_pda(
    State(_state): State<AppState>,
    Path(_address): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    // This endpoint needs both address and program_id, but we only have address
    // We'll need to search for any PDA with this address
    // For now, return not implemented
    Err(ApiError::NotImplemented("PDA lookup by address only not implemented yet".to_string()))
}

pub async fn search_pdas(
    State(state): State<AppState>,
    Query(query): Query<PdaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(50).min(500) as i64;

    let pdas = if let Some(pattern) = query.pattern {
        state.database.search_pdas_by_pattern(&pattern, limit).await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to search PDAs: {}", e)))?
    } else {
        state.database.get_recent_pdas(limit).await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch PDAs: {}", e)))?
    };

    Ok(ApiResponse::success(pdas))
}

pub async fn get_recent_pdas(
    State(state): State<AppState>,
    Query(query): Query<PdaQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(50).min(500) as i64;
    let pdas = state.database.get_recent_pdas(limit).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch recent PDAs: {}", e)))?;

    Ok(ApiResponse::success(pdas))
}

// Analytics handlers
pub async fn get_database_metrics(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let stats = state.database.get_stats().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch database stats: {}", e)))?;

    Ok(ApiResponse::success(stats))
}

pub async fn get_pattern_distribution(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let stats = state.database.get_stats().await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch pattern distribution: {}", e)))?;

    Ok(ApiResponse::success(stats.patterns_distribution))
}

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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_pda_request_deserialization() {
        let json = r#"{"address": "11111111111111111111111111111111", "program_id": "11111111111111111111111111111111"}"#;
        let request: AnalyzePdaRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.address, "11111111111111111111111111111111");
        assert_eq!(request.program_id, "11111111111111111111111111111111");
    }
}