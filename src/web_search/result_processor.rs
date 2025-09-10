//! Result processing with semantic embeddings and official source ranking
//!
//! This module processes raw search results by:
//! - Using semantic embeddings for similarity scoring
//! - Applying official source priority boosts
//! - Ranking results by combined relevance + authority scores

use crate::rag::embeddings::EmbeddingModel;
use crate::web_search::official_sources::{OfficialSourceManager, SourceTier};
use crate::web_search::{ProcessedSearchResult, RawSearchResult, query_analyzer};
use anyhow::Result;


/// Process search results with semantic similarity and query analysis context
pub async fn process_with_embeddings_and_analysis(
    query_analysis: &query_analyzer::QueryAnalysis,
    raw_results: &[RawSearchResult],
    embedding_model: &EmbeddingModel,
    official_sources: &OfficialSourceManager,
    similarity_threshold: f32,
) -> Result<Vec<ProcessedSearchResult>> {
    log::info!(
        "Processing {} results with semantic embeddings + query analysis (framework: {:?})",
        raw_results.len(),
        query_analysis.detected_frameworks.first().map(|f| &f.name)
    );

    // Use enhanced query for embedding generation (better context)
    let embedding_query = &query_analysis.enhanced_query;
    let query_embedding = embedding_model.embed_text(embedding_query).await?;

    let mut processed_results = Vec::new();

    for (index, result) in raw_results.iter().enumerate() {
        // Create enhanced text for embedding with domain context
        let mut combined_text = format!("{} {}", result.title, result.snippet);
        
        // Add framework context to help embeddings understand domain
        for framework in &query_analysis.detected_frameworks {
            combined_text.push_str(&format!(" {}", framework.name));
        }
        
        // Add domain context keywords
        for keyword in &query_analysis.domain_context.context_keywords {
            combined_text.push_str(&format!(" {}", keyword));
        }

        // Generate result embedding with enhanced context
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

        // Apply framework-specific similarity threshold adjustment
        let adjusted_threshold = if !query_analysis.detected_frameworks.is_empty() {
            // Lower threshold for framework-specific queries (more lenient)
            similarity_threshold * 0.8
        } else {
            similarity_threshold
        };

        // Skip results below adjusted threshold
        if similarity_score < adjusted_threshold {
            log::debug!(
                "Filtering out result with low similarity: {} (score: {:.3}, threshold: {:.3})",
                result.title,
                similarity_score,
                adjusted_threshold
            );
            continue;
        }

        // Determine source tier and official status
        let source_tier = official_sources.get_source_tier(&result.source_domain, &result.url);
        let is_official = matches!(
            source_tier,
            SourceTier::OfficialDocs | SourceTier::OfficialRepos
        );

        // Apply framework-specific boost
        let mut source_boost = official_sources.get_score_boost(&source_tier);
        
        // Extra boost if result matches detected framework domains
        for framework in &query_analysis.detected_frameworks {
            if framework.official_sites.iter().any(|site| result.source_domain.contains(site)) {
                source_boost *= 1.5; // Extra framework match boost
                log::debug!("Applied framework domain boost for {}", framework.name);
            }
        }

        // Apply query type specific adjustments
        let type_boost = match query_analysis.query_type {
            query_analyzer::QueryType::Reference => 1.2, // Boost official documentation
            query_analyzer::QueryType::Example => 1.1,   // Slightly boost examples
            query_analyzer::QueryType::Troubleshoot => 0.9, // Allow more diverse sources
            _ => 1.0,
        };

        // Calculate final score with all boosts
        let final_score = similarity_score * source_boost * type_boost;

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
            "Enhanced result: {} | Similarity: {:.3} | Source boost: {:.1}x | Type boost: {:.1}x | Final: {:.3}",
            result.source_domain,
            similarity_score,
            source_boost,
            type_boost,
            final_score
        );
    }

    // Sort by final score (descending)
    processed_results.sort_by(|a, b| b.final_score.partial_cmp(&a.final_score).unwrap());

    log::info!(
        "Enhanced processing: {} relevant results (filtered {} below threshold)",
        processed_results.len(),
        raw_results.len() - processed_results.len()
    );

    Ok(processed_results)
}

/// Filter out non-technical domains when processing technical queries (LLM-enhanced only)
pub fn filter_non_technical_domains(
    results: Vec<ProcessedSearchResult>,
    query_analysis: &query_analyzer::QueryAnalysis,
    has_llm: bool,
) -> Vec<ProcessedSearchResult> {
    if !has_llm || query_analysis.detected_frameworks.is_empty() {
        // No filtering without LLM or if not a framework-specific query
        return results;
    }

    // Non-technical domains to filter out for technical queries
    let non_technical_domains = vec![
        "amazon.com",
        "ebay.com", 
        "etsy.com",
        "walmart.com",
        "target.com",
        "houzz.com",
        "wayfair.com",
        "overstock.com",
        "perigold.com",
        "safavieh.com",
        "furniture.com",
        "shopping.com",
        "bestbuy.com",
        "lowes.com",
        "homedepot.com",
    ];

    let original_count = results.len();
    let filtered_results: Vec<ProcessedSearchResult> = results
        .into_iter()
        .filter(|result| {
            let domain_lower = result.source_domain.to_lowercase();
            
            // Check if it's a non-technical domain
            let is_non_technical = non_technical_domains
                .iter()
                .any(|nt_domain| domain_lower.contains(nt_domain));

            if is_non_technical {
                log::debug!(
                    "LLM filter: Removing non-technical result: {} from {}",
                    result.title,
                    result.source_domain
                );
                false
            } else {
                true
            }
        })
        .collect();

    let filtered_count = original_count - filtered_results.len();
    if filtered_count > 0 {
        log::info!(
            "🧠 LLM-enhanced filtering: Removed {} non-technical results (e.g., shopping, furniture)",
            filtered_count
        );
    }

    filtered_results
}

/// Process search results without embeddings (fallback method)
pub fn process_without_embeddings(
    query: &str,
    raw_results: &[RawSearchResult],
    official_sources: &OfficialSourceManager,
) -> Vec<ProcessedSearchResult> {
    log::info!(
        "Processing {} results with text matching (no embeddings)",
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
    fn test_process_without_embeddings() {
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

        let results = process_without_embeddings("python", &raw_results, &official_sources);

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
