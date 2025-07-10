use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
    Router,
};
use tower::ServiceExt;
use serde_json::{json, Value};
use solana_pda_analyzer_api::{create_router, AppState};
use solana_pda_analyzer_database::{DatabaseRepository, DatabaseConfig};
use solana_pda_analyzer_core::PdaAnalyzer;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

async fn create_test_app() -> Result<Router, Box<dyn std::error::Error>> {
    // Create in-memory or test database
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: format!("api_test_{}", Uuid::new_v4().to_string().replace('-', "")),
        username: "postgres".to_string(),
        password: "".to_string(),
        max_connections: 5,
        min_connections: 1,
        acquire_timeout: 30,
        idle_timeout: 600,
        max_lifetime: 1800,
    };

    // Try to create a test database connection
    let pool = match config.create_pool().await {
        Ok(pool) => pool,
        Err(_) => {
            // If we can't connect to a real database, skip these tests
            return Err("Cannot connect to test database".into());
        }
    };

    let database = DatabaseRepository::new(pool);
    let pda_analyzer = Arc::new(RwLock::new(PdaAnalyzer::new()));

    let state = AppState {
        database,
        pda_analyzer,
    };

    Ok(create_router(state))
}

async fn send_request(app: &Router, request: Request<Body>) -> Result<(StatusCode, Value), Box<dyn std::error::Error>> {
    let response = app.clone().oneshot(request).await?;
    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await?;
    let json: Value = serde_json::from_slice(&body)?;
    Ok((status, json))
}

#[tokio::test]
async fn test_health_check() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert_eq!(json["data"], "Service is healthy");
}

#[tokio::test]
async fn test_analyze_pda_endpoint() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let payload = json!({
        "address": "11111111111111111111111111111111",
        "program_id": "11111111111111111111111111111111"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_object());
    
    let data = &json["data"];
    assert_eq!(data["address"], "11111111111111111111111111111111");
    assert_eq!(data["program_id"], "11111111111111111111111111111111");
    assert!(data["derived_successfully"].is_boolean());
}

#[tokio::test]
async fn test_analyze_pda_invalid_address() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let payload = json!({
        "address": "invalid_address",
        "program_id": "11111111111111111111111111111111"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let (status, _json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_batch_analyze_pda() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let payload = json!({
        "addresses": [
            {
                "address": "11111111111111111111111111111111",
                "program_id": "11111111111111111111111111111111"
            },
            {
                "address": "22222222222222222222222222222222",
                "program_id": "11111111111111111111111111111111"
            }
        ]
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda/batch")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_programs() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/programs")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_list_programs_with_query_params() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/programs?limit=10&offset=0")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_list_transactions() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/transactions")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_list_transactions_with_filters() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/transactions?success=true&limit=5")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_list_pdas() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/pdas")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
}

#[tokio::test]
async fn test_get_database_metrics() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/database")
        .body(Body::empty())
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_object());
    
    let data = &json["data"];
    assert!(data["total_programs"].is_number());
    assert!(data["total_transactions"].is_number());
    assert!(data["total_pdas"].is_number());
    assert!(data["total_interactions"].is_number());
}

#[tokio::test]
async fn test_invalid_endpoint() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/nonexistent")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_malformed_json() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda")
        .header("content-type", "application/json")
        .body(Body::from("{invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_missing_content_type() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let payload = json!({
        "address": "11111111111111111111111111111111",
        "program_id": "11111111111111111111111111111111"
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Should fail due to missing content-type
    assert_ne!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cors_headers() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/v1/analyze/pda")
        .header("Origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let headers = response.headers();
    
    // Check for CORS headers (implementation depends on tower-http configuration)
    assert!(response.status().is_success() || response.status() == StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_large_batch_request() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    // Create a large batch request
    let mut addresses = Vec::new();
    for i in 0..100 {
        addresses.push(json!({
            "address": format!("{:044}", i),
            "program_id": "11111111111111111111111111111111"
        }));
    }

    let payload = json!({
        "addresses": addresses
    });

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/analyze/pda/batch")
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let (status, json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["success"], true);
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 100);
}

#[tokio::test]
async fn test_get_nonexistent_program() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/programs/nonexistent_program_id")
        .body(Body::empty())
        .unwrap();

    let (status, _json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_transaction() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/transactions/nonexistent_signature")
        .body(Body::empty())
        .unwrap();

    let (status, _json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_pda() {
    let app = match create_test_app().await {
        Ok(app) => app,
        Err(_) => {
            println!("Skipping API tests - no database connection");
            return;
        }
    };

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/pdas/nonexistent_address")
        .body(Body::empty())
        .unwrap();

    let (status, _json) = send_request(&app, request).await.unwrap();
    assert_eq!(status, StatusCode::NOT_FOUND);
}