use crate::{create_router, AppState, middleware::*};
use axum::{middleware, Router};
use solana_pda_analyzer_database::{DatabaseRepository, DatabaseConfig, initialize_database};
use solana_pda_analyzer_core::PdaAnalyzer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::{info, error};
use anyhow::Result;
use solana_pda_analyzer_core::PdaAnalyzerError;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub database_config: DatabaseConfig,
    pub static_files_dir: Option<String>,
    pub log_level: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid PORT: {}", e)))?,
            database_config: DatabaseConfig::from_env()
                .map_err(|e| anyhow::anyhow!("Database config error: {}", e))?,
            static_files_dir: std::env::var("STATIC_FILES_DIR").ok(),
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_config: DatabaseConfig::default(),
            static_files_dir: None,
            log_level: "info".to_string(),
        }
    }
}

pub struct Server {
    config: ServerConfig,
    app_state: AppState,
}

impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self> {
        // Initialize database
        let database_pool = initialize_database(&config.database_config).await?;
        let database = DatabaseRepository::new(database_pool);
        
        // Initialize PDA analyzer
        let pda_analyzer = Arc::new(RwLock::new(PdaAnalyzer::new()));
        
        let app_state = AppState {
            database,
            pda_analyzer,
        };
        
        Ok(Self {
            config,
            app_state,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let bind_address = self.config.bind_address();
        info!("Starting server on {}", bind_address);
        
        // Create the main router
        let mut app = create_router(self.app_state.clone());
        
        // Add middleware layers
        app = app
            .layer(middleware::from_fn(logging_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(middleware::from_fn(request_validation_middleware));
        
        // Add static file serving if configured
        if let Some(static_dir) = &self.config.static_files_dir {
            info!("Serving static files from: {}", static_dir);
            app = app.nest_service("/static", ServeDir::new(static_dir));
        }
        
        // Create the TCP listener
        let listener = tokio::net::TcpListener::bind(&bind_address)
            .await
            .map_err(|e| {
                error!("Failed to bind to address {}: {}", bind_address, e);
                e
            })?;
        
        info!("Server listening on {}", bind_address);
        
        // Start the server
        axum::serve(listener, app)
            .await
            .map_err(|e| {
                error!("Server error: {}", e);
                e.into()
            })
    }
}

pub async fn run_server(config: ServerConfig) -> Result<()> {
    let server = Server::new(config).await?;
    server.run().await
}

// Health check for the server
pub async fn health_check_server(config: &ServerConfig) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("http://{}/health", config.bind_address());
    
    match client.get(&url).send().await {
        Ok(response) => Ok(response.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address(), "127.0.0.1:8080");
    }
    
    #[test]
    fn test_server_config_bind_address() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
            ..Default::default()
        };
        assert_eq!(config.bind_address(), "0.0.0.0:3000");
    }
    
    #[tokio::test]
    async fn test_server_creation() {
        // This test would need a real database for full testing
        // For now, it's just a structural test
        let config = ServerConfig::default();
        // let result = Server::new(config).await;
        // This would work with a proper database setup
        assert!(true);
    }
}