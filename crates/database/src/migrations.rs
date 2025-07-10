use sqlx::{PgPool, migrate::MigrateDatabase, Postgres};
use solana_pda_analyzer_core::{PdaAnalyzerError, Result};
use tracing::{info, error};

pub struct DatabaseMigrator {
    database_url: String,
}

impl DatabaseMigrator {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }

    pub async fn ensure_database_exists(&self) -> Result<()> {
        if !Postgres::database_exists(&self.database_url).await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
        {
            info!("Creating database...");
            Postgres::create_database(&self.database_url).await
                .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;
            info!("Database created successfully");
        } else {
            info!("Database already exists");
        }
        Ok(())
    }

    pub async fn run_migrations(&self, pool: &PgPool) -> Result<()> {
        info!("Running database migrations...");
        
        // Run the initial schema migration
        let migration_sql = include_str!("../../../migrations/001_initial_schema.sql");
        
        // Split by semicolon and execute each statement
        let statements: Vec<&str> = migration_sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();

        for statement in statements {
            if !statement.is_empty() {
                sqlx::query(statement)
                    .execute(pool)
                    .await
                    .map_err(|e| {
                        error!("Failed to execute migration statement: {}", e);
                        error!("Statement: {}", statement);
                        PdaAnalyzerError::DatabaseError(e.to_string())
                    })?;
            }
        }

        info!("Database migrations completed successfully");
        Ok(())
    }

    pub async fn setup_database(&self) -> Result<PgPool> {
        // Ensure database exists
        self.ensure_database_exists().await?;

        // Create connection pool
        let pool = PgPool::connect(&self.database_url)
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        // Run migrations
        self.run_migrations(&pool).await?;

        Ok(pool)
    }

    pub async fn reset_database(&self) -> Result<()> {
        info!("Resetting database...");
        
        if Postgres::database_exists(&self.database_url).await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?
        {
            Postgres::drop_database(&self.database_url).await
                .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;
        }
        
        self.ensure_database_exists().await?;
        
        info!("Database reset completed");
        Ok(())
    }
}

// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_PORT: {}", e)))?,
            database: std::env::var("DATABASE_NAME").unwrap_or_else(|_| "solana_pda_analyzer".to_string()),
            username: std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD").unwrap_or_else(|_| "".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_MAX_CONNECTIONS: {}", e)))?,
            min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_MIN_CONNECTIONS: {}", e)))?,
            acquire_timeout: std::env::var("DATABASE_ACQUIRE_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_ACQUIRE_TIMEOUT: {}", e)))?,
            idle_timeout: std::env::var("DATABASE_IDLE_TIMEOUT")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_IDLE_TIMEOUT: {}", e)))?,
            max_lifetime: std::env::var("DATABASE_MAX_LIFETIME")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .map_err(|e| PdaAnalyzerError::ConfigurationError(format!("Invalid DATABASE_MAX_LIFETIME: {}", e)))?,
        })
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub async fn create_pool(&self) -> Result<PgPool> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(self.acquire_timeout))
            .idle_timeout(std::time::Duration::from_secs(self.idle_timeout))
            .max_lifetime(std::time::Duration::from_secs(self.max_lifetime))
            .connect(&self.database_url())
            .await
            .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;

        Ok(pool)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "solana_pda_analyzer".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            max_connections: 10,
            min_connections: 1,
            acquire_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 1800,
        }
    }
}

// Helper functions for common database operations
pub async fn initialize_database(config: &DatabaseConfig) -> Result<PgPool> {
    let migrator = DatabaseMigrator::new(config.database_url());
    migrator.setup_database().await
}

pub async fn health_check(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| PdaAnalyzerError::DatabaseError(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "solana_pda_analyzer");
    }

    #[test]
    fn test_database_url_generation() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "test_db".to_string(),
            username: "test_user".to_string(),
            password: "test_pass".to_string(),
            ..Default::default()
        };

        let url = config.database_url();
        assert_eq!(url, "postgresql://test_user:test_pass@localhost:5432/test_db");
    }

    #[test]
    fn test_database_migrator_creation() {
        let migrator = DatabaseMigrator::new("postgresql://test".to_string());
        assert_eq!(migrator.database_url, "postgresql://test");
    }
}