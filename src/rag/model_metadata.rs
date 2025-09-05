use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Metadata about an installed embedding model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_name: String,
    pub provider_type: String,
    pub dimension: usize,
    pub size_mb: f64,
    pub model_path: Option<PathBuf>,
    pub api_endpoint: Option<String>,
    pub installed_date: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub checksum: Option<String>,
    pub description: Option<String>,
    pub max_input_length: Option<usize>,
}

/// Manager for model metadata storage and retrieval
pub struct ModelMetadataManager {
    metadata_file: PathBuf,
    models: HashMap<String, ModelMetadata>,
}

impl ModelMetadataManager {
    /// Create a new metadata manager
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("manx")
            .join("models");

        std::fs::create_dir_all(&cache_dir)?;
        let metadata_file = cache_dir.join("metadata.json");

        let models = if metadata_file.exists() {
            let content = std::fs::read_to_string(&metadata_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            metadata_file,
            models,
        })
    }

    /// Get metadata for a specific model
    pub fn get_model(&self, model_name: &str) -> Option<&ModelMetadata> {
        self.models.get(model_name)
    }

    /// Add or update model metadata
    pub fn add_model(&mut self, metadata: ModelMetadata) -> Result<()> {
        self.models.insert(metadata.model_name.clone(), metadata);
        self.save()
    }

    /// Remove model metadata
    pub fn remove_model(&mut self, model_name: &str) -> Result<()> {
        self.models.remove(model_name);
        self.save()
    }

    /// List all installed models
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// Update last used timestamp for a model
    pub fn mark_used(&mut self, model_name: &str) -> Result<()> {
        if let Some(model) = self.models.get_mut(model_name) {
            model.last_used = Some(Utc::now());
            self.save()?;
        }
        Ok(())
    }

    /// Get the cache directory for models
    pub fn get_models_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("manx")
            .join("models")
    }

    /// Save metadata to disk
    fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.models)?;
        std::fs::write(&self.metadata_file, content)?;
        Ok(())
    }
}
