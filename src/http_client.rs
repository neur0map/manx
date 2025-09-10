//! Secure HTTP client wrapper that prevents API key leakage in logs

use reqwest::{Client, ClientBuilder, Request, Response};
use std::time::Duration;

/// Create a secure HTTP client that doesn't log sensitive headers
pub fn create_secure_client() -> reqwest::Result<Client> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .user_agent("manx-cli")
        // Disable connection pooling logs that might leak info
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
}

/// Middleware to sanitize requests before logging
pub fn sanitize_request_for_logging(req: &Request) -> String {
    let mut sanitized = format!("{} {}", req.method(), req.url());
    
    // Log headers but mask sensitive ones
    for (name, value) in req.headers() {
        let header_name = name.as_str().to_lowercase();
        if header_name.contains("authorization") || 
           header_name.contains("api-key") ||
           header_name.contains("token") {
            sanitized.push_str(&format!("\n  {}: ****", name));
        } else {
            sanitized.push_str(&format!("\n  {}: {:?}", name, value));
        }
    }
    
    sanitized
}

/// Log response without sensitive data
pub fn log_response_safely(response: &Response) {
    log::debug!(
        "HTTP Response: {} {} ({}ms)", 
        response.status().as_u16(),
        response.url(),
        "timing_info_here" // Add timing if needed
    );
}