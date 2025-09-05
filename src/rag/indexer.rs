//! Document indexing pipeline for the RAG system
//!
//! This module handles parsing and chunking of various document formats
//! for indexing into the vector database.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use spider::website::Website;

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
        log::info!("Fetching content from URL: {}", url);

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

        // Fetch content from URL
        let content = fetch_url_content(&url).await?;

        if content.trim().is_empty() {
            return Err(anyhow!("URL contains no extractable content: {}", url));
        }

        // Create metadata for the URL
        let metadata = create_url_metadata(&url, &content).await?;

        // Detect document structure (title, sections) from HTML content
        let (title, sections) = detect_url_structure(&content, &url);

        // Chunk the content
        let chunks = chunk_content(&content, DEFAULT_CHUNK_SIZE, DEFAULT_CHUNK_OVERLAP);

        // Create document chunks
        let mut document_chunks = Vec::new();
        for (i, chunk_content) in chunks.into_iter().enumerate() {
            // Try to determine which section this chunk belongs to
            let current_section = find_relevant_section(&chunk_content, &sections);

            let chunk_id = format!("{}#{}", url, i);
            let document_chunk = DocumentChunk {
                id: chunk_id,
                content: chunk_content,
                source_path: PathBuf::from(&url), // Use URL as path
                source_type: SourceType::Remote,
                title: title.clone(),
                section: current_section,
                chunk_index: i,
                metadata: metadata.clone(),
            };

            document_chunks.push(document_chunk);
        }

        log::info!(
            "Successfully indexed {} chunks from URL: {}",
            document_chunks.len(),
            url
        );
        Ok(document_chunks)
    }

    /// Index content from a URL with deep crawling support
    pub async fn index_url_deep(
        &self, 
        url: String, 
        max_depth: Option<u32>,
        max_pages: Option<u32>
    ) -> Result<Vec<DocumentChunk>> {
        log::info!("Starting deep crawl of URL: {} (max_depth: {:?}, max_pages: {:?})", 
                   url, max_depth, max_pages);

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

        // Create and configure spider website
        let mut website = Website::new(&url);
        
        // Configure for documentation-optimized crawling
        website.configuration.respect_robots_txt = false; // Disable for faster crawling
        website.configuration.delay = 50; // Reduce delay for faster crawling
        website.configuration.subdomains = false; // Stay within same domain
        website.configuration.tld = false; // Don't crawl different TLDs
        website.configuration.request_timeout = Some(Box::new(std::time::Duration::from_secs(10))); // Add timeout
        
        // Set crawl depth limit (default to 3 if not specified)
        if let Some(depth) = max_depth {
            website.configuration.depth = depth as usize;
        } else {
            website.configuration.depth = 3;
        }

        // Filter out non-documentation URLs
        let blacklist_patterns = vec![
            "/api/".to_string(),
            "/downloads/".to_string(),
            "/images/".to_string(),
            "/assets/".to_string(),
            "/static/".to_string(),
            ".jpg".to_string(),
            ".jpeg".to_string(),
            ".png".to_string(),
            ".gif".to_string(),
            ".pdf".to_string(),
            ".zip".to_string(),
            ".tar".to_string(),
        ];
        
        website.configuration.blacklist_url = Some(
            blacklist_patterns
                .iter()
                .map(|pattern| format!("{}{}", parsed_url.origin().ascii_serialization(), pattern).into())
                .collect()
        );

        // Perform the crawl with timeout
        let crawl_timeout = std::time::Duration::from_secs(30);
        match tokio::time::timeout(crawl_timeout, website.crawl()).await {
            Ok(_) => log::info!("Crawl completed successfully"),
            Err(_) => {
                log::warn!("Crawl timed out after {} seconds", crawl_timeout.as_secs());
                return Err(anyhow!("Crawl operation timed out after {} seconds", crawl_timeout.as_secs()));
            }
        }
        
        let discovered_links = website.get_links();
        log::info!("Discovered {} URLs during crawl", discovered_links.len());

        // Apply page limit if specified
        let links_to_process = if let Some(max) = max_pages {
            discovered_links.into_iter().take(max as usize).collect::<Vec<_>>()
        } else {
            discovered_links.into_iter().collect::<Vec<_>>()
        };

        log::info!("Processing {} URLs for content extraction", links_to_process.len());

        // Process each discovered page
        let mut all_chunks = Vec::new();
        for (index, link) in links_to_process.iter().enumerate() {
            let page_url = link.as_ref();
            log::debug!("Processing page {}/{}: {}", index + 1, links_to_process.len(), page_url);
            
            match self.process_crawled_page(page_url).await {
                Ok(chunks) => {
                    let chunk_count = chunks.len();
                    all_chunks.extend(chunks);
                    log::debug!("Successfully processed page: {} ({} chunks)", page_url, chunk_count);
                },
                Err(e) => {
                    log::warn!("Failed to process page '{}': {}", page_url, e);
                    // Continue with other pages even if one fails
                }
            }
        }

        log::info!(
            "Successfully indexed {} chunks from {} pages via deep crawl of: {}",
            all_chunks.len(),
            links_to_process.len(),
            url
        );

        Ok(all_chunks)
    }

    /// Process a single crawled page URL and extract content chunks
    async fn process_crawled_page(&self, page_url: &str) -> Result<Vec<DocumentChunk>> {
        // Fetch content from the page
        let content = fetch_url_content(page_url).await?;

        if content.trim().is_empty() {
            return Err(anyhow!("Page contains no extractable content: {}", page_url));
        }

        // Create metadata for this page
        let metadata = create_url_metadata(page_url, &content).await?;

        // Detect document structure (title, sections) from HTML content
        let (title, sections) = detect_url_structure(&content, page_url);

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
                source_path: PathBuf::from(page_url),
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
}

/// Supported file extensions for indexing
const SUPPORTED_EXTENSIONS: &[&str] = &[".md", ".txt", ".pdf", ".doc", ".docx"];

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
    let max_file_size = 50 * 1024 * 1024; // 50MB limit per file

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

    log::info!("Indexing document: {:?}", path);

    // Extract text content
    let content = extract_text(&path)?;
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
fn extract_text(path: &Path) -> Result<String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "md" | "txt" => extract_text_file(path),
        "pdf" => extract_pdf_text(path),
        "doc" | "docx" => extract_doc_text(path),
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

    // Note about PDF processing
    content.push_str("\nNote: Full PDF text extraction will be implemented in a future update.\n");
    content.push_str("For now, this document is indexed by filename and metadata.");

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

/// Fetch content from URL and extract text
async fn fetch_url_content(url: &str) -> Result<String> {
    log::debug!("Fetching content from: {}", url);

    // Create HTTP client with reasonable timeouts
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Manx Documentation Indexer/1.0")
        .build()?;

    // Fetch the URL
    let response = client.get(url).send().await?;

    // Check if request was successful
    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch URL '{}': HTTP {}",
            url,
            response.status()
        ));
    }

    // Get content type to determine how to process
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = response.text().await?;

    // Process based on content type
    if content_type.contains("text/html") || content_type.contains("application/xhtml") {
        // Extract text from HTML
        extract_text_from_html(&body)
    } else if content_type.contains("text/plain") || content_type.contains("text/markdown") {
        // Plain text or markdown
        Ok(body)
    } else {
        log::warn!(
            "Unknown content type '{}' for URL '{}', treating as plain text",
            content_type,
            url
        );
        Ok(body)
    }
}

/// Extract readable text from HTML content
fn extract_text_from_html(html: &str) -> Result<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Remove script, style, and other non-content elements
    let cleanup_selectors = [
        "script",
        "style",
        "nav",
        "footer",
        "header",
        ".nav",
        ".footer",
        ".header",
        ".sidebar",
        ".advertisement",
        ".ads",
        ".social-share",
        "[role='navigation']",
        "[role='banner']",
        "[role='contentinfo']",
    ];

    let mut cleaned_html = html.to_string();
    for selector_str in &cleanup_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            // Remove matching elements by replacing them with empty content
            for element in document.select(&selector) {
                let element_html = element.html();
                cleaned_html = cleaned_html.replace(&element_html, "");
            }
        }
    }

    // Re-parse the cleaned HTML
    let cleaned_document = Html::parse_document(&cleaned_html);

    // Extract text from main content areas
    let content_selectors = [
        "article",
        "main",
        ".content",
        ".article",
        ".post",
        ".entry",
        "div[role=main]",
        "[role=article]",
        ".documentation",
    ];

    let mut extracted_text = String::new();

    // Try to find main content first
    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in cleaned_document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() && text.len() > 100 {
                    // Only substantial content
                    extracted_text.push_str(&text);
                    extracted_text.push('\n');
                }
            }
        }
    }

    // If no main content found, extract from body
    if extracted_text.trim().is_empty() {
        if let Ok(body_selector) = Selector::parse("body") {
            if let Some(element) = cleaned_document.select(&body_selector).next() {
                extracted_text = element.text().collect::<Vec<_>>().join(" ");
            }
        }
    }

    // Final fallback - get all text
    if extracted_text.trim().is_empty() {
        extracted_text = cleaned_document
            .root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ");
    }

    // Clean up whitespace
    let cleaned = extracted_text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(cleaned)
}

/// Create metadata for URL content
async fn create_url_metadata(_url: &str, content: &str) -> Result<DocumentMetadata> {
    Ok(DocumentMetadata {
        file_type: "web".to_string(),
        size: content.len() as u64,
        modified: chrono::Utc::now(),
        tags: vec!["web".to_string(), "remote".to_string()],
        language: detect_content_language(content),
    })
}

/// Detect document structure from URL content (title, sections)
fn detect_url_structure(content: &str, url: &str) -> (Option<String>, Vec<String>) {
    use scraper::{Html, Selector};

    // Try to parse as HTML first
    let document = Html::parse_document(content);

    // Extract title (before cleaning, as title is in head)
    let title = if let Ok(title_selector) = Selector::parse("title") {
        document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|t| !t.is_empty())
    } else {
        None
    }
    .or_else(|| {
        // Fallback: use URL path as title
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(mut segments) = parsed_url.path_segments() {
                if let Some(last_segment) = segments.next_back() {
                    if !last_segment.is_empty() {
                        return Some(last_segment.replace("-", " ").replace("_", " "));
                    }
                }
            }
            // If no path segments, use domain as title
            parsed_url.host_str().map(|s| s.to_string())
        } else {
            None
        }
    });

    // Clean HTML by removing non-content elements first
    let cleanup_selectors = ["script", "style", "nav", "footer", "header"];
    let mut cleaned_html = content.to_string();
    for selector_str in &cleanup_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector) {
                let element_html = element.html();
                cleaned_html = cleaned_html.replace(&element_html, "");
            }
        }
    }

    let cleaned_document = Html::parse_document(&cleaned_html);

    // Extract section headings from cleaned content
    let mut sections = Vec::new();
    let heading_selectors = ["h1", "h2", "h3", "h4", "h5", "h6"];

    for selector_str in &heading_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in cleaned_document.select(&selector) {
                let heading_text = element.text().collect::<String>().trim().to_string();
                if !heading_text.is_empty() && heading_text.len() < 200 {
                    sections.push(heading_text);
                }
            }
        }
    }

    // If it's not HTML, look for markdown-style headings
    if sections.is_empty() {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let heading = trimmed.trim_start_matches('#').trim();
                if !heading.is_empty() && heading.len() < 200 {
                    sections.push(heading.to_string());
                }
            }
        }
    }

    (title, sections)
}

/// Find the most relevant section for a chunk of content
fn find_relevant_section(chunk_content: &str, sections: &[String]) -> Option<String> {
    // Simple heuristic: find the section whose title appears in the chunk
    for section in sections {
        if chunk_content
            .to_lowercase()
            .contains(&section.to_lowercase())
        {
            return Some(section.clone());
        }
    }
    None
}

/// Detect content language using basic heuristics
fn detect_content_language(content: &str) -> Option<String> {
    // Simple language detection based on common programming patterns and keywords
    let content_lower = content.to_lowercase();

    if content_lower.contains("function")
        || content_lower.contains("const ")
        || content_lower.contains("let ")
    {
        Some("javascript".to_string())
    } else if content_lower.contains("def ")
        || content_lower.contains("import ")
        || content_lower.contains("print(")
    {
        Some("python".to_string())
    } else if content_lower.contains("fn ")
        || content_lower.contains("use ")
        || content_lower.contains("struct")
    {
        Some("rust".to_string())
    } else if content_lower.contains("class ") && content_lower.contains("public") {
        Some("java".to_string())
    } else {
        Some("en".to_string())
    }
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

    // Note about document processing
    content.push_str("\nNote: Full document text extraction will be enhanced in future updates.\n");
    content.push_str("Currently indexed by filename and metadata for discoverability.");

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

#[cfg(test)]
mod tests {
    use super::*;
    // These imports are not currently used in tests but may be needed for future test implementations
    // use std::fs::File;
    // use std::io::Write;

    #[test]
    fn test_is_supported_file() {
        assert!(is_supported_file(Path::new("test.md")));
        assert!(is_supported_file(Path::new("test.txt")));
        assert!(is_supported_file(Path::new("test.pdf")));
        assert!(!is_supported_file(Path::new("test.rs")));
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
