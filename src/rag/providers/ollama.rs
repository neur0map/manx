use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};

/// Ollama API embedding provider
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    dimension: Option<usize>, // Cached dimension
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Serialize)]
#[allow(dead_code)]
struct OllamaShowRequest {
    name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct OllamaShowResponse {
    pub details: Option<ModelDetails>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct ModelDetails {
    pub parameter_size: Option<String>,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(model: String, base_url: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap();

        let base_url = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());

        Self {
            client,
            base_url,
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
            "Detecting embedding dimension for Ollama model: {}",
            self.model
        );

        let test_embedding = self.call_api("test").await?;
        let dimension = test_embedding.len();

        self.dimension = Some(dimension);
        log::info!("Detected dimension: {} for model {}", dimension, self.model);

        Ok(dimension)
    }

    /// Get model information from Ollama
    #[allow(dead_code)]
    pub async fn get_model_info(&self) -> Result<OllamaShowResponse> {
        let request = OllamaShowRequest {
            name: self.model.clone(),
        };

        let url = format!("{}/api/show", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama show API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let show_response: OllamaShowResponse = response.json().await?;
        Ok(show_response)
    }

    /// Make API call to Ollama embeddings endpoint
    async fn call_api(&self, text: &str) -> Result<Vec<f32>> {
        let request = OllamaEmbeddingRequest {
            model: self.model.clone(),
            prompt: text.to_string(),
        };

        let url = format!("{}/api/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Ollama API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        let embedding_response: OllamaEmbeddingResponse = response.json().await?;

        if embedding_response.embedding.is_empty() {
            return Err(anyhow!("No embeddings returned from Ollama API"));
        }

        Ok(embedding_response.embedding)
    }

    /// Check if Ollama server is available
    pub async fn check_server(&self) -> Result<()> {
        let url = format!("{}/api/version", self.base_url);

        let response = self.client.get(&url).send().await.map_err(|e| {
            anyhow!(
                "Failed to connect to Ollama server at {}: {}",
                self.base_url,
                e
            )
        })?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Ollama server returned error: HTTP {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Get common Ollama model information (dimension estimates)
    pub fn get_common_model_info(model: &str) -> (Option<usize>, usize) {
        match model {
            "nomic-embed-text" => (Some(768), 2048),
            "mxbai-embed-large" => (Some(1024), 512),
            "all-minilm" => (Some(384), 512),
            _ => (None, 2048), // Unknown model, will detect dynamically
        }
    }
}

#[async_trait]
impl ProviderTrait for OllamaProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        // Ollama typically handles longer texts well, but let's be conservative
        let (_, max_chars) = Self::get_common_model_info(&self.model);
        let truncated_text = if text.len() > max_chars * 4 {
            // Rough token approximation
            &text[..max_chars * 4]
        } else {
            text
        };

        self.call_api(truncated_text).await
    }

    async fn get_dimension(&self) -> Result<usize> {
        if let Some(dim) = self.dimension {
            Ok(dim)
        } else {
            // Try to use known dimensions for common models
            let (known_dim, _) = Self::get_common_model_info(&self.model);
            if let Some(dim) = known_dim {
                Ok(dim)
            } else {
                // Need to detect dynamically
                Err(anyhow!(
                    "Dimension not known for model {}. Use 'manx embedding test' to detect it.",
                    self.model
                ))
            }
        }
    }

    async fn health_check(&self) -> Result<()> {
        self.check_server().await?;
        self.call_api("test").await.map(|_| ())
    }

    fn get_info(&self) -> ProviderInfo {
        let (_, max_length) = Self::get_common_model_info(&self.model);

        ProviderInfo {
            name: "Ollama Local Server".to_string(),
            provider_type: "ollama".to_string(),
            model_name: Some(self.model.clone()),
            description: format!("Ollama model: {} ({})", self.model, self.base_url),
            max_input_length: Some(max_length),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
