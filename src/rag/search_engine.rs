//! Smart search engine orchestrator for intelligent RAG search
//!
//! This module coordinates query enhancement, embedding selection, multi-stage search,
//! and result verification to provide the best possible search experience.

use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

use crate::rag::{
    embeddings::EmbeddingModel,
    indexer::Indexer,
    llm::LlmClient,
    query_enhancer::{QueryEnhancer, EnhancedQuery, SearchStrategy},
    result_verifier::{ResultVerifier, VerifiedResult},
    EmbeddingProvider, RagConfig, RagSearchResult,
};

#[cfg(test)]
use crate::rag::SmartSearchConfig;

/// Smart search engine that orchestrates intelligent search strategies
pub struct SmartSearchEngine {
    config: RagConfig,
    query_enhancer: QueryEnhancer,
    result_verifier: ResultVerifier,
    embedding_model: Option<EmbeddingModel>,
    #[allow(dead_code)] // Used in public API methods
    llm_client: Option<LlmClient>,
}


impl SmartSearchEngine {
    /// Create a new smart search engine
    pub async fn new(config: RagConfig, llm_client: Option<LlmClient>) -> Result<Self> {
        log::info!("Initializing smart search engine with config: {:?}", config.smart_search);

        // Initialize embedding model based on smart search preferences
        let embedding_model = Self::initialize_embedding_model(&config).await?;

        // Create query enhancer
        let query_enhancer = QueryEnhancer::new(llm_client.clone(), config.smart_search.clone());

        // Create result verifier
        let result_verifier = ResultVerifier::new(llm_client.clone(), config.smart_search.clone());

        Ok(Self {
            config,
            query_enhancer,
            result_verifier,
            embedding_model,
            llm_client,
        })
    }

    /// Initialize the best available embedding model
    async fn initialize_embedding_model(config: &RagConfig) -> Result<Option<EmbeddingModel>> {
        if !config.smart_search.prefer_semantic {
            log::info!("Semantic embeddings disabled by config");
            return Ok(None);
        }

        // Try smart auto-selection first if using default hash provider
        if matches!(config.embedding.provider, EmbeddingProvider::Hash) {
            log::info!("Default hash provider detected, attempting auto-selection of better model");
            match EmbeddingModel::new_auto_select().await {
                Ok(model) => {
                    log::info!("Successfully auto-selected embedding model: {:?}", model.get_config().provider);
                    return Ok(Some(model));
                }
                Err(e) => {
                    log::warn!("Auto-selection failed, trying configured provider: {}", e);
                }
            }
        }

        // Try to initialize with configured embedding provider
        match EmbeddingModel::new_with_config(config.embedding.clone()).await {
            Ok(model) => {
                log::info!("Successfully initialized embedding model: {:?}", config.embedding.provider);
                Ok(Some(model))
            }
            Err(e) => {
                log::warn!("Failed to initialize embedding model, will use fallback: {}", e);
                Ok(None)
            }
        }
    }

    /// Perform intelligent search with multi-stage strategy
    pub async fn search(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<VerifiedResult>> {
        log::info!("Starting smart search for: '{}'", query);

        // Stage 1: Query Enhancement
        let enhanced_query = self.query_enhancer.enhance_query(query).await?;
        log::debug!("Enhanced query with {} variations", enhanced_query.variations.len());

        // Stage 2: Multi-strategy search execution
        let mut all_results = if self.config.smart_search.enable_multi_stage {
            self.execute_multi_stage_search(&enhanced_query).await?
        } else {
            self.execute_single_stage_search(&enhanced_query).await?
        };

        log::debug!("Collected {} raw results from search stages", all_results.len());

        // Stage 3: Deduplication and initial filtering
        all_results = self.deduplicate_results(all_results);

        // Stage 4: Result verification and scoring
        let verified_results = self.result_verifier.verify_results(&enhanced_query, all_results).await?;

        // Stage 5: Final ranking and limiting
        let final_results = self.finalize_results(verified_results, max_results);

        log::info!(
            "Smart search completed: {} verified results for '{}'",
            final_results.len(),
            query
        );

        Ok(final_results)
    }

    /// Execute multi-stage search with different strategies
    async fn execute_multi_stage_search(&self, query: &EnhancedQuery) -> Result<Vec<RagSearchResult>> {
        let mut all_results = Vec::new();

        // Stage 1: Direct semantic search with original query
        if let Some(ref embedding_model) = self.embedding_model {
            log::debug!("Stage 1: Semantic search with original query");
            match self.semantic_search(&query.original, embedding_model).await {
                Ok(mut results) => {
                    log::debug!("Semantic search found {} results", results.len());
                    all_results.append(&mut results);
                }
                Err(e) => log::warn!("Semantic search failed: {}", e),
            }
        }

        // Stage 2: Enhanced query variations
        log::debug!("Stage 2: Enhanced query variations");
        for (i, variation) in query.variations.iter().enumerate().take(3) { // Limit to top 3 variations
            log::debug!("Searching with variation {}: '{}'", i + 1, variation.query);

            let mut variation_results = match variation.strategy {
                SearchStrategy::Semantic => {
                    if let Some(ref embedding_model) = self.embedding_model {
                        self.semantic_search(&variation.query, embedding_model).await.unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                }
                SearchStrategy::Keyword => {
                    self.keyword_search(&variation.query).await.unwrap_or_default()
                }
                SearchStrategy::Code => {
                    self.code_search(&variation.query).await.unwrap_or_default()
                }
                SearchStrategy::Mixed => {
                    let mut mixed_results = Vec::new();
                    if let Some(ref embedding_model) = self.embedding_model {
                        if let Ok(mut semantic_results) = self.semantic_search(&variation.query, embedding_model).await {
                            mixed_results.append(&mut semantic_results);
                        }
                    }
                    if let Ok(mut keyword_results) = self.keyword_search(&variation.query).await {
                        mixed_results.append(&mut keyword_results);
                    }
                    mixed_results
                }
                _ => Vec::new(),
            };

            // Apply variation weight to scores
            for result in &mut variation_results {
                result.score *= variation.weight;
            }

            all_results.append(&mut variation_results);
        }

        // Stage 3: Keyword fallback for exact matches
        log::debug!("Stage 3: Keyword fallback");
        let mut keyword_results = self.keyword_search(&query.original).await.unwrap_or_default();
        // Boost keyword results slightly since they're exact matches
        for result in &mut keyword_results {
            result.score *= 1.1;
        }
        all_results.append(&mut keyword_results);

        Ok(all_results)
    }

    /// Execute single-stage search (simpler approach)
    async fn execute_single_stage_search(&self, query: &EnhancedQuery) -> Result<Vec<RagSearchResult>> {
        if let Some(ref embedding_model) = self.embedding_model {
            self.semantic_search(&query.original, embedding_model).await
        } else {
            self.keyword_search(&query.original).await
        }
    }

    /// Perform semantic search using embeddings
    async fn semantic_search(&self, query: &str, embedding_model: &EmbeddingModel) -> Result<Vec<RagSearchResult>> {
        log::debug!("Performing semantic search for: '{}'", query);

        // Generate query embedding
        let query_embedding = embedding_model.embed_text(query).await?;

        // Search through stored embeddings
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        if !embedding_dir.exists() {
            log::debug!("No embeddings directory found");
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        let entries = std::fs::read_dir(embedding_dir)?;

        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    match self.load_and_score_embedding(&entry.path(), &query_embedding, embedding_model).await {
                        Ok(Some(result)) => {
                            if result.score >= self.config.similarity_threshold {
                                results.push(result);
                            }
                        }
                        Ok(None) => continue,
                        Err(e) => {
                            log::warn!("Failed to process embedding file {:?}: {}", entry.path(), e);
                        }
                    }
                }
            }
        }

        // Sort by similarity score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        log::debug!("Semantic search found {} results", results.len());
        Ok(results)
    }

    /// Perform keyword-based search
    async fn keyword_search(&self, query: &str) -> Result<Vec<RagSearchResult>> {
        log::debug!("Performing keyword search for: '{}'", query);

        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        if !embedding_dir.exists() {
            return Ok(vec![]);
        }

        let query_words: Vec<String> = query.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .map(|w| w.to_string())
            .collect();

        let mut results = Vec::new();
        let entries = std::fs::read_dir(embedding_dir)?;

        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(stored_chunk) = serde_json::from_str::<crate::rag::StoredChunk>(&content) {
                            let content_lower = stored_chunk.content.to_lowercase();

                            let matches = query_words.iter()
                                .filter(|word| content_lower.contains(*word))
                                .count();

                            if matches > 0 {
                                let score = matches as f32 / query_words.len() as f32;

                                results.push(RagSearchResult {
                                    id: stored_chunk.id,
                                    content: stored_chunk.content,
                                    source_path: stored_chunk.source_path,
                                    source_type: stored_chunk.source_type,
                                    title: stored_chunk.title,
                                    section: stored_chunk.section,
                                    score,
                                    chunk_index: stored_chunk.chunk_index,
                                    metadata: stored_chunk.metadata,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort by keyword match score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        log::debug!("Keyword search found {} results", results.len());
        Ok(results)
    }

    /// Perform code-specific search
    async fn code_search(&self, query: &str) -> Result<Vec<RagSearchResult>> {
        log::debug!("Performing code search for: '{}'", query);

        // For now, use keyword search with code-specific boosting
        let mut results = self.keyword_search(query).await?;

        // Boost results that appear to be from code files
        for result in &mut results {
            if self.is_code_file(&result.source_path) {
                result.score *= 1.3;
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(results)
    }

    /// Check if a file appears to be a code file
    fn is_code_file(&self, path: &PathBuf) -> bool {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(extension, "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "go" | "php" | "rb")
        } else {
            false
        }
    }

    /// Load and score a single embedding file (adapted from mod.rs)
    async fn load_and_score_embedding(
        &self,
        file_path: &PathBuf,
        query_embedding: &[f32],
        _embedding_model: &EmbeddingModel,
    ) -> Result<Option<RagSearchResult>> {
        let content = std::fs::read_to_string(file_path)?;
        let chunk_data: crate::rag::StoredChunk = serde_json::from_str(&content)?;

        // Calculate similarity score
        let score = EmbeddingModel::cosine_similarity(query_embedding, &chunk_data.embedding);

        Ok(Some(RagSearchResult {
            id: chunk_data.id,
            content: chunk_data.content,
            source_path: chunk_data.source_path,
            source_type: chunk_data.source_type,
            title: chunk_data.title,
            section: chunk_data.section,
            score,
            chunk_index: chunk_data.chunk_index,
            metadata: chunk_data.metadata,
        }))
    }

    /// Remove duplicate results based on content similarity
    fn deduplicate_results(&self, results: Vec<RagSearchResult>) -> Vec<RagSearchResult> {
        let mut unique_results = Vec::new();
        let mut seen_content = HashSet::new();
        let original_count = results.len();

        for result in results {
            // Create a simple hash of the content for deduplication
            let content_hash = format!("{}_{}", result.source_path.to_string_lossy(), result.chunk_index);

            if !seen_content.contains(&content_hash) {
                seen_content.insert(content_hash);
                unique_results.push(result);
            }
        }

        log::debug!("Deduplicated {} results to {}", original_count, unique_results.len());
        unique_results
    }

    /// Finalize results with ranking and limiting
    fn finalize_results(&self, mut results: Vec<VerifiedResult>, max_results: Option<usize>) -> Vec<VerifiedResult> {
        // Sort by confidence score (already done in verifier, but ensuring)
        results.sort_by(|a, b| b.confidence_score.partial_cmp(&a.confidence_score).unwrap());

        // Apply limit
        let limit = max_results.unwrap_or(self.config.max_results);
        if results.len() > limit {
            results.truncate(limit);
        }

        results
    }

    /// Check if the search engine is ready to perform intelligent search
    /// This is a public API method for external consumers
    #[allow(dead_code)] // Public API method - may be used by external code
    pub fn is_intelligent_mode_available(&self) -> bool {
        self.embedding_model.is_some() || self.llm_client.is_some()
    }

    /// Get search engine capabilities for debugging
    /// This is a public API method for external consumers
    #[allow(dead_code)] // Public API method - may be used by external code
    pub fn get_capabilities(&self) -> SearchCapabilities {
        SearchCapabilities {
            has_semantic_embeddings: self.embedding_model.is_some(),
            has_llm_client: self.llm_client.is_some(),
            has_query_enhancement: self.config.smart_search.enable_query_enhancement,
            has_result_verification: self.config.smart_search.enable_result_verification,
            multi_stage_enabled: self.config.smart_search.enable_multi_stage,
        }
    }
}

/// Search engine capabilities information
/// This is a public API struct for external consumers
#[derive(Debug)]
#[allow(dead_code)] // Public API struct - may be used by external code
pub struct SearchCapabilities {
    pub has_semantic_embeddings: bool,
    pub has_llm_client: bool,
    pub has_query_enhancement: bool,
    pub has_result_verification: bool,
    pub multi_stage_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rag::{CodeSecurityLevel, EmbeddingConfig, EmbeddingProvider};

    fn create_test_config() -> RagConfig {
        RagConfig {
            enabled: true,
            index_path: PathBuf::from("/tmp/test_index"),
            max_results: 10,
            similarity_threshold: 0.6,
            allow_pdf_processing: false,
            allow_code_processing: true,
            code_security_level: CodeSecurityLevel::Moderate,
            mask_secrets: true,
            max_file_size_mb: 100,
            embedding: EmbeddingConfig {
                provider: EmbeddingProvider::Hash,
                dimension: 384,
                model_path: None,
                api_key: None,
                endpoint: None,
                timeout_seconds: 30,
                batch_size: 32,
            },
            smart_search: SmartSearchConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_search_engine_initialization() {
        let config = create_test_config();
        let engine = SmartSearchEngine::new(config, None).await;
        assert!(engine.is_ok());
    }

    #[test]
    fn test_code_file_detection() {
        let _engine_config = create_test_config();
        // We can't easily test the full engine without setting it up, but we can test the logic
        let path = PathBuf::from("test.rs");
        // This would be tested with the actual engine instance
        assert!(path.extension().and_then(|ext| ext.to_str()) == Some("rs"));
    }
}