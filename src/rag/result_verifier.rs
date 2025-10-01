//! Result verification system for RAG search quality assurance
//!
//! This module provides intelligent result verification using LLM when available,
//! with statistical fallback methods to ensure search results are relevant.

use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

use crate::rag::{
    llm::LlmClient,
    query_enhancer::{EnhancedQuery, QueryIntent},
    RagSearchResult, SmartSearchConfig,
};

/// Verified search result with confidence scoring
#[derive(Debug, Clone)]
pub struct VerifiedResult {
    pub result: RagSearchResult,
    pub confidence_score: f32,
    #[allow(dead_code)]
    pub relevance_explanation: Option<String>,
    #[allow(dead_code)]
    pub extracted_context: Option<String>,
    #[allow(dead_code)]
    pub verification_method: VerificationMethod,
}

/// Method used for result verification
#[derive(Debug, Clone)]
pub enum VerificationMethod {
    LlmBased,
    Statistical,
    #[allow(dead_code)]
    Keyword,
    #[allow(dead_code)]
    Hybrid,
}

/// Result verification system
pub struct ResultVerifier {
    llm_client: Option<Arc<LlmClient>>,
    config: SmartSearchConfig,
}

impl ResultVerifier {
    /// Create a new result verifier
    pub fn new(llm_client: Option<Arc<LlmClient>>, config: SmartSearchConfig) -> Self {
        Self { llm_client, config }
    }

    /// Verify and score search results based on query relevance
    pub async fn verify_results(
        &self,
        query: &EnhancedQuery,
        results: Vec<RagSearchResult>,
    ) -> Result<Vec<VerifiedResult>> {
        log::debug!(
            "Verifying {} results for query: '{}'",
            results.len(),
            query.original
        );

        let mut verified_results = Vec::new();

        for result in results {
            let verified = self.verify_single_result(query, result).await?;

            // Only include results that meet the confidence threshold
            if verified.confidence_score >= self.config.min_confidence_score {
                verified_results.push(verified);
            } else {
                log::debug!(
                    "Filtered out result with confidence {} < threshold {}",
                    verified.confidence_score,
                    self.config.min_confidence_score
                );
            }
        }

        // Sort by confidence score (highest first)
        verified_results
            .sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

        log::debug!(
            "Verification complete: {} results passed threshold",
            verified_results.len()
        );

        Ok(verified_results)
    }

    /// Verify a single search result
    async fn verify_single_result(
        &self,
        query: &EnhancedQuery,
        result: RagSearchResult,
    ) -> Result<VerifiedResult> {
        // Try LLM verification first if available
        if let Some(ref llm_client) = self.llm_client {
            if self.config.enable_result_verification {
                match self.verify_with_llm(query, &result, llm_client).await {
                    Ok(verified) => return Ok(verified),
                    Err(e) => {
                        log::warn!("LLM verification failed, using fallback: {}", e);
                    }
                }
            }
        }

        // Fallback to statistical verification
        Ok(self.verify_with_fallback(query, result))
    }

    /// Verify result using LLM
    async fn verify_with_llm(
        &self,
        query: &EnhancedQuery,
        result: &RagSearchResult,
        llm_client: &LlmClient,
    ) -> Result<VerifiedResult> {
        let system_prompt = self.build_verification_prompt(&query.detected_intent);

        let user_message = format!(
            "Query: \"{}\"\n\nContent to verify:\n{}\n\nPlease analyze if this content actually answers or relates to the query. Respond in JSON format with:\n- 'relevant': boolean\n- 'confidence': 0.0-1.0 score\n- 'explanation': brief reason\n- 'key_context': most relevant excerpt (max 200 chars)",
            query.original,
            self.truncate_content(&result.content, 1000)
        );

        // Simplified LLM call (placeholder)
        let response = self
            .call_llm_for_verification(llm_client, &system_prompt, &user_message)
            .await?;

        let parsed_response: serde_json::Value =
            serde_json::from_str(&response).unwrap_or_else(|_| {
                json!({
                    "relevant": true,
                    "confidence": 0.5,
                    "explanation": "Unable to parse LLM response",
                    "key_context": null
                })
            });

        let confidence = parsed_response["confidence"].as_f64().unwrap_or(0.5) as f32;

        let explanation = parsed_response["explanation"]
            .as_str()
            .map(|s| s.to_string());

        let key_context = parsed_response["key_context"]
            .as_str()
            .map(|s| s.to_string());

        Ok(VerifiedResult {
            result: result.clone(),
            confidence_score: confidence,
            relevance_explanation: explanation,
            extracted_context: key_context,
            verification_method: VerificationMethod::LlmBased,
        })
    }

    /// Build verification prompt based on query intent
    fn build_verification_prompt(&self, intent: &QueryIntent) -> String {
        match intent {
            QueryIntent::CodeSearch { language, component_type } => {
                format!(
                    "You are a code search verification expert. Analyze if content contains relevant {} {} code. Focus on function definitions, implementations, and usage patterns.",
                    component_type.as_deref().unwrap_or("programming"),
                    language.as_deref().unwrap_or("code")
                )
            },
            QueryIntent::Documentation => {
                "You are a documentation verification expert. Analyze if content provides explanatory information, guides, or instructional material relevant to the query.".to_string()
            },
            QueryIntent::Configuration => {
                "You are a configuration verification expert. Analyze if content contains settings, environment variables, or configuration patterns relevant to the query.".to_string()
            },
            QueryIntent::Debugging => {
                "You are a debugging verification expert. Analyze if content contains error solutions, troubleshooting steps, or problem resolution information.".to_string()
            },
            _ => {
                "You are a relevance verification expert. Analyze if the content is relevant to the search query.".to_string()
            }
        }
    }

    /// Placeholder for LLM verification call
    async fn call_llm_for_verification(
        &self,
        _llm_client: &LlmClient,
        _system_prompt: &str,
        _user_message: &str,
    ) -> Result<String> {
        // Placeholder implementation
        Ok(json!({
            "relevant": true,
            "confidence": 0.7,
            "explanation": "Content appears relevant to query",
            "key_context": null
        })
        .to_string())
    }

    /// Fallback verification using statistical methods
    fn verify_with_fallback(
        &self,
        query: &EnhancedQuery,
        result: RagSearchResult,
    ) -> VerifiedResult {
        let mut confidence_score = result.score; // Start with embedding similarity

        // Keyword matching boost
        let keyword_score = self.calculate_keyword_score(&query.original, &result.content);
        confidence_score = (confidence_score + keyword_score) / 2.0;

        // Intent-specific scoring adjustments
        confidence_score =
            self.apply_intent_adjustments(confidence_score, &query.detected_intent, &result);

        // Apply query variation matching
        let variation_score = self.calculate_variation_score(query, &result.content);
        confidence_score = (confidence_score * 0.7) + (variation_score * 0.3);

        // Extract key context using simple heuristics
        let extracted_context = self.extract_key_context(&query.original, &result.content);

        VerifiedResult {
            result,
            confidence_score: confidence_score.clamp(0.0, 1.0),
            relevance_explanation: Some("Statistical relevance analysis".to_string()),
            extracted_context,
            verification_method: VerificationMethod::Statistical,
        }
    }

    /// Calculate keyword-based relevance score
    fn calculate_keyword_score(&self, query: &str, content: &str) -> f32 {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip very short words
            .collect();

        if query_words.is_empty() {
            return 0.0;
        }

        let content_lower = content.to_lowercase();
        let matches = query_words
            .iter()
            .filter(|&&word| content_lower.contains(word))
            .count();

        matches as f32 / query_words.len() as f32
    }

    /// Apply intent-specific scoring adjustments
    fn apply_intent_adjustments(
        &self,
        base_score: f32,
        intent: &QueryIntent,
        result: &RagSearchResult,
    ) -> f32 {
        let mut adjusted_score = base_score;

        match intent {
            QueryIntent::CodeSearch {
                language,
                component_type,
            } => {
                // Boost if content appears to be code
                if self.looks_like_code(&result.content) {
                    adjusted_score *= 1.2;
                }

                // Boost if language matches
                if let Some(lang) = language {
                    if result.content.to_lowercase().contains(&lang.to_lowercase()) {
                        adjusted_score *= 1.1;
                    }
                }

                // Boost if component type matches
                if let Some(comp_type) = component_type {
                    if result
                        .content
                        .to_lowercase()
                        .contains(&comp_type.to_lowercase())
                    {
                        adjusted_score *= 1.15;
                    }
                }
            }
            QueryIntent::Documentation => {
                // Boost documentation-like files
                if result.source_path.to_string_lossy().contains("doc")
                    || result.source_path.to_string_lossy().contains("readme")
                    || result.source_path.to_string_lossy().ends_with(".md")
                {
                    adjusted_score *= 1.1;
                }
            }
            QueryIntent::Configuration => {
                // Boost config-like files
                if result.source_path.to_string_lossy().contains("config")
                    || result.source_path.to_string_lossy().ends_with(".json")
                    || result.source_path.to_string_lossy().ends_with(".yaml")
                    || result.source_path.to_string_lossy().ends_with(".toml")
                {
                    adjusted_score *= 1.2;
                }
            }
            _ => {}
        }

        adjusted_score
    }

    /// Check if content looks like code
    fn looks_like_code(&self, content: &str) -> bool {
        let code_indicators = [
            "function", "class", "struct", "impl", "def", "fn", "public", "private", "const",
            "let", "var", "import", "use", "include", "package", "{", "}", "(", ")", ";", "=>",
            "->",
        ];

        let indicator_count = code_indicators
            .iter()
            .filter(|&&indicator| content.contains(indicator))
            .count();

        indicator_count >= 3
    }

    /// Calculate score based on query variations
    fn calculate_variation_score(&self, query: &EnhancedQuery, content: &str) -> f32 {
        let mut best_score: f32 = 0.0;

        for variation in &query.variations {
            let score = self.calculate_keyword_score(&variation.query, content) * variation.weight;
            best_score = best_score.max(score);
        }

        best_score
    }

    /// Extract most relevant context from content
    fn extract_key_context(&self, query: &str, content: &str) -> Option<String> {
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        // Find sentences containing query terms
        let sentences: Vec<&str> = content
            .split(['.', '\n', ';'])
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut best_sentence = "";
        let mut best_score = 0;

        for sentence in sentences {
            let sentence_lower = sentence.to_lowercase();
            let matches = query_words
                .iter()
                .filter(|&&word| sentence_lower.contains(word))
                .count();

            if matches > best_score {
                best_score = matches;
                best_sentence = sentence;
            }
        }

        if best_score > 0 {
            Some(self.truncate_content(best_sentence.trim(), 200))
        } else {
            None
        }
    }

    /// Truncate content to specified length (Unicode-safe)
    fn truncate_content(&self, content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            content.to_string()
        } else {
            // Use char_indices to find a valid Unicode boundary
            let mut truncate_at = max_length;
            while truncate_at > 0 && !content.is_char_boundary(truncate_at) {
                truncate_at -= 1;
            }
            format!("{}...", &content[..truncate_at])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{DocumentMetadata, SourceType};
    use std::path::PathBuf;

    #[allow(dead_code)]
    fn create_test_result(content: &str, score: f32) -> RagSearchResult {
        RagSearchResult {
            id: "test".to_string(),
            content: content.to_string(),
            source_path: PathBuf::from("test.rs"),
            source_type: SourceType::Local,
            title: None,
            section: None,
            score,
            chunk_index: 0,
            metadata: DocumentMetadata {
                file_type: "rust".to_string(),
                size: 100,
                modified: chrono::Utc::now(),
                tags: vec![],
                language: Some("rust".to_string()),
            },
        }
    }

    #[tokio::test]
    async fn test_keyword_scoring() {
        let verifier = ResultVerifier::new(None, SmartSearchConfig::default());

        let score = verifier.calculate_keyword_score(
            "validate security function",
            "fn validate_security() { /* security validation */ }",
        );

        assert!(score > 0.5);
    }

    #[tokio::test]
    async fn test_code_detection() {
        let verifier = ResultVerifier::new(None, SmartSearchConfig::default());

        assert!(verifier.looks_like_code("fn main() { println!(\"hello\"); }"));
        assert!(!verifier.looks_like_code("This is just regular text content."));
    }
}
