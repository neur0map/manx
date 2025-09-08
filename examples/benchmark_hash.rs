//! Simple benchmark runner for hash-based embeddings

use manx_cli::rag::benchmarks::{benchmark_provider, print_benchmark_results, BenchmarkTestData};
use manx_cli::rag::providers::hash::HashProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Manx Embedding Provider Benchmarks");
    println!("=====================================\n");

    // Test different hash dimensions
    let dimensions = vec![128, 256, 384, 512, 768];
    let test_data = BenchmarkTestData::new_default();
    let extended_data = BenchmarkTestData::extended();

    println!("📊 Testing Hash Provider with Different Dimensions");
    println!(
        "Test Data: {} texts with {} similarity pairs\n",
        test_data.texts.len(),
        test_data.semantic_pairs.len()
    );

    let mut results = Vec::new();

    for dim in &dimensions {
        let provider = HashProvider::new(*dim);
        let result = benchmark_provider(&provider, &test_data).await?;
        results.push(result);
    }

    print_benchmark_results(&results);

    println!("\n📈 Extended Dataset Benchmark (Hash-384)");
    println!(
        "Extended Data: {} texts with {} similarity pairs\n",
        extended_data.texts.len(),
        extended_data.semantic_pairs.len()
    );

    let provider_384 = HashProvider::new(384);
    let extended_result = benchmark_provider(&provider_384, &extended_data).await?;
    print_benchmark_results(&[extended_result]);

    println!("\n✅ Benchmark Complete!");
    println!("💡 Next: Compare with ONNX-based embeddings for quality improvements");

    Ok(())
}
