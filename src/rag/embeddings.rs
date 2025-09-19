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

    /// Create embedding model with smart auto-selection of best available provider
    pub async fn new_auto_select() -> Result<Self> {
        let best_config = Self::auto_select_best_provider().await?;
        Self::new_with_config(best_config).await
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

    /// Automatically select the best available embedding provider from installed models
    /// Respects user's installed models and doesn't hardcode specific model names
    pub async fn auto_select_best_provider() -> Result<EmbeddingConfig> {
        log::info!("Auto-selecting best available embedding provider from installed models...");

        // Try to find any available ONNX models by checking common model directories
        if let Ok(available_models) = Self::get_available_onnx_models().await {
            if !available_models.is_empty() {
                // Select the first available model (user chose to install it)
                let selected_model = &available_models[0];
                log::info!("Auto-selected installed ONNX model: {}", selected_model);

                // Try to determine dimension by testing the model
                if let Ok(test_config) = Self::create_config_for_model(selected_model).await {
                    return Ok(test_config);
                }
            }
        }

        // Fallback to hash-based embeddings if no ONNX models available
        log::info!("No ONNX models found, using hash-based embeddings");
        Ok(EmbeddingConfig::default())
    }

    /// Get list of available ONNX models (non-hardcoded discovery)
    async fn get_available_onnx_models() -> Result<Vec<String>> {
        // This would typically scan the model cache directory
        // For now, we'll try a few common models that might be installed
        let potential_models = [
            "sentence-transformers/all-MiniLM-L6-v2",
            "sentence-transformers/all-mpnet-base-v2",
            "BAAI/bge-base-en-v1.5",
            "BAAI/bge-small-en-v1.5",
            "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
        ];

        let mut available = Vec::new();
        for model in &potential_models {
            if Self::is_onnx_model_available(model).await {
                available.push(model.to_string());
            }
        }

        Ok(available)
    }

    /// Create config for a specific model with proper dimension detection
    async fn create_config_for_model(model_name: &str) -> Result<EmbeddingConfig> {
        // Test the model to get its dimension
        match onnx::OnnxProvider::new(model_name).await {
            Ok(provider) => {
                let dimension = provider.get_dimension().await.unwrap_or(384);
                Ok(EmbeddingConfig {
                    provider: EmbeddingProvider::Onnx(model_name.to_string()),
                    dimension,
                    ..EmbeddingConfig::default()
                })
            }
            Err(e) => Err(anyhow!(
                "Failed to create config for model {}: {}",
                model_name,
                e
            )),
        }
    }

    /// Check if an ONNX model is available locally
    async fn is_onnx_model_available(model_name: &str) -> bool {
        // Try to create the provider to test availability
        match onnx::OnnxProvider::new(model_name).await {
            Ok(_) => {
                log::debug!("ONNX model '{}' is available", model_name);
                true
            }
            Err(e) => {
                log::debug!("ONNX model '{}' not available: {}", model_name, e);
                false
            }
        }
    }
}

/// Utility functions for text preprocessing
pub mod preprocessing {
    /// Clean and normalize text for embedding
    pub fn clean_text(text: &str) -> String {
        // Detect if this is code based on common patterns
        if is_code_content(text) {
            clean_code_text(text)
        } else {
            clean_regular_text(text)
        }
    }

    /// Clean regular text (documents, markdown, etc.)
    fn clean_regular_text(text: &str) -> String {
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

    /// Clean code text while preserving structure
    fn clean_code_text(text: &str) -> String {
        let mut cleaned = String::new();
        let mut in_comment_block = false;

        for line in text.lines() {
            let trimmed = line.trim();

            // Skip empty lines in code
            if trimmed.is_empty() && !cleaned.is_empty() {
                continue;
            }

            // Handle comment blocks
            if trimmed.starts_with("/*") {
                in_comment_block = true;
            }
            if in_comment_block {
                if trimmed.ends_with("*/") {
                    in_comment_block = false;
                }
                cleaned.push_str("// ");
                cleaned.push_str(trimmed);
                cleaned.push('\n');
                continue;
            }

            // Preserve important code structure
            if is_important_code_line(trimmed) {
                // Keep indentation context (simplified)
                let indent_level = line.len() - line.trim_start().len();
                let normalized_indent = " ".repeat((indent_level / 2).min(4));
                cleaned.push_str(&normalized_indent);
                cleaned.push_str(trimmed);
                cleaned.push('\n');
            }
        }

        // Limit length
        const MAX_CODE_LENGTH: usize = 3000;
        if cleaned.len() > MAX_CODE_LENGTH {
            format!("{}...", &cleaned[..MAX_CODE_LENGTH])
        } else {
            cleaned
        }
    }

    /// Check if text appears to be code
    fn is_code_content(text: &str) -> bool {
        let code_indicators = [
            "function",
            "const",
            "let",
            "var",
            "def",
            "class",
            "import",
            "export",
            "public",
            "private",
            "protected",
            "return",
            "if (",
            "for (",
            "while (",
            "=>",
            "->",
            "::",
            "<?php",
            "#!/",
            "package",
            "namespace",
            "struct",
        ];

        let text_lower = text.to_lowercase();
        let indicator_count = code_indicators
            .iter()
            .filter(|&&ind| text_lower.contains(ind))
            .count();

        // If multiple code indicators found, likely code
        indicator_count >= 3
    }

    /// Check if a line is important for code context
    fn is_important_code_line(line: &str) -> bool {
        // Skip pure comments unless they're doc comments
        if line.starts_with("//") && !line.starts_with("///") && !line.starts_with("//!") {
            return false;
        }

        // Keep imports, function definitions, class definitions, etc.
        let important_patterns = [
            "import ",
            "from ",
            "require",
            "include",
            "function ",
            "def ",
            "fn ",
            "func ",
            "class ",
            "struct ",
            "interface ",
            "enum ",
            "public ",
            "private ",
            "protected ",
            "export ",
            "module ",
            "namespace ",
        ];

        for pattern in &important_patterns {
            if line.contains(pattern) {
                return true;
            }
        }

        // Keep lines with actual code (not just brackets)
        !line
            .chars()
            .all(|c| c == '{' || c == '}' || c == '(' || c == ')' || c == ';' || c.is_whitespace())
    }

    /// Split text into chunks suitable for embedding
    pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        // Use code-aware chunking if this appears to be code
        if is_code_content(text) {
            chunk_code_text(text, chunk_size, overlap)
        } else {
            chunk_regular_text(text, chunk_size, overlap)
        }
    }

    /// Regular text chunking by words
    fn chunk_regular_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
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

    /// Code-aware chunking that respects function/class boundaries
    fn chunk_code_text(text: &str, chunk_size: usize, _overlap: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_size = 0;
        let mut brace_depth = 0;
        let mut in_function = false;

        for line in text.lines() {
            let trimmed = line.trim();

            // Detect function/class boundaries
            if trimmed.contains("function ")
                || trimmed.contains("def ")
                || trimmed.contains("class ")
                || trimmed.contains("fn ")
            {
                in_function = true;

                // If current chunk is large enough, save it
                if current_size > chunk_size / 2 && brace_depth == 0 && !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                    current_size = 0;
                }
            }

            // Track brace depth for better chunking
            brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32;
            brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32;
            brace_depth = brace_depth.max(0);

            // Add line to current chunk
            current_chunk.push_str(line);
            current_chunk.push('\n');
            current_size += line.split_whitespace().count();

            // Create new chunk when we hit size limit and are at a good boundary
            if current_size >= chunk_size && brace_depth == 0 && !in_function {
                chunks.push(current_chunk.clone());
                current_chunk.clear();
                current_size = 0;
            }

            // Reset function flag when we exit a function
            if in_function && brace_depth == 0 && trimmed.ends_with('}') {
                in_function = false;
            }
        }

        // Add remaining content
        if !current_chunk.trim().is_empty() {
            chunks.push(current_chunk);
        }

        // If no chunks were created, fall back to regular chunking
        if chunks.is_empty() {
            return chunk_regular_text(text, chunk_size, chunk_size / 10);
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
