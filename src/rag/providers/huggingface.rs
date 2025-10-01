use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};

/// HuggingFace Inference API embedding provider
pub struct HuggingFaceProvider {
    client: Client,
    api_key: String,
    model: String,
    dimension: Option<usize>, // Cached dimension
}

#[derive(Serialize)]
struct HfEmbeddingRequest {
    inputs: String,
    options: HfOptions,
}

#[derive(Serialize)]
struct HfOptions {
    wait_for_model: bool,
}

impl HuggingFaceProvider {
    /// Create a new HuggingFace provider
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60)) // HF can be slower
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
            "Detecting embedding dimension for HuggingFace model: {}",
            self.model
        );

        let test_embedding = self.call_api("test").await?;
        let dimension = test_embedding.len();

        self.dimension = Some(dimension);
        log::info!("Detected dimension: {} for model {}", dimension, self.model);

        Ok(dimension)
    }

    /// Make API call to HuggingFace Inference API
    async fn call_api(&self, text: &str) -> Result<Vec<f32>> {
        let request = HfEmbeddingRequest {
            inputs: text.to_string(),
            options: HfOptions {
                wait_for_model: true,
            },
        };

        let url = format!("https://api-inference.huggingface.co/models/{}", self.model);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "HuggingFace API error: HTTP {} - {}",
                status,
                error_text
            ));
        }

        // HuggingFace returns embeddings as a flat array
        let embeddings: Vec<f32> = response.json().await?;

        if embeddings.is_empty() {
            return Err(anyhow!("No embeddings returned from HuggingFace API"));
        }

        Ok(embeddings)
    }

    /// Get common HuggingFace model information (dimension, max_length)
    pub fn get_model_info(model: &str) -> (Option<usize>, usize) {
        match model {
            "sentence-transformers/all-MiniLM-L6-v2" => (Some(384), 512),
            "sentence-transformers/all-mpnet-base-v2" => (Some(768), 512),
            "sentence-transformers/multi-qa-MiniLM-L6-cos-v1" => (Some(384), 512),
            "BAAI/bge-small-en-v1.5" => (Some(384), 512),
            "BAAI/bge-base-en-v1.5" => (Some(768), 512),
            "BAAI/bge-large-en-v1.5" => (Some(1024), 512),
            _ => (None, 512), // Unknown model, will detect dynamically
        }
    }
}

#[async_trait]
impl ProviderTrait for HuggingFaceProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        // Truncate text if too long
        let (_, max_chars) = Self::get_model_info(&self.model);
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
            let (known_dim, _) = Self::get_model_info(&self.model);
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
        self.call_api("test").await.map(|_| ())
    }

    fn get_info(&self) -> ProviderInfo {
        let (_, max_length) = Self::get_model_info(&self.model);

        ProviderInfo {
            name: "HuggingFace Inference API".to_string(),
            provider_type: "huggingface".to_string(),
            model_name: Some(self.model.clone()),
            description: format!("HuggingFace model: {}", self.model),
            max_input_length: Some(max_length),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
