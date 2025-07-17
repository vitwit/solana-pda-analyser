//! API Client Tests for Solana PDA Analyzer
//! 
//! This module contains tests for the API client functionality,
//! including comprehensive unit tests for request/response handling,
//! error scenarios, and client configuration.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use anyhow::{Result, Context};
use colored::*;

/// API response wrapper
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    message: Option<String>,
}

/// PDA analysis request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalyzePdaRequest {
    pub address: String,
    pub program_id: String,
}

/// Batch PDA analysis request
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchAnalyzePdaRequest {
    pub pdas: Vec<AnalyzePdaRequest>,
}

/// PDA analysis result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PdaAnalysisResult {
    pub address: String,
    pub program_id: String,
    pub pattern: Option<String>,
    pub confidence: Option<f64>,
    pub seeds: Vec<SeedValue>,
    pub bump: u8,
    pub analysis_time_ms: u64,
}

/// Seed value types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum SeedValue {
    String(String),
    Bytes(Vec<u8>),
    Pubkey(String),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
}

/// Program information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgramInfo {
    pub program_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub total_pdas: i64,
    pub total_transactions: i64,
}

/// Database statistics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseStats {
    pub total_programs: i64,
    pub total_pdas: i64,
    pub total_transactions: i64,
    pub total_interactions: i64,
}

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

/// API client for interacting with the Solana PDA Analyzer
pub struct ApiClient {
    client: Client,
    config: ApiClientConfig,
    stats: Arc<RwLock<ClientStats>>,
}

#[derive(Debug, Default)]
struct ClientStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_response_time: Duration,
    cache_hits: u64,
    cache_misses: u64,
}

impl ApiClient {
    pub fn new(config: ApiClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            config,
            stats: Arc::new(RwLock::new(ClientStats::default())),
        })
    }

    pub fn new_with_default_config() -> Result<Self> {
        Self::new(ApiClientConfig::default())
    }

    async fn record_request(&self, success: bool, response_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.total_response_time += response_time;
        
        if success {
            stats.successful_requests += 1;
        } else {
            stats.failed_requests += 1;
        }
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<T> {
        let start = std::time::Instant::now();
        let url = format!("{}{}", self.config.base_url, endpoint);
        
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_retries {
            let mut request = match method {
                "GET" => self.client.get(&url),
                "POST" => self.client.post(&url),
                "PUT" => self.client.put(&url),
                "DELETE" => self.client.delete(&url),
                _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
            };

            if let Some(ref body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    
                    if response.status().is_success() {
                        match response.json::<ApiResponse<T>>().await {
                            Ok(api_response) => {
                                self.record_request(true, elapsed).await;
                                
                                if api_response.success {
                                    return api_response.data.ok_or_else(|| {
                                        anyhow::anyhow!("API response missing data field")
                                    });
                                } else {
                                    let error_msg = api_response.error
                                        .or(api_response.message)
                                        .unwrap_or_else(|| "Unknown API error".to_string());
                                    return Err(anyhow::anyhow!("API error: {}", error_msg));
                                }
                            }
                            Err(e) => {
                                last_error = Some(anyhow::anyhow!("Failed to parse response: {}", e));
                            }
                        }
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        last_error = Some(anyhow::anyhow!("HTTP {}: {}", status, error_text));
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));
                }
            }
            
            if attempt < self.config.max_retries {
                tokio::time::sleep(self.config.retry_delay).await;
            }
        }
        
        let elapsed = start.elapsed();
        self.record_request(false, elapsed).await;
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Check if the API server is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let health: serde_json::Value = self.make_request("GET", "/health", None).await?;
        Ok(health.get("status").and_then(|s| s.as_str()) == Some("healthy"))
    }

    /// Analyze a single PDA
    pub async fn analyze_pda(&self, address: &str, program_id: &str) -> Result<PdaAnalysisResult> {
        let request = AnalyzePdaRequest {
            address: address.to_string(),
            program_id: program_id.to_string(),
        };

        let body = serde_json::to_value(&request)?;
        self.make_request("POST", "/api/v1/analyze/pda", Some(body)).await
    }

    /// Analyze multiple PDAs in batch
    pub async fn analyze_pdas_batch(&self, pdas: Vec<AnalyzePdaRequest>) -> Result<Vec<Option<PdaAnalysisResult>>> {
        let request = BatchAnalyzePdaRequest { pdas };
        let body = serde_json::to_value(&request)?;
        self.make_request("POST", "/api/v1/analyze/pda/batch", Some(body)).await
    }

    /// List all programs
    pub async fn list_programs(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ProgramInfo>> {
        let mut endpoint = "/api/v1/programs".to_string();
        let mut params = Vec::new();
        
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = offset {
            params.push(format!("offset={}", offset));
        }
        
        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }
        
        self.make_request("GET", &endpoint, None).await
    }

    /// Get program details
    pub async fn get_program(&self, program_id: &str) -> Result<ProgramInfo> {
        let endpoint = format!("/api/v1/programs/{}", program_id);
        self.make_request("GET", &endpoint, None).await
    }

    /// List PDAs with optional filtering
    pub async fn list_pdas(&self, limit: Option<i64>, program_id: Option<&str>) -> Result<Vec<Value>> {
        let mut endpoint = "/api/v1/pdas".to_string();
        let mut params = Vec::new();
        
        if let Some(limit) = limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(program_id) = program_id {
            params.push(format!("program_id={}", program_id));
        }
        
        if !params.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&params.join("&"));
        }
        
        self.make_request("GET", &endpoint, None).await
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats> {
        self.make_request("GET", "/api/v1/analytics/database", None).await
    }

    /// Get performance metrics
    pub async fn get_performance_metrics(&self) -> Result<HashMap<String, Value>> {
        self.make_request("GET", "/api/v1/analytics/performance", None).await
    }

    /// Get client statistics
    pub async fn get_stats(&self) -> ClientStats {
        self.stats.read().await.clone()
    }

    /// Reset client statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ClientStats::default();
    }
}

impl Clone for ClientStats {
    fn clone(&self) -> Self {
        Self {
            total_requests: self.total_requests,
            successful_requests: self.successful_requests,
            failed_requests: self.failed_requests,
            total_response_time: self.total_response_time,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
        }
    }
}

/// Test harness for API client functionality
pub struct ApiClientTester {
    client: ApiClient,
    test_results: Vec<TestResult>,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    success: bool,
    duration: Duration,
    error: Option<String>,
    details: Option<String>,
}

impl ApiClientTester {
    pub fn new() -> Result<Self> {
        let config = ApiClientConfig::default();
        let client = ApiClient::new(config)?;
        
        Ok(Self {
            client,
            test_results: Vec::new(),
        })
    }

    pub fn new_with_url(base_url: String) -> Result<Self> {
        let config = ApiClientConfig {
            base_url,
            ..Default::default()
        };
        let client = ApiClient::new(config)?;
        
        Ok(Self {
            client,
            test_results: Vec::new(),
        })
    }

    fn log_info(&self, message: &str) {
        println!("{} {}", "[INFO]".blue(), message);
    }

    fn log_success(&self, message: &str) {
        println!("{} {}", "[SUCCESS]".green(), message);
    }

    fn log_error(&self, message: &str) {
        println!("{} {}", "[ERROR]".red(), message);
    }

    async fn run_test<F, Fut>(&mut self, name: &str, test_fn: F) 
    where
        F: FnOnce(&ApiClient) -> Fut,
        Fut: std::future::Future<Output = Result<String>>,
    {
        self.log_info(&format!("Running test: {}", name));
        let start = std::time::Instant::now();
        
        match test_fn(&self.client).await {
            Ok(details) => {
                let duration = start.elapsed();
                self.log_success(&format!("{} passed ({:.3}s)", name, duration.as_secs_f64()));
                
                let result = TestResult {
                    name: name.to_string(),
                    success: true,
                    duration,
                    error: None,
                    details: Some(details),
                };
                self.test_results.push(result);
            }
            Err(e) => {
                let duration = start.elapsed();
                self.log_error(&format!("{} failed ({:.3}s): {}", name, duration.as_secs_f64(), e));
                
                let result = TestResult {
                    name: name.to_string(),
                    success: false,
                    duration,
                    error: Some(e.to_string()),
                    details: None,
                };
                self.test_results.push(result);
            }
        }
    }

    pub async fn test_client_configuration(&mut self) -> Result<()> {
        self.run_test("Client Configuration", |_client| async {
            // Test that client can be created with custom configuration
            let custom_config = ApiClientConfig {
                base_url: "http://localhost:8080".to_string(),
                timeout: Duration::from_secs(10),
                max_retries: 5,
                retry_delay: Duration::from_millis(100),
            };
            
            let _custom_client = ApiClient::new(custom_config)?;
            Ok("Client created with custom configuration".to_string())
        }).await;
        
        Ok(())
    }

    pub async fn test_health_check(&mut self) -> Result<()> {
        self.run_test("Health Check", |client| async {
            let is_healthy = client.health_check().await?;
            if is_healthy {
                Ok("Server is healthy".to_string())
            } else {
                Err(anyhow::anyhow!("Server is not healthy"))
            }
        }).await;
        
        Ok(())
    }

    pub async fn test_pda_analysis(&mut self) -> Result<()> {
        self.run_test("PDA Analysis", |client| async {
            let result = client.analyze_pda(
                "11111111111111111111111111111111",
                "11111111111111111111111111111111"
            ).await?;
            
            Ok(format!("PDA analyzed: {}, pattern: {:?}", 
                result.address, result.pattern))
        }).await;
        
        Ok(())
    }

    pub async fn test_batch_analysis(&mut self) -> Result<()> {
        self.run_test("Batch Analysis", |client| async {
            let pdas = vec![
                AnalyzePdaRequest {
                    address: "11111111111111111111111111111111".to_string(),
                    program_id: "11111111111111111111111111111111".to_string(),
                },
                AnalyzePdaRequest {
                    address: "22222222222222222222222222222222".to_string(),
                    program_id: "11111111111111111111111111111111".to_string(),
                },
            ];
            
            let results = client.analyze_pdas_batch(pdas).await?;
            Ok(format!("Batch analysis completed: {} results", results.len()))
        }).await;
        
        Ok(())
    }

    pub async fn test_list_programs(&mut self) -> Result<()> {
        self.run_test("List Programs", |client| async {
            let programs = client.list_programs(Some(10), None).await?;
            Ok(format!("Found {} programs", programs.len()))
        }).await;
        
        Ok(())
    }

    pub async fn test_list_pdas(&mut self) -> Result<()> {
        self.run_test("List PDAs", |client| async {
            let pdas = client.list_pdas(Some(10), None).await?;
            Ok(format!("Found {} PDAs", pdas.len()))
        }).await;
        
        Ok(())
    }

    pub async fn test_database_stats(&mut self) -> Result<()> {
        self.run_test("Database Stats", |client| async {
            let stats = client.get_database_stats().await?;
            Ok(format!("Database stats: {} programs, {} PDAs, {} transactions", 
                stats.total_programs, stats.total_pdas, stats.total_transactions))
        }).await;
        
        Ok(())
    }

    pub async fn test_performance_metrics(&mut self) -> Result<()> {
        self.run_test("Performance Metrics", |client| async {
            let metrics = client.get_performance_metrics().await?;
            Ok(format!("Performance metrics retrieved: {} entries", metrics.len()))
        }).await;
        
        Ok(())
    }

    pub async fn test_error_handling(&mut self) -> Result<()> {
        self.run_test("Error Handling", |client| async {
            // Test with invalid PDA address
            let result = client.analyze_pda("invalid_address", "11111111111111111111111111111111").await;
            
            match result {
                Err(e) => {
                    if e.to_string().contains("Invalid") || e.to_string().contains("400") {
                        Ok("Correctly handled invalid PDA address".to_string())
                    } else {
                        Err(anyhow::anyhow!("Unexpected error: {}", e))
                    }
                }
                Ok(_) => Err(anyhow::anyhow!("Expected error for invalid address, but got success")),
            }
        }).await;
        
        Ok(())
    }

    pub async fn test_client_stats(&mut self) -> Result<()> {
        self.run_test("Client Stats", |client| async {
            // Reset stats
            client.reset_stats().await;
            
            // Make a few requests
            let _ = client.health_check().await;
            let _ = client.list_programs(Some(5), None).await;
            
            let stats = client.get_stats().await;
            
            if stats.total_requests > 0 {
                Ok(format!("Client stats: {} total requests, {} successful", 
                    stats.total_requests, stats.successful_requests))
            } else {
                Err(anyhow::anyhow!("No requests recorded in stats"))
            }
        }).await;
        
        Ok(())
    }

    pub async fn test_concurrent_requests(&mut self) -> Result<()> {
        self.run_test("Concurrent Requests", |client| async {
            let mut handles = Vec::new();
            
            // Send 5 concurrent health checks
            for _ in 0..5 {
                let client = client.clone();
                let handle = tokio::spawn(async move {
                    client.health_check().await
                });
                handles.push(handle);
            }
            
            let mut success_count = 0;
            for handle in handles {
                match handle.await {
                    Ok(Ok(true)) => success_count += 1,
                    _ => {}
                }
            }
            
            if success_count == 5 {
                Ok("All 5 concurrent requests succeeded".to_string())
            } else {
                Err(anyhow::anyhow!("Only {}/5 concurrent requests succeeded", success_count))
            }
        }).await;
        
        Ok(())
    }

    pub async fn run_all_tests(&mut self) -> Result<()> {
        self.log_info("Starting API client tests...");
        
        self.test_client_configuration().await?;
        self.test_health_check().await?;
        self.test_pda_analysis().await?;
        self.test_batch_analysis().await?;
        self.test_list_programs().await?;
        self.test_list_pdas().await?;
        self.test_database_stats().await?;
        self.test_performance_metrics().await?;
        self.test_error_handling().await?;
        self.test_client_stats().await?;
        self.test_concurrent_requests().await?;
        
        Ok(())
    }

    pub fn print_summary(&self) {
        let passed = self.test_results.iter().filter(|r| r.success).count();
        let total = self.test_results.len();

        println!("\n{}", "=".repeat(60));
        println!("API CLIENT TEST SUMMARY");
        println!("{}", "=".repeat(60));

        for result in &self.test_results {
            let status = if result.success { "PASS".green() } else { "FAIL".red() };
            println!("{} {} ({:.3}s)", status, result.name, result.duration.as_secs_f64());
            
            if let Some(details) = &result.details {
                println!("      Details: {}", details);
            }
            
            if let Some(error) = &result.error {
                println!("      Error: {}", error);
            }
        }

        println!("\n{}", "-".repeat(60));
        let overall_color = if passed == total { "green" } else { "red" };
        
        match overall_color {
            "green" => println!("{}", format!("Overall: {}/{} tests passed", passed, total).green()),
            _ => println!("{}", format!("Overall: {}/{} tests passed", passed, total).red()),
        }

        if passed == total {
            println!("{}", "All API client tests passed!".green());
        } else {
            println!("{}", format!("{} tests failed", total - passed).red());
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    println!("{}", "Solana PDA Analyzer - API Client Tests".blue().bold());
    println!("Target URL: {}", base_url);
    println!();

    let mut tester = ApiClientTester::new_with_url(base_url)?;
    tester.run_all_tests().await?;
    tester.print_summary();

    let passed = tester.test_results.iter().filter(|r| r.success).count();
    let total = tester.test_results.len();

    std::process::exit(if passed == total { 0 } else { 1 });
}