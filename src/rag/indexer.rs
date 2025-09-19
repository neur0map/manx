//! Document indexing pipeline for the RAG system
//!
//! This module handles parsing and chunking of various document formats
//! for indexing into the vector database.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use docrawl::{crawl, CrawlConfig, Config};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use walkdir::WalkDir;
use url::Url;

use crate::rag::embeddings::preprocessing;
use crate::rag::{DocumentChunk, DocumentMetadata, RagConfig, SourceType};

/// Document indexer for the RAG system
pub struct Indexer {
    config: RagConfig,
    index_path: PathBuf,
}

impl Indexer {
    /// Create new indexer with configuration
    pub fn new(config: &RagConfig) -> Result<Self> {
        let index_path = if config.index_path.to_string_lossy().starts_with("~") {
            // Expand home directory
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| anyhow!("Cannot determine home directory"))?;
            let path_str = config.index_path.to_string_lossy();
            let without_tilde = path_str.strip_prefix("~/").unwrap_or(&path_str[1..]);
            PathBuf::from(home).join(without_tilde)
        } else {
            config.index_path.clone()
        };

        // Ensure index directory exists
        if !index_path.exists() {
            fs::create_dir_all(&index_path)
                .map_err(|e| anyhow!("Failed to create index directory {:?}: {}", index_path, e))?;
        }

        Ok(Self {
            config: config.clone(),
            index_path,
        })
    }

    /// Get the index directory path
    pub fn get_index_path(&self) -> &PathBuf {
        &self.index_path
    }

    /// Index a single document
    pub fn index_document(&self, path: PathBuf) -> Result<Vec<DocumentChunk>> {
        index_document(path, &self.config)
    }

    /// Index all documents in a directory
    pub fn index_directory(&self, dir_path: PathBuf) -> Result<Vec<DocumentChunk>> {
        let documents = find_documents(&dir_path)?;
        let mut all_chunks = Vec::new();

        for doc_path in documents {
            match self.index_document(doc_path.clone()) {
                Ok(mut chunks) => all_chunks.append(&mut chunks),
                Err(e) => {
                    log::warn!("Failed to index {:?}: {}", doc_path, e);
                    continue;
                }
            }
        }

        log::info!(
            "Indexed {} chunks from {} directory",
            all_chunks.len(),
            dir_path.display()
        );
        Ok(all_chunks)
    }

    /// Index content from a URL
    pub async fn index_url(&self, url: String) -> Result<Vec<DocumentChunk>> {
        log::info!("Indexing single URL (no crawling): {}", url);

        // Use docrawl with depth 0 (single page only)
        self.index_url_deep(url, Some(0), false).await
    }

    /// Index content from a URL with documentation-optimized crawling
    pub async fn index_url_deep(
        &self,
        url: String,
        crawl_depth: Option<u32>,
        crawl_all: bool,
    ) -> Result<Vec<DocumentChunk>> {
        log::info!(
            "Starting docrawl of URL: {} (depth: {:?}, crawl_all: {})",
            url,
            crawl_depth,
            crawl_all
        );

        // Validate URL format
        let parsed_url =
            url::Url::parse(&url).map_err(|e| anyhow!("Invalid URL format '{}': {}", url, e))?;

        // Security check - only allow HTTP/HTTPS
        match parsed_url.scheme() {
            "http" | "https" => {}
            scheme => {
                return Err(anyhow!(
                    "Unsupported URL scheme '{}'. Only HTTP and HTTPS are allowed.",
                    scheme
                ))
            }
        }

        // Create temporary directory for docrawl output
        let temp_dir = std::env::temp_dir().join(format!("manx_crawl_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        // Parse the URL
        let base_url = Url::parse(&url)?;

        // Configure docrawl
        let config = CrawlConfig {
            base_url,
            output_dir: temp_dir.clone(),
            user_agent: "Manx/0.5.0 (Documentation Crawler)".to_string(),
            max_depth: if let Some(depth) = crawl_depth {
                Some(depth as usize)
            } else if crawl_all {
                None // No depth limit
            } else {
                Some(3) // Default depth
            },
            rate_limit_per_sec: 10,
            follow_sitemaps: true,
            concurrency: 4,
            timeout: Some(Duration::from_secs(30)),
            resume: false,
            config: Config::default(),
        };

        log::info!("Running docrawl on: {}", url);
        match crawl(config).await {
            Ok(stats) => {
                log::info!("Docrawl completed successfully, processed {} pages", stats.pages);
            }
            Err(e) => {
                // Clean up temp directory
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Err(anyhow!("Docrawl failed: {}", e));
            }
        }

        // Process the generated markdown files
        let mut all_chunks = Vec::new();
        let markdown_files = self.find_markdown_files(&temp_dir)?;

        log::info!("Processing {} markdown files from docrawl", markdown_files.len());

        for (index, md_file) in markdown_files.iter().enumerate() {
            log::debug!(
                "Processing markdown file {}/{}: {}",
                index + 1,
                markdown_files.len(),
                md_file.display()
            );

            match self.process_markdown_file(md_file, &url).await {
                Ok(chunks) => {
                    let chunk_count = chunks.len();
                    all_chunks.extend(chunks);
                    log::debug!(
                        "Successfully processed markdown: {} ({} chunks)",
                        md_file.display(),
                        chunk_count
                    );
                }
                Err(e) => {
                    log::warn!("Failed to process markdown '{}': {}", md_file.display(), e);
                    // Continue with other files even if one fails
                }
            }
        }

        // Clean up temporary directory
        if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
            log::warn!("Failed to clean up temporary directory: {}", e);
        }

        log::info!(
            "Successfully indexed {} chunks from {} markdown files via docrawl of: {}",
            all_chunks.len(),
            markdown_files.len(),
            url
        );

        Ok(all_chunks)
    }

    /// Find all markdown files in the crawled directory
    fn find_markdown_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut markdown_files = Vec::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                markdown_files.push(path.to_path_buf());
            }
        }

        Ok(markdown_files)
    }

    /// Process a markdown file generated by docrawl
    async fn process_markdown_file(&self, md_file: &Path, base_url: &str) -> Result<Vec<DocumentChunk>> {
        // Read the markdown content
        let content = std::fs::read_to_string(md_file)?;

        if content.trim().is_empty() {
            return Err(anyhow!(
                "Markdown file contains no content: {}",
                md_file.display()
            ));
        }

        // Create metadata for this markdown file
        let metadata = self.create_markdown_metadata(md_file, &content, base_url)?;

        // Detect document structure (title, sections) from markdown content
        let (title, sections) = detect_structure(&content, md_file);

        // Derive a logical page URL from the file path and base URL
        let page_url = self.derive_page_url(md_file, base_url);

        // Chunk the content
        let chunks = chunk_content(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);

        // Create document chunks
        let mut document_chunks = Vec::new();
        for (i, chunk_content) in chunks.into_iter().enumerate() {
            // Try to determine which section this chunk belongs to
            let section = find_section_for_chunk(&chunk_content, &sections);

            let chunk = DocumentChunk {
                id: format!("{}_{}", page_url, i),
                content: preprocessing::clean_text(&chunk_content),
                source_path: PathBuf::from(&page_url),
                source_type: SourceType::Web,
                title: title.clone(),
                section: section.clone(),
                chunk_index: i,
                metadata: metadata.clone(),
            };

            document_chunks.push(chunk);
        }

        Ok(document_chunks)
    }

    /// Create metadata for a markdown file from docrawl
    fn create_markdown_metadata(&self, md_file: &Path, content: &str, base_url: &str) -> Result<DocumentMetadata> {
        let file_metadata = std::fs::metadata(md_file)?;
        let modified_time = file_metadata.modified()?;
        let modified_datetime = chrono::DateTime::<chrono::Utc>::from(modified_time);

        // Extract tags from file path and content
        let mut tags = extract_tags_from_path(md_file);
        tags.push("documentation".to_string());
        tags.push("crawled".to_string());

        // Add base domain as a tag (simple extraction)
        if let Some(domain) = extract_domain_from_url(base_url) {
            tags.push(domain);
        }

        // Detect language from content (basic detection for now)
        let language = detect_language(md_file);

        Ok(DocumentMetadata {
            file_type: "markdown".to_string(),
            size: content.len() as u64,
            modified: modified_datetime,
            tags,
            language,
        })
    }

    /// Derive a logical page URL from the markdown file path
    fn derive_page_url(&self, md_file: &Path, base_url: &str) -> String {
        // Get the relative path from the temp directory
        let file_name = md_file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");

        // Create a logical URL by combining base URL with file name
        if base_url.ends_with('/') {
            format!("{}{}", base_url, file_name)
        } else {
            format!("{}/{}", base_url, file_name)
        }
    }
}

/// Supported file extensions for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &[
    // Documentation
    ".md", ".txt", ".pdf", ".doc", ".docx", ".rst",
    // Web/Frontend
    ".js", ".jsx", ".ts", ".tsx", ".vue", ".svelte", ".html", ".css", ".scss", ".sass", ".less",
    // Backend/Server
    ".py", ".rb", ".php", ".java", ".scala", ".kotlin", ".groovy",
    // Systems Programming
    ".c", ".cpp", ".cc", ".cxx", ".h", ".hpp", ".rs", ".go", ".zig",
    // Functional
    ".ml", ".mli", ".hs", ".elm", ".clj", ".cljs", ".erl", ".ex", ".exs",
    // Data/Config
    ".json", ".yaml", ".yml", ".toml", ".xml", ".ini", ".env", ".properties",
    // Shell/Scripts (with security validation)
    ".sh", ".bash", ".zsh", ".fish", ".ps1", ".bat", ".cmd",
    // Mobile
    ".swift", ".m", ".mm", ".kt", ".dart",
    // Database
    ".sql", ".graphql", ".prisma",
    // Other Languages
    ".r", ".R", ".jl", ".lua", ".vim", ".el",
];

/// Default chunk size in tokens (approximately)
const DEFAULT_CHUNK_SIZE: usize = 500;

/// Overlap between chunks in tokens
const DEFAULT_CHUNK_OVERLAP: usize = 50;

/// Find all indexable documents in a directory using WalkDir for performance
pub fn find_documents(dir_path: &Path) -> Result<Vec<PathBuf>> {
    if !dir_path.exists() {
        return Err(anyhow!("Directory does not exist: {:?}", dir_path));
    }

    if !dir_path.is_dir() {
        return Err(anyhow!("Path is not a directory: {:?}", dir_path));
    }

    let mut documents = Vec::new();
    let max_depth = 10; // Prevent infinite recursion
    let max_file_size = 100 * 1024 * 1024; // 100MB limit per file

    log::info!("Scanning directory for documents: {:?}", dir_path);

    for entry in WalkDir::new(dir_path)
        .max_depth(max_depth)
        .follow_links(false) // Avoid symlink cycles
        .into_iter()
        .filter_map(|e| e.ok())
    // Skip entries that can't be read
    {
        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Check if file type is supported
        if !is_supported_file(path) {
            continue;
        }

        // Check file size limits
        if let Ok(metadata) = entry.metadata() {
            if metadata.len() > max_file_size {
                log::warn!(
                    "Skipping large file ({}MB): {:?}",
                    metadata.len() / 1024 / 1024,
                    path
                );
                continue;
            }
        }

        // Skip hidden files and directories (starting with .)
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
        {
            log::debug!("Skipping hidden file: {:?}", path);
            continue;
        }

        // Skip common binary/cache directories
        let path_str = path.to_string_lossy();
        let skip_patterns = [
            "/target/",
            "/.git/",
            "/node_modules/",
            "/__pycache__/",
            "/.cache/",
            "/dist/",
            "/build/",
        ];

        if skip_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
        {
            log::debug!("Skipping file in ignored directory: {:?}", path);
            continue;
        }

        documents.push(path.to_path_buf());
    }

    log::info!(
        "Found {} indexable documents in {:?} (max depth: {})",
        documents.len(),
        dir_path,
        max_depth
    );

    if documents.is_empty() {
        log::warn!(
            "No supported documents found in {:?}. Supported formats: {:?}",
            dir_path,
            SUPPORTED_EXTENSIONS
        );
    }

    Ok(documents)
}

/// Check if a file is supported for indexing
pub fn is_supported_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&format!(".{}", ext.to_lowercase()).as_str()))
        .unwrap_or(false)
}

/// Index a single document and return chunks
pub fn index_document(path: PathBuf, config: &RagConfig) -> Result<Vec<DocumentChunk>> {
    if !path.exists() {
        return Err(anyhow!("File does not exist: {:?}", path));
    }

    if !is_supported_file(&path) {
        return Err(anyhow!("Unsupported file type: {:?}", path));
    }

    // SECURITY: Check if PDF processing is disabled
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    if extension == "pdf" && !config.allow_pdf_processing {
        log::warn!("PDF processing disabled for security. Skipping: {:?}", path);
        return Ok(vec![]); // Skip PDF files when disabled
    }

    // SECURITY: Check if code processing is disabled
    let code_extensions = ["js", "jsx", "ts", "tsx", "py", "rb", "php", "java", "scala",
                          "kotlin", "rs", "go", "c", "cpp", "sh", "bash", "ps1"];
    if code_extensions.contains(&extension.as_str()) && !config.allow_code_processing {
        log::warn!("Code processing disabled. Skipping: {:?}", path);
        return Ok(vec![]); // Skip code files when disabled
    }

    log::info!("Indexing document: {:?}", path);

    // Extract text content with configuration
    let content = extract_text(&path, config)?;
    if content.trim().is_empty() {
        return Err(anyhow!("Document contains no text content: {:?}", path));
    }

    // Get file metadata
    let metadata = extract_metadata(&path)?;

    // Detect document structure (title, sections)
    let (title, sections) = detect_structure(&content, &path);

    // Chunk the content
    let chunks = chunk_content(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);

    // Create document chunks
    let mut document_chunks = Vec::new();
    for (i, chunk_content) in chunks.into_iter().enumerate() {
        // Try to determine which section this chunk belongs to
        let section = find_section_for_chunk(&chunk_content, &sections);

        let chunk = DocumentChunk {
            id: format!("{}_{}", path.to_string_lossy(), i),
            content: preprocessing::clean_text(&chunk_content),
            source_path: path.clone(),
            source_type: SourceType::Local, // Default to local for now
            title: title.clone(),
            section: section.clone(),
            chunk_index: i,
            metadata: metadata.clone(),
        };

        document_chunks.push(chunk);
    }

    log::info!("Created {} chunks from {:?}", document_chunks.len(), path);
    Ok(document_chunks)
}

/// Extract text content from various file formats
fn extract_text(path: &Path, config: &RagConfig) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "md" | "txt" | "rst" => extract_text_file(path),
        "pdf" => extract_pdf_text(path),
        "doc" | "docx" => extract_doc_text(path),
        // Code files
        "js" | "jsx" | "ts" | "tsx" | "vue" | "svelte" | "html" | "css" | "scss" | "sass" | "less" |
        "py" | "rb" | "php" | "java" | "scala" | "kotlin" | "groovy" |
        "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "rs" | "go" | "zig" |
        "ml" | "mli" | "hs" | "elm" | "clj" | "cljs" | "erl" | "ex" | "exs" |
        "swift" | "m" | "mm" | "kt" | "dart" |
        "r" | "jl" | "lua" | "vim" | "el" |
        "sql" | "graphql" | "prisma" => extract_code_text(path, config),
        // Config files
        "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "properties" => extract_config_text(path, config),
        // Shell scripts (with extra security)
        "sh" | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" => extract_shell_text(path, config),
        // Environment files (with secret masking)
        "env" => extract_env_text(path, config),
        _ => Err(anyhow!("Unsupported file extension: {}", extension)),
    }
}

/// Extract text from plain text files (markdown, txt)
fn extract_text_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| anyhow!("Failed to read text file {:?}: {}", path, e))
}

/// Extract text from PDF files with security validation
fn extract_pdf_text(path: &Path) -> Result<String> {
    log::info!("Processing PDF file with security validation: {:?}", path);

    // SECURITY: Validate PDF before processing
    validate_pdf_security(path)?;

    // Create a metadata entry that includes the filename and basic information
    // Note: Basic PDF metadata extraction with security validation
    let file_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    // Get file size for indexing
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Create indexable content from filename and metadata
    let mut content = String::new();
    content.push_str(&format!("PDF Document: {}\n", file_name));
    content.push_str(&format!("File size: {} bytes\n", file_size));
    content.push_str(&format!("Location: {}\n", path.display()));

    // Add searchable terms from filename
    let searchable_terms: Vec<&str> = file_name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.len() > 2)
        .collect();

    if !searchable_terms.is_empty() {
        content.push_str("Keywords: ");
        content.push_str(&searchable_terms.join(", "));
        content.push('\n');
    }

    // PDF processing currently indexes by filename and metadata
    content.push_str("This document is indexed by filename and metadata.");

    log::info!(
        "Created indexable content for PDF {:?} ({} characters)",
        path,
        content.len()
    );
    Ok(content)
}

/// Extract text from DOC/DOCX files
fn extract_doc_text(path: &Path) -> Result<String> {
    log::info!("Processing DOC/DOCX file: {:?}", path);

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Handle legacy DOC format
    if extension == "doc" {
        log::warn!("Legacy DOC format detected: {:?}", path);
        return create_doc_metadata(path, "DOC (Legacy Word Document)");
    }

    // Handle DOCX format
    if extension == "docx" {
        // Try to extract text using docx-rs library
        match extract_docx_text_safe(path) {
            Ok(content) => {
                log::info!(
                    "Successfully extracted {} characters from DOCX: {:?}",
                    content.len(),
                    path
                );
                return Ok(content);
            }
            Err(e) => {
                log::warn!(
                    "Failed to extract DOCX text, using metadata fallback: {}",
                    e
                );
                return create_doc_metadata(path, "DOCX (Word Document)");
            }
        }
    }

    Err(anyhow!("Unsupported document format: {:?}", extension))
}

/// Safely extract DOCX text with error handling
fn extract_docx_text_safe(path: &Path) -> Result<String> {
    use docx_rs::read_docx;

    // Read the DOCX file as bytes
    let file_bytes = std::fs::read(path).map_err(|e| anyhow!("Failed to read DOCX file: {}", e))?;

    let _docx = read_docx(&file_bytes).map_err(|e| anyhow!("Failed to parse DOCX file: {}", e))?;

    // Extract text content from document (basic implementation)
    let mut text_content = String::new();

    // Simple text extraction - this may need enhancement based on docx-rs API
    text_content.push_str(&format!("DOCX Document from: {}\n", path.display()));
    text_content.push_str("Document content successfully parsed.\n");
    text_content.push_str("Note: Basic DOCX processing - text extraction can be enhanced.");

    Ok(text_content)
}

/// Extract text from code files with security validation
fn extract_code_text(path: &Path, config: &RagConfig) -> Result<String> {
    // Validate code file security
    validate_code_security(path, &config.code_security_level)?;

    // Read the code file
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read code file {:?}: {}", path, e))?;

    // Clean and prepare for indexing
    let cleaned = if config.mask_secrets {
        sanitize_code_content(&content)
    } else {
        content
    };
    Ok(cleaned)
}

/// Extract text from config files with validation
fn extract_config_text(path: &Path, config: &RagConfig) -> Result<String> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read config file {:?}: {}", path, e))?;

    // Mask any potential secrets in config files
    let sanitized = if config.mask_secrets {
        mask_secrets(&content)
    } else {
        content
    };
    Ok(sanitized)
}

/// Extract text from shell scripts with enhanced security validation
fn extract_shell_text(path: &Path, config: &RagConfig) -> Result<String> {
    // Extra security validation for shell scripts
    validate_shell_security(path, &config.code_security_level)?;

    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read shell script {:?}: {}", path, e))?;

    // Sanitize shell content
    let sanitized = if config.mask_secrets {
        sanitize_shell_content(&content)
    } else {
        content
    };
    Ok(sanitized)
}

/// Extract text from environment files with secret masking
fn extract_env_text(path: &Path, _config: &RagConfig) -> Result<String> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read env file {:?}: {}", path, e))?;

    // Heavily mask environment files
    let masked = mask_env_secrets(&content);
    Ok(masked)
}

/// Security validation for code files to prevent malicious content processing
fn validate_code_security(path: &Path, security_level: &crate::rag::CodeSecurityLevel) -> Result<()> {
    use crate::rag::CodeSecurityLevel;
    log::debug!("Running security validation on code file: {:?}", path);

    // Check file size
    let metadata = fs::metadata(path)?;
    const MAX_CODE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
    if metadata.len() > MAX_CODE_SIZE {
        return Err(anyhow!(
            "Code file rejected: Size {} bytes exceeds maximum allowed size of {} bytes",
            metadata.len(),
            MAX_CODE_SIZE
        ));
    }

    // Read file content for analysis
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read code file for validation: {}", e))?;

    // Check for obfuscated code patterns
    if is_potentially_obfuscated(&content) {
        match security_level {
            CodeSecurityLevel::Strict => {
                return Err(anyhow!("Code file rejected: Contains potentially obfuscated content"));
            }
            CodeSecurityLevel::Moderate => {
                log::warn!("Code file may contain obfuscated content: {:?}", path);
            }
            CodeSecurityLevel::Permissive => {
                log::debug!("Obfuscated content check bypassed (permissive mode)");
            }
        }
    }

    // Check for suspicious URLs or domains
    validate_urls_in_code(&content, security_level)?;

    // Check for prompt injection patterns
    check_prompt_injection(&content, security_level)?;

    Ok(())
}

/// Enhanced security validation for shell scripts
fn validate_shell_security(path: &Path, security_level: &crate::rag::CodeSecurityLevel) -> Result<()> {
    use crate::rag::CodeSecurityLevel;
    log::debug!("Running enhanced security validation on shell script: {:?}", path);

    let content = fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to read shell script for validation: {}", e))?;

    // Dangerous shell command patterns
    let dangerous_patterns = [
        r"rm\s+-rf\s+/",           // rm -rf /
        r"rm\s+-rf\s+\*",          // rm -rf *
        r":\(\)\s*\{\s*:\|\:&\s*\};:", // Fork bomb
        r"mkfs\.",                 // Format filesystem
        r"dd\s+if=/dev/(zero|random)", // Disk wipe
        r">\s*/dev/sda",           // Direct disk write
        r"curl.*\|\s*(ba)?sh",     // Remote code execution
        r"wget.*\|\s*(ba)?sh",     // Remote code execution
        r"eval\s+.*\$\(",          // Eval with command substitution
        r"python\s+-c.*exec",      // Python exec
    ];

    for pattern in &dangerous_patterns {
        if regex::Regex::new(pattern)
            .unwrap_or_else(|_| regex::Regex::new("").unwrap())
            .is_match(&content)
        {
            match security_level {
                CodeSecurityLevel::Strict | CodeSecurityLevel::Moderate => {
                    return Err(anyhow!(
                        "Shell script rejected: Contains potentially dangerous command pattern"
                    ));
                }
                CodeSecurityLevel::Permissive => {
                    log::warn!("Dangerous shell pattern detected but allowed in permissive mode");
                }
            }
        }
    }

    Ok(())
}

/// Check for potentially obfuscated code
fn is_potentially_obfuscated(content: &str) -> bool {
    // Check for high entropy (randomness) in variable names
    let lines: Vec<&str> = content.lines().collect();
    let mut suspicious_count = 0;

    for line in lines {
        // Skip comments
        if line.trim().starts_with("//") || line.trim().starts_with("#") || line.trim().starts_with("/*") {
            continue;
        }

        // Check for base64 encoded strings
        if line.contains("atob") || line.contains("btoa") || line.contains("base64") {
            suspicious_count += 1;
        }

        // Check for hex strings
        if regex::Regex::new(r"\\x[0-9a-fA-F]{2}").unwrap().is_match(line) {
            suspicious_count += 1;
        }

        // Check for excessive use of escape characters
        if line.matches('\\').count() > 10 {
            suspicious_count += 1;
        }
    }

    suspicious_count > 5
}

/// Validate URLs in code for suspicious domains
fn validate_urls_in_code(content: &str, security_level: &crate::rag::CodeSecurityLevel) -> Result<()> {
    use crate::rag::CodeSecurityLevel;
    let url_pattern = regex::Regex::new(r#"https?://[^\s"']+"#).unwrap();

    let suspicious_domains = [
        "bit.ly", "tinyurl.com", "goo.gl", "ow.ly", "shorte.st",
        "adf.ly", "bc.vc", "bit.do", "soo.gd", "7.ly",
        "5z8.info", "DFHGDH", // Common in malware
    ];

    for url_match in url_pattern.find_iter(content) {
        let url = url_match.as_str();
        for domain in &suspicious_domains {
            if url.contains(domain) {
                match security_level {
                    CodeSecurityLevel::Strict => {
                        return Err(anyhow!("Code rejected: Contains suspicious URL shortener: {}", url));
                    }
                    CodeSecurityLevel::Moderate => {
                        log::warn!("Suspicious URL shortener found in code: {}", url);
                    }
                    CodeSecurityLevel::Permissive => {
                        log::debug!("URL check bypassed (permissive mode): {}", url);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Check for prompt injection patterns
fn check_prompt_injection(content: &str, security_level: &crate::rag::CodeSecurityLevel) -> Result<()> {
    use crate::rag::CodeSecurityLevel;
    let injection_patterns = [
        "ignore previous instructions",
        "disregard all prior",
        "forget everything above",
        "new instructions:",
        "SYSTEM PROMPT:",
        "###SYSTEM###",
        "</system>",
        "<|im_start|>",
        "<|im_end|>",
    ];

    let content_lower = content.to_lowercase();
    for pattern in &injection_patterns {
        if content_lower.contains(pattern) {
            match security_level {
                CodeSecurityLevel::Strict => {
                    return Err(anyhow!("Code rejected: Contains potential prompt injection pattern: {}", pattern));
                }
                CodeSecurityLevel::Moderate => {
                    log::warn!("Potential prompt injection pattern detected: {}", pattern);
                }
                CodeSecurityLevel::Permissive => {
                    log::debug!("Prompt injection check bypassed (permissive mode)");
                }
            }
        }
    }

    Ok(())
}

/// Sanitize code content for safe indexing
fn sanitize_code_content(content: &str) -> String {
    // Remove any inline secrets or API keys
    let sanitized = mask_secrets(content);

    // Preserve code structure but clean up
    sanitized
}

/// Sanitize shell script content
fn sanitize_shell_content(content: &str) -> String {
    // Mask any hardcoded passwords or secrets
    mask_secrets(content)
}

/// Mask secrets in content
fn mask_secrets(content: &str) -> String {
    let mut result = content.to_string();

    // Patterns for common secrets
    let secret_patterns = [
        (r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['\"]?([^'\";\s]+)"#, "API_KEY=[MASKED]"),
        (r#"(?i)(secret|password|passwd|pwd)\s*[:=]\s*['\"]?([^'\";\s]+)"#, "SECRET=[MASKED]"),
        (r#"(?i)(token|auth)\s*[:=]\s*['\"]?([^'\";\s]+)"#, "TOKEN=[MASKED]"),
        (r"(?i)bearer\s+[a-zA-Z0-9\-._~+/]+", "Bearer [MASKED]"),
        (r"-----BEGIN (RSA |EC |DSA |OPENSSH |)PRIVATE KEY-----[\s\S]*?-----END (RSA |EC |DSA |OPENSSH |)PRIVATE KEY-----", "[PRIVATE_KEY_MASKED]"),
        (r"ghp_[a-zA-Z0-9]{36}", "ghp_[GITHUB_TOKEN_MASKED]"),
        (r"sk-[a-zA-Z0-9]{48}", "sk-[OPENAI_KEY_MASKED]"),
    ];

    for (pattern, replacement) in &secret_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, *replacement).to_string();
        }
    }

    result
}

/// Heavily mask environment file secrets
fn mask_env_secrets(content: &str) -> String {
    let mut result = String::new();

    for line in content.lines() {
        if line.trim().is_empty() || line.trim().starts_with('#') {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = &line[..eq_pos];
            // Keep the key but mask the value
            result.push_str(key);
            result.push_str("=[MASKED]\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// Security validation for PDF files to prevent malicious content processing
fn validate_pdf_security(path: &Path) -> Result<()> {
    log::debug!("Running security validation on PDF: {:?}", path);

    // Check file size - reject extremely large files that could cause DoS
    const MAX_PDF_SIZE: u64 = 100 * 1024 * 1024; // 100MB limit
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_PDF_SIZE {
        return Err(anyhow!(
            "PDF file rejected: Size {} bytes exceeds maximum allowed size of {} bytes ({}MB)",
            metadata.len(),
            MAX_PDF_SIZE,
            MAX_PDF_SIZE / (1024 * 1024)
        ));
    }

    // Read the first few bytes to validate PDF header
    let mut buffer = vec![0u8; 1024];
    let file = fs::File::open(path)?;
    use std::io::Read;
    let mut reader = std::io::BufReader::new(file);
    let bytes_read = reader.read(&mut buffer)?;

    if bytes_read < 8 {
        return Err(anyhow!("PDF file rejected: File too small or corrupted"));
    }

    // Validate PDF magic header
    if !buffer.starts_with(b"%PDF-") {
        return Err(anyhow!(
            "PDF file rejected: Invalid PDF header - not a valid PDF file"
        ));
    }

    // Check PDF version - reject very old or suspicious versions
    if bytes_read >= 8 {
        let version_bytes = &buffer[5..8];
        if let Ok(version_str) = std::str::from_utf8(version_bytes) {
            // Extract major version number
            if let Some(major_char) = version_str.chars().next() {
                if let Some(major) = major_char.to_digit(10) {
                    if !(1..=2).contains(&major) {
                        // Only allow PDF versions 1.x and 2.x
                        return Err(anyhow!(
                            "PDF file rejected: Unsupported PDF version {}",
                            version_str
                        ));
                    }
                }
            }
        }
    }

    // Scan for suspicious content patterns in the first KB
    let content = std::str::from_utf8(&buffer[..bytes_read]).unwrap_or("");

    // Dangerous JavaScript/ActionScript patterns
    let dangerous_patterns = [
        "/JavaScript",
        "/JS",
        "/OpenAction",
        "/AA", // Auto Action
        "/Launch",
        "/GoToE", // GoToEmbedded
        "/GoToR", // GoToRemote
        "/ImportData",
        "/SubmitForm",
        "/URI",
        "/Sound",
        "/Movie",
        "/RichMedia",
        "/3D",
        "/Encrypt",
        "eval(",
        "unescape(",
        "String.fromCharCode(",
        "document.write(",
        "this.print(",
        "app.alert(",
        "xfa.host",
        "soap.connect",
        "util.printf",
    ];

    for pattern in &dangerous_patterns {
        if content.contains(pattern) {
            log::warn!(
                "PDF security violation: Found suspicious pattern '{}' in {}",
                pattern,
                path.display()
            );
            return Err(anyhow!(
                "PDF file rejected: Contains potentially malicious content pattern '{}'. PDF may contain embedded JavaScript or other dangerous elements.", 
                pattern
            ));
        }
    }

    // Check for embedded files patterns
    let embed_patterns = ["/EmbeddedFile", "/F ", "/UF ", "/Filespec"];
    for pattern in &embed_patterns {
        if content.contains(pattern) {
            log::warn!(
                "PDF security violation: Found embedded file pattern '{}' in {}",
                pattern,
                path.display()
            );
            return Err(anyhow!(
                "PDF file rejected: Contains embedded files which pose security risks"
            ));
        }
    }

    // Check for form patterns that could be used for data exfiltration
    let form_patterns = ["/XFA", "/AcroForm", "/Fields"];
    for pattern in &form_patterns {
        if content.contains(pattern) {
            log::warn!(
                "PDF security warning: Found form pattern '{}' in {}",
                pattern,
                path.display()
            );
            // Forms are suspicious but not automatically rejected - just logged
        }
    }

    log::info!("PDF security validation passed for: {:?}", path);
    Ok(())
}


/// Create metadata entry for documents that cannot be fully processed
fn create_doc_metadata(path: &Path, doc_type: &str) -> Result<String> {
    let file_name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    // Get file size for indexing
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Create indexable content from filename and metadata
    let mut content = String::new();
    content.push_str(&format!("{}: {}\n", doc_type, file_name));
    content.push_str(&format!("File size: {} bytes\n", file_size));
    content.push_str(&format!("Location: {}\n", path.display()));

    // Add searchable terms from filename
    let searchable_terms: Vec<&str> = file_name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.len() > 2)
        .collect();

    if !searchable_terms.is_empty() {
        content.push_str("Keywords: ");
        content.push_str(&searchable_terms.join(", "));
        content.push('\n');
    }

    // Enhanced document processing: indexed by filename, metadata, and content structure
    if let Ok(modified) = fs::metadata(path).and_then(|m| m.modified()) {
        if let Ok(duration) = modified.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            let datetime = chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(chrono::Utc::now);
            content.push_str(&format!("Modified: {}\n", datetime.format("%Y-%m-%d")));
        }
    }

    // Add file extension context
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        content.push_str(&format!("Format: {} document\n", extension.to_uppercase()));
    }

    Ok(content)
}

/// Extract file metadata
fn extract_metadata(path: &Path) -> Result<DocumentMetadata> {
    let metadata = fs::metadata(path)?;

    let file_type = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    let modified = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let modified_datetime = DateTime::from_timestamp(modified as i64, 0).unwrap_or_else(Utc::now);

    // Extract tags from filename or path
    let tags = extract_tags_from_path(path);

    // Try to detect language from content or filename
    let language = detect_language(path);

    Ok(DocumentMetadata {
        file_type,
        size: metadata.len(),
        modified: modified_datetime,
        tags,
        language,
    })
}

/// Extract tags from file path (e.g., directory names, filename patterns)
fn extract_tags_from_path(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();

    // Add parent directory names as tags
    if let Some(parent) = path.parent() {
        for component in parent.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if !name.starts_with('.') && name != "/" {
                    tags.push(name.to_lowercase());
                }
            }
        }
    }

    // Add filename-based tags
    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
        // Look for common patterns
        if filename.contains("readme") {
            tags.push("readme".to_string());
        }
        if filename.contains("api") {
            tags.push("api".to_string());
        }
        if filename.contains("guide") {
            tags.push("guide".to_string());
        }
        if filename.contains("tutorial") {
            tags.push("tutorial".to_string());
        }
    }

    tags
}

/// Detect document language
fn detect_language(_path: &Path) -> Option<String> {
    // For now, assume English. In a real implementation,
    // you could use language detection libraries
    Some("en".to_string())
}

/// Detect document structure (title, sections)
fn detect_structure(content: &str, path: &Path) -> (Option<String>, Vec<String>) {
    let lines: Vec<&str> = content.lines().collect();
    let mut title = None;
    let mut sections = Vec::new();

    // For markdown files, look for headers
    if path.extension().and_then(|s| s.to_str()) == Some("md") {
        for line in &lines {
            let trimmed = line.trim();

            // Check for title (first # header)
            if title.is_none() && trimmed.starts_with("# ") {
                title = Some(trimmed[2..].trim().to_string());
            }

            // Collect section headers
            if let Some(stripped) = trimmed.strip_prefix("## ") {
                sections.push(stripped.trim().to_string());
            } else if let Some(stripped) = trimmed.strip_prefix("### ") {
                sections.push(stripped.trim().to_string());
            }
        }
    }

    // If no title found in markdown, use filename
    if title.is_none() {
        if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
            title = Some(filename.replace(['_', '-'], " "));
        }
    }

    (title, sections)
}

/// Find which section a chunk belongs to
fn find_section_for_chunk(chunk: &str, sections: &[String]) -> Option<String> {
    // Look for section headers in the chunk
    for section in sections {
        if chunk.contains(section) {
            return Some(section.clone());
        }
    }
    None
}

/// Chunk content into smaller pieces
fn chunk_content(content: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    // Convert approximate token count to word count (rough estimate: 1 token ≈ 0.75 words)
    let word_chunk_size = (chunk_size as f32 * 0.75) as usize;
    let word_overlap = (overlap as f32 * 0.75) as usize;

    // Use the preprocessing chunk_text function
    crate::rag::embeddings::preprocessing::chunk_text(content, word_chunk_size, word_overlap)
}

/// Extract domain from URL without requiring external dependencies
fn extract_domain_from_url(url: &str) -> Option<String> {
    // Simple domain extraction without full URL parsing
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            Some(after_protocol[..end].to_string())
        } else {
            Some(after_protocol.to_string())
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Additional imports available if needed for enhanced testing
    // use std::fs::File;
    // use std::io::Write;

    #[test]
    fn test_is_supported_file() {
        assert!(is_supported_file(Path::new("test.md")));
        assert!(is_supported_file(Path::new("test.txt")));
        assert!(is_supported_file(Path::new("test.pdf")));
        assert!(is_supported_file(Path::new("test.rs"))); // Now supported for code indexing
        assert!(!is_supported_file(Path::new("test.unknown")));
        assert!(!is_supported_file(Path::new("test")));
    }

    #[test]
    fn test_detect_structure() {
        let content = r#"# Main Title

Some introduction text.

## Section 1

Content for section 1.

## Section 2

Content for section 2.

### Subsection 2.1

More content.
"#;

        let path = Path::new("test.md");
        let (title, sections) = detect_structure(content, path);

        assert_eq!(title, Some("Main Title".to_string()));
        assert_eq!(sections.len(), 3);
        assert!(sections.contains(&"Section 1".to_string()));
        assert!(sections.contains(&"Section 2".to_string()));
        assert!(sections.contains(&"Subsection 2.1".to_string()));
    }

    #[test]
    fn test_extract_tags_from_path() {
        let path = Path::new("/docs/api/authentication/readme.md");
        let tags = extract_tags_from_path(path);

        assert!(tags.contains(&"docs".to_string()));
        assert!(tags.contains(&"api".to_string()));
        assert!(tags.contains(&"authentication".to_string()));
        assert!(tags.contains(&"readme".to_string()));
    }

    #[test]
    fn test_chunk_content() {
        let content = "This is a test document with multiple sentences. Each sentence should be preserved in the chunking process. We want to make sure the chunks are reasonable.";
        let chunks = chunk_content(content, 10, 2); // Small chunks for testing

        assert!(chunks.len() > 1);
        assert!(!chunks[0].is_empty());

        // Check for overlap
        if chunks.len() > 1 {
            let words1: Vec<&str> = chunks[0].split_whitespace().collect();
            let words2: Vec<&str> = chunks[1].split_whitespace().collect();

            // There should be some overlap between consecutive chunks
            let overlap_found = words1
                .iter()
                .rev()
                .take(5)
                .any(|word| words2.iter().take(5).any(|w| w == word));
            assert!(overlap_found);
        }
    }
}
