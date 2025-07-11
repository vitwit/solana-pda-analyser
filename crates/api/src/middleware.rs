use axum::{
    http::{HeaderMap, StatusCode, Request},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = Uuid::new_v4();
    
    // Log the incoming request
    info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Incoming request"
    );
    
    let response = next.run(request).await;
    
    let elapsed = start.elapsed();
    let status = response.status();
    
    // Log the response
    if status.is_success() {
        info!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = elapsed.as_millis(),
            "Request completed successfully"
        );
    } else {
        warn!(
            request_id = %request_id,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = elapsed.as_millis(),
            "Request completed with error"
        );
    }
    
    response
}

pub async fn rate_limiting_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Simple rate limiting based on IP address
    // In a production system, you'd want to use a proper rate limiting solution
    // like Redis or a dedicated rate limiting service
    
    let headers = request.headers();
    let client_ip = extract_client_ip(headers).unwrap_or("unknown".to_string());
    
    // For now, just log the client IP and allow all requests
    info!(client_ip = %client_ip, "Request from client");
    
    Ok(next.run(request).await)
}

pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".parse().unwrap(),
    );
    
    response
}

pub async fn cors_middleware(request: Request, next: Next) -> Response {
    let origin = request.headers().get("Origin").cloned();
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add CORS headers
    if let Some(origin) = origin {
        headers.insert("Access-Control-Allow-Origin", origin);
    } else {
        headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
    }
    
    headers.insert(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization, X-Requested-With".parse().unwrap(),
    );
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());
    
    response
}

pub async fn request_validation_middleware(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Basic request validation
    let headers = request.headers();
    let method = request.method();
    let uri = request.uri();
    
    // Check for required headers on POST requests
    if method == "POST" {
        if !headers.contains_key("content-type") {
            warn!(uri = %uri, "POST request missing Content-Type header");
            return Err(StatusCode::BAD_REQUEST);
        }
        
        let content_type = headers.get("content-type").unwrap().to_str().unwrap_or("");
        if !content_type.starts_with("application/json") {
            warn!(uri = %uri, content_type = %content_type, "POST request with unsupported Content-Type");
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
    }
    
    // Check for excessively long URIs
    if uri.path().len() > 2048 {
        warn!(uri = %uri, "Request URI too long");
        return Err(StatusCode::URI_TOO_LONG);
    }
    
    Ok(next.run(request).await)
}

fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Try to extract client IP from various headers
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(forwarded_for_str) = forwarded_for.to_str() {
            // Take the first IP in the list
            if let Some(ip) = forwarded_for_str.split(',').next() {
                return Some(ip.trim().to_string());
            }
        }
    }
    
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            return Some(real_ip_str.to_string());
        }
    }
    
    if let Some(forwarded) = headers.get("Forwarded") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Parse the Forwarded header (simplified)
            if let Some(for_part) = forwarded_str.split(';').find(|part| part.trim().starts_with("for=")) {
                if let Some(ip) = for_part.split('=').nth(1) {
                    return Some(ip.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    
    #[test]
    fn test_extract_client_ip_from_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", HeaderValue::from_static("192.168.1.1, 10.0.0.1"));
        
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }
    
    #[test]
    fn test_extract_client_ip_from_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", HeaderValue::from_static("192.168.1.1"));
        
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, Some("192.168.1.1".to_string()));
    }
    
    #[test]
    fn test_extract_client_ip_no_headers() {
        let headers = HeaderMap::new();
        let ip = extract_client_ip(&headers);
        assert_eq!(ip, None);
    }
}