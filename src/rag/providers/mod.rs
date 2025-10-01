use anyhow::Result;

pub mod custom;
pub mod hash;
pub mod huggingface;
pub mod ollama;
pub mod onnx;
pub mod openai;

/// Trait for embedding providers
#[async_trait::async_trait]
pub trait EmbeddingProvider: std::any::Any {
    /// Generate embeddings for a single text
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;

    /// Get the dimension of embeddings produced by this provider
    async fn get_dimension(&self) -> Result<usize>;

    /// Test if the provider is available and working
    async fn health_check(&self) -> Result<()>;

    /// Get provider-specific information
    fn get_info(&self) -> ProviderInfo;

    /// Downcast support for accessing provider-specific methods
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Information about an embedding provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub provider_type: String,
    pub model_name: Option<String>,
    pub description: String,
    pub max_input_length: Option<usize>,
}
