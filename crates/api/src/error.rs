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

    pub fn BadRequest(message: String) -> Self {
        Self::new("Bad Request".to_string(), message, StatusCode::BAD_REQUEST)
    }

    pub fn NotFound(message: String) -> Self {
        Self::new("Not Found".to_string(), message, StatusCode::NOT_FOUND)
    }

    pub fn InternalServerError(message: String) -> Self {
        Self::new("Internal Server Error".to_string(), message, StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn NotImplemented(message: String) -> Self {
        Self::new("Not Implemented".to_string(), message, StatusCode::NOT_IMPLEMENTED)
    }

    pub fn UnprocessableEntity(message: String) -> Self {
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
            PdaAnalyzerError::InvalidSeedData(msg) => ApiError::BadRequest(msg),
            PdaAnalyzerError::PdaDerivationFailed(msg) => ApiError::UnprocessableEntity(msg),
            PdaAnalyzerError::InvalidProgramId(msg) => ApiError::BadRequest(msg),
            PdaAnalyzerError::InvalidPublicKey(msg) => ApiError::BadRequest(msg),
            PdaAnalyzerError::TransactionParsingError(msg) => ApiError::UnprocessableEntity(msg),
            PdaAnalyzerError::DatabaseError(msg) => ApiError::InternalServerError(msg),
            PdaAnalyzerError::SerializationError(msg) => ApiError::InternalServerError(msg),
            PdaAnalyzerError::NetworkError(msg) => ApiError::InternalServerError(msg),
            PdaAnalyzerError::ConfigurationError(msg) => ApiError::InternalServerError(msg),
        }
    }
}

// impl From<sqlx::Error> for ApiError {
//     fn from(err: sqlx::Error) -> Self {
//         ApiError::InternalServerError(err.to_string())
//     }
// }

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(err.to_string())
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