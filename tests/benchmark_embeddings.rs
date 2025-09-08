//! Integration tests for embedding provider benchmarks

use manx_cli::rag::benchmarks::{benchmark_provider, print_benchmark_results, BenchmarkTestData};
use manx_cli::rag::providers::hash::HashProvider;

#[tokio::test]
async fn benchmark_hash_provider() {
    let provider = HashProvider::new(384);
    let test_data = BenchmarkTestData::new_default();

    let result = benchmark_provider(&provider, &test_data).await.unwrap();

    println!("Hash Provider Benchmark Results:");
    print_benchmark_results(std::slice::from_ref(&result));

    // Assertions for baseline performance
    assert_eq!(result.dimension, 384);
    assert!(
        result.embeddings_per_second > 100.0,
        "Hash provider should be very fast"
    );
    assert!(
        result.avg_embedding_time.as_millis() < 10,
        "Should embed in under 10ms"
    );

    // Quality score will be low for hash-based (expected)
    if let Some(quality) = result.semantic_quality_score {
        println!("Semantic quality score: {:.3}", quality);
        assert!(
            (0.0..=1.0).contains(&quality),
            "Quality score should be 0-1"
        );
    }
}

#[tokio::test]
async fn benchmark_hash_provider_extended() {
    let provider = HashProvider::new(384);
    let test_data = BenchmarkTestData::extended();

    let result = benchmark_provider(&provider, &test_data).await.unwrap();

    println!("Hash Provider Extended Benchmark:");
    print_benchmark_results(&[result]);
}

#[tokio::test]
async fn compare_hash_dimensions() {
    let dimensions = vec![128, 256, 384, 512, 768];
    let test_data = BenchmarkTestData::new_default();
    let mut results = Vec::new();

    for dim in dimensions {
        let provider = HashProvider::new(dim);
        let result = benchmark_provider(&provider, &test_data).await.unwrap();
        results.push(result);
    }

    println!("Hash Provider Dimension Comparison:");
    print_benchmark_results(&results);

    // Higher dimensions should not significantly impact speed for hash provider
    for result in &results {
        assert!(
            result.embeddings_per_second > 50.0,
            "Should maintain good throughput"
        );
    }
}
