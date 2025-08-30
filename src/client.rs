use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

const CONTEXT7_URL: &str = "https://mcp.context7.com/mcp";
const REQUEST_TIMEOUT: u64 = 30;

#[derive(Debug, Clone)]
pub struct Context7Client {
    client: Client,
    api_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LibraryInfo {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Documentation {
    pub library: LibraryInfo,
    pub sections: Vec<DocSection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DocSection {
    pub id: String,
    pub title: String,
    pub content: String,
    pub code_examples: Vec<CodeExample>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CodeExample {
    pub language: String,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub library: String,
    pub title: String,
    pub excerpt: String,
    pub url: Option<String>,
    pub relevance_score: f32,
}

impl Context7Client {
    pub fn new(api_key: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT))
            .user_agent(format!("manx/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;
        
        Ok(Self { client, api_key })
    }
    
    pub async fn resolve_library(&self, library_name: &str) -> Result<LibraryInfo> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "resolve-library-id".to_string(),
            params: json!({
                "library": library_name
            }),
            id: 1,
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(error) = response.error {
            anyhow::bail!("API error: {} (code: {})", error.message, error.code);
        }
        
        let result = response.result
            .context("No result in response")?;
        
        serde_json::from_value(result)
            .context("Failed to parse library info")
    }
    
    pub async fn get_documentation(
        &self,
        library: &str,
        query: Option<&str>,
    ) -> Result<Documentation> {
        let mut params = json!({
            "library": library
        });
        
        if let Some(q) = query {
            params["query"] = json!(q);
        }
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get-library-docs".to_string(),
            params,
            id: 2,
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(error) = response.error {
            anyhow::bail!("API error: {} (code: {})", error.message, error.code);
        }
        
        let result = response.result
            .context("No result in response")?;
        
        serde_json::from_value(result)
            .context("Failed to parse documentation")
    }
    
    pub async fn search(
        &self,
        library: &str,
        query: &str,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let params = json!({
            "library": library,
            "query": query,
            "limit": limit.unwrap_or(10)
        });
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "search".to_string(),
            params,
            id: 3,
        };
        
        let response = self.send_request(request).await?;
        
        if let Some(error) = response.error {
            anyhow::bail!("API error: {} (code: {})", error.message, error.code);
        }
        
        let result = response.result
            .context("No result in response")?;
        
        serde_json::from_value(result)
            .context("Failed to parse search results")
    }
    
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let mut req = self.client
            .post(CONTEXT7_URL)
            .json(&request);
        
        if let Some(key) = &self.api_key {
            req = req.header("CONTEXT7_API_KEY", key);
        }
        
        let response = req.send().await
            .context("Failed to send request to Context7")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("HTTP {} error: {}", status, error_text);
        }
        
        response.json::<JsonRpcResponse>().await
            .context("Failed to parse JSON-RPC response")
    }
}