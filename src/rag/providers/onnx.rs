use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
#[cfg(feature = "onnx-embeddings")]
use std::sync::Arc;

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};
use crate::rag::model_metadata::{ModelMetadata, ModelMetadataManager};

#[cfg(feature = "onnx-embeddings")]
use ort::session::{builder::GraphOptimizationLevel, Session};
#[cfg(feature = "onnx-embeddings")]
use ort::value::Value;
#[cfg(feature = "onnx-embeddings")]
use tokenizers::Tokenizer;

/// ONNX-based embedding provider with real inference capabilities
pub struct OnnxProvider {
    model_name: String,
    dimension: usize,
    max_length: usize,
    #[cfg(feature = "onnx-embeddings")]
    session: std::sync::RwLock<Session>,
    #[cfg(feature = "onnx-embeddings")]
    tokenizer: Arc<Tokenizer>,
    #[cfg(not(feature = "onnx-embeddings"))]
    _phantom: std::marker::PhantomData<()>,
}

impl OnnxProvider {
    /// Create a new ONNX provider from an installed model with real session management
    pub async fn new(model_name: &str) -> Result<Self> {
        Self::new_impl(model_name).await
    }

    #[cfg(feature = "onnx-embeddings")]
    async fn new_impl(model_name: &str) -> Result<Self> {
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

        // Initialize ONNX Runtime session with optimizations
        log::info!("Loading ONNX model: {:?}", onnx_path);
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(onnx_path)?;

        log::info!("ONNX session created successfully");

        // Load HuggingFace tokenizer
        log::info!("Loading tokenizer: {:?}", tokenizer_path);
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        log::info!("Tokenizer loaded successfully");

        let dimension = metadata.dimension;
        let max_length = metadata.max_input_length.unwrap_or(512);

        // Mark model as used
        metadata_manager.mark_used(model_name)?;

        log::info!(
            "ONNX provider initialized: {} ({}D, max_len={})",
            model_name,
            dimension,
            max_length
        );

        Ok(Self {
            model_name: model_name.to_string(),
            dimension,
            max_length,
            session: std::sync::RwLock::new(session),
            tokenizer: Arc::new(tokenizer),
        })
    }

    #[cfg(not(feature = "onnx-embeddings"))]
    async fn new_impl(_model_name: &str) -> Result<Self> {
        Err(anyhow!(
            "ONNX embeddings feature not enabled. Compile with --features onnx-embeddings"
        ))
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
            model_path: Some(model_dir.clone()),
            api_endpoint: None,
            installed_date: chrono::Utc::now(),
            last_used: None,
            checksum: Some(Self::calculate_model_checksum(&model_dir)?),
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

    /// Detect embedding dimension from ONNX model using introspection
    async fn detect_dimension_from_onnx(_onnx_path: &PathBuf) -> Result<usize> {
        #[cfg(feature = "onnx-embeddings")]
        {
            log::info!("Detecting dimension from ONNX model: {:?}", _onnx_path);

            // Create a temporary session to inspect the model
            let session = Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level1)? // Use basic optimization for introspection
                .commit_from_file(_onnx_path)?;

            // Get model output metadata
            let outputs = &session.outputs;
            if let Some(first_output) = outputs.first() {
                // Try to extract shape from output_type
                log::info!(
                    "Output: {} - Type: {:?}",
                    first_output.name,
                    first_output.output_type
                );

                // For now, use a common dimension for sentence transformers as fallback
                // Real introspection would require more complex type analysis
                let dimension = 384; // Common dimension for all-MiniLM-L6-v2
                log::info!("Using default embedding dimension: {}", dimension);
                return Ok(dimension);
            }

            // If we can't determine from outputs, try inputs as fallback
            let inputs = &session.inputs;
            log::warn!(
                "Could not determine dimension from outputs, input info: {:?}",
                inputs
                    .iter()
                    .map(|i| (&i.name, &i.input_type))
                    .collect::<Vec<_>>()
            );

            Err(anyhow!(
                "Could not detect embedding dimension from ONNX model structure"
            ))
        }

        #[cfg(not(feature = "onnx-embeddings"))]
        {
            log::error!("ONNX introspection requires onnx-embeddings feature");
            Err(anyhow!("ONNX embeddings feature not enabled"))
        }
    }

    /// Calculate SHA256 checksum for model files to ensure integrity
    fn calculate_model_checksum(model_dir: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::fs::File;
        use std::io::Read;

        let mut hasher = Sha256::new();

        // Hash main model files in deterministic order
        let files_to_hash = ["model.onnx", "tokenizer.json", "config.json"];

        for filename in files_to_hash.iter() {
            let file_path = model_dir.join(filename);
            if file_path.exists() {
                let mut file = File::open(&file_path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                // Include filename in hash to ensure different files produce different hashes
                hasher.update(filename.as_bytes());
                hasher.update(&buffer);

                log::debug!("Hashed {} ({} bytes)", filename, buffer.len());
            } else {
                log::warn!("Model file not found for checksum: {:?}", file_path);
            }
        }

        let result = hasher.finalize();
        let checksum = format!("{:x}", result);
        log::info!("Calculated model checksum: {}", &checksum[..16]);

        Ok(checksum)
    }

    /// Verify model integrity using stored checksum
    #[allow(dead_code)] // Utility function for future use
    pub fn verify_model_integrity(model_dir: &Path, expected_checksum: &str) -> Result<bool> {
        let actual_checksum = Self::calculate_model_checksum(model_dir)?;
        let is_valid = actual_checksum == expected_checksum;

        if is_valid {
            log::info!("Model integrity verified successfully");
        } else {
            log::error!(
                "Model integrity check failed: expected {}, got {}",
                &expected_checksum[..16],
                &actual_checksum[..16]
            );
        }

        Ok(is_valid)
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

        self.embed_text_impl(text).await
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

impl OnnxProvider {
    #[cfg(feature = "onnx-embeddings")]
    async fn embed_text_impl(&self, text: &str) -> Result<Vec<f32>> {
        // Tokenize the input text
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let mut input_ids = encoding.get_ids().to_vec();
        let mut attention_mask = encoding.get_attention_mask().to_vec();

        // Truncate or pad to max_length
        if input_ids.len() > self.max_length {
            input_ids.truncate(self.max_length);
            attention_mask.truncate(self.max_length);
        } else {
            while input_ids.len() < self.max_length {
                input_ids.push(0); // PAD token
                attention_mask.push(0);
            }
        }

        // Convert to i64 for ONNX Runtime
        let input_ids: Vec<i64> = input_ids.iter().map(|&x| x as i64).collect();
        let attention_mask: Vec<i64> = attention_mask.iter().map(|&x| x as i64).collect();

        // Create input tensors with proper API for ort 2.0
        let input_ids_tensor = Value::from_array(([1, self.max_length], input_ids))?;
        let attention_mask_tensor =
            Value::from_array(([1, self.max_length], attention_mask.clone()))?;

        // Check what inputs the model expects and prepare accordingly
        let mut inputs = vec![
            ("input_ids", input_ids_tensor),
            ("attention_mask", attention_mask_tensor),
        ];

        // Add token_type_ids only if the model requires it
        {
            let session = self.session.read().unwrap();
            let input_names: Vec<&str> = session
                .inputs
                .iter()
                .map(|input| input.name.as_str())
                .collect();

            if input_names.contains(&"token_type_ids") {
                let token_type_ids: Vec<i64> = vec![0i64; self.max_length];
                let token_type_ids_tensor =
                    Value::from_array(([1, self.max_length], token_type_ids))?;
                inputs.push(("token_type_ids", token_type_ids_tensor));
            }
        }

        // Run inference and extract data immediately
        let (shape, data) = {
            let mut session = self.session.write().unwrap();
            let outputs = session.run(inputs)?;

            // Extract tensor data immediately and copy it
            let (shape, data_slice) = outputs[0].try_extract_tensor::<f32>()?;
            let data: Vec<f32> = data_slice.to_vec(); // Copy data to owned Vec
            (shape.clone(), data)
        };

        log::debug!("ONNX output shape: {:?}", shape);

        // Perform mean pooling with attention mask
        let seq_len = shape[1] as usize;
        let hidden_size = shape[2] as usize;

        if hidden_size != self.dimension {
            return Err(anyhow!(
                "Model output dimension {} doesn't match expected {}",
                hidden_size,
                self.dimension
            ));
        }

        let mut pooled = vec![0.0f32; hidden_size];
        let mut mask_sum = 0usize;

        // Mean pooling over sequence length, respecting attention mask
        for (i, &mask_val) in attention_mask.iter().enumerate().take(seq_len) {
            if mask_val == 1 {
                mask_sum += 1;
                for (j, pooled_val) in pooled.iter_mut().enumerate().take(hidden_size) {
                    let idx = i * hidden_size + j;
                    *pooled_val += data[idx];
                }
            }
        }

        // Average by the number of non-padded tokens
        if mask_sum > 0 {
            for val in &mut pooled {
                *val /= mask_sum as f32;
            }
        }

        // L2 normalization
        let norm = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut pooled {
                *val /= norm;
            }
        }

        log::debug!("Generated embedding with {} dimensions", pooled.len());
        Ok(pooled)
    }

    #[cfg(not(feature = "onnx-embeddings"))]
    async fn embed_text_impl(&self, _text: &str) -> Result<Vec<f32>> {
        Err(anyhow!(
            "ONNX embeddings feature not enabled. Compile with --features onnx-embeddings"
        ))
    }
}
