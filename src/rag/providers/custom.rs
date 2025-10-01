use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};

/// Custom endpoint embedding provider
pub struct CustomProvider {
    client: Client,
    endpoint_url: String,
    api_key: Option<String>,
    dimension: Option<usize>, // Cached dimension
}

#[derive(Serialize)]
struct CustomEmbeddingRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
}

#[derive(Deserialize)]
struct CustomEmbeddingResponse {
    embedding: Vec<f32>,
    #[serde(default)]
    dimension: Option<usize>,
}

impl CustomProvider {
    /// Create a new custom endpoint provider
    pub fn new(endpoint_url: String, api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            endpoint_url,
            api_key,
            dimension: None,
        }
    }

    /// Detect dimension by making a test API call
    #[allow(dead_code)]
    pub async fn detect_dimension(&mut self) -> Result<usize> {
        if let Some(dim) = self.dimension {
            return Ok(dim);
        }

        log::info!(
            "Detecting embedding dimension for custom endpoint: {}",
            self.endpoint_url
        );

        let test_embedding = self.call_api("test").await?;
        let dimension = test_embedding.len();

        self.dimension = Some(dimension);
        log::info!(
            "Detected dimension: {} for endpoint {}",
            dimension,
            self.endpoint_url
        );

        Ok(dimension)
    }

    /// Make API call to custom endpoint
    async fn call_api(&self, text: &str) -> Result<Vec<f32>> {
        let request = CustomEmbeddingRequest {
            text: text.to_string(),
            model: None,
        };

        let mut request_builder = self
            .client
            .post(&self.endpoint_url)
            .header("Content-Type", "application/json")
            .json(&request);

        // Add API key if provided
        if let Some(ref api_key) = self.api_key {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request_builder.send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Custom endpoint error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let embedding_response: CustomEmbeddingResponse = response.json().await?;

        if embedding_response.embedding.is_empty() {
            return Err(anyhow!("No embeddings returned from custom endpoint"));
        }

        // Cache dimension if provided in response
        if let Some(dim) = embedding_response.dimension {
            if self.dimension.is_none() {
                // Note: This is a mutable operation but we're in an immutable context
                // In practice, this would need to be handled differently
                log::info!("Custom endpoint reported dimension: {}", dim);
            }
        }

        Ok(embedding_response.embedding)
    }

    /// Health check for custom endpoint
    pub async fn check_endpoint(&self) -> Result<()> {
        // Try a simple GET request first to see if endpoint is reachable
        let response = self
            .client
            .get(&self.endpoint_url)
            .send()
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to connect to custom endpoint {}: {}",
                    self.endpoint_url,
                    e
                )
            })?;

        // Accept any non-server-error response for basic connectivity
        if response.status().as_u16() >= 500 {
            return Err(anyhow!(
                "Custom endpoint returned server error: HTTP {}",
                response.status()
            ));
        }

        Ok(())
    }
}

#[async_trait]
impl ProviderTrait for CustomProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        self.call_api(text).await
    }

    async fn get_dimension(&self) -> Result<usize> {
        if let Some(dim) = self.dimension {
            Ok(dim)
        } else {
            Err(anyhow!("Dimension not known for custom endpoint {}. Use 'manx embedding test' to detect it.", self.endpoint_url))
        }
    }

    async fn health_check(&self) -> Result<()> {
        self.check_endpoint().await?;
        self.call_api("test").await.map(|_| ())
    }

    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Custom Endpoint".to_string(),
            provider_type: "custom".to_string(),
            model_name: None,
            description: format!("Custom embedding endpoint: {}", self.endpoint_url),
            max_input_length: None, // Unknown
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
