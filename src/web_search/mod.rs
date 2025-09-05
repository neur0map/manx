//! Intelligent documentation search with DuckDuckGo + BERT embeddings
//!
//! This module provides official-first documentation search that:
//! - Prioritizes official documentation sites by default
//! - Falls back to trusted community sources with clear notifications
//! - Uses BERT embeddings for semantic relevance filtering
//! - Optionally uses LLM for authenticity verification and summarization
//! - Maintains privacy with anonymous DuckDuckGo searches

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod llm_verifier;
pub mod official_sources;
pub mod result_processor;
pub mod search_engine;

/// Configuration for documentation search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchConfig {
    pub enabled: bool,
    pub max_results: usize,
    pub similarity_threshold: f32,
    pub search_timeout_seconds: u64,
    pub user_agent: String,
    pub min_official_results: usize, // Minimum official results before fallback
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 8,
            similarity_threshold: 0.6,
            search_timeout_seconds: 10,
            user_agent: "Manx/0.3.5 Documentation Finder (+https://github.com/neur0map/manx)"
                .to_string(),
            min_official_results: 3,
        }
    }
}

/// Raw search result from DuckDuckGo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_domain: String,
    pub timestamp: Option<DateTime<Utc>>,
}

/// Processed search result with relevance scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_domain: String,
    pub is_official: bool,
    pub source_tier: u8, // 1=Official docs, 2=Official repos, 3=Trusted community, 4=General
    pub similarity_score: f32,
    pub final_score: f32, // Combined similarity + official boost
    pub timestamp: Option<DateTime<Utc>>,
}

/// Final documentation search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSearchResponse {
    pub query: String,
    pub summary: String,
    pub results: Vec<ProcessedSearchResult>,
    pub official_results_count: usize,
    pub used_fallback: bool,
    pub total_found: usize,
    pub search_time_ms: u64,
    pub sources: Vec<String>,
    pub used_llm_verification: bool,
    pub verification_passed: Option<bool>,
}

/// LLM verification response for search authenticity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub is_authentic: bool,
    pub confidence: f32,
    pub reasoning: String,
    pub suggested_refinement: Option<String>, // If search should be refined
}

/// Documentation search system
pub struct DocumentationSearchSystem {
    config: WebSearchConfig,
    embedding_model: Option<crate::rag::embeddings::EmbeddingModel>,
    llm_client: Option<crate::rag::llm::LlmClient>,
    official_sources: official_sources::OfficialSourceManager,
}

impl DocumentationSearchSystem {
    /// Create new documentation search system
    pub async fn new(
        config: WebSearchConfig,
        llm_config: Option<crate::rag::llm::LlmConfig>,
    ) -> Result<Self> {
        if !config.enabled {
            return Err(anyhow!("Documentation search is disabled"));
        }

        // Initialize BERT embeddings for semantic similarity
        let embedding_model = match crate::rag::embeddings::EmbeddingModel::new().await {
            Ok(model) => {
                log::info!("BERT embeddings initialized for semantic search");
                Some(model)
            }
            Err(e) => {
                log::warn!("BERT embeddings unavailable, using text matching: {}", e);
                None
            }
        };

        // Initialize LLM client if configured
        let llm_client = if let Some(llm_cfg) = llm_config {
            match crate::rag::llm::LlmClient::new(llm_cfg) {
                Ok(client) => {
                    log::info!("LLM client initialized for result verification");
                    Some(client)
                }
                Err(e) => {
                    log::warn!("LLM client unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let official_sources = official_sources::OfficialSourceManager::new();

        Ok(Self {
            config,
            embedding_model,
            llm_client,
            official_sources,
        })
    }

    /// Search for documentation with official-first strategy
    pub async fn search(&mut self, query: &str) -> Result<DocumentationSearchResponse> {
        let start_time = std::time::Instant::now();

        log::info!("🔍 Searching official documentation for: {}", query);

        // Step 1: Search official documentation sites first
        let official_query = self.official_sources.build_official_query(query);
        let mut all_results = search_engine::search_duckduckgo(
            &official_query,
            self.config.max_results,
            &self.config.user_agent,
            self.config.search_timeout_seconds,
        )
        .await?;

        let mut used_fallback = false;

        // Step 2: Check if we have enough official results
        let official_results_count = all_results
            .iter()
            .filter(|r| self.official_sources.is_official_domain(&r.source_domain))
            .count();

        // Step 3: Fallback to general search if insufficient official results
        if official_results_count < self.config.min_official_results {
            log::info!(
                "⚠️ Only {} official results found, expanding search...",
                official_results_count
            );
            used_fallback = true;

            // Search without site restrictions
            let fallback_results = search_engine::search_duckduckgo(
                query,
                self.config.max_results,
                &self.config.user_agent,
                self.config.search_timeout_seconds,
            )
            .await?;

            // Merge results, avoiding duplicates
            for result in fallback_results {
                if !all_results.iter().any(|r| r.url == result.url) {
                    all_results.push(result);
                }
            }
        }

        if all_results.is_empty() {
            return Ok(DocumentationSearchResponse {
                query: query.to_string(),
                summary: "No relevant documentation found".to_string(),
                results: vec![],
                official_results_count: 0,
                used_fallback: false,
                total_found: 0,
                search_time_ms: start_time.elapsed().as_millis() as u64,
                sources: vec![],
                used_llm_verification: false,
                verification_passed: None,
            });
        }

        // Step 4: Process results with BERT semantic filtering and official source ranking
        let mut processed_results = if let Some(ref embedding_model) = self.embedding_model {
            result_processor::process_with_bert(
                query,
                &all_results,
                embedding_model,
                &self.official_sources,
                self.config.similarity_threshold,
            )
            .await?
        } else {
            result_processor::process_without_bert(query, &all_results, &self.official_sources)
        };

        // Step 4a: Enhance results with additional metadata
        result_processor::enhance_results(&mut processed_results, &self.official_sources);

        // Log tier information for debugging
        for result in &processed_results {
            let tier = self
                .official_sources
                .get_source_tier(&result.source_domain, &result.url);
            log::debug!(
                "Source: {} - Tier: {} - Score: {}",
                result.source_domain,
                self.official_sources.get_tier_description(&tier),
                result.final_score
            );
        }

        // Step 4b: Filter out low-quality results
        processed_results = result_processor::filter_quality_results(processed_results, 30);

        // Step 4c: Remove duplicates
        let processed_results = result_processor::deduplicate_results(processed_results);

        // Step 5: LLM verification if available
        let verification_result = if let Some(ref llm_client) = self.llm_client {
            if llm_client.is_available() {
                log::info!("Verifying results with LLM");
                match llm_verifier::verify_search_results(query, &processed_results, llm_client)
                    .await
                {
                    Ok(verification) => Some(verification),
                    Err(e) => {
                        log::warn!("LLM verification failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // Step 6: Generate summary
        let summary = self.generate_summary(query, &processed_results).await?;

        // Calculate final stats
        let final_official_count = processed_results.iter().filter(|r| r.is_official).count();

        let sources: Vec<String> = processed_results
            .iter()
            .map(|r| r.source_domain.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let search_time = start_time.elapsed().as_millis() as u64;

        Ok(DocumentationSearchResponse {
            query: query.to_string(),
            summary,
            results: processed_results,
            official_results_count: final_official_count,
            used_fallback,
            total_found: all_results.len(),
            search_time_ms: search_time,
            sources,
            used_llm_verification: verification_result.is_some(),
            verification_passed: verification_result.as_ref().map(|v| v.is_authentic),
        })
    }

    /// Generate concise summary without AI fluff
    async fn generate_summary(
        &self,
        query: &str,
        results: &[ProcessedSearchResult],
    ) -> Result<String> {
        if results.is_empty() {
            return Ok("No relevant documentation found".to_string());
        }

        // Use LLM for intelligent summarization if available
        if let Some(ref llm_client) = self.llm_client {
            if llm_client.is_available() {
                let _context = results
                    .iter()
                    .take(3) // Top 3 most relevant
                    .map(|r| {
                        format!(
                            "Source: {} ({})\nContent: {}",
                            r.source_domain,
                            if r.is_official {
                                "Official"
                            } else {
                                "Community"
                            },
                            r.snippet
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n");

                // Create mock search results for LLM synthesis
                let mock_results: Vec<crate::rag::RagSearchResult> = results
                    .iter()
                    .take(3)
                    .map(|r| crate::rag::RagSearchResult {
                        id: r.url.clone(),
                        content: r.snippet.clone(),
                        source_path: std::path::PathBuf::from(&r.url),
                        source_type: if r.is_official {
                            crate::rag::SourceType::Curated
                        } else {
                            crate::rag::SourceType::Remote
                        },
                        title: Some(r.title.clone()),
                        section: None,
                        score: r.final_score,
                        metadata: crate::rag::DocumentMetadata {
                            file_type: "web".to_string(),
                            size: r.snippet.len() as u64,
                            modified: r.timestamp.unwrap_or_else(chrono::Utc::now),
                            tags: vec!["documentation".to_string()],
                            language: Some("en".to_string()),
                        },
                    })
                    .collect();

                match llm_client.synthesize_answer(query, &mock_results).await {
                    Ok(response) => return Ok(response.answer),
                    Err(e) => log::warn!("LLM summarization failed, using fallback: {}", e),
                }
            }
        }

        // Fallback: Generate summary from top results
        let official_count = results.iter().filter(|r| r.is_official).count();
        let summary_prefix = if official_count > 0 {
            format!("From {} official sources", official_count)
        } else {
            "From community sources".to_string()
        };

        let top_content = results
            .iter()
            .take(2)
            .map(|r| r.snippet.split('.').next().unwrap_or(&r.snippet))
            .collect::<Vec<_>>()
            .join(". ");

        Ok(format!("{}: {}", summary_prefix, top_content))
    }

    /// Check if system is ready for searches
    pub fn is_available(&self) -> bool {
        self.config.enabled
    }

    /// Get configuration
    pub fn config(&self) -> &WebSearchConfig {
        &self.config
    }
}
