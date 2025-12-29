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
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum CodeSecurityLevel {
    /// Strict: Reject files with any suspicious patterns
    Strict,
    /// Moderate: Log warnings but allow most files
    #[default]
    Moderate,
    /// Permissive: Minimal security checks
    Permissive,
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

        // If explicitly depth 0 and not crawl-all, do single-page fetch without the crawler
        if matches!(max_depth, Some(0)) && !crawl_all {
            eprintln!("\nIndexing single page (no crawl): {}", url);
            let embedding_model = std::sync::Arc::new(
                EmbeddingModel::new_with_config(self.config.embedding.clone()).await?,
            );
            let indexer = Indexer::new(&self.config)?;
            let chunks = indexer.index_single_url_no_crawl(url).await?;
            let total_stored =
                store_chunks_with_model_config(&self.config, &chunks, &embedding_model).await?;

            let index_path = indexer.get_index_path();
            eprintln!("\n==== Manx Index Summary ====");
            eprintln!("Mode: Single page (no crawl)");
            eprintln!("Chunks created: {}", chunks.len());
            eprintln!("Chunks stored: {}", total_stored);
            eprintln!("Index path: {}", index_path.display());
            return Ok(total_stored);
        }

        // If depth is 1 (shallow), prefer our manual shallow crawler to avoid docrawl host-scope quirks
        if matches!(max_depth, Some(1)) && !crawl_all {
            eprintln!("\nShallow crawl (depth 1) for: {}", url);
            let embedding_model = std::sync::Arc::new(
                EmbeddingModel::new_with_config(self.config.embedding.clone()).await?,
            );
            let indexer = Indexer::new(&self.config)?;
            let chunks = indexer.index_shallow_url(url, crawl_max_pages).await?;
            let total_stored =
                store_chunks_with_model_config(&self.config, &chunks, &embedding_model).await?;

            let index_path = indexer.get_index_path();
            eprintln!("\n==== Manx Index Summary ====");
            eprintln!("Mode: Shallow crawl (depth 1)");
            eprintln!("Chunks created: {}", chunks.len());
            eprintln!("Chunks stored: {}", total_stored);
            eprintln!("Index path: {}", index_path.display());
            return Ok(total_stored);
        }

        let temp_dir = std::env::temp_dir().join(format!("manx_crawl_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Show initial status
        eprintln!("\nStarting document crawl for: {}", url);
        eprintln!("   This will: 1) Crawl pages -> 2) Chunk content -> 3) Create embeddings");
        log::debug!("Temp directory: {}", temp_dir.display());
        eprintln!();

        // Resolve potential redirects to get canonical host (e.g., kali.org -> www.kali.org)
        let base_url = if let Ok(resp) = reqwest::Client::new().get(url).send().await {
            resp.url().clone()
        } else {
            url::Url::parse(url)?
        };
        // Configure docrawl secondary options (use defaults)
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
            silence: true,          // Silence docrawl; Manx renders its own progress UI
            rate_limit_per_sec: 20, // Reasonable rate limit for documentation sites
            follow_sitemaps: true,
            concurrency: std::cmp::max(8, num_cpus::get()), // Use more threads for faster crawling
            timeout: Some(std::time::Duration::from_secs(30)),
            resume: false,
            // Additional crawler behavior configuration
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
        let processed_pages_counter = Arc::new(AtomicUsize::new(0));
        let chunks_counter = Arc::new(AtomicUsize::new(0));

        // Track when crawl is done
        let crawl_done = Arc::new(AtomicBool::new(false));
        let crawl_done_clone = crawl_done.clone();

        // Spawn crawler; suppress its stdout to avoid competing spinner
        let crawl_handle = tokio::spawn(async move {
            let result = crawl(crawl_config).await;
            crawl_done_clone.store(true, Ordering::Relaxed);
            // Convert the error to a string to make it Send
            result.map_err(|e| e.to_string())
        });

        // Spawn scanner: discover new markdown files while crawler runs
        let temp_dir_clone = temp_dir.clone();
        let scanner_tx = tx.clone();
        let pc = pages_counter.clone();
        let crawl_done_scanner = crawl_done.clone();
        let scanner_handle = tokio::spawn(async move {
            // Give docrawl a head start before we start scanning
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Start with longer interval during active crawling, speed up when crawl is done
            let mut scan_interval_ms = 1000; // Start with 1 second interval to reduce overhead
            let mut ticker = interval(Duration::from_millis(scan_interval_ms));
            let mut seen: HashSet<PathBuf> = HashSet::new();
            let mut idle_ticks = 0u32;
            let mut total_files_scanned;
            // Reduced verbosity - only show important messages
            log::debug!(
                "Scanner: Starting to monitor directory: {}",
                temp_dir_clone.display()
            );

            loop {
                ticker.tick().await;
                let mut new_found = 0usize;
                let mut current_scan_count = 0usize;

                for entry in WalkDir::new(&temp_dir_clone)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    current_scan_count += 1;

                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if ext == "md" {
                                let pb = path.to_path_buf();
                                if !seen.contains(&pb) {
                                    log::debug!(
                                        "Scanner: Found new markdown file: {}",
                                        pb.file_name().unwrap_or_default().to_string_lossy()
                                    );
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
                    }
                }

                total_files_scanned = current_scan_count;

                if new_found > 0 {
                    log::debug!(
                        "Scanner: Found {} new markdown files this scan (total: {})",
                        new_found,
                        seen.len()
                    );
                    idle_ticks = 0;
                    // Speed up scanning when we're finding files
                    if scan_interval_ms > 300 {
                        scan_interval_ms = 300;
                        ticker = interval(Duration::from_millis(scan_interval_ms));
                    }
                } else {
                    idle_ticks += 1;

                    // If crawl is done and we're idle, speed up scanning to finish quickly
                    if crawl_done_scanner.load(Ordering::Relaxed) && scan_interval_ms > 200 {
                        scan_interval_ms = 200;
                        ticker = interval(Duration::from_millis(scan_interval_ms));
                    }

                    if idle_ticks.is_multiple_of(10) {
                        log::debug!("Scanner: No new files found for {} ticks (scanned {} total files in directory)", idle_ticks, total_files_scanned);
                    }
                }

                if seen.len() >= crawl_max_pages {
                    eprintln!("Scanner: Reached max pages limit ({})", crawl_max_pages);
                    break;
                }

                // Only exit on idle if crawl is done
                if idle_ticks > 20 && crawl_done_scanner.load(Ordering::Relaxed) {
                    log::debug!(
                        "Scanner: Crawl is done and no new files for {} ticks, exiting",
                        idle_ticks
                    );
                    break;
                }

                // If crawl is still running but we've been idle for a long time, keep waiting
                if idle_ticks > 100 {
                    log::debug!(
                        "Scanner: Safety exit after {} ticks of no activity",
                        idle_ticks
                    );
                    break;
                }
            }
            let files_found = seen.len();
            // Final count will be shown by the main thread
            drop(scanner_tx);
            files_found
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
            let processed_pages_counter = processed_pages_counter.clone();
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
                            // count this page as processed after storing its chunks
                            processed_pages_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                stored
            });
            joins.push(join);
        }

        // Wait for crawl to complete first
        let crawl_result = crawl_handle.await;
        let crawled_pages = if let Ok(Ok(stats)) = &crawl_result {
            eprintln!("\nCrawl completed: {} pages crawled", stats.pages);
            stats.pages
        } else if let Ok(Err(e)) = &crawl_result {
            eprintln!("\nCrawl completed with error: {}", e);
            0
        } else {
            eprintln!("\nCrawl status unknown");
            0
        };

        // Monitor file discovery with a proper progress spinner
        eprintln!("\nScanning for markdown files...");

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(200));

        // Create a monitoring task that updates the progress bar
        let pages_counter_clone = pages_counter.clone();
        let crawl_done_clone = crawl_done.clone();
        let pb_clone = pb.clone();
        let start_time = std::time::Instant::now();
        let monitor_handle = tokio::spawn(async move {
            let mut last_count = 0usize;
            let mut stable_cycles = 0;

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let current_count = pages_counter_clone.load(Ordering::Relaxed);
                let is_crawl_done = crawl_done_clone.load(Ordering::Relaxed);
                let elapsed = start_time.elapsed().as_secs_f32().max(0.1);
                let rate = (current_count as f32) / elapsed;

                let message = if current_count != last_count {
                    last_count = current_count;
                    stable_cycles = 0;
                    format!("Found {} files | {:.1}/s", current_count, rate)
                } else {
                    stable_cycles += 1;
                    if stable_cycles < 4 {
                        format!(
                            "Found {} files (scanning...) | {:.1}/s",
                            current_count, rate
                        )
                    } else {
                        format!(
                            "Found {} files (finalizing...) | {:.1}/s",
                            current_count, rate
                        )
                    }
                };

                pb_clone.set_message(message);

                // Stop monitoring once crawl is done and we've been stable for a bit
                if is_crawl_done && stable_cycles > 6 {
                    break;
                }
                // Safety exit after 30 seconds
                if stable_cycles > 60 {
                    break;
                }
            }
        });

        // Wait for scanner to finish
        let scanner_result = scanner_handle.await;
        let _scanner_files = scanner_result.unwrap_or(0);

        // Stop the monitor and finish the progress bar
        monitor_handle.abort();
        let final_count = pages_counter.load(Ordering::Relaxed);
        pb.finish_with_message(format!("Found {} markdown files", final_count));

        drop(tx);

        // If scanner already printed the count, we don't need to print it again
        // but we'll use the value for logic below

        // Get final count for processing phase
        let total_pages_found = pages_counter.load(Ordering::Relaxed);

        // Show status for chunking phase
        let mut processed_so_far = processed_pages_counter.load(Ordering::Relaxed);

        // If we have pages to process, show chunking progress with a progress bar
        if total_pages_found > 0 {
            eprintln!("\nProcessing {} markdown files...", total_pages_found);

            // Create a progress bar for chunking
            let pb = ProgressBar::new(total_pages_found as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) | {msg}")
                    .unwrap()
                    .progress_chars("█▉▊▋▌▍▎▏  ")
            );
            pb.set_message("Chunking... 0 chunks created".to_string());

            // Monitor chunking progress
            let mut last_processed = processed_so_far;
            let mut stall_counter = 0;
            while processed_so_far < total_pages_found && stall_counter < 60 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                processed_so_far = processed_pages_counter.load(Ordering::Relaxed);
                let chunks_so_far = chunks_counter.load(Ordering::Relaxed);

                if processed_so_far != last_processed {
                    pb.set_position(processed_so_far as u64);
                    pb.set_message(format!("{} chunks created", chunks_so_far));
                    last_processed = processed_so_far;
                    stall_counter = 0;
                } else {
                    stall_counter += 1;
                    // Update spinner even when stalled to show activity
                    pb.tick();
                }
            }

            // Ensure we show completion
            pb.set_position(processed_so_far as u64);

            if processed_so_far == total_pages_found {
                pb.finish_with_message("All files processed");
            } else {
                pb.abandon_with_message("Processing incomplete - some files may have failed");
            }
        } else {
            eprintln!("\nNo markdown files found to process");
            eprintln!(
                "   The crawler processed {} pages but docrawl generated no markdown files.",
                crawled_pages
            );
            eprintln!("   This can happen when:");
            eprintln!("   • The site uses JavaScript rendering that docrawl can't parse");
            eprintln!("   • The pages contain mostly non-text content (images, PDFs, etc.)");
            eprintln!("   • The site structure isn't compatible with the crawler");
            // Extra newline intentionally removed to satisfy clippy
            eprintln!("   Try:");
            eprintln!("   • Using a different URL that points to documentation pages");
            eprintln!("   • Indexing local files instead if you have them downloaded");
        }

        // Wait for all workers to complete
        let mut total_stored = 0usize;

        if total_pages_found > 0 {
            // Only show spinner if we had files to process
            eprintln!("\nWaiting for workers to finish...");
            let pb_final = ProgressBar::new_spinner();
            pb_final.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            pb_final.set_message("Finalizing embeddings and storing to database...");
            pb_final.enable_steady_tick(std::time::Duration::from_millis(100));

            for j in joins {
                pb_final.tick();
                if let Ok(count) = j.await {
                    total_stored += count;
                }
            }

            pb_final.finish_with_message("Index finalized");
        } else {
            // Just wait for workers without showing spinner
            for j in joins {
                if let Ok(count) = j.await {
                    total_stored += count;
                }
            }
        }

        // Clean up temp directory (silently)
        let _ = std::fs::remove_dir_all(&temp_dir);

        // Final summary (always show, even if no files found)
        let total_pages = pages_counter.load(Ordering::Relaxed);
        let final_processed = processed_pages_counter.load(Ordering::Relaxed);
        let final_chunks = chunks_counter.load(Ordering::Relaxed);
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();

        eprintln!();
        eprintln!("==== Manx Index Summary ====");
        eprintln!("Markdown files found: {}", total_pages);
        eprintln!("Files processed: {}", final_processed);
        eprintln!("Chunks created: {}", final_chunks);
        eprintln!("Chunks stored: {}", total_stored);
        eprintln!("Index path: {}", index_path.display());

        if total_pages == 0 {
            eprintln!();
            eprintln!("No markdown files were found. Docrawl may not have generated any content.");
            eprintln!("   This could mean the site structure is not compatible with crawling.");
        } else if total_stored == 0 {
            eprintln!();
            eprintln!("No chunks were stored. The markdown files may have been empty.");
        }

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
        log::info!("Embedding model loaded successfully");

        // Check if index directory exists and is accessible
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();

        if index_path.exists() {
            log::info!("Local index directory exists: {:?}", index_path);

            // Check embeddings directory
            let embedding_dir = index_path.join("embeddings");
            if embedding_dir.exists() {
                // Count existing embeddings
                match std::fs::read_dir(&embedding_dir) {
                    Ok(entries) => {
                        let count = entries.filter_map(|e| e.ok()).count();
                        log::info!(
                            "Local vector storage accessible with {} embedding files",
                            count
                        );
                    }
                    Err(e) => {
                        log::warn!(
                            "Local vector storage directory exists but cannot read contents: {}",
                            e
                        );
                    }
                }
            } else {
                log::info!("Local vector storage will be created when needed");
            }
        } else {
            log::info!("Local index directory will be created: {:?}", index_path);
        }

        // Test file system write access
        let test_file = index_path.join(".health_check");
        match std::fs::create_dir_all(index_path) {
            Ok(_) => {
                match std::fs::write(&test_file, "health_check") {
                    Ok(_) => {
                        log::info!("File system write access confirmed");
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
