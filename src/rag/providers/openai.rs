use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};

/// OpenAI API embedding provider
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
    dimension: Option<usize>, // Cached dimension
}

#[derive(Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
    encoding_format: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Usage,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    total_tokens: u32,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            api_key,
            model,
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
            "Detecting embedding dimension for OpenAI model: {}",
            self.model
        );

        let test_embedding = self.call_api("test").await?;
        let dimension = test_embedding.len();

        self.dimension = Some(dimension);
        log::info!("Detected dimension: {} for model {}", dimension, self.model);

        Ok(dimension)
    }

    /// Get token usage from last API call
    #[allow(dead_code)]
    pub fn get_usage_stats(&self) -> Option<(u32, u32)> {
        // This would store the last usage from call_api
        // For now return None as we don't track it
        None
    }

    /// Make API call to OpenAI embeddings endpoint
    async fn call_api(&self, text: &str) -> Result<Vec<f32>> {
        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone(),
            encoding_format: "float".to_string(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let embedding_response: EmbeddingResponse = response.json().await?;

        if embedding_response.data.is_empty() {
            return Err(anyhow!("No embeddings returned from OpenAI API"));
        }

        // Log usage statistics
        log::debug!(
            "OpenAI API usage: {} prompt tokens, {} total tokens",
            embedding_response.usage.prompt_tokens,
            embedding_response.usage.total_tokens
        );

        // Ensure we have the right embedding (index should match)
        if embedding_response.data[0].index != 0 {
            log::warn!(
                "Unexpected embedding index: {}",
                embedding_response.data[0].index
            );
        }

        // Verify model matches request
        if embedding_response.model != self.model {
            log::info!(
                "API returned model: {} (requested: {})",
                embedding_response.model,
                self.model
            );
        }

        Ok(embedding_response.data[0].embedding.clone())
    }

    /// Get common OpenAI model information
    pub fn get_model_info(model: &str) -> (usize, usize) {
        match model {
            "text-embedding-3-small" => (1536, 8191),
            "text-embedding-3-large" => (3072, 8191),
            "text-embedding-ada-002" => (1536, 8191),
            _ => (1536, 8191), // Default
        }
    }
}

#[async_trait]
impl ProviderTrait for OpenAiProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        // Truncate text if too long (OpenAI models have token limits)
        let (_, max_chars) = Self::get_model_info(&self.model);
        let truncated_text = if text.len() > max_chars {
            &text[..max_chars]
        } else {
            text
        };

        self.call_api(truncated_text).await
    }

    async fn get_dimension(&self) -> Result<usize> {
        if let Some(dim) = self.dimension {
            Ok(dim)
        } else {
            // Use known dimensions for common models
            let (dim, _) = Self::get_model_info(&self.model);
            Ok(dim)
        }
    }

    async fn health_check(&self) -> Result<()> {
        self.call_api("test").await.map(|_| ())
    }

    fn get_info(&self) -> ProviderInfo {
        let (_, max_length) = Self::get_model_info(&self.model);

        ProviderInfo {
            name: "OpenAI Embeddings".to_string(),
            provider_type: "openai".to_string(),
            model_name: Some(self.model.clone()),
            description: format!("OpenAI embeddings model: {}", self.model),
            max_input_length: Some(max_length),
        }
    }
}
