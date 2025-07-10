use crate::{ApiResult, ApiError, ApiResponse};
use axum::{
    extract::{Path, Query, State},
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use solana_pda_analyzer_database::{DatabaseRepository, ProgramFilter, TransactionFilter, PdaFilter};
use solana_pda_analyzer_core::PdaAnalyzer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseRepository,
    pub pda_analyzer: Arc<RwLock<PdaAnalyzer>>,
}

// Request/Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzePdaRequest {
    pub address: String,
    pub program_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzePdaResponse {
    pub address: String,
    pub program_id: String,
    pub seeds: Option<Vec<serde_json::Value>>,
    pub bump: Option<u8>,
    pub derived_successfully: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeTransactionRequest {
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAnalyzePdaRequest {
    pub addresses: Vec<AnalyzePdaRequest>,
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
}

// Health check handler
pub async fn health_check() -> impl IntoResponse {
    ApiResponse::success("Service is healthy")
}

// PDA analysis handlers
pub async fn analyze_pda(
    State(state): State<AppState>,
    Json(request): Json<AnalyzePdaRequest>,
) -> ApiResult<impl IntoResponse> {
    let address = Pubkey::from_str(&request.address)
        .map_err(|_| ApiError::bad_request("Invalid address".to_string()))?;
    
    let program_id = Pubkey::from_str(&request.program_id)
        .map_err(|_| ApiError::bad_request("Invalid program ID".to_string()))?;

    let mut analyzer = state.pda_analyzer.write().await;
    let analysis_result = analyzer.analyze_pda(&address, &program_id)
        .map_err(ApiError::from)?;

    let response = match analysis_result {
        Some(pda_info) => AnalyzePdaResponse {
            address: request.address,
            program_id: request.program_id,
            seeds: Some(pda_info.seeds.iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
            bump: Some(pda_info.bump),
            derived_successfully: true,
        },
        None => AnalyzePdaResponse {
            address: request.address,
            program_id: request.program_id,
            seeds: None,
            bump: None,
            derived_successfully: false,
        },
    };

    Ok(ApiResponse::success(response))
}

pub async fn batch_analyze_pda(
    State(state): State<AppState>,
    Json(request): Json<BatchAnalyzePdaRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut results = Vec::new();
    let mut analyzer = state.pda_analyzer.write().await;

    for pda_request in request.addresses {
        let address = Pubkey::from_str(&pda_request.address)
            .map_err(|_| ApiError::bad_request(format!("Invalid address: {}", pda_request.address)))?;
        
        let program_id = Pubkey::from_str(&pda_request.program_id)
            .map_err(|_| ApiError::bad_request(format!("Invalid program ID: {}", pda_request.program_id)))?;

        let analysis_result = analyzer.analyze_pda(&address, &program_id)
            .map_err(ApiError::from)?;

        let response = match analysis_result {
            Some(pda_info) => AnalyzePdaResponse {
                address: pda_request.address,
                program_id: pda_request.program_id,
                seeds: Some(pda_info.seeds.iter().map(|s| serde_json::to_value(s).unwrap()).collect()),
                bump: Some(pda_info.bump),
                derived_successfully: true,
            },
            None => AnalyzePdaResponse {
                address: pda_request.address,
                program_id: pda_request.program_id,
                seeds: None,
                bump: None,
                derived_successfully: false,
            },
        };

        results.push(response);
    }

    Ok(ApiResponse::success(results))
}

// Program handlers
pub async fn list_programs(
    State(state): State<AppState>,
    Query(query): Query<ProgramQuery>,
) -> ApiResult<impl IntoResponse> {
    let filter = ProgramFilter {
        program_id: None,
        name: query.name,
        limit: query.limit,
        offset: query.offset,
    };

    let programs = state.database.list_programs(filter).await.map_err(ApiError::from)?;
    Ok(ApiResponse::success(programs))
}

pub async fn get_program(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let program = state.database.get_program_by_id(&program_id).await.map_err(ApiError::from)?;
    
    match program {
        Some(program) => Ok(ApiResponse::success(program)),
        None => Err(ApiError::not_found("Program not found".to_string())),
    }
}

pub async fn get_program_stats(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let program_uuid = Uuid::from_str(&program_id)
        .map_err(|_| ApiError::bad_request("Invalid program ID format".to_string()))?;

    let stats = state.database.get_program_stats(program_uuid).await.map_err(ApiError::from)?;
    Ok(ApiResponse::success(stats))
}

// Transaction handlers
pub async fn list_transactions(
    State(state): State<AppState>,
    Query(query): Query<TransactionQuery>,
) -> ApiResult<impl IntoResponse> {
    let filter = TransactionFilter {
        signature: None,
        slot_range: match (query.min_slot, query.max_slot) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        },
        success: query.success,
        limit: query.limit,
        offset: query.offset,
    };

    let transactions = state.database.list_transactions(filter).await.map_err(ApiError::from)?;
    Ok(ApiResponse::success(transactions))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(signature): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let transaction = state.database.get_transaction_by_signature(&signature).await.map_err(ApiError::from)?;
    
    match transaction {
        Some(transaction) => Ok(ApiResponse::success(transaction)),
        None => Err(ApiError::not_found("Transaction not found".to_string())),
    }
}

// PDA handlers
pub async fn list_pdas(
    State(state): State<AppState>,
    Query(query): Query<PdaQuery>,
) -> ApiResult<impl IntoResponse> {
    let program_id = match query.program_id {
        Some(id) => Some(Uuid::from_str(&id)
            .map_err(|_| ApiError::bad_request("Invalid program ID format".to_string()))?),
        None => None,
    };

    let filter = PdaFilter {
        address: None,
        program_id,
        limit: query.limit,
        offset: query.offset,
    };

    let pdas = state.database.list_pdas(filter).await.map_err(ApiError::from)?;
    Ok(ApiResponse::success(pdas))
}

pub async fn get_pda(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let pda = state.database.get_pda_by_address(&address).await.map_err(ApiError::from)?;
    
    match pda {
        Some(pda) => Ok(ApiResponse::success(pda)),
        None => Err(ApiError::not_found("PDA not found".to_string())),
    }
}

// Analytics handlers
pub async fn get_database_metrics(
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let metrics = state.database.get_database_metrics().await.map_err(ApiError::from)?;
    Ok(ApiResponse::success(metrics))
}

// Pattern analysis handlers
pub async fn analyze_program_patterns(
    State(state): State<AppState>,
    Path(program_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let program_uuid = Uuid::from_str(&program_id)
        .map_err(|_| ApiError::bad_request("Invalid program ID format".to_string()))?;

    let filter = PdaFilter {
        address: None,
        program_id: Some(program_uuid),
        limit: None,
        offset: None,
    };

    let pdas = state.database.list_pdas(filter).await.map_err(ApiError::from)?;
    
    // Convert database records to core types (simplified)
    let pda_infos = pdas.into_iter().map(|pda| {
        solana_pda_analyzer_core::PdaInfo {
            address: Pubkey::from_str(&pda.address).unwrap_or_default(),
            program_id: Pubkey::new_unique(), // Would need to resolve this from database
            seeds: Vec::new(), // Would need to deserialize from JSON
            bump: pda.bump as u8,
            first_seen_slot: None,
            first_seen_transaction: None,
        }
    }).collect::<Vec<_>>();

    // Use pattern registry to analyze patterns
    let mut pattern_registry = solana_pda_analyzer_analyzer::PatternRegistry::new();
    let program_pubkey = Pubkey::new_unique(); // Would need to resolve from database
    let patterns = pattern_registry.detect_patterns(&program_pubkey, &pda_infos)
        .map_err(ApiError::from)?;

    Ok(ApiResponse::success(patterns))
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

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }
}