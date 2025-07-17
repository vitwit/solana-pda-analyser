//! Integration tests for the Solana PDA Analyzer API
//! 
//! These tests verify the API endpoints work correctly and handle
//! various scenarios including error conditions.

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use serde_json::Value;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use colored::*;

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnalyzePdaRequest {
    address: String,
    program_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct BatchAnalyzePdaRequest {
    pdas: Vec<AnalyzePdaRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    database_connected: bool,
    version: String,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    success: bool,
    duration: Duration,
    error: Option<String>,
    details: Option<String>,
}

impl TestResult {
    fn new(name: String, success: bool, duration: Duration) -> Self {
        Self {
            name,
            success,
            duration,
            error: None,
            details: None,
        }
    }

    fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self
    }

    fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

struct IntegrationTester {
    client: Client,
    base_url: String,
    results: Vec<TestResult>,
}

impl IntegrationTester {
    fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            results: Vec::new(),
        }
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

    fn log_warning(&self, message: &str) {
        println!("{} {}", "[WARNING]".yellow(), message);
    }

    async fn make_request(&self, method: &str, endpoint: &str, body: Option<Value>) -> Result<Response> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        if let Some(body) = body {
            request = request.json(&body);
        }

        request.send().await.context("Failed to send request")
    }

    async fn test_health_check(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Health Check".to_string();

        match self.make_request("GET", "/health", None).await {
            Ok(response) => {
                let duration = start.elapsed();
                
                if response.status().is_success() {
                    match response.json::<ApiResponse<HealthResponse>>().await {
                        Ok(health) => {
                            if health.success {
                                let details = format!("Status: {}, DB Connected: {}", 
                                    health.data.as_ref().map(|d| &d.status).unwrap_or(&"unknown".to_string()),
                                    health.data.as_ref().map(|d| d.database_connected).unwrap_or(false)
                                );
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error("Health check returned success=false".to_string());
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}", response.status()));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_pda_analysis(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "PDA Analysis".to_string();

        let request = AnalyzePdaRequest {
            address: "11111111111111111111111111111111".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
        };

        let body = serde_json::to_value(&request)?;

        match self.make_request("POST", "/api/v1/analyze/pda", Some(body)).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status.is_success() {
                    match response.json::<ApiResponse<Value>>().await {
                        Ok(api_response) => {
                            if api_response.success {
                                let details = format!("Analysis completed successfully");
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let error = api_response.error.unwrap_or("Unknown error".to_string());
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error(error);
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}: {}", status, error_text));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_batch_pda_analysis(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Batch PDA Analysis".to_string();

        let request = BatchAnalyzePdaRequest {
            pdas: vec![
                AnalyzePdaRequest {
                    address: "11111111111111111111111111111111".to_string(),
                    program_id: "11111111111111111111111111111111".to_string(),
                },
                AnalyzePdaRequest {
                    address: "22222222222222222222222222222222".to_string(),
                    program_id: "11111111111111111111111111111111".to_string(),
                },
            ],
        };

        let body = serde_json::to_value(&request)?;

        match self.make_request("POST", "/api/v1/analyze/pda/batch", Some(body)).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status.is_success() {
                    match response.json::<ApiResponse<Vec<Value>>>().await {
                        Ok(api_response) => {
                            if api_response.success {
                                let count = api_response.data.as_ref().map(|d| d.len()).unwrap_or(0);
                                let details = format!("Batch analysis completed: {} results", count);
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let error = api_response.error.unwrap_or("Unknown error".to_string());
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error(error);
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}: {}", status, error_text));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_list_programs(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "List Programs".to_string();

        match self.make_request("GET", "/api/v1/programs", None).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status.is_success() {
                    match response.json::<ApiResponse<Vec<Value>>>().await {
                        Ok(api_response) => {
                            if api_response.success {
                                let count = api_response.data.as_ref().map(|d| d.len()).unwrap_or(0);
                                let details = format!("Found {} programs", count);
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let error = api_response.error.unwrap_or("Unknown error".to_string());
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error(error);
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}: {}", status, error_text));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_list_pdas(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "List PDAs".to_string();

        match self.make_request("GET", "/api/v1/pdas?limit=10", None).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status.is_success() {
                    match response.json::<ApiResponse<Vec<Value>>>().await {
                        Ok(api_response) => {
                            if api_response.success {
                                let count = api_response.data.as_ref().map(|d| d.len()).unwrap_or(0);
                                let details = format!("Found {} PDAs", count);
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let error = api_response.error.unwrap_or("Unknown error".to_string());
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error(error);
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}: {}", status, error_text));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_analytics_endpoints(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Analytics Endpoints".to_string();

        // Test database metrics
        match self.make_request("GET", "/api/v1/analytics/database", None).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status.is_success() {
                    match response.json::<ApiResponse<Value>>().await {
                        Ok(api_response) => {
                            if api_response.success {
                                let details = "Database metrics retrieved successfully".to_string();
                                let result = TestResult::new(test_name, true, duration)
                                    .with_details(details);
                                self.results.push(result);
                            } else {
                                let error = api_response.error.unwrap_or("Unknown error".to_string());
                                let result = TestResult::new(test_name, false, duration)
                                    .with_error(error);
                                self.results.push(result);
                            }
                        }
                        Err(e) => {
                            let result = TestResult::new(test_name, false, duration)
                                .with_error(format!("Invalid response format: {}", e));
                            self.results.push(result);
                        }
                    }
                } else {
                    let error_text = response.text().await.unwrap_or_default();
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("HTTP {}: {}", status, error_text));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_error_handling(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Error Handling".to_string();

        // Test with invalid PDA address
        let request = AnalyzePdaRequest {
            address: "invalid_address".to_string(),
            program_id: "11111111111111111111111111111111".to_string(),
        };

        let body = serde_json::to_value(&request)?;

        match self.make_request("POST", "/api/v1/analyze/pda", Some(body)).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                // Should return 400 for invalid address
                if status == 400 {
                    let details = "Correctly returned 400 for invalid address".to_string();
                    let result = TestResult::new(test_name, true, duration)
                        .with_details(details);
                    self.results.push(result);
                } else {
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("Expected 400, got {}", status));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_nonexistent_endpoints(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Nonexistent Endpoints".to_string();

        match self.make_request("GET", "/api/v1/nonexistent", None).await {
            Ok(response) => {
                let duration = start.elapsed();
                let status = response.status();

                if status == 404 {
                    let details = "Correctly returned 404 for nonexistent endpoint".to_string();
                    let result = TestResult::new(test_name, true, duration)
                        .with_details(details);
                    self.results.push(result);
                } else {
                    let result = TestResult::new(test_name, false, duration)
                        .with_error(format!("Expected 404, got {}", status));
                    self.results.push(result);
                }
            }
            Err(e) => {
                let duration = start.elapsed();
                let result = TestResult::new(test_name, false, duration)
                    .with_error(e.to_string());
                self.results.push(result);
            }
        }

        Ok(())
    }

    async fn test_concurrent_requests(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "Concurrent Requests".to_string();

        // Send 10 concurrent health check requests
        let mut handles = Vec::new();

        for _ in 0..10 {
            let client = self.client.clone();
            let url = format!("{}/health", self.base_url);
            
            let handle = tokio::spawn(async move {
                client.get(&url).send().await
            });
            
            handles.push(handle);
        }

        let mut success_count = 0;
        let mut total_count = 0;

        for handle in handles {
            total_count += 1;
            match handle.await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        success_count += 1;
                    }
                }
                _ => {}
            }
        }

        let duration = start.elapsed();

        if success_count == 10 {
            let details = format!("All {} concurrent requests succeeded", success_count);
            let result = TestResult::new(test_name, true, duration)
                .with_details(details);
            self.results.push(result);
        } else {
            let result = TestResult::new(test_name, false, duration)
                .with_error(format!("Only {}/{} requests succeeded", success_count, total_count));
            self.results.push(result);
        }

        Ok(())
    }

    async fn run_all_tests(&mut self) -> Result<()> {
        self.log_info("Starting integration tests...");

        // Test server availability first
        self.log_info("Checking server availability...");
        let health_check = self.make_request("GET", "/health", None).await;
        if health_check.is_err() {
            self.log_error("Server is not accessible. Make sure it's running.");
            return Ok(());
        }

        // Run all tests
        let tests = vec![
            ("Health Check", Self::test_health_check),
            ("PDA Analysis", Self::test_pda_analysis),
            ("Batch PDA Analysis", Self::test_batch_pda_analysis),
            ("List Programs", Self::test_list_programs),
            ("List PDAs", Self::test_list_pdas),
            ("Analytics Endpoints", Self::test_analytics_endpoints),
            ("Error Handling", Self::test_error_handling),
            ("Nonexistent Endpoints", Self::test_nonexistent_endpoints),
            ("Concurrent Requests", Self::test_concurrent_requests),
        ];

        for (test_name, test_func) in tests {
            self.log_info(&format!("Running test: {}", test_name));
            
            if let Err(e) = test_func(self).await {
                self.log_error(&format!("Test {} failed with error: {}", test_name, e));
                let result = TestResult::new(test_name.to_string(), false, Duration::from_secs(0))
                    .with_error(e.to_string());
                self.results.push(result);
            }
            
            // Small delay between tests
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    fn print_summary(&self) {
        let passed = self.results.iter().filter(|r| r.success).count();
        let total = self.results.len();

        println!("\n{}", "=".repeat(60));
        println!("TEST SUMMARY");
        println!("{}", "=".repeat(60));

        for result in &self.results {
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
            println!("{}", "All tests passed!".green());
        } else {
            println!("{}", format!("{} tests failed", total - passed).red());
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    println!("{}", "Solana PDA Analyzer - Integration Tests".blue().bold());
    println!("Target URL: {}", base_url);
    println!();

    let mut tester = IntegrationTester::new(base_url);
    tester.run_all_tests().await?;
    tester.print_summary();

    let passed = tester.results.iter().filter(|r| r.success).count();
    let total = tester.results.len();

    std::process::exit(if passed == total { 0 } else { 1 });
}