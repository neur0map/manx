//! Local RAG (Retrieval-Augmented Generation) system for Manx
//!
//! Provides document indexing, semantic search, and LLM integration
//! for enhanced documentation discovery and AI synthesis.

use crate::rag::embeddings::EmbeddingModel;
use crate::rag::indexer::Indexer;
use crate::rag::llm::LlmClient;
use crate::rag::search_engine::SmartSearchEngine;
use anyhow::Result;
use docrawl::{crawl, Config as DocrawlConfig, CrawlConfig};
// gag disabled: let docrawl manage its own spinner
#[cfg(unix)]
// removed libc imports; gag handles stdout/stderr capture cross‑platform
// no longer used: previous attempt to redirect crawler output
// #[cfg(unix)]
// use libc::{close, dup, dup2, open, O_WRONLY};
use serde::{Deserialize, Serialize};
// no need for Write trait; summary prints are plain
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

pub mod benchmarks;
pub mod embeddings;
pub mod indexer;
pub mod llm;
pub mod model_metadata;
pub mod providers;
pub mod query_enhancer;
pub mod result_verifier;
pub mod search_engine;

/// Embedding provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum EmbeddingProvider {
    #[default]
    Hash, // Default hash-based embeddings (current implementation)
    Onnx(String),        // Local ONNX model path
    Ollama(String),      // Ollama model name
    OpenAI(String),      // OpenAI model name (requires API key)
    HuggingFace(String), // HuggingFace model name (requires API key)
    Custom(String),      // Custom endpoint URL
}

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub dimension: usize,
    pub model_path: Option<PathBuf>, // For local models
    pub api_key: Option<String>,     // For API providers
    pub endpoint: Option<String>,    // For custom endpoints
    pub timeout_seconds: u64,
    pub batch_size: usize,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::Hash,
            dimension: 384, // Hash provider default (will be updated dynamically for others)
            model_path: None,
            api_key: None,
            endpoint: None,
            timeout_seconds: 30,
            batch_size: 32,
        }
    }
}

impl EmbeddingConfig {
    /// Update dimension from actual provider detection
    pub async fn detect_and_update_dimension(&mut self) -> Result<()> {
        use crate::rag::embeddings::EmbeddingModel;

        let model = EmbeddingModel::new_with_config(self.clone()).await?;
        let detected_dimension = model.get_dimension().await?;

        if self.dimension != detected_dimension {
            log::info!(
                "Updating dimension from {} to {} for provider {:?}",
                self.dimension,
                detected_dimension,
                self.provider
            );
            self.dimension = detected_dimension;
        }

        Ok(())
    }
}

/// Security level for code processing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodeSecurityLevel {
    /// Strict: Reject files with any suspicious patterns
    Strict,
    /// Moderate: Log warnings but allow most files
    Moderate,
    /// Permissive: Minimal security checks
    Permissive,
}

impl Default for CodeSecurityLevel {
    fn default() -> Self {
        Self::Moderate
    }
}

/// Configuration for smart search capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSearchConfig {
    pub prefer_semantic: bool,            // Use ONNX over hash when available
    pub enable_query_enhancement: bool,   // Use LLM for query expansion
    pub enable_result_verification: bool, // Use LLM for relevance checking
    pub min_confidence_score: f32,        // Minimum relevance threshold
    pub max_query_variations: usize,      // Number of query variations to try
    pub enable_multi_stage: bool,         // Enable multi-stage search strategy
    pub adaptive_chunking: bool,          // Use smart code-aware chunking
}

impl Default for SmartSearchConfig {
    fn default() -> Self {
        Self {
            prefer_semantic: true,
            enable_query_enhancement: true,
            enable_result_verification: true,
            min_confidence_score: 0.7,
            max_query_variations: 3,
            enable_multi_stage: true,
            adaptive_chunking: true,
        }
    }
}

/// Configuration for the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub enabled: bool,
    pub index_path: PathBuf,
    pub max_results: usize,
    pub similarity_threshold: f32,
    pub allow_pdf_processing: bool,
    pub allow_code_processing: bool,
    pub code_security_level: CodeSecurityLevel,
    pub mask_secrets: bool,
    pub max_file_size_mb: u64,
    pub embedding: EmbeddingConfig,
    pub smart_search: SmartSearchConfig,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enabled by default
            index_path: PathBuf::from("~/.cache/manx/rag_index"),
            max_results: 10,
            similarity_threshold: 0.6,
            allow_pdf_processing: false, // Disabled by default for security
            allow_code_processing: true, // Enabled by default with security checks
            code_security_level: CodeSecurityLevel::Moderate,
            mask_secrets: true,    // Mask secrets by default
            max_file_size_mb: 100, // 100MB default limit
            embedding: EmbeddingConfig::default(),
            smart_search: SmartSearchConfig::default(),
        }
    }
}

/// Document chunk for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub title: Option<String>,
    pub section: Option<String>,
    pub chunk_index: usize,
    pub metadata: DocumentMetadata,
}

/// Type of document source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    Local,
    Remote,
    Curated,
    Web,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub file_type: String,
    pub size: u64,
    pub modified: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub language: Option<String>,
}

/// Search result from RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSearchResult {
    pub id: String,
    pub content: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub title: Option<String>,
    pub section: Option<String>,
    pub score: f32,
    pub chunk_index: usize,
    pub metadata: DocumentMetadata,
}

/// RAG system stats
#[derive(Debug, Serialize, Deserialize)]
pub struct RagStats {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub index_size_mb: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub sources: Vec<String>,
}

/// Stored chunk with embedding for file-based vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredChunk {
    pub id: String,
    pub content: String,
    pub source_path: PathBuf,
    pub source_type: SourceType,
    pub title: Option<String>,
    pub section: Option<String>,
    pub chunk_index: usize,
    pub metadata: DocumentMetadata,
    pub embedding: Vec<f32>,
}

/// Local file-based RAG system
pub struct RagSystem {
    config: RagConfig,
    llm_client: Option<LlmClient>,
}

impl RagSystem {
    pub async fn new(config: RagConfig) -> Result<Self> {
        Self::new_with_llm(config, None).await
    }

    pub async fn new_with_llm(config: RagConfig, llm_client: Option<LlmClient>) -> Result<Self> {
        if !config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        // Initialize the local vector storage system
        let indexer = Indexer::new(&config)?;
        let index_path = indexer.get_index_path();

        // Create index directory if it doesn't exist
        std::fs::create_dir_all(index_path)?;

        log::info!(
            "RAG system initialized with local vector storage at {:?}",
            index_path
        );
        Ok(Self { config, llm_client })
    }

    pub async fn index_document(&mut self, path: PathBuf) -> Result<usize> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        let indexer = Indexer::new(&self.config)?;
        let chunks = indexer.index_document(path)?;
        let chunk_count = chunks.len();

        // Store chunks in local vector storage
        self.store_chunks_locally(&chunks).await?;

        log::info!("Successfully indexed and stored {} chunks", chunk_count);
        Ok(chunk_count)
    }

    pub async fn index_directory(&mut self, path: PathBuf) -> Result<usize> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        let indexer = Indexer::new(&self.config)?;
        let chunks = indexer.index_directory(path)?;
        let chunk_count = chunks.len();

        // Store chunks in local vector storage
        self.store_chunks_locally(&chunks).await?;

        log::info!(
            "Successfully indexed and stored {} chunks from directory",
            chunk_count
        );
        Ok(chunk_count)
    }

    #[allow(dead_code)]
    pub async fn index_url(&mut self, url: &str) -> Result<usize> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        log::info!("Indexing URL: {}", url);

        let indexer = Indexer::new(&self.config)?;
        let chunks = indexer.index_url(url.to_string()).await?;
        let chunk_count = chunks.len();

        // Store chunks in local vector storage
        self.store_chunks_locally(&chunks).await?;

        log::info!(
            "Successfully indexed and stored {} chunks from URL",
            chunk_count
        );
        Ok(chunk_count)
    }

    #[allow(dead_code)]
    pub async fn index_url_deep(
        &mut self,
        url: &str,
        max_depth: Option<u32>,
        max_pages: Option<u32>,
    ) -> Result<usize> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        log::info!(
            "Deep indexing URL: {} (depth: {:?}, pages: {:?})",
            url,
            max_depth,
            max_pages
        );

        let indexer = Indexer::new(&self.config)?;
        // Convert old parameters to new docrawl-based parameters
        let crawl_all = max_pages.is_none(); // If no page limit, crawl all
        let chunks = indexer
            .index_url_deep(url.to_string(), max_depth, crawl_all)
            .await?;
        let chunk_count = chunks.len();

        // Store chunks in local vector storage
        self.store_chunks_locally(&chunks).await?;

        log::info!(
            "Successfully deep indexed and stored {} chunks from URL",
            chunk_count
        );
        Ok(chunk_count)
    }

    /// Streamed deep indexing: overlaps crawling and embedding using Tokio for speed
    pub async fn index_url_deep_stream(
        &self,
        url: &str,
        max_depth: Option<u32>,
        crawl_all: bool,
        embed_concurrency: Option<usize>,
        crawl_max_pages: Option<usize>,
    ) -> Result<usize> {
        use std::collections::HashSet;
        use std::sync::Arc;
        use tokio::sync::mpsc;
        use tokio::time::{interval, Duration};

        let temp_dir = std::env::temp_dir().join(format!("manx_crawl_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Build crawler config
        let base_url = url::Url::parse(url)?;
        let crawl_config = CrawlConfig {
            base_url,
            output_dir: temp_dir.clone(),
            user_agent: "Manx/0.5.0 (Documentation Crawler)".to_string(),
            max_depth: if let Some(d) = max_depth {
                Some(d as usize)
            } else if crawl_all {
                None
            } else {
                Some(3)
            },
            rate_limit_per_sec: 20,
            follow_sitemaps: true,
            concurrency: std::cmp::max(8, num_cpus::get()),
            timeout: Some(std::time::Duration::from_secs(30)),
            resume: false,
            config: DocrawlConfig::default(),
        };

        // Create embedding model once
        let embedding_model =
            Arc::new(EmbeddingModel::new_with_config(self.config.embedding.clone()).await?);

        // Channel for discovered markdown files
        let (tx, rx) = mpsc::channel::<PathBuf>(200);
        let rx = std::sync::Arc::new(tokio::sync::Mutex::new(rx));
        let crawl_max_pages = crawl_max_pages.unwrap_or(usize::MAX);

        // Counters for summary (no live prints during crawl)
        let pages_counter = Arc::new(AtomicUsize::new(0));
        let chunks_counter = Arc::new(AtomicUsize::new(0));

        // Spawn crawler; suppress its stdout to avoid competing spinner
        let crawl_handle = tokio::spawn(async move {
            let _ = crawl(crawl_config).await;
        });

        // Spawn scanner: discover new markdown files while crawler runs
        let temp_dir_clone = temp_dir.clone();
        let scanner_tx = tx.clone();
        let pc = pages_counter.clone();
        let scanner_handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(300));
            let mut seen: HashSet<PathBuf> = HashSet::new();
            let mut idle_ticks = 0u32;
            loop {
                ticker.tick().await;
                let mut new_found = 0usize;
                for entry in WalkDir::new(&temp_dir_clone)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                        let pb = path.to_path_buf();
                        if !seen.contains(&pb) {
                            seen.insert(pb.clone());
                            if scanner_tx.send(pb).await.is_err() {
                                break;
                            }
                            new_found += 1;
                            pc.fetch_add(1, Ordering::Relaxed);
                            if seen.len() >= crawl_max_pages {
                                break;
                            }
                        }
                    }
                }
                if new_found == 0 {
                    idle_ticks += 1;
                } else {
                    idle_ticks = 0;
                }
                if seen.len() >= crawl_max_pages {
                    break;
                }
                if idle_ticks > 20 {
                    break;
                }
            }
            drop(scanner_tx);
        });

        // Worker pool
        let workers = embed_concurrency.unwrap_or_else(|| std::cmp::max(4, num_cpus::get()));
        let mut joins = Vec::new();
        let config_clone = self.config.clone();
        let url_for_worker = url.to_string();
        for _ in 0..workers {
            let rx = rx.clone();
            let embedding_model = embedding_model.clone();
            let config_clone = config_clone.clone();
            let url_clone = url_for_worker.clone();
            let chunks_counter = chunks_counter.clone();
            let join = tokio::spawn(async move {
                let mut stored = 0usize;
                let idx = match Indexer::new(&config_clone) {
                    Ok(i) => i,
                    Err(_) => return 0usize,
                };
                loop {
                    let opt_path = { rx.lock().await.recv().await };
                    let Some(md_path) = opt_path else { break };
                    if let Ok(chunks) = idx.process_markdown_file(&md_path, &url_clone).await {
                        if let Ok(count) =
                            store_chunks_with_model_config(&config_clone, &chunks, &embedding_model)
                                .await
                        {
                            stored += count;
                            chunks_counter.fetch_add(count, Ordering::Relaxed);
                        }
                    }
                }
                stored
            });
            joins.push(join);
        }

        let _ = scanner_handle.await;
        let _ = crawl_handle.await;
        drop(tx);

        let mut total_stored = 0usize;
        for j in joins {
            if let Ok(count) = j.await {
                total_stored += count;
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
        // Final summary (separate from docrawl spinner output)
        let pages = pages_counter.load(Ordering::Relaxed);
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        eprintln!();
        eprintln!("==== Manx Index Summary ====");
        eprintln!("Stored chunks: {}", total_stored);
        eprintln!("Pages discovered: {}", pages);
        eprintln!("Index path: {}", index_path.display());
        Ok(total_stored)
    }

    pub async fn search(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<RagSearchResult>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        log::info!("Starting intelligent search for: '{}'", query);

        // Create smart search engine
        let search_engine =
            SmartSearchEngine::new(self.config.clone(), self.llm_client.clone()).await?;

        // Perform intelligent search
        let verified_results = search_engine.search(query, max_results).await?;

        // Convert VerifiedResult back to RagSearchResult for compatibility
        let results: Vec<RagSearchResult> = verified_results
            .into_iter()
            .map(|verified| RagSearchResult {
                id: verified.result.id,
                content: verified.result.content,
                source_path: verified.result.source_path,
                source_type: verified.result.source_type,
                title: verified.result.title,
                section: verified.result.section,
                score: verified.confidence_score, // Use the verified confidence score
                chunk_index: verified.result.chunk_index,
                metadata: verified.result.metadata,
            })
            .collect();

        log::info!(
            "Intelligent search completed with {} results",
            results.len()
        );
        Ok(results)
    }

    pub async fn get_stats(&self) -> Result<RagStats> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        if !embedding_dir.exists() {
            return Ok(RagStats {
                total_documents: 0,
                total_chunks: 0,
                index_size_mb: 0.0,
                last_updated: chrono::Utc::now(),
                sources: vec![],
            });
        }

        // Count chunks and calculate size
        let mut total_chunks = 0;
        let mut total_size = 0u64;
        let mut sources = std::collections::HashSet::new();
        let mut last_modified = std::time::UNIX_EPOCH;

        let entries = std::fs::read_dir(&embedding_dir)?;
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    total_chunks += 1;

                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();

                        // Track most recent modification
                        if let Ok(modified) = metadata.modified() {
                            if modified > last_modified {
                                last_modified = modified;
                            }
                        }
                    }

                    // Try to extract source info from chunk data
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        if let Ok(chunk_data) = serde_json::from_str::<StoredChunk>(&content) {
                            if let Some(source_str) = chunk_data.source_path.to_str() {
                                sources.insert(source_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Convert sources to unique document count estimate
        let total_documents = sources.len();
        let index_size_mb = total_size as f64 / (1024.0 * 1024.0);

        let last_updated = chrono::DateTime::<chrono::Utc>::from(last_modified);

        let sources_vec: Vec<String> = sources.into_iter().collect();

        Ok(RagStats {
            total_documents,
            total_chunks,
            index_size_mb,
            last_updated,
            sources: sources_vec,
        })
    }

    pub async fn clear_index(&self) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        log::info!("Clearing local vector storage");

        // Get index path and embeddings directory
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        if embedding_dir.exists() {
            // Remove all embedding files
            let entries = std::fs::read_dir(&embedding_dir)?;
            let mut cleared_count = 0;

            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        if let Err(e) = std::fs::remove_file(entry.path()) {
                            log::warn!("Failed to remove embedding file {:?}: {}", entry.path(), e);
                        } else {
                            cleared_count += 1;
                        }
                    }
                }
            }

            log::info!(
                "Successfully cleared {} embedding files from local vector storage",
                cleared_count
            );
        } else {
            log::info!("Local vector storage directory does not exist, nothing to clear");
        }

        Ok(())
    }

    pub async fn health_check(&self) -> Result<()> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        log::info!("Running RAG system health check...");

        // Check if embedding model can be loaded
        let _embedding_model = EmbeddingModel::new_with_config(self.config.embedding.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Embedding model unavailable: {}", e))?;
        log::info!("✓ Embedding model loaded successfully");

        // Check if index directory exists and is accessible
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();

        if index_path.exists() {
            log::info!("✓ Local index directory exists: {:?}", index_path);

            // Check embeddings directory
            let embedding_dir = index_path.join("embeddings");
            if embedding_dir.exists() {
                // Count existing embeddings
                match std::fs::read_dir(&embedding_dir) {
                    Ok(entries) => {
                        let count = entries.filter_map(|e| e.ok()).count();
                        log::info!(
                            "✓ Local vector storage accessible with {} embedding files",
                            count
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "⚠ Local vector storage directory exists but cannot read contents: {}",
                            e
                        );
                    }
                }
            } else {
                log::info!("✓ Local vector storage will be created when needed");
            }
        } else {
            log::info!("✓ Local index directory will be created: {:?}", index_path);
        }

        // Test file system write access
        let test_file = index_path.join(".health_check");
        match std::fs::create_dir_all(index_path) {
            Ok(_) => {
                match std::fs::write(&test_file, "health_check") {
                    Ok(_) => {
                        log::info!("✓ File system write access confirmed");
                        let _ = std::fs::remove_file(&test_file); // Clean up test file
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("File system write access failed: {}", e));
                    }
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Cannot create index directory: {}", e));
            }
        }

        log::info!("RAG system health check: All systems operational");
        Ok(())
    }

    /// Store document chunks in local file-based vector storage
    async fn store_chunks_locally(&self, chunks: &[DocumentChunk]) -> Result<()> {
        use uuid::Uuid;

        if chunks.is_empty() {
            log::info!("No chunks to store locally");
            return Ok(());
        }

        log::info!("Storing {} chunks in local vector storage", chunks.len());

        // Initialize embedding model
        let embedding_model =
            EmbeddingModel::new_with_config(self.config.embedding.clone()).await?;

        // Get index path and create embeddings directory
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        // Create directories if they don't exist
        std::fs::create_dir_all(&embedding_dir)?;

        // Process chunks and store with embeddings
        let mut stored_count = 0;

        for (i, chunk) in chunks.iter().enumerate() {
            // Generate embedding for chunk content
            let embedding = match embedding_model.embed_text(&chunk.content).await {
                Ok(embedding) => embedding,
                Err(e) => {
                    log::warn!("Failed to generate embedding for chunk {}: {}", chunk.id, e);
                    continue;
                }
            };

            // Create stored chunk with embedding
            let stored_chunk = StoredChunk {
                id: chunk.id.clone(),
                content: chunk.content.clone(),
                source_path: chunk.source_path.clone(),
                source_type: chunk.source_type.clone(),
                title: chunk.title.clone(),
                section: chunk.section.clone(),
                chunk_index: chunk.chunk_index,
                metadata: chunk.metadata.clone(),
                embedding,
            };

            // Save to JSON file
            let file_id = Uuid::new_v4().to_string();
            let file_path = embedding_dir.join(format!("{}.json", file_id));

            let json_content = serde_json::to_string_pretty(&stored_chunk)?;
            std::fs::write(&file_path, json_content)?;

            stored_count += 1;
            log::debug!("Stored chunk {} to {:?}", chunk.id, file_path);
            if (i + 1) % 100 == 0 || i + 1 == chunks.len() {
                println!("Stored {}/{} chunks...", i + 1, chunks.len());
            }
        }

        log::info!(
            "Successfully stored {} chunks in local vector storage",
            stored_count
        );
        Ok(())
    }
}

/// Store chunks using a shared embedding model (config-based helper)
pub async fn store_chunks_with_model_config(
    config: &RagConfig,
    chunks: &[DocumentChunk],
    embedding_model: &EmbeddingModel,
) -> Result<usize> {
    use uuid::Uuid;
    if chunks.is_empty() {
        return Ok(0);
    }

    let indexer = Indexer::new(config)?;
    let index_path = indexer.get_index_path();
    let embedding_dir = index_path.join("embeddings");
    std::fs::create_dir_all(&embedding_dir)?;

    let mut stored_count = 0usize;
    for chunk in chunks {
        let embedding = match embedding_model.embed_text(&chunk.content).await {
            Ok(embedding) => embedding,
            Err(_) => continue,
        };

        let stored_chunk = StoredChunk {
            id: chunk.id.clone(),
            content: chunk.content.clone(),
            source_path: chunk.source_path.clone(),
            source_type: chunk.source_type.clone(),
            title: chunk.title.clone(),
            section: chunk.section.clone(),
            chunk_index: chunk.chunk_index,
            metadata: chunk.metadata.clone(),
            embedding,
        };

        let file_id = Uuid::new_v4().to_string();
        let file_path = embedding_dir.join(format!("{}.json", file_id));
        let json_content = serde_json::to_string_pretty(&stored_chunk)?;
        std::fs::write(&file_path, json_content)?;
        stored_count += 1;
    }

    Ok(stored_count)
}
