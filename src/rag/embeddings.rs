//! Text embedding generation using configurable embedding providers
//!
//! This module provides flexible text embedding functionality for semantic similarity search.
//! Supports multiple embedding providers: hash-based (default), local models, and API services.
//! Users can configure their preferred embedding method via `manx config --embedding-provider`.

use crate::rag::providers::{
    custom, hash, huggingface, ollama, onnx, openai, EmbeddingProvider as ProviderTrait,
};
use crate::rag::{EmbeddingConfig, EmbeddingProvider};
use anyhow::{anyhow, Result};

/// Text embedding model wrapper with configurable providers
/// Supports hash-based embeddings (default), local ONNX models, and API services.
/// Users can configure their preferred embedding method via `manx config`.
pub struct EmbeddingModel {
    provider: Box<dyn ProviderTrait + Send + Sync>,
    config: EmbeddingConfig,
}

impl EmbeddingModel {
    /// Create a new embedding model with default hash-based provider
    pub async fn new() -> Result<Self> {
        Self::new_with_config(EmbeddingConfig::default()).await
    }

    /// Create a new embedding model with custom configuration
    pub async fn new_with_config(config: EmbeddingConfig) -> Result<Self> {
        log::info!(
            "Initializing embedding model with provider: {:?}",
            config.provider
        );

        let provider: Box<dyn ProviderTrait + Send + Sync> = match &config.provider {
            EmbeddingProvider::Hash => {
                log::info!("Using hash-based embeddings (default provider)");
                Box::new(hash::HashProvider::new(384)) // Hash provider always uses 384 dimensions
            }
            EmbeddingProvider::Onnx(model_name) => {
                log::info!("Loading ONNX model: {}", model_name);
                let onnx_provider = onnx::OnnxProvider::new(model_name).await?;
                Box::new(onnx_provider)
            }
            EmbeddingProvider::Ollama(model_name) => {
                log::info!("Connecting to Ollama model: {}", model_name);
                let ollama_provider =
                    ollama::OllamaProvider::new(model_name.clone(), config.endpoint.clone());
                // Test connection
                ollama_provider.health_check().await?;
                Box::new(ollama_provider)
            }
            EmbeddingProvider::OpenAI(model_name) => {
                log::info!("Connecting to OpenAI model: {}", model_name);
                let api_key = config.api_key.as_ref().ok_or_else(|| {
                    anyhow!("OpenAI API key required. Use 'manx config --embedding-api-key <key>'")
                })?;
                let openai_provider =
                    openai::OpenAiProvider::new(api_key.clone(), model_name.clone());
                Box::new(openai_provider)
            }
            EmbeddingProvider::HuggingFace(model_name) => {
                log::info!("Connecting to HuggingFace model: {}", model_name);
                let api_key = config.api_key.as_ref().ok_or_else(|| {
                    anyhow!(
                        "HuggingFace API key required. Use 'manx config --embedding-api-key <key>'"
                    )
                })?;
                let hf_provider =
                    huggingface::HuggingFaceProvider::new(api_key.clone(), model_name.clone());
                Box::new(hf_provider)
            }
            EmbeddingProvider::Custom(endpoint) => {
                log::info!("Connecting to custom endpoint: {}", endpoint);
                let custom_provider =
                    custom::CustomProvider::new(endpoint.clone(), config.api_key.clone());
                Box::new(custom_provider)
            }
        };

        Ok(Self { provider, config })
    }

    /// Generate embeddings for a single text using configured provider
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        self.provider.embed_text(text).await
    }

    /// Get the dimension of embeddings produced by this model
    pub async fn get_dimension(&self) -> Result<usize> {
        self.provider.get_dimension().await
    }

    /// Test if the embedding model is working correctly
    pub async fn health_check(&self) -> Result<()> {
        self.provider.health_check().await
    }

    /// Get information about the current provider
    pub fn get_provider_info(&self) -> crate::rag::providers::ProviderInfo {
        self.provider.get_info()
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &EmbeddingConfig {
        &self.config
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

/// Utility functions for text preprocessing
pub mod preprocessing {
    /// Clean and normalize text for embedding
    pub fn clean_text(text: &str) -> String {
        // Remove excessive whitespace
        let cleaned = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Limit length to prevent very long embeddings
        const MAX_LENGTH: usize = 2048;
        if cleaned.len() > MAX_LENGTH {
            format!("{}...", &cleaned[..MAX_LENGTH])
        } else {
            cleaned
        }
    }

    /// Split text into chunks suitable for embedding
    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.len() <= chunk_size {
            chunks.push(text.to_string());
            return chunks;
        }

        let mut start = 0;
        while start < words.len() {
            let end = std::cmp::min(start + chunk_size, words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);

            if end == words.len() {
                break;
            }

            start = end - overlap;
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_model() {
        let model = EmbeddingModel::new().await.unwrap();

        let text = "This is a test sentence for embedding.";
        let embedding = model.embed_text(text).await.unwrap();

        assert_eq!(embedding.len(), 384); // Hash provider default
        assert!(embedding.iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = EmbeddingModel::cosine_similarity(&a, &b);
        assert!((similarity - 1.0).abs() < 0.001);

        let c = vec![-1.0, -2.0, -3.0];
        let similarity2 = EmbeddingModel::cosine_similarity(&a, &c);
        assert!((similarity2 + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_text_preprocessing() {
        let text = "  This is   a test\n\n  with  multiple   lines  \n  ";
        let cleaned = preprocessing::clean_text(text);
        assert_eq!(cleaned, "This is a test with multiple lines");
    }

    #[test]
    fn test_text_chunking() {
        let text = "one two three four five six seven eight nine ten";
        let chunks = preprocessing::chunk_text(text, 3, 1);

        assert_eq!(chunks.len(), 5);
        assert_eq!(chunks[0], "one two three");
        assert_eq!(chunks[1], "three four five");
        assert_eq!(chunks[2], "five six seven");
        assert_eq!(chunks[3], "seven eight nine");
        assert_eq!(chunks[4], "nine ten");
    }

    #[tokio::test]
    async fn test_similarity_detection() {
        let model = EmbeddingModel::new().await.unwrap();

        let text1 = "React hooks useState";
        let text2 = "useState React hooks";
        let text3 = "Python Django models";

        let emb1 = model.embed_text(text1).await.unwrap();
        let emb2 = model.embed_text(text2).await.unwrap();
        let emb3 = model.embed_text(text3).await.unwrap();

        let sim_12 = EmbeddingModel::cosine_similarity(&emb1, &emb2);
        let sim_13 = EmbeddingModel::cosine_similarity(&emb1, &emb3);

        // Similar texts should have higher similarity
        assert!(sim_12 > sim_13);
    }
}
