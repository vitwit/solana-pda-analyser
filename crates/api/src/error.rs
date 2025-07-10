use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use solana_pda_analyzer_core::PdaAnalyzerError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

impl ApiError {
    pub fn new(error: String, message: String, status_code: StatusCode) -> Self {
        Self {
            error,
            message,
            status_code: status_code.as_u16(),
        }
    }

    pub fn bad_request(message: String) -> Self {
        Self::new("Bad Request".to_string(), message, StatusCode::BAD_REQUEST)
    }

    pub fn not_found(message: String) -> Self {
        Self::new("Not Found".to_string(), message, StatusCode::NOT_FOUND)
    }

    pub fn internal_server_error(message: String) -> Self {
        Self::new("Internal Server Error".to_string(), message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn unprocessable_entity(message: String) -> Self {
        Self::new("Unprocessable Entity".to_string(), message, StatusCode::UNPROCESSABLE_ENTITY)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status_code, Json(self)).into_response()
    }
}

impl From<PdaAnalyzerError> for ApiError {
    fn from(err: PdaAnalyzerError) -> Self {
        match err {
            PdaAnalyzerError::InvalidSeedData(msg) => ApiError::bad_request(msg),
            PdaAnalyzerError::PdaDerivationFailed(msg) => ApiError::unprocessable_entity(msg),
            PdaAnalyzerError::InvalidProgramId(msg) => ApiError::bad_request(msg),
            PdaAnalyzerError::InvalidPublicKey(msg) => ApiError::bad_request(msg),
            PdaAnalyzerError::TransactionParsingError(msg) => ApiError::unprocessable_entity(msg),
            PdaAnalyzerError::DatabaseError(msg) => ApiError::internal_server_error(msg),
            PdaAnalyzerError::SerializationError(msg) => ApiError::internal_server_error(msg),
            PdaAnalyzerError::NetworkError(msg) => ApiError::internal_server_error(msg),
            PdaAnalyzerError::ConfigurationError(msg) => ApiError::internal_server_error(msg),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::internal_server_error(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::bad_request(err.to_string())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}