use anyhow::Result;
use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::{EmbeddingProvider as ProviderTrait, ProviderInfo};
use crate::rag::embeddings::preprocessing;

/// Hash-based embedding provider (fast, lightweight, no dependencies)
pub struct HashProvider {
    dimension: usize,
}

impl HashProvider {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl ProviderTrait for HashProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow::anyhow!("Cannot embed empty text"));
        }

        let embedding = self.generate_hash_embedding(text);
        Ok(embedding)
    }

    async fn get_dimension(&self) -> Result<usize> {
        Ok(self.dimension)
    }

    async fn health_check(&self) -> Result<()> {
        // Hash provider is always available
        Ok(())
    }

    fn get_info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "Hash-based Embeddings".to_string(),
            provider_type: "hash".to_string(),
            model_name: None,
            description: "Fast hash-based embeddings for basic semantic similarity".to_string(),
            max_input_length: Some(2048),
        }
    }
}

impl HashProvider {
    /// Generate hash-based embedding vector
    fn generate_hash_embedding(&self, text: &str) -> Vec<f32> {
        let cleaned_text = preprocessing::clean_text(text);
        let words: Vec<&str> = cleaned_text.split_whitespace().collect();

        let mut embedding = vec![0.0; self.dimension];

        // Generate hash-based features for each word
        for word in &words {
            let mut hasher = DefaultHasher::new();
            word.to_lowercase().hash(&mut hasher);
            let hash = hasher.finish();

            // Distribute hash across embedding dimensions
            for i in 0..self.dimension {
                let feature_hash = (hash.wrapping_mul(i as u64 + 1)) as usize % self.dimension;
                embedding[feature_hash] += 1.0;
            }
        }

        // Add n-gram features for better similarity
        for window in words.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            let mut hasher = DefaultHasher::new();
            bigram.to_lowercase().hash(&mut hasher);
            let hash = hasher.finish();

            for i in 0..self.dimension {
                let feature_hash = (hash.wrapping_mul(i as u64 + 1)) as usize % self.dimension;
                embedding[feature_hash] += 0.5; // Lower weight for n-grams
            }
        }

        // Normalize the embedding vector
        let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}
