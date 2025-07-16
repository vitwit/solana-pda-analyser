use crate::{create_simple_router, AppState, middleware::*};
use axum::middleware;
use solana_pda_analyzer_core::PdaAnalyzer;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::{info, error};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SimpleServerConfig {
    pub host: String,
    pub port: u16,
    pub static_files_dir: Option<String>,
}

impl SimpleServerConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            static_files_dir: std::env::var("STATIC_FILES_DIR").ok(),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Default for SimpleServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            static_files_dir: None,
        }
    }
}

pub struct SimpleServer {
    config: SimpleServerConfig,
    app_state: AppState,
}

impl SimpleServer {
    pub async fn new(config: SimpleServerConfig) -> Result<Self> {
        // Initialize PDA analyzer
        let pda_analyzer = Arc::new(RwLock::new(PdaAnalyzer::new()));
        
        let app_state = AppState {
            pda_analyzer,
        };
        
        Ok(Self {
            config,
            app_state,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let bind_address = self.config.bind_address();
        info!("Starting PDA Analyzer API server on {}", bind_address);
        
        // Create the main router
        let mut app = create_simple_router(self.app_state.clone());
        
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
        
        info!("🚀 PDA Analyzer API server listening on {}", bind_address);
        info!("📖 API Documentation: http://{}/docs", bind_address);
        info!("❤️  Health Check: http://{}/health", bind_address);
        info!("🔍 Try analyzing a PDA: POST http://{}/api/v1/analyze/pda", bind_address);
        
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

pub async fn run_simple_server(config: SimpleServerConfig) -> Result<()> {
    let server = SimpleServer::new(config).await?;
    server.run().await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_server_config_default() {
        let config = SimpleServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address(), "127.0.0.1:8080");
    }
}