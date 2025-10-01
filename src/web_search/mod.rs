//! Intelligent documentation search with DuckDuckGo + semantic embeddings
//!
//! This module provides official-first documentation search that:
//! - Prioritizes official documentation sites by default
//! - Falls back to trusted community sources with clear notifications
//! - Uses semantic embeddings for relevance filtering
//! - Optionally uses LLM for authenticity verification and summarization
//! - Maintains privacy with anonymous DuckDuckGo searches

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod llm_verifier;
pub mod official_sources;
pub mod query_analyzer;
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
    embedding_model: Option<Arc<crate::rag::embeddings::EmbeddingModel>>,
    llm_client: Option<Arc<crate::rag::llm::LlmClient>>,
    official_sources: official_sources::OfficialSourceManager,
    query_analyzer: query_analyzer::QueryAnalyzer,
}

impl DocumentationSearchSystem {
    /// Create new documentation search system
    pub async fn new(
        config: WebSearchConfig,
        llm_config: Option<crate::rag::llm::LlmConfig>,
        embedding_config: Option<crate::rag::EmbeddingConfig>,
    ) -> Result<Self> {
        if !config.enabled {
            return Err(anyhow!("Documentation search is disabled"));
        }

        // Initialize semantic embeddings for similarity scoring (with Arc for sharing)
        let embedding_model = match match &embedding_config {
            Some(cfg) => crate::rag::embeddings::EmbeddingModel::new_with_config(cfg.clone()).await,
            None => crate::rag::embeddings::EmbeddingModel::new().await,
        } {
            Ok(model) => {
                log::info!("Semantic embeddings initialized for search (pooled)");
                Some(Arc::new(model))
            }
            Err(e) => {
                log::warn!(
                    "Semantic embeddings unavailable, using text matching: {}",
                    e
                );
                None
            }
        };

        // Initialize LLM client if configured (with Arc for sharing)
        let llm_client = if let Some(llm_cfg) = llm_config {
            match crate::rag::llm::LlmClient::new(llm_cfg) {
                Ok(client) => {
                    log::info!("LLM client initialized for result verification (pooled)");
                    Some(Arc::new(client))
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
        let query_analyzer = query_analyzer::QueryAnalyzer::new();

        Ok(Self {
            config,
            embedding_model,
            llm_client,
            official_sources,
            query_analyzer,
        })
    }

    /// Search for documentation with official-first strategy
    pub async fn search(&mut self, query: &str) -> Result<DocumentationSearchResponse> {
        let start_time = std::time::Instant::now();

        log::info!("🔍 Searching official documentation for: {}", query);

        // Step 0: Analyze query intent to enhance search strategy
        let query_analysis = self
            .query_analyzer
            .analyze_query(query, self.llm_client.as_deref())
            .await?;
        log::info!(
            "🧠 Query analysis: {} -> {} (confidence: {:.1}%)",
            query_analysis.original_query,
            query_analysis.enhanced_query,
            query_analysis.confidence * 100.0
        );

        // Use enhanced query for better search results
        let search_query = &query_analysis.enhanced_query;
        // Prefer keeping key phrases intact (e.g., "hello world")
        fn extract_key_phrase(q: &str) -> Option<String> {
            let q = q.to_lowercase();
            if let Some(start) = q.find('"') {
                if let Some(end_rel) = q[start + 1..].find('"') {
                    let end = start + 1 + end_rel;
                    let phrase = &q[start + 1..end];
                    if !phrase.trim().is_empty() {
                        return Some(phrase.trim().to_string());
                    }
                }
            }
            // Fallback: first two content words (basic stopword filter)
            let stop: std::collections::HashSet<&str> = [
                "a", "an", "and", "the", "in", "on", "of", "to", "for", "how", "do", "i", "with",
                "using", "is", "are", "be", "this", "that", "it", "from", "by", "into", "as",
            ]
            .into_iter()
            .collect();
            let content: Vec<&str> = q
                .split_whitespace()
                .filter(|w| !stop.contains(*w))
                .collect();
            if content.len() >= 2 {
                Some(format!("{} {}", content[0], content[1]))
            } else {
                None
            }
        }
        let phrase_query = if let Some(p) = extract_key_phrase(&query_analysis.original_query) {
            format!("\"{}\" {}", p, search_query)
        } else {
            search_query.to_string()
        };

        // Step 1: Apply smart search strategy (only when LLM is available)
        let official_query = if self.llm_client.is_some() {
            match &query_analysis.search_strategy {
                query_analyzer::SearchStrategy::FrameworkSpecific { framework, sites } => {
                    log::info!("🎯 Using LLM-enhanced framework search for {}", framework);
                    self.build_technical_search_query(&phrase_query, sites)
                }
                query_analyzer::SearchStrategy::OfficialDocsFirst { frameworks } => {
                    log::info!(
                        "📚 Using LLM-enhanced prioritized search for: {}",
                        frameworks.join(", ")
                    );
                    self.build_dev_focused_query(&phrase_query, frameworks)
                }
                _ => {
                    if self.is_technical_query(&query_analysis) {
                        log::info!("🔧 Using LLM-enhanced technical search");
                        self.build_dev_focused_query(&phrase_query, &[])
                    } else {
                        self.official_sources.build_official_query(&phrase_query)
                    }
                }
            }
        } else {
            // Default behavior when no LLM - unchanged original functionality
            log::debug!("Using standard search (no LLM configured)");
            self.official_sources.build_official_query(&phrase_query)
        };
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
                &phrase_query,
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

        // Step 4: Process results with enhanced semantic filtering and query analysis
        let mut processed_results = if let Some(ref embedding_model) = self.embedding_model {
            result_processor::process_with_embeddings_and_analysis(
                &query_analysis,
                &all_results,
                embedding_model,
                &self.official_sources,
                self.config.similarity_threshold,
            )
            .await?
        } else {
            result_processor::process_without_embeddings(
                query,
                &all_results,
                &self.official_sources,
            )
        };

        // Fallback: if nothing survived filtering, retry with text matching (softer) before final filters
        if processed_results.is_empty() {
            log::info!("No results after semantic filtering; retrying with text matching fallback");
            processed_results = result_processor::process_without_embeddings(
                query,
                &all_results,
                &self.official_sources,
            );
        }

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

        // Step 4b: Filter out non-technical domains (LLM-enhanced only)
        processed_results = result_processor::filter_non_technical_domains(
            processed_results,
            &query_analysis,
            self.llm_client.is_some(),
        );

        // Step 4c: Filter out low-quality results
        processed_results = result_processor::filter_quality_results(processed_results, 20);

        // Step 4d: Remove duplicates
        let mut processed_results = result_processor::deduplicate_results(processed_results);

        // Second-chance fallback: if filters resulted in zero items, retry with softer text matching
        if processed_results.is_empty() {
            log::info!("No results after filtering; retrying with softer text-based processing");
            let mut soft_results = result_processor::process_without_embeddings(
                query,
                &all_results,
                &self.official_sources,
            );
            // Use a smaller snippet length minimum for soft pass
            soft_results = result_processor::filter_quality_results(soft_results, 10);
            processed_results = result_processor::deduplicate_results(soft_results);
        }

        // Step 5: LLM verification if available
        let verification_result = if let Some(ref llm_client) = self.llm_client {
            if llm_client.is_available() {
                log::info!("Verifying results with LLM");
                match llm_verifier::verify_search_results(query, &processed_results).await {
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
                        chunk_index: 0,
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

    /// Build technical search query prioritizing dev docs, GitHub, StackOverflow
    fn build_technical_search_query(&self, query: &str, framework_sites: &[String]) -> String {
        let dev_domains = [
            "github.com",
            "stackoverflow.com",
            "docs.rs",
            "developer.mozilla.org",
            "reactjs.org",
            "nodejs.org",
            "python.org",
            "rust-lang.org",
            "tauri.app",
            "electronjs.org",
            "dev.to",
            "medium.com/@",
        ];

        // Combine framework-specific sites with general dev domains
        let mut all_sites = framework_sites.to_vec();
        all_sites.extend(dev_domains.iter().map(|s| s.to_string()));

        // Remove duplicates
        all_sites.sort();
        all_sites.dedup();

        // Build site-restricted query
        let site_filters: String = all_sites
            .iter()
            .map(|site| format!("site:{}", site))
            .collect::<Vec<_>>()
            .join(" OR ");

        format!("({}) {}", site_filters, query)
    }

    /// Build developer-focused query with technical domain prioritization
    fn build_dev_focused_query(&self, query: &str, frameworks: &[String]) -> String {
        let mut dev_query = query.to_string();

        // Add framework-specific terms to boost relevance
        for framework in frameworks {
            if !dev_query.to_lowercase().contains(&framework.to_lowercase()) {
                dev_query = format!("{} {}", framework, dev_query);
            }
        }

        // Technical domains to prioritize
        let tech_domains = [
            "site:github.com",
            "site:stackoverflow.com",
            "site:docs.rs",
            "site:developer.mozilla.org",
            "site:dev.to",
        ];

        // Add technical domain boost (not exclusive, just prioritized)
        format!("({}) OR {}", tech_domains.join(" OR "), dev_query)
    }

    /// Check if query is technical based on analysis
    fn is_technical_query(&self, analysis: &query_analyzer::QueryAnalysis) -> bool {
        // Technical indicators
        !analysis.detected_frameworks.is_empty()
            || analysis
                .domain_context
                .primary_domain
                .contains("development")
            || analysis
                .domain_context
                .primary_domain
                .contains("programming")
            || analysis.query_type == query_analyzer::QueryType::Reference
            || analysis.original_query.to_lowercase().contains("api")
            || analysis.original_query.to_lowercase().contains("code")
            || analysis.original_query.to_lowercase().contains("library")
            || analysis.original_query.to_lowercase().contains("function")
            || analysis.original_query.to_lowercase().contains("method")
            || analysis.original_query.to_lowercase().contains("component")
    }
}
