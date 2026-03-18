use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

const CONTEXT7_MCP_URL: &str = "https://mcp.context7.com/mcp";
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
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub id: String,
    pub library: String,
    pub title: String,
    pub excerpt: String,
    pub url: Option<String>,
    pub relevance_score: f32,
    /// Full section content (not truncated). Falls back to excerpt if unavailable.
    #[serde(default)]
    pub full_content: Option<String>,
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

    fn get_base_url(&self) -> &str {
        // For now, always use MCP URL until we confirm the correct API endpoint
        CONTEXT7_MCP_URL
    }

    pub async fn resolve_library(
        &self,
        library_name: &str,
        query: &str,
    ) -> Result<(String, String)> {
        // Always use MCP tools/call format for now
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "resolve-library-id",
                "arguments": {
                    "libraryName": library_name,
                    "query": query
                }
            }),
            id: 1,
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("API error: {} (code: {})", error.message, error.code);
        }

        let result = response.result.context("No result in response")?;

        // Extract the library ID from the response text
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
            .context("Failed to extract content from response")?;

        // Parse the response following Context7's selection criteria:
        // 1. First result is pre-ranked by Context7 (prioritize it)
        // 2. For ties, prefer exact name matches
        // 3. Secondary: higher snippet count, benchmark score
        let lines: Vec<&str> = content.lines().collect();
        let mut libraries = Vec::new();

        // Parse all libraries from response - collect all fields before pushing
        let mut current_id = String::new();
        let mut current_title = String::new();
        let mut current_score: f64 = 0.0;
        let mut current_snippets: u32 = 0;
        let mut has_current = false;

        for line in &lines {
            // Look for library title (start of a new library block)
            if let Some(stripped) = line.strip_prefix("- Title: ") {
                // Push previous library if we have one
                if has_current && !current_id.is_empty() {
                    libraries.push((
                        current_id.clone(),
                        current_title.clone(),
                        current_score,
                        current_snippets,
                    ));
                }
                current_title = stripped.trim().to_string();
                current_id = String::new();
                current_score = 0.0;
                current_snippets = 0;
                has_current = true;
            }
            // Look for library ID
            else if has_current && line.contains("Context7-compatible library ID:") {
                if let Some(start) = line.find('/') {
                    let id_part = &line[start..];
                    let end = id_part.find(char::is_whitespace).unwrap_or(id_part.len());
                    current_id = id_part[..end].trim().to_string();
                }
            }
            // Look for code snippets count
            else if has_current && line.contains("Code Snippets:") {
                if let Some(count_str) = line.split("Code Snippets:").nth(1) {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        current_snippets = count;
                    }
                }
            }
            // Look for benchmark score (replaces old Trust Score)
            else if has_current && line.contains("Benchmark Score:") {
                if let Some(score_str) = line.split("Benchmark Score:").nth(1) {
                    if let Ok(score) = score_str.trim().parse::<f64>() {
                        current_score = score;
                    }
                }
            }
        }
        // Push the last library
        if has_current && !current_id.is_empty() {
            libraries.push((current_id, current_title, current_score, current_snippets));
        }

        log::debug!(
            "Found {} library candidates for '{}'",
            libraries.len(),
            library_name
        );
        for (i, (id, title, benchmark, snippets)) in libraries.iter().enumerate() {
            log::debug!(
                "  {}: {} ({}) - Benchmark: {}, Snippets: {}",
                i + 1,
                title,
                id,
                benchmark,
                snippets
            );
        }

        // Apply Context7 selection criteria to find the best match
        let selected_library = libraries.iter().enumerate().max_by_key(
            |(index, (_id, title, benchmark_score, snippet_count))| {
                let mut score = 0;

                // 1. First result gets highest priority (Context7 pre-ranks)
                score += (1000 - index) * 100;

                // 2. Exact name match gets bonus
                if title.to_lowercase() == library_name.to_lowercase() {
                    score += 500;
                }

                // 3. Partial name match gets smaller bonus
                if title.to_lowercase().contains(&library_name.to_lowercase()) {
                    score += 200;
                }

                // 4. Higher benchmark score (0-100) gets bonus
                score += *benchmark_score as usize;

                // 5. Higher snippet count indicates better documentation
                score += (*snippet_count as usize).min(100);

                log::debug!(
                    "Library '{}' score: {} (index: {}, benchmark: {}, snippets: {})",
                    title,
                    score,
                    index,
                    benchmark_score,
                    snippet_count
                );

                score
            },
        );

        if let Some((index, (library_id, title, benchmark_score, snippet_count))) = selected_library
        {
            log::debug!(
                "Selected library: '{}' ({}), Benchmark: {}, Snippets: {}, Position: {}",
                title,
                library_id,
                benchmark_score,
                snippet_count,
                index + 1
            );
            Ok((library_id.clone(), title.clone()))
        } else {
            // Extract available library names for suggestions
            let available_libraries: Vec<String> = lines
                .iter()
                .filter_map(|line| {
                    if line.contains("- Title: ") {
                        Some(line.replace("- Title: ", "").trim().to_string())
                    } else {
                        None
                    }
                })
                .collect();

            if !available_libraries.is_empty() {
                let suggestions =
                    crate::search::fuzzy_find_libraries(library_name, &available_libraries);
                if !suggestions.is_empty() {
                    let suggestion_text: Vec<String> =
                        suggestions.iter().map(|(name, _)| name.clone()).collect();
                    anyhow::bail!(
                        "Library '{}' not found. Did you mean one of: {}?",
                        library_name,
                        suggestion_text.join(", ")
                    );
                }
            }

            anyhow::bail!(
                "No library ID found in response for '{}': {}",
                library_name,
                content
            );
        }
    }

    pub async fn get_documentation(&self, library_id: &str, topic: Option<&str>) -> Result<String> {
        let mut params = json!({
            "libraryId": library_id
        });

        params["query"] = json!(topic.unwrap_or_default());

        // Always use MCP tools/call format for now
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tools/call".to_string(),
            params: json!({
                "name": "query-docs",
                "arguments": params
            }),
            id: 2,
        };

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            anyhow::bail!("API error: {} (code: {})", error.message, error.code);
        }

        let result = response.result.context("No result in response")?;

        // Extract the documentation text from the response
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|text| text.as_str())
            .context("Failed to extract documentation from response")?;

        log::debug!("{:?}", content);
        Ok(content.to_string())
    }

    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let base_url = self.get_base_url();
        let mut req = self
            .client
            .post(base_url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(key) = &self.api_key {
            req = req.header("CONTEXT7_API_KEY", key);
        }

        let response = req
            .send()
            .await
            .context("Failed to send request to Context7")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("HTTP {} error: {}", status, error_text);
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Handle different response types
        if content_type.contains("text/event-stream") {
            // Handle SSE response
            let text = response.text().await?;
            log::debug!("SSE Response: {}", text);

            // Parse SSE to get the JSON data
            if let Some(json_line) = text.lines().find(|line| line.starts_with("data: ")) {
                let json_data = &json_line[6..]; // Remove "data: " prefix
                serde_json::from_str(json_data).context("Failed to parse SSE JSON data")
            } else {
                anyhow::bail!("No JSON data found in SSE response");
            }
        } else {
            // Regular JSON response
            response
                .json::<JsonRpcResponse>()
                .await
                .context("Failed to parse JSON-RPC response")
        }
    }
}
