//! Performance and Load Tests for Solana PDA Analyzer
//! 
//! This module contains comprehensive performance tests that measure
//! API response times, throughput, and system behavior under load.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug, Clone)]
struct PerformanceResult {
    test_name: String,
    total_requests: u64,
    duration: Duration,
    successful_requests: u64,
    failed_requests: u64,
    avg_response_time: Duration,
    min_response_time: Duration,
    max_response_time: Duration,
    percentile_95: Duration,
    percentile_99: Duration,
    requests_per_second: f64,
    errors: Vec<String>,
    throughput_over_time: Vec<(Duration, u64)>,
}

#[derive(Debug, Clone)]
struct RequestResult {
    success: bool,
    response_time: Duration,
    error: Option<String>,
}

pub struct PerformanceTester {
    client: Client,
    base_url: String,
    results: Vec<PerformanceResult>,
}

impl PerformanceTester {
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(100)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            results: Vec::new(),
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

    fn log_warning(&self, message: &str) {
        println!("{} {}", "[WARNING]".yellow(), message);
    }

    async fn make_request(&self, method: &str, endpoint: &str, body: Option<Value>) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut request = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => {
                return RequestResult {
                    success: false,
                    response_time: start.elapsed(),
                    error: Some(format!("Unsupported HTTP method: {}", method)),
                };
            }
        };

        if let Some(body) = body {
            request = request.json(&body);
        }

        match request.send().await {
            Ok(response) => {
                let response_time = start.elapsed();
                let success = response.status().is_success();
                
                if success {
                    // Try to consume the response body to simulate real usage
                    let _ = response.bytes().await;
                    RequestResult {
                        success: true,
                        response_time,
                        error: None,
                    }
                } else {
                    let error = format!("HTTP {}", response.status());
                    RequestResult {
                        success: false,
                        response_time,
                        error: Some(error),
                    }
                }
            }
            Err(e) => {
                RequestResult {
                    success: false,
                    response_time: start.elapsed(),
                    error: Some(e.to_string()),
                }
            }
        }
    }

    async fn run_concurrent_requests(
        &self,
        test_name: &str,
        method: &str,
        endpoint: &str,
        body: Option<Value>,
        concurrent_users: u64,
        requests_per_user: u64,
    ) -> PerformanceResult {
        let total_requests = concurrent_users * requests_per_user;
        
        self.log_info(&format!(
            "Running {} - {} requests ({} concurrent users, {} requests each)",
            test_name, total_requests, concurrent_users, requests_per_user
        ));

        let semaphore = Arc::new(Semaphore::new(concurrent_users as usize));
        let request_counter = Arc::new(AtomicU64::new(0));
        let start_time = Instant::now();
        let mut tasks = Vec::new();

        // Track throughput over time
        let throughput_tracker = Arc::new(AtomicU64::new(0));
        let throughput_results = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        // Spawn throughput monitoring task
        let throughput_monitor = {
            let counter = throughput_tracker.clone();
            let results = throughput_results.clone();
            let start = start_time;
            
            tokio::spawn(async move {
                let mut last_count = 0;
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                
                loop {
                    interval.tick().await;
                    let current_count = counter.load(Ordering::Relaxed);
                    let current_throughput = current_count - last_count;
                    let elapsed = start.elapsed();
                    
                    results.lock().await.push((elapsed, current_throughput));
                    last_count = current_count;
                    
                    if current_count >= total_requests {
                        break;
                    }
                }
            })
        };

        // Create all request tasks
        for _ in 0..total_requests {
            let semaphore = semaphore.clone();
            let client = self.client.clone();
            let base_url = self.base_url.clone();
            let method = method.to_string();
            let endpoint = endpoint.to_string();
            let body = body.clone();
            let counter = request_counter.clone();
            let throughput_counter = throughput_tracker.clone();

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                let result = Self::make_single_request(&client, &base_url, &method, &endpoint, body).await;
                
                counter.fetch_add(1, Ordering::Relaxed);
                throughput_counter.fetch_add(1, Ordering::Relaxed);
                
                result
            });
            
            tasks.push(task);
        }

        // Wait for all requests to complete
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(RequestResult {
                        success: false,
                        response_time: Duration::from_secs(0),
                        error: Some(format!("Task error: {}", e)),
                    });
                }
            }
        }

        // Stop throughput monitoring
        let _ = throughput_monitor.await;
        let throughput_over_time = throughput_results.lock().await.clone();

        let total_duration = start_time.elapsed();

        self.calculate_performance_metrics(
            test_name.to_string(),
            results,
            total_duration,
            throughput_over_time,
        )
    }

    async fn make_single_request(
        client: &Client,
        base_url: &str,
        method: &str,
        endpoint: &str,
        body: Option<Value>,
    ) -> RequestResult {
        let start = Instant::now();
        let url = format!("{}{}", base_url, endpoint);
        
        let mut request = match method {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => {
                return RequestResult {
                    success: false,
                    response_time: start.elapsed(),
                    error: Some(format!("Unsupported HTTP method: {}", method)),
                };
            }
        };

        if let Some(body) = body {
            request = request.json(&body);
        }

        match request.send().await {
            Ok(response) => {
                let response_time = start.elapsed();
                let success = response.status().is_success();
                
                if success {
                    let _ = response.bytes().await;
                    RequestResult {
                        success: true,
                        response_time,
                        error: None,
                    }
                } else {
                    let error = format!("HTTP {}", response.status());
                    RequestResult {
                        success: false,
                        response_time,
                        error: Some(error),
                    }
                }
            }
            Err(e) => {
                RequestResult {
                    success: false,
                    response_time: start.elapsed(),
                    error: Some(e.to_string()),
                }
            }
        }
    }

    fn calculate_performance_metrics(
        &self,
        test_name: String,
        results: Vec<RequestResult>,
        total_duration: Duration,
        throughput_over_time: Vec<(Duration, u64)>,
    ) -> PerformanceResult {
        let total_requests = results.len() as u64;
        let successful_requests = results.iter().filter(|r| r.success).count() as u64;
        let failed_requests = total_requests - successful_requests;

        let mut response_times: Vec<Duration> = results
            .iter()
            .map(|r| r.response_time)
            .collect();
        
        response_times.sort();

        let avg_response_time = if !response_times.is_empty() {
            let total_time: Duration = response_times.iter().sum();
            total_time / response_times.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let min_response_time = response_times.first().cloned().unwrap_or(Duration::from_secs(0));
        let max_response_time = response_times.last().cloned().unwrap_or(Duration::from_secs(0));

        let percentile_95 = if !response_times.is_empty() {
            let index = (response_times.len() as f64 * 0.95) as usize;
            response_times.get(index.min(response_times.len() - 1)).cloned().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };

        let percentile_99 = if !response_times.is_empty() {
            let index = (response_times.len() as f64 * 0.99) as usize;
            response_times.get(index.min(response_times.len() - 1)).cloned().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };

        let requests_per_second = if total_duration.as_secs_f64() > 0.0 {
            total_requests as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let errors: Vec<String> = results
            .iter()
            .filter_map(|r| r.error.clone())
            .take(10)
            .collect();

        PerformanceResult {
            test_name,
            total_requests,
            duration: total_duration,
            successful_requests,
            failed_requests,
            avg_response_time,
            min_response_time,
            max_response_time,
            percentile_95,
            percentile_99,
            requests_per_second,
            errors,
            throughput_over_time,
        }
    }

    pub async fn test_health_endpoint_load(&mut self) -> Result<()> {
        let result = self.run_concurrent_requests(
            "Health Endpoint Load Test",
            "GET",
            "/health",
            None,
            50,
            10,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub async fn test_pda_analysis_load(&mut self) -> Result<()> {
        let body = serde_json::json!({
            "address": "11111111111111111111111111111111",
            "program_id": "11111111111111111111111111111111"
        });

        let result = self.run_concurrent_requests(
            "PDA Analysis Load Test",
            "POST",
            "/api/v1/analyze/pda",
            Some(body),
            20,
            5,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub async fn test_batch_analysis_load(&mut self) -> Result<()> {
        let pdas = vec![
            serde_json::json!({
                "address": "11111111111111111111111111111111",
                "program_id": "11111111111111111111111111111111"
            }),
            serde_json::json!({
                "address": "22222222222222222222222222222222",
                "program_id": "11111111111111111111111111111111"
            }),
            serde_json::json!({
                "address": "33333333333333333333333333333333",
                "program_id": "11111111111111111111111111111111"
            }),
        ];

        let body = serde_json::json!({
            "pdas": pdas
        });

        let result = self.run_concurrent_requests(
            "Batch Analysis Load Test",
            "POST",
            "/api/v1/analyze/pda/batch",
            Some(body),
            10,
            3,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub async fn test_database_queries_load(&mut self) -> Result<()> {
        let result = self.run_concurrent_requests(
            "Database Queries Load Test",
            "GET",
            "/api/v1/analytics/database",
            None,
            30,
            10,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub async fn test_list_endpoints_load(&mut self) -> Result<()> {
        let result = self.run_concurrent_requests(
            "List Endpoints Load Test",
            "GET",
            "/api/v1/programs?limit=20",
            None,
            25,
            8,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub async fn test_sustained_load(&mut self, duration_seconds: u64) -> Result<()> {
        self.log_info(&format!("Running sustained load test for {} seconds", duration_seconds));
        
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(duration_seconds);
        
        let counter = Arc::new(AtomicU64::new(0));
        let successful_counter = Arc::new(AtomicU64::new(0));
        let failed_counter = Arc::new(AtomicU64::new(0));
        let response_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let errors = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let throughput_tracker = Arc::new(AtomicU64::new(0));
        let throughput_results = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        // Throughput monitoring
        let throughput_monitor = {
            let counter = throughput_tracker.clone();
            let results = throughput_results.clone();
            let start = start_time;
            
            tokio::spawn(async move {
                let mut last_count = 0;
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                
                while start.elapsed() < Duration::from_secs(duration_seconds) {
                    interval.tick().await;
                    let current_count = counter.load(Ordering::Relaxed);
                    let current_throughput = current_count - last_count;
                    let elapsed = start.elapsed();
                    
                    results.lock().await.push((elapsed, current_throughput));
                    last_count = current_count;
                }
            })
        };

        // Request generation
        let mut tasks = Vec::new();
        let semaphore = Arc::new(Semaphore::new(50)); // Limit concurrent requests
        
        while Instant::now() < end_time {
            if tasks.len() < 100 {
                let semaphore = semaphore.clone();
                let client = self.client.clone();
                let base_url = self.base_url.clone();
                let counter = counter.clone();
                let successful_counter = successful_counter.clone();
                let failed_counter = failed_counter.clone();
                let response_times = response_times.clone();
                let errors = errors.clone();
                let throughput_counter = throughput_tracker.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    let result = Self::make_single_request(&client, &base_url, "GET", "/health", None).await;
                    
                    counter.fetch_add(1, Ordering::Relaxed);
                    throughput_counter.fetch_add(1, Ordering::Relaxed);
                    
                    if result.success {
                        successful_counter.fetch_add(1, Ordering::Relaxed);
                    } else {
                        failed_counter.fetch_add(1, Ordering::Relaxed);
                        if let Some(error) = result.error {
                            errors.lock().await.push(error);
                        }
                    }
                    
                    response_times.lock().await.push(result.response_time);
                });
                
                tasks.push(task);
            }
            
            // Clean up completed tasks
            tasks.retain(|task| !task.is_finished());
            
            sleep(Duration::from_millis(10)).await;
        }

        // Wait for remaining tasks
        for task in tasks {
            let _ = task.await;
        }

        let _ = throughput_monitor.await;

        let total_duration = start_time.elapsed();
        let total_requests = counter.load(Ordering::Relaxed);
        let successful_requests = successful_counter.load(Ordering::Relaxed);
        let failed_requests = failed_counter.load(Ordering::Relaxed);
        
        let response_times_vec = response_times.lock().await.clone();
        let errors_vec = errors.lock().await.clone();
        let throughput_over_time = throughput_results.lock().await.clone();

        // Calculate metrics
        let mut sorted_times = response_times_vec.clone();
        sorted_times.sort();

        let avg_response_time = if !sorted_times.is_empty() {
            let total_time: Duration = sorted_times.iter().sum();
            total_time / sorted_times.len() as u32
        } else {
            Duration::from_secs(0)
        };

        let min_response_time = sorted_times.first().cloned().unwrap_or(Duration::from_secs(0));
        let max_response_time = sorted_times.last().cloned().unwrap_or(Duration::from_secs(0));

        let percentile_95 = if !sorted_times.is_empty() {
            let index = (sorted_times.len() as f64 * 0.95) as usize;
            sorted_times.get(index.min(sorted_times.len() - 1)).cloned().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };

        let percentile_99 = if !sorted_times.is_empty() {
            let index = (sorted_times.len() as f64 * 0.99) as usize;
            sorted_times.get(index.min(sorted_times.len() - 1)).cloned().unwrap_or(Duration::from_secs(0))
        } else {
            Duration::from_secs(0)
        };

        let requests_per_second = if total_duration.as_secs_f64() > 0.0 {
            total_requests as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let result = PerformanceResult {
            test_name: format!("Sustained Load Test ({}s)", duration_seconds),
            total_requests,
            duration: total_duration,
            successful_requests,
            failed_requests,
            avg_response_time,
            min_response_time,
            max_response_time,
            percentile_95,
            percentile_99,
            requests_per_second,
            errors: errors_vec.into_iter().take(10).collect(),
            throughput_over_time,
        };

        self.results.push(result);
        Ok(())
    }

    pub async fn test_memory_usage_load(&mut self) -> Result<()> {
        // Test with large payloads
        let mut large_pdas = Vec::new();
        for i in 0..100 {
            large_pdas.push(serde_json::json!({
                "address": format!("{:044}", i),
                "program_id": "11111111111111111111111111111111"
            }));
        }

        let body = serde_json::json!({
            "pdas": large_pdas
        });

        let result = self.run_concurrent_requests(
            "Memory Usage Load Test",
            "POST",
            "/api/v1/analyze/pda/batch",
            Some(body),
            5,
            3,
        ).await;

        self.results.push(result);
        Ok(())
    }

    pub fn print_result(&self, result: &PerformanceResult) {
        println!("\n{}", "=".repeat(60).blue());
        println!("{}", format!("Test: {}", result.test_name).blue());
        println!("{}", "=".repeat(60).blue());

        // Basic metrics
        println!("Total Requests:      {}", result.total_requests);
        println!("Duration:            {:.2}s", result.duration.as_secs_f64());
        println!("Successful:          {}", result.successful_requests);
        println!("Failed:              {}", result.failed_requests);
        println!("Success Rate:        {:.1}%", 
            (result.successful_requests as f64 / result.total_requests as f64) * 100.0);
        println!("Requests/Second:     {:.2}", result.requests_per_second);

        // Response time metrics
        println!("\nResponse Times:");
        println!("  Average:           {:.2}ms", result.avg_response_time.as_millis());
        println!("  Minimum:           {:.2}ms", result.min_response_time.as_millis());
        println!("  Maximum:           {:.2}ms", result.max_response_time.as_millis());
        println!("  95th Percentile:   {:.2}ms", result.percentile_95.as_millis());
        println!("  99th Percentile:   {:.2}ms", result.percentile_99.as_millis());

        // Throughput over time
        if !result.throughput_over_time.is_empty() {
            println!("\nThroughput over time (last 5 seconds):");
            for (time, throughput) in result.throughput_over_time.iter().rev().take(5).rev() {
                println!("  {:>3}s: {:>3} req/s", time.as_secs(), throughput);
            }
        }

        // Error summary
        if !result.errors.is_empty() {
            println!("\nErrors (first 10):");
            for (i, error) in result.errors.iter().take(10).enumerate() {
                println!("  {}. {}", i + 1, error);
            }
        }

        // Performance assessment
        self.assess_performance(result);
    }

    fn assess_performance(&self, result: &PerformanceResult) {
        println!("\n{}", "Performance Assessment:".yellow());

        let success_rate = (result.successful_requests as f64 / result.total_requests as f64) * 100.0;
        if success_rate >= 99.0 {
            println!("{} Excellent success rate ({:.1}%)", "✓".green(), success_rate);
        } else if success_rate >= 95.0 {
            println!("{} Good success rate ({:.1}%)", "⚠".yellow(), success_rate);
        } else {
            println!("{} Poor success rate ({:.1}%)", "✗".red(), success_rate);
        }

        let avg_time_ms = result.avg_response_time.as_millis();
        if avg_time_ms <= 100 {
            println!("{} Excellent average response time ({}ms)", "✓".green(), avg_time_ms);
        } else if avg_time_ms <= 500 {
            println!("{} Good average response time ({}ms)", "⚠".yellow(), avg_time_ms);
        } else {
            println!("{} Poor average response time ({}ms)", "✗".red(), avg_time_ms);
        }

        if result.requests_per_second >= 100.0 {
            println!("{} Excellent throughput ({:.1} req/s)", "✓".green(), result.requests_per_second);
        } else if result.requests_per_second >= 50.0 {
            println!("{} Good throughput ({:.1} req/s)", "⚠".yellow(), result.requests_per_second);
        } else {
            println!("{} Poor throughput ({:.1} req/s)", "✗".red(), result.requests_per_second);
        }

        let p99_ms = result.percentile_99.as_millis();
        if p99_ms <= 1000 {
            println!("{} Good 99th percentile ({}ms)", "✓".green(), p99_ms);
        } else {
            println!("{} High 99th percentile ({}ms)", "✗".red(), p99_ms);
        }
    }

    pub async fn run_all_tests(&mut self) -> Result<()> {
        self.log_info("Starting performance tests...");

        // Check server availability
        match self.make_request("GET", "/health", None).await {
            RequestResult { success: true, .. } => {
                self.log_success("Server is accessible");
            }
            _ => {
                self.log_error("Server is not accessible");
                return Ok(());
            }
        }

        // Run performance tests
        self.test_health_endpoint_load().await?;
        self.test_pda_analysis_load().await?;
        self.test_batch_analysis_load().await?;
        self.test_database_queries_load().await?;
        self.test_list_endpoints_load().await?;
        self.test_memory_usage_load().await?;

        Ok(())
    }

    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60).blue());
        println!("{}", "OVERALL PERFORMANCE SUMMARY".blue());
        println!("{}", "=".repeat(60).blue());

        let total_requests: u64 = self.results.iter().map(|r| r.total_requests).sum();
        let total_successful: u64 = self.results.iter().map(|r| r.successful_requests).sum();
        let overall_success_rate = if total_requests > 0 {
            (total_successful as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        println!("Total Tests:         {}", self.results.len());
        println!("Total Requests:      {}", total_requests);
        println!("Total Successful:    {}", total_successful);
        println!("Overall Success Rate: {:.1}%", overall_success_rate);

        if !self.results.is_empty() {
            let avg_rps: f64 = self.results.iter().map(|r| r.requests_per_second).sum::<f64>() / self.results.len() as f64;
            let avg_response_time: f64 = self.results.iter()
                .map(|r| r.avg_response_time.as_millis() as f64)
                .sum::<f64>() / self.results.len() as f64;

            println!("Average RPS:         {:.2}", avg_rps);
            println!("Average Response:    {:.2}ms", avg_response_time);

            if overall_success_rate >= 95.0 && avg_response_time <= 500.0 {
                println!("{}", "✓ Overall performance is GOOD".green());
            } else {
                println!("{}", "⚠ Overall performance needs IMPROVEMENT".yellow());
            }
        }

        // Print individual test results
        for result in &self.results {
            self.print_result(result);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_url = std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let quick_test = std::env::var("QUICK_TEST").is_ok();
    let sustained_duration = std::env::var("SUSTAINED_DURATION")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    println!("{}", "Solana PDA Analyzer - Performance Tests".blue().bold());
    println!("Target URL: {}", base_url);
    println!();

    let mut tester = PerformanceTester::new(base_url)?;

    if sustained_duration > 0 {
        tester.test_sustained_load(sustained_duration).await?;
    } else if quick_test {
        tester.log_info("Running quick performance tests...");
        tester.test_health_endpoint_load().await?;
        tester.test_pda_analysis_load().await?;
    } else {
        tester.run_all_tests().await?;
    }

    tester.print_summary();

    let total_requests: u64 = tester.results.iter().map(|r| r.total_requests).sum();
    let total_successful: u64 = tester.results.iter().map(|r| r.successful_requests).sum();
    let overall_success_rate = if total_requests > 0 {
        (total_successful as f64 / total_requests as f64) * 100.0
    } else {
        0.0
    };

    let avg_response_time: f64 = if !tester.results.is_empty() {
        tester.results.iter()
            .map(|r| r.avg_response_time.as_millis() as f64)
            .sum::<f64>() / tester.results.len() as f64
    } else {
        0.0
    };

    let exit_code = if overall_success_rate >= 95.0 && avg_response_time <= 500.0 {
        0
    } else {
        1
    };

    std::process::exit(exit_code);
}