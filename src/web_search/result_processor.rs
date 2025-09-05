//! Result processing with BERT embeddings and official source ranking
//!
//! This module processes raw search results by:
//! - Using BERT embeddings for semantic similarity scoring
//! - Applying official source priority boosts
//! - Ranking results by combined relevance + authority scores

use crate::rag::embeddings::EmbeddingModel;
use crate::web_search::official_sources::{OfficialSourceManager, SourceTier};
use crate::web_search::{ProcessedSearchResult, RawSearchResult};
use anyhow::Result;

/// Process search results with BERT semantic similarity
pub async fn process_with_bert(
    query: &str,
    raw_results: &[RawSearchResult],
    embedding_model: &EmbeddingModel,
    official_sources: &OfficialSourceManager,
    similarity_threshold: f32,
) -> Result<Vec<ProcessedSearchResult>> {
    log::info!(
        "Processing {} results with BERT embeddings",
        raw_results.len()
    );

    // Generate query embedding
    let query_embedding = embedding_model.embed_text(query).await?;

    let mut processed_results = Vec::new();

    for (index, result) in raw_results.iter().enumerate() {
        // Create combined text for embedding (title + snippet)
        let combined_text = format!("{} {}", result.title, result.snippet);

        // Generate result embedding
        let result_embedding = match embedding_model.embed_text(&combined_text).await {
            Ok(embedding) => embedding,
            Err(e) => {
                log::warn!("Failed to embed result {}: {}", index, e);
                continue;
            }
        };

        // Calculate semantic similarity
        let similarity_score =
            EmbeddingModel::cosine_similarity(&query_embedding, &result_embedding);

        // Skip results below similarity threshold
        if similarity_score < similarity_threshold {
            log::debug!(
                "Filtering out result with low similarity: {} (score: {:.3})",
                result.title,
                similarity_score
            );
            continue;
        }

        // Determine source tier and official status
        let source_tier = official_sources.get_source_tier(&result.source_domain, &result.url);
        let is_official = matches!(
            source_tier,
            SourceTier::OfficialDocs | SourceTier::OfficialRepos
        );

        // Calculate official source boost
        let source_boost = official_sources.get_score_boost(&source_tier);

        // Calculate final score (similarity * source_boost)
        let final_score = similarity_score * source_boost;

        processed_results.push(ProcessedSearchResult {
            title: result.title.clone(),
            url: result.url.clone(),
            snippet: result.snippet.clone(),
            source_domain: result.source_domain.clone(),
            is_official,
            source_tier: source_tier as u8,
            similarity_score,
            final_score,
            timestamp: result.timestamp,
        });

        log::debug!(
            "Processed result: {} | Similarity: {:.3} | Boost: {:.1}x | Final: {:.3}",
            result.source_domain,
            similarity_score,
            source_boost,
            final_score
        );
    }

    // Sort by final score (descending)
    processed_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

    log::info!(
        "Processed {} relevant results (filtered {} below threshold)",
        processed_results.len(),
        raw_results.len() - processed_results.len()
    );

    Ok(processed_results)
}

/// Process search results without BERT (fallback method)
pub fn process_without_bert(
    query: &str,
    raw_results: &[RawSearchResult],
    official_sources: &OfficialSourceManager,
) -> Vec<ProcessedSearchResult> {
    log::info!(
        "Processing {} results with text matching (no BERT)",
        raw_results.len()
    );

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let mut processed_results = Vec::new();

    for result in raw_results {
        // Simple text-based similarity scoring
        let combined_text = format!("{} {}", result.title, result.snippet).to_lowercase();

        // Count query word matches
        let word_matches = query_words
            .iter()
            .filter(|word| combined_text.contains(*word))
            .count();

        // Calculate basic similarity score (percentage of query words found)
        let similarity_score = if query_words.is_empty() {
            0.5 // Default score if no query words
        } else {
            word_matches as f32 / query_words.len() as f32
        };

        // Apply minimum threshold
        if similarity_score < 0.3 {
            continue;
        }

        // Determine source tier and official status
        let source_tier = official_sources.get_source_tier(&result.source_domain, &result.url);
        let is_official = matches!(
            source_tier,
            SourceTier::OfficialDocs | SourceTier::OfficialRepos
        );

        // Calculate official source boost
        let source_boost = official_sources.get_score_boost(&source_tier);

        // Calculate final score
        let final_score = similarity_score * source_boost;

        processed_results.push(ProcessedSearchResult {
            title: result.title.clone(),
            url: result.url.clone(),
            snippet: result.snippet.clone(),
            source_domain: result.source_domain.clone(),
            is_official,
            source_tier: source_tier as u8,
            similarity_score,
            final_score,
            timestamp: result.timestamp,
        });
    }

    // Sort by final score (descending)
    processed_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

    log::info!(
        "Processed {} results with text matching",
        processed_results.len()
    );
    processed_results
}

/// Enhance results with additional metadata
pub fn enhance_results(
    processed_results: &mut [ProcessedSearchResult],
    _official_sources: &OfficialSourceManager,
) {
    for result in processed_results.iter_mut() {
        // Add content type detection
        if result.url.contains("/docs/") || result.url.contains("/documentation/") {
            // This is likely documentation
        } else if result.url.contains("/api/") {
            // This is likely API documentation
        } else if result.url.contains("/tutorial") || result.url.contains("/guide") {
            // This is likely a tutorial or guide
        }

        // Boost results that mention exact query terms in title
        // This could be implemented for better ranking
    }
}

/// Filter results to remove low-quality or duplicate content
pub fn filter_quality_results(
    processed_results: Vec<ProcessedSearchResult>,
    min_snippet_length: usize,
) -> Vec<ProcessedSearchResult> {
    processed_results
        .into_iter()
        .filter(|result| {
            // Filter out results with very short snippets
            if result.snippet.len() < min_snippet_length {
                log::debug!("Filtering short snippet: {}", result.title);
                return false;
            }

            // Filter out obvious spam or low-quality indicators
            let snippet_lower = result.snippet.to_lowercase();
            if snippet_lower.contains("lorem ipsum")
                || snippet_lower.contains("click here for more")
                || snippet_lower.contains("subscribe now")
            {
                log::debug!("Filtering low-quality content: {}", result.title);
                return false;
            }

            // Filter out results that are just lists of links
            if result.snippet.matches("http").count() > 3 {
                log::debug!("Filtering link-heavy content: {}", result.title);
                return false;
            }

            true
        })
        .collect()
}

/// Deduplicate results based on content similarity
pub fn deduplicate_results(
    mut processed_results: Vec<ProcessedSearchResult>,
) -> Vec<ProcessedSearchResult> {
    // Simple deduplication based on URL domain + title similarity
    processed_results.sort_by(|a, b| {
        let domain_cmp = a.source_domain.cmp(&b.source_domain);
        if domain_cmp == std::cmp::Ordering::Equal {
            a.title.cmp(&b.title)
        } else {
            domain_cmp
        }
    });

    let mut unique_results = Vec::new();
    let mut last_domain = String::new();
    let mut last_title_words = Vec::new();

    let result_count = processed_results.len();
    for result in &processed_results {
        let current_title_words: Vec<&str> = result.title.split_whitespace().take(5).collect();

        // Check if this is a duplicate based on domain + title similarity
        let is_duplicate = result.source_domain == last_domain
            && title_similarity(&current_title_words, &last_title_words) > 0.8;

        if !is_duplicate {
            unique_results.push(result.clone());
        } else {
            log::debug!(
                "Removing duplicate: {} from {}",
                result.title,
                result.source_domain
            );
        }

        last_domain = result.source_domain.clone();
        last_title_words = current_title_words
            .into_iter()
            .map(|s| s.to_string())
            .collect();
    }

    // Re-sort by final score
    unique_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

    log::info!(
        "Deduplicated results: {} -> {}",
        result_count,
        unique_results.len()
    );

    unique_results
}

/// Calculate title similarity for deduplication
fn title_similarity(words1: &[&str], words2: &[String]) -> f32 {
    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }

    let matches = words1
        .iter()
        .filter(|word1| {
            words2
                .iter()
                .any(|word2| word1.to_lowercase() == word2.to_lowercase())
        })
        .count();

    matches as f32 / words1.len().max(words2.len()) as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_process_without_bert() {
        let official_sources = OfficialSourceManager::new();

        let raw_results = vec![
            RawSearchResult {
                title: "Python Documentation".to_string(),
                url: "https://docs.python.org/3/".to_string(),
                snippet: "Python programming language documentation".to_string(),
                source_domain: "docs.python.org".to_string(),
                timestamp: Some(Utc::now()),
            },
            RawSearchResult {
                title: "Random Blog".to_string(),
                url: "https://random-blog.com/python".to_string(),
                snippet: "Some random python content".to_string(),
                source_domain: "random-blog.com".to_string(),
                timestamp: Some(Utc::now()),
            },
        ];

        let results = process_without_bert("python", &raw_results, &official_sources);

        assert_eq!(results.len(), 2);
        assert!(results[0].is_official); // Official source should rank higher
        assert!(results[0].final_score > results[1].final_score);
    }

    #[test]
    fn test_filter_quality_results() {
        let results = vec![
            ProcessedSearchResult {
                title: "Good Result".to_string(),
                url: "https://example.com".to_string(),
                snippet: "This is a good quality result with sufficient content".to_string(),
                source_domain: "example.com".to_string(),
                is_official: false,
                source_tier: 4,
                similarity_score: 0.8,
                final_score: 0.8,
                timestamp: Some(Utc::now()),
            },
            ProcessedSearchResult {
                title: "Short Result".to_string(),
                url: "https://short.com".to_string(),
                snippet: "Too short".to_string(),
                source_domain: "short.com".to_string(),
                is_official: false,
                source_tier: 4,
                similarity_score: 0.5,
                final_score: 0.5,
                timestamp: Some(Utc::now()),
            },
        ];

        let filtered = filter_quality_results(results, 20);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Good Result");
    }

    #[test]
    fn test_title_similarity() {
        let words1 = vec!["Python", "Documentation", "Guide"];
        let words2 = vec![
            "Python".to_string(),
            "Docs".to_string(),
            "Tutorial".to_string(),
        ];

        let similarity = title_similarity(&words1, &words2);
        assert!(similarity > 0.0 && similarity <= 1.0);
    }
}
