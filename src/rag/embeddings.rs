//! Text embedding generation using simple hash-based embeddings
//!
//! This module provides basic text embedding functionality for semantic similarity search.
//! For production use with advanced semantic understanding, users can integrate
//! external embedding services or install PyTorch-based models separately.

use anyhow::{anyhow, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Text embedding model wrapper using hash-based embeddings
/// This is a simplified implementation suitable for basic semantic search.
/// For advanced BERT-based embeddings, users can integrate external services.
pub struct EmbeddingModel {
    dimension: usize,
}

impl EmbeddingModel {
    /// Create a new embedding model
    /// Uses hash-based embeddings for basic semantic similarity
    pub async fn new() -> Result<Self> {
        log::info!("Initializing hash-based embedding model...");

        Ok(Self {
            dimension: 384, // Standard embedding dimension
        })
    }

    /// Generate embeddings for a single text using hash-based approach
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Err(anyhow!("Cannot embed empty text"));
        }

        // Simple hash-based embedding generation
        // This provides basic similarity matching for identical/similar terms
        let embedding = self.generate_hash_embedding(text);
        Ok(embedding)
    }

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

        assert_eq!(embedding.len(), 384);
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
