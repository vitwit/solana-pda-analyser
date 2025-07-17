use crate::{create_router, middleware::*};
use crate::routes::AppState;
use axum::{middleware, Router};
use solana_pda_analyzer_core::PdaAnalyzer;
use solana_pda_analyzer_database::DatabaseRepository as DatabaseManager;
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
    pub database_url: String,
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
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:password@localhost/solana_pda_analyzer".to_string()),
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
            database_url: "postgresql://postgres:password@localhost/solana_pda_analyzer".to_string(),
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
        let database = DatabaseManager::from_url(&config.database_url).await?;
        
        // Run migrations
        database.migrate().await?;
        
        // Initialize PDA analyzer
        let pda_analyzer = Arc::new(RwLock::new(PdaAnalyzer::new()));
        
        let app_state = AppState {
            database: Arc::new(database),
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
            .layer(middleware::from_fn(cors_middleware));
        
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
        info!("API Documentation: http://{}/docs", bind_address);
        info!("Health Check: http://{}/health", bind_address);
        
        // Start the server
        axum::Server::from_tcp(listener.into_std().unwrap())
            .unwrap()
            .serve(app.into_make_service())
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
    // Simple health check without requiring HTTP client
    // In a real implementation, this would make an HTTP request
    // For now, we'll just return true if the config is valid
    Ok(!config.bind_address().is_empty())
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
}