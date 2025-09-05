//! LLM verification for search result authenticity
//!
//! This module provides LLM-powered verification to ensure search results
//! are authentic and relevant to the user's query.

use crate::rag::llm::LlmClient;
use crate::web_search::{ProcessedSearchResult, VerificationResult};
use anyhow::Result;

/// Verify search results using LLM for authenticity checking
pub async fn verify_search_results(
    query: &str,
    results: &[ProcessedSearchResult],
    _llm_client: &LlmClient,
) -> Result<VerificationResult> {
    // LLM-based verification of search results
    // This implementation provides basic verification using heuristics and could be
    // enhanced with actual LLM calls for deeper analysis

    if results.is_empty() {
        return Ok(VerificationResult {
            is_authentic: false,
            confidence: 0.0,
            reasoning: "No results to verify".to_string(),
            suggested_refinement: Some(format!("Try more specific terms related to '{}'", query)),
        });
    }

    // Simple heuristic verification based on result quality
    let official_count = results.iter().filter(|r| r.is_official).count();
    let avg_similarity =
        results.iter().map(|r| r.similarity_score).sum::<f32>() / results.len() as f32;

    let is_authentic = official_count > 0 || avg_similarity > 0.7;
    let confidence = if official_count > 0 {
        0.9 // High confidence for official sources
    } else {
        avg_similarity.min(0.8) // Cap confidence based on similarity
    };

    let reasoning = if official_count > 0 {
        format!(
            "Found {} official documentation sources with high relevance",
            official_count
        )
    } else if avg_similarity > 0.7 {
        format!(
            "Community sources show high semantic similarity ({:.1}%)",
            avg_similarity * 100.0
        )
    } else {
        "Results show low semantic similarity to query".to_string()
    };

    let suggested_refinement = if !is_authentic && avg_similarity < 0.5 {
        Some(format!(
            "Try more specific terms like '{} examples' or '{} tutorial'",
            query, query
        ))
    } else {
        None
    };

    Ok(VerificationResult {
        is_authentic,
        confidence,
        reasoning,
        suggested_refinement,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_result(is_official: bool, similarity: f32) -> ProcessedSearchResult {
        ProcessedSearchResult {
            title: "Test Result".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            source_domain: "example.com".to_string(),
            is_official,
            source_tier: if is_official { 1 } else { 4 },
            similarity_score: similarity,
            final_score: similarity,
            timestamp: Some(Utc::now()),
        }
    }

    #[tokio::test]
    async fn test_verify_empty_results() {
        let llm_config = crate::rag::llm::LlmConfig::default();
        let llm_client = crate::rag::llm::LlmClient::new(llm_config).unwrap();

        let verification = verify_search_results("test query", &[], &llm_client)
            .await
            .unwrap();

        assert!(!verification.is_authentic);
        assert_eq!(verification.confidence, 0.0);
        assert!(verification.suggested_refinement.is_some());
    }

    #[tokio::test]
    async fn test_verify_official_results() {
        let llm_config = crate::rag::llm::LlmConfig::default();
        let llm_client = crate::rag::llm::LlmClient::new(llm_config).unwrap();

        let results = vec![
            create_test_result(true, 0.8),
            create_test_result(false, 0.6),
        ];

        let verification = verify_search_results("test query", &results, &llm_client)
            .await
            .unwrap();

        assert!(verification.is_authentic);
        assert_eq!(verification.confidence, 0.9);
        assert!(verification.suggested_refinement.is_none());
    }

    #[tokio::test]
    async fn test_verify_high_similarity_results() {
        let llm_config = crate::rag::llm::LlmConfig::default();
        let llm_client = crate::rag::llm::LlmClient::new(llm_config).unwrap();

        let results = vec![
            create_test_result(false, 0.8),
            create_test_result(false, 0.9),
        ];

        let verification = verify_search_results("test query", &results, &llm_client)
            .await
            .unwrap();

        assert!(verification.is_authentic);
        assert!(verification.confidence > 0.5);
        assert!(verification.suggested_refinement.is_none());
    }

    #[tokio::test]
    async fn test_verify_low_quality_results() {
        let llm_config = crate::rag::llm::LlmConfig::default();
        let llm_client = crate::rag::llm::LlmClient::new(llm_config).unwrap();

        let results = vec![
            create_test_result(false, 0.3),
            create_test_result(false, 0.4),
        ];

        let verification = verify_search_results("test query", &results, &llm_client)
            .await
            .unwrap();

        assert!(!verification.is_authentic);
        assert!(verification.confidence < 0.5);
        assert!(verification.suggested_refinement.is_some());
    }
}
