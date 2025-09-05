//! Local RAG (Retrieval-Augmented Generation) system for Manx
//!
//! Provides document indexing, semantic search, and LLM integration
//! for enhanced documentation discovery and AI synthesis.

use crate::rag::embeddings::EmbeddingModel;
use crate::rag::indexer::Indexer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod embeddings;
pub mod indexer;
pub mod llm;

/// Configuration for the RAG system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub enabled: bool,
    pub index_path: PathBuf,
    pub max_results: usize,
    pub similarity_threshold: f32,
    pub allow_pdf_processing: bool,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            index_path: PathBuf::from("~/.cache/manx/rag_index"),
            max_results: 10,
            similarity_threshold: 0.6,
            allow_pdf_processing: false, // Disabled by default for security
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
struct StoredChunk {
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
    #[allow(dead_code)]
    config: RagConfig,
}

impl RagSystem {
    pub async fn new(config: RagConfig) -> Result<Self> {
        if !config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        // Initialize the local vector storage system
        let indexer = Indexer::new(&config)?;
        let index_path = indexer.get_index_path();

        // Create index directory if it doesn't exist
        std::fs::create_dir_all(&index_path)?;

        log::info!(
            "RAG system initialized with local vector storage at {:?}",
            index_path
        );
        Ok(Self { config })
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

    pub async fn search(
        &self,
        query: &str,
        max_results: Option<usize>,
    ) -> Result<Vec<RagSearchResult>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("RAG system is disabled"));
        }

        let limit = max_results.unwrap_or(10);

        log::info!("Searching local vector storage for: '{}'", query);

        // Load the embedding model for semantic search
        let embedding_model = EmbeddingModel::new().await?;
        let query_embedding = embedding_model.embed_text(query).await?;

        // Search through stored embeddings
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();

        if !index_path.exists() {
            log::info!("No indexed documents found yet");
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        let embedding_dir = index_path.join("embeddings");

        if !embedding_dir.exists() {
            log::info!("No embeddings directory found yet");
            return Ok(vec![]);
        }

        // Read all embedding files
        let entries = std::fs::read_dir(embedding_dir)?;

        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    match self
                        .load_and_score_embedding(&entry.path(), &query_embedding, &embedding_model)
                        .await
                    {
                        Ok(Some(result)) => {
                            if result.score >= self.config.similarity_threshold {
                                results.push(result);
                            }
                        }
                        Ok(None) => continue,
                        Err(e) => {
                            log::warn!(
                                "Failed to process embedding file {:?}: {}",
                                entry.path(),
                                e
                            );
                            continue;
                        }
                    }
                }
            }
        }

        // Sort by similarity score (highest first)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        log::info!("Found {} results from local vector storage", results.len());
        Ok(results)
    }

    /// Load and score an embedding file against the query
    async fn load_and_score_embedding(
        &self,
        embedding_path: &std::path::Path,
        query_embedding: &[f32],
        _embedding_model: &EmbeddingModel,
    ) -> Result<Option<RagSearchResult>> {
        // Read the stored chunk data
        let content = std::fs::read_to_string(embedding_path)?;
        let chunk_data: StoredChunk = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse chunk data: {}", e))?;

        // Calculate similarity score
        let score = EmbeddingModel::cosine_similarity(query_embedding, &chunk_data.embedding);

        // Convert to RagSearchResult
        Ok(Some(RagSearchResult {
            id: chunk_data.id,
            content: chunk_data.content,
            source_path: chunk_data.source_path,
            source_type: chunk_data.source_type,
            title: chunk_data.title,
            section: chunk_data.section,
            score,
            metadata: chunk_data.metadata,
        }))
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

        let last_updated =
            chrono::DateTime::<chrono::Utc>::from(std::time::SystemTime::from(last_modified));

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
        let _embedding_model = EmbeddingModel::new()
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
        match std::fs::create_dir_all(&index_path) {
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
        let embedding_model = EmbeddingModel::new().await?;

        // Get index path and create embeddings directory
        let indexer = Indexer::new(&self.config)?;
        let index_path = indexer.get_index_path();
        let embedding_dir = index_path.join("embeddings");

        // Create directories if they don't exist
        std::fs::create_dir_all(&embedding_dir)?;

        // Process chunks and store with embeddings
        let mut stored_count = 0;

        for chunk in chunks {
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
        }

        log::info!(
            "Successfully stored {} chunks in local vector storage",
            stored_count
        );
        Ok(())
    }
}
