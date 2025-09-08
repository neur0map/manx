//! Benchmarking utilities for embedding providers
//!
//! This module provides tools to benchmark and compare different embedding providers,
//! measuring both performance and quality metrics.

#![allow(dead_code)] // Benchmarking infrastructure used by examples

use crate::rag::providers::EmbeddingProvider as ProviderTrait;
use anyhow::Result;
use std::time::{Duration, Instant};

/// Benchmark results for an embedding provider
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    pub provider_name: String,
    pub total_texts: usize,
    pub total_duration: Duration,
    pub avg_embedding_time: Duration,
    pub embeddings_per_second: f64,
    pub dimension: usize,
    pub memory_usage_mb: Option<f64>,
    pub semantic_quality_score: Option<f64>,
}

/// Test data for benchmarking
pub struct BenchmarkTestData {
    pub texts: Vec<&'static str>,
    pub semantic_pairs: Vec<(usize, usize, f32)>, // (text1_idx, text2_idx, expected_similarity)
}

impl BenchmarkTestData {
    /// Get default test data for benchmarking
    pub fn new_default() -> Self {
        let texts = vec![
            "React hooks useState for state management",
            "useState React hook for managing component state",
            "Python Django models for database operations",
            "Django Python framework for web development",
            "JavaScript async await for asynchronous programming",
            "Node.js Express framework for web servers",
            "Rust memory safety without garbage collection",
            "C++ manual memory management with pointers",
            "Machine learning with neural networks",
            "Deep learning artificial intelligence models",
        ];

        // Expected high similarity pairs (same topic)
        let semantic_pairs = vec![
            (0, 1, 0.8), // React hooks related
            (2, 3, 0.7), // Django related
            (8, 9, 0.8), // ML/AI related
            (0, 2, 0.2), // Different topics - low similarity
            (4, 6, 0.1), // Very different topics
        ];

        Self {
            texts,
            semantic_pairs,
        }
    }

    /// Get extended test data for more comprehensive benchmarking
    pub fn extended() -> Self {
        let texts = vec![
            // Programming concepts (should cluster together)
            "React hooks useState for state management in functional components",
            "useState React hook manages local component state efficiently",
            "Vue.js reactive data binding with computed properties",
            "Angular component lifecycle hooks and state management",
            // Database concepts
            "PostgreSQL relational database with ACID transactions",
            "MongoDB document database with flexible schema design",
            "Redis in-memory data structure store for caching",
            "SQLite lightweight embedded database for applications",
            // Machine Learning
            "Deep neural networks for computer vision tasks",
            "Convolutional neural networks process image data effectively",
            "Natural language processing with transformer models",
            "BERT transformer model for text understanding tasks",
            // Web Development
            "RESTful API design principles and best practices",
            "GraphQL flexible query language for API development",
            "Microservices architecture pattern for scalable systems",
            "Docker containerization for application deployment",
            // Unrelated content
            "Cooking pasta requires boiling water and salt",
            "Weather forecast shows rain tomorrow afternoon",
            "Basketball game ended with a score of 95-87",
            "Garden flowers bloom beautifully in spring season",
        ];

        let semantic_pairs = vec![
            // High similarity pairs
            (0, 1, 0.85),   // React useState
            (9, 10, 0.80),  // CNN concepts
            (11, 12, 0.75), // NLP/BERT
            (4, 5, 0.65),   // Databases
            // Medium similarity pairs
            (0, 2, 0.45),   // React vs Vue (both frontend)
            (13, 14, 0.60), // API concepts
            // Low similarity pairs
            (0, 16, 0.05), // Programming vs cooking
            (4, 17, 0.05), // Database vs weather
            (9, 18, 0.05), // ML vs basketball
        ];

        Self {
            texts,
            semantic_pairs,
        }
    }
}

/// Benchmark a single embedding provider
pub async fn benchmark_provider<T: ProviderTrait + Send + Sync + ?Sized>(
    provider: &T,
    test_data: &BenchmarkTestData,
) -> Result<BenchmarkResults> {
    let provider_info = provider.get_info();
    let dimension = provider.get_dimension().await?;

    let mut embeddings = Vec::new();
    let start_time = Instant::now();

    // Generate embeddings for all test texts
    for text in &test_data.texts {
        let embedding = provider.embed_text(text).await?;
        embeddings.push(embedding);
    }

    let total_duration = start_time.elapsed();
    let avg_embedding_time = total_duration / test_data.texts.len() as u32;
    let embeddings_per_second = test_data.texts.len() as f64 / total_duration.as_secs_f64();

    // Calculate semantic quality score
    let semantic_quality_score = calculate_semantic_quality(&embeddings, test_data);

    Ok(BenchmarkResults {
        provider_name: provider_info.name,
        total_texts: test_data.texts.len(),
        total_duration,
        avg_embedding_time,
        embeddings_per_second,
        dimension,
        memory_usage_mb: Some(get_process_memory_mb()),
        semantic_quality_score: Some(semantic_quality_score),
    })
}

/// Calculate semantic quality score based on expected similarity pairs
fn calculate_semantic_quality(embeddings: &[Vec<f32>], test_data: &BenchmarkTestData) -> f64 {
    if test_data.semantic_pairs.is_empty() {
        return 0.0;
    }

    let mut total_error = 0.0;

    for (idx1, idx2, expected_sim) in &test_data.semantic_pairs {
        if *idx1 < embeddings.len() && *idx2 < embeddings.len() {
            let actual_sim = cosine_similarity(&embeddings[*idx1], &embeddings[*idx2]);
            let error = (actual_sim - expected_sim).abs();
            total_error += error;
        }
    }

    let avg_error = total_error / test_data.semantic_pairs.len() as f32;
    // Convert to quality score (1.0 = perfect, 0.0 = worst)
    (1.0 - avg_error.min(1.0)) as f64
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
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

/// Run comprehensive benchmark comparing multiple providers
pub async fn compare_providers(
    providers: Vec<(&str, Box<dyn ProviderTrait + Send + Sync>)>,
    test_data: &BenchmarkTestData,
) -> Result<Vec<BenchmarkResults>> {
    let mut results = Vec::new();

    for (name, provider) in providers {
        println!("Benchmarking provider: {}", name);
        match benchmark_provider(provider.as_ref(), test_data).await {
            Ok(result) => {
                println!("✅ {} completed", name);
                results.push(result);
            }
            Err(e) => {
                println!("❌ {} failed: {}", name, e);
            }
        }
    }

    Ok(results)
}

/// Print benchmark results in a readable format
pub fn print_benchmark_results(results: &[BenchmarkResults]) {
    println!("\n📊 Embedding Provider Benchmark Results");
    println!("{}", "=".repeat(80));

    for result in results {
        println!("\n🔧 Provider: {}", result.provider_name);
        println!("   Texts processed: {}", result.total_texts);
        println!("   Total time: {:.2}ms", result.total_duration.as_millis());
        println!(
            "   Avg per embedding: {:.2}ms",
            result.avg_embedding_time.as_millis()
        );
        println!(
            "   Throughput: {:.1} embeddings/sec",
            result.embeddings_per_second
        );
        println!("   Embedding dimension: {}", result.dimension);

        if let Some(quality) = result.semantic_quality_score {
            println!("   Semantic quality: {:.3} (0.0-1.0)", quality);
        }

        if let Some(memory) = result.memory_usage_mb {
            println!("   Memory usage: {:.1}MB", memory);
        }
    }

    println!("\n{}", "=".repeat(80));
}

/// Get current process memory usage in MB
fn get_process_memory_mb() -> f64 {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        // Read from /proc/self/status on Linux
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    // Extract memory in kB
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<f64>() {
                            return kb / 1024.0; // Convert kB to MB
                        }
                    }
                    break;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, we could use mach APIs but it's complex
        // For now, return 0.0 as a fallback
        // This could be enhanced with proper mach_task_info calls
        0.0
    }

    #[cfg(target_os = "windows")]
    {
        // Windows API not available, return 0.0
        0.0
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        // Fallback for unsupported platforms
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::providers::hash::HashProvider;

    #[tokio::test]
    async fn test_hash_provider_benchmark() {
        let provider = HashProvider::new(384);
        let test_data = BenchmarkTestData::new_default();

        let result = benchmark_provider(&provider, &test_data).await.unwrap();

        assert_eq!(result.total_texts, test_data.texts.len());
        assert_eq!(result.dimension, 384);
        assert!(result.embeddings_per_second > 0.0);
        assert!(result.semantic_quality_score.is_some());
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }
}
