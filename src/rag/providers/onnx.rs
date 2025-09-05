use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::PathBuf;

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};
use crate::rag::model_metadata::{ModelMetadata, ModelMetadataManager};

/// ONNX-based embedding provider
pub struct OnnxProvider {
    model_name: String,
    dimension: usize,
    max_length: usize,
}

impl OnnxProvider {
    /// Create a new ONNX provider from an installed model
    pub async fn new(model_name: &str) -> Result<Self> {
        let mut metadata_manager = ModelMetadataManager::new()?;

        // Get model metadata
        let metadata = metadata_manager.get_model(model_name).ok_or_else(|| {
            anyhow!(
                "Model '{}' not found. Use 'manx embedding download {}' first",
                model_name,
                model_name
            )
        })?;

        // Load model files
        let model_dir = metadata
            .model_path
            .as_ref()
            .ok_or_else(|| anyhow!("No model path found for {}", model_name))?;

        let onnx_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !onnx_path.exists() {
            return Err(anyhow!("ONNX model file not found at {:?}", onnx_path));
        }

        if !tokenizer_path.exists() {
            return Err(anyhow!("Tokenizer file not found at {:?}", tokenizer_path));
        }

        // TODO: Initialize ONNX Runtime (placeholder implementation)
        log::info!("ONNX model loaded (placeholder): {:?}", onnx_path);

        // TODO: Load tokenizer (placeholder implementation)
        log::info!("Tokenizer loaded (placeholder): {:?}", tokenizer_path);

        let dimension = metadata.dimension;
        let max_length = metadata.max_input_length.unwrap_or(512);

        // Mark model as used
        metadata_manager.mark_used(model_name)?;

        Ok(Self {
            model_name: model_name.to_string(),
            dimension,
            max_length,
        })
    }

    /// Download and install an ONNX model from HuggingFace
    pub async fn download_model(model_name: &str, force: bool) -> Result<()> {
        let mut metadata_manager = ModelMetadataManager::new()?;

        // Check if already installed
        if !force && metadata_manager.get_model(model_name).is_some() {
            return Err(anyhow!(
                "Model '{}' already installed. Use --force to reinstall",
                model_name
            ));
        }

        log::info!("Downloading model: {}", model_name);

        // Create model directory
        let models_dir = ModelMetadataManager::get_models_dir();
        let model_dir = models_dir.join(model_name.replace('/', "_"));
        std::fs::create_dir_all(&model_dir)?;

        // Download files from HuggingFace
        let files_to_download = vec![
            ("onnx/model.onnx", "model.onnx"),
            ("tokenizer.json", "tokenizer.json"),
            ("config.json", "config.json"),
        ];

        let client = reqwest::Client::new();
        let mut total_size = 0u64;
        let mut dimension = None;

        for (remote_path, local_filename) in files_to_download {
            let url = format!(
                "https://huggingface.co/{}/resolve/main/{}",
                model_name, remote_path
            );
            let local_path = model_dir.join(local_filename);

            log::info!("Downloading: {} -> {:?}", url, local_path);

            let response = client.get(&url).send().await?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Failed to download {}: HTTP {}",
                    url,
                    response.status()
                ));
            }

            let bytes = response.bytes().await?;
            std::fs::write(&local_path, &bytes)?;
            total_size += bytes.len() as u64;

            log::info!("Downloaded {} ({} bytes)", local_filename, bytes.len());

            // Try to extract dimension from config.json
            if local_filename == "config.json" {
                if let Ok(config_str) = std::fs::read_to_string(&local_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                        if let Some(hidden_size) =
                            config.get("hidden_size").and_then(|v| v.as_u64())
                        {
                            dimension = Some(hidden_size as usize);
                        }
                    }
                }
            }
        }

        // If we couldn't get dimension from config, try to detect from ONNX model
        if dimension.is_none() {
            dimension =
                Some(Self::detect_dimension_from_onnx(&model_dir.join("model.onnx")).await?);
        }

        let dimension = dimension
            .ok_or_else(|| anyhow!("Could not detect dimension from model config or ONNX file"))?;

        // Create metadata
        let metadata = ModelMetadata {
            model_name: model_name.to_string(),
            provider_type: "onnx".to_string(),
            dimension,
            size_mb: total_size as f64 / 1_048_576.0, // Convert to MB
            model_path: Some(model_dir),
            api_endpoint: None,
            installed_date: chrono::Utc::now(),
            last_used: None,
            checksum: None, // TODO: Add SHA256 checksums
            description: Some(format!("ONNX model: {}", model_name)),
            max_input_length: Some(512), // Common default
        };

        metadata_manager.add_model(metadata)?;
        log::info!(
            "Successfully installed model: {} ({}D, {:.1}MB)",
            model_name,
            dimension,
            total_size as f64 / 1_048_576.0
        );

        Ok(())
    }

    /// Detect embedding dimension from ONNX model
    async fn detect_dimension_from_onnx(onnx_path: &PathBuf) -> Result<usize> {
        // TODO: Implement proper ONNX model introspection
        log::info!("Detecting dimension from ONNX model: {:?}", onnx_path);

        // Fallback: try a test inference
        // This is more complex and might require proper tokenization
        Err(anyhow!("Could not detect dimension from ONNX model"))
    }

    /// List available models that can be downloaded
    pub fn list_available_models() -> Vec<&'static str> {
        vec![
            "sentence-transformers/all-MiniLM-L6-v2",
            "sentence-transformers/all-mpnet-base-v2",
            "sentence-transformers/multi-qa-MiniLM-L6-cos-v1",
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
            "BAAI/bge-small-en-v1.5",
            "BAAI/bge-base-en-v1.5",
            "BAAI/bge-large-en-v1.5",
        ]
    }
}

#[async_trait]
impl ProviderTrait for OnnxProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        // Simple tokenization placeholder (would use proper tokenizer in real implementation)
        let words: Vec<&str> = text.split_whitespace().collect();
        let _token_count = std::cmp::min(words.len(), self.max_length);

        // For now, return a mock embedding since ONNX integration is complex
        // This would be replaced with proper ONNX tensor operations
        log::warn!("ONNX inference not fully implemented yet, returning mock embedding");

        // Create a simple deterministic embedding based on text content
        let mut sentence_embedding = Vec::with_capacity(self.dimension);
        let text_bytes = text.as_bytes();

        for i in 0..self.dimension {
            let byte_idx = i % text_bytes.len();
            let val = (text_bytes[byte_idx] as f32 + i as f32) / 255.0;
            sentence_embedding.push(val.sin()); // Add some variation
        }

        // Normalize
        let norm = sentence_embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            Ok(sentence_embedding.iter().map(|x| x / norm).collect())
        } else {
            Ok(sentence_embedding)
        }
    }

    async fn get_dimension(&self) -> Result<usize> {
        Ok(self.dimension)
    }

    async fn health_check(&self) -> Result<()> {
        // Try a simple inference
        self.embed_text("test").await.map(|_| ())
    }

    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "ONNX Local Model".to_string(),
            provider_type: "onnx".to_string(),
            model_name: Some(self.model_name.clone()),
            description: format!("Local ONNX model: {}", self.model_name),
            max_input_length: Some(self.max_length),
        }
    }
}
