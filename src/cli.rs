use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "manx",
    about = "A blazing-fast CLI documentation finder",
    long_about = r#"🚀 Intelligent documentation finder with native RAG and AI synthesis

CORE COMMANDS:
  snippet <lib> [query]          Search code snippets and examples (official + local docs)
  search <query>                 Search official documentation across the web
  doc <lib> [topic]              Browse comprehensive documentation  
  get <id>                       Retrieve specific results by ID

LOCAL RAG COMMANDS:
  index <path>                   Index your documents for semantic search
  sources list                   View indexed document sources
  sources clear                  Clear all indexed documents

EMBEDDING SYSTEM - Smart semantic search (works great out of the box):
  embedding status               View current embedding configuration
  embedding list                 Show installed models
  embedding download <model>     Install neural models from HuggingFace
  embedding test <query>         Test embedding quality

DEFAULT MODE (No setup required):
  ⚡ Hash-based embeddings       Built-in algorithm (0ms, offline, 0MB storage)
  📚 Official documentation      Context7 API integration
  🔍 Keyword matching           Excellent for exact phrases and terms

ENHANCED MODE (Optional setup for better results):
  🧠 Neural embeddings          Install: sentence-transformers/all-MiniLM-L6-v2
  🎯 Semantic understanding     "database connection" = "data storage"
  📊 Intent matching            Superior relevance ranking
  🔄 Easy switching             manx config --embedding-provider onnx:model-name

AI SYNTHESIS - Get comprehensive answers with citations (optional):
  manx config --llm-api "sk-key"           Enable AI answer synthesis
  manx snippet react hooks                 Search + AI explanation (if configured)

LOCAL RAG - Search your own documents and code (optional):
  manx index /path/to/docs                 Index your documentation
  manx config --rag-enabled                Enable local document search
  manx search "authentication" --rag       Search indexed documents only

QUICK START:
  manx snippet react "state management"    Works great with defaults
  manx embedding download all-MiniLM-L6-v2 Optional: Better semantic search
  manx config --llm-api "sk-openai-key"    Optional: AI synthesis

Use 'manx <command> --help' for detailed options."#,
    version = get_version_info(),
    author,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Show detailed debug information and API requests
    #[arg(long, help_heading = "DEBUG OPTIONS")]
    pub debug: bool,

    /// Output JSON format (useful for scripts and automation)
    #[arg(short = 'q', long, help_heading = "OUTPUT OPTIONS")]
    pub quiet: bool,

    /// Clear all cached documentation and start fresh
    #[arg(long, help_heading = "CACHE OPTIONS")]
    pub clear_cache: bool,

    /// Enable automatic caching of all search results
    #[arg(long, help_heading = "CACHE OPTIONS")]
    pub auto_cache_on: bool,

    /// Disable automatic caching (manual caching only)
    #[arg(long, help_heading = "CACHE OPTIONS")]
    pub auto_cache_off: bool,

    /// Override API key for this session
    #[arg(long, help_heading = "GLOBAL OPTIONS")]
    pub api_key: Option<String>,

    /// Override cache directory for this session
    #[arg(long, help_heading = "GLOBAL OPTIONS")]
    pub cache_dir: Option<PathBuf>,

    /// Work offline using only cached results
    #[arg(long, help_heading = "GLOBAL OPTIONS")]
    pub offline: bool,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Commands {
    /// 📚 Browse comprehensive documentation sections and guides
    Doc {
        /// Library name (examples: 'fastapi', 'react@18', 'django')
        #[arg(value_name = "LIBRARY")]
        library: String,
        /// Topic to search for within documentation (optional - omit for general docs)
        #[arg(value_name = "TOPIC", default_value = "")]
        query: String,
        /// Save documentation to file (auto-detects format)
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Limit number of sections shown (default: 10, use 0 for unlimited)
        #[arg(short = 'l', long, value_name = "NUMBER")]
        limit: Option<usize>,
        /// Force retrieval-only mode (disable LLM synthesis even if API key configured)
        #[arg(long)]
        no_llm: bool,
        /// Search locally indexed documents instead of Context7 API
        #[arg(long)]
        rag: bool,
    },

    /// 🔍 Search code snippets and examples with AI-powered understanding
    ///
    /// ENHANCED SEARCH:
    ///   • Searches official docs (Context7) + your indexed documents (RAG)
    ///   • Semantic understanding finds relevant content with different wording
    ///   • Quote prioritization: "useEffect cleanup" gets 10x higher relevance
    ///   • Optional AI synthesis provides comprehensive answers with citations
    ///
    /// SEMANTIC FEATURES:
    ///   • "memory leaks" finds: "memory cleanup", "performance issues", "leak prevention"
    ///   • "authentication" finds: "auth", "login", "security", "credentials"
    ///   • Version-specific: react@18, django@4.2
    ///
    /// AI SYNTHESIS:
    ///   • Configure: manx config --llm-api "sk-your-key"
    ///   • Get answers: manx snippet "react hooks best practices"
    ///   • Force retrieval: manx snippet react hooks --no-llm
    ///
    /// EXAMPLES:
    ///   manx snippet react "useEffect cleanup"           # Semantic search with phrase priority
    ///   manx snippet "database pooling" --llm-api        # Get AI answer with citations  
    ///   manx snippet fastapi middleware --no-llm         # Raw results only
    ///   manx snippet python "async functions" --rag      # Search your indexed code files
    Snippet {
        /// Library name (examples: 'fastapi', 'react@18', 'vue@3')
        #[arg(value_name = "LIBRARY")]
        library: String,
        /// Search query for specific code snippets
        #[arg(value_name = "QUERY")]
        query: Option<String>,
        /// Export results to file (format auto-detected by extension: .md, .json)
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Work offline using only cached results (no network requests)
        #[arg(long)]
        offline: bool,
        /// Save specific search results by number (e.g., --save 1,3,7)
        #[arg(long, value_name = "NUMBERS")]
        save: Option<String>,
        /// Save all search results to file
        #[arg(long)]
        save_all: bool,
        /// Export in JSON format instead of Markdown (use with --save or --save-all)
        #[arg(long)]
        json: bool,
        /// Limit number of results shown (default: 10, use 0 for unlimited)
        #[arg(short = 'l', long, value_name = "NUMBER")]
        limit: Option<usize>,
        /// Force retrieval-only mode (disable LLM synthesis even if API key configured)
        #[arg(long)]
        no_llm: bool,
        /// Search locally indexed documents instead of Context7 API (requires: manx config --rag-enabled)
        #[arg(long)]
        rag: bool,
    },

    /// 🔍 Search official documentation across the web
    ///
    /// INTELLIGENT WEB SEARCH:
    ///   • Prioritizes official documentation sites (docs.python.org, reactjs.org, etc.)
    ///   • Uses semantic embeddings for relevance matching  
    ///   • Falls back to trusted community sources with clear notification
    ///   • Optional LLM verification ensures result authenticity
    ///
    /// OFFICIAL-FIRST STRATEGY:
    ///   • Always searches official sources first (10x relevance boost)
    ///   • Expands to community sources only if insufficient official results
    ///   • Transparent fallback notifications: "⚠️ Expanded to community sources"
    ///
    /// EXAMPLES:
    ///   manx search "hydra configuration commands"      # Auto-detects LLM availability  
    ///   manx search "react hooks best practices"        # Uses LLM if API key configured
    ///   manx search "python async await" --no-llm       # Force embeddings-only mode
    ///   manx search "authentication" --rag              # Search your indexed documents
    Search {
        /// Search query for official documentation
        #[arg(value_name = "QUERY")]
        query: String,
        /// Disable LLM verification (use embeddings-only mode even if API key is configured)
        #[arg(long)]
        no_llm: bool,
        /// Export results to file (format auto-detected by extension: .md, .json)
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
        /// Limit number of results shown (default: 8)
        #[arg(short = 'l', long, value_name = "NUMBER")]
        limit: Option<usize>,
        /// Search locally indexed documents instead of web search (requires: manx config --rag-enabled)
        #[arg(long)]
        rag: bool,
    },

    /// 📥 Get specific item by ID (doc-3, section-5, etc.)
    Get {
        /// Item ID from previous search or doc command output
        #[arg(value_name = "ITEM_ID")]
        id: String,
        /// Save retrieved item to file
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// 🗂️ Manage local documentation cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },

    /// ⚙️ Configure Manx settings, API keys, and AI integration
    Config {
        /// Display current configuration settings
        #[arg(long)]
        show: bool,
        /// Set Context7 API key (get one at context7.com)
        #[arg(long, value_name = "KEY")]
        api_key: Option<String>,
        /// Set custom cache directory path
        #[arg(long, value_name = "PATH")]
        cache_dir: Option<PathBuf>,
        /// Enable/disable automatic caching (values: on, off)
        #[arg(long, value_name = "MODE")]
        auto_cache: Option<String>,
        /// Set cache expiration time in hours (default: 24)
        #[arg(long, value_name = "HOURS")]
        cache_ttl: Option<u64>,
        /// Set maximum cache size in MB (default: 100)
        #[arg(long, value_name = "SIZE")]
        max_cache_size: Option<u64>,
        /// Set OpenAI API key for GPT models
        #[arg(long, value_name = "API_KEY")]
        openai_api: Option<String>,
        /// Set Anthropic API key for Claude models
        #[arg(long, value_name = "API_KEY")]
        anthropic_api: Option<String>,
        /// Set Groq API key for fast inference
        #[arg(long, value_name = "API_KEY")]
        groq_api: Option<String>,
        /// Set OpenRouter API key for multi-model access
        #[arg(long, value_name = "API_KEY")]
        openrouter_api: Option<String>,
        /// Set HuggingFace API key for open-source models
        #[arg(long, value_name = "API_KEY")]
        huggingface_api: Option<String>,
        /// Set custom endpoint URL for self-hosted models
        #[arg(long, value_name = "URL")]
        custom_endpoint: Option<String>,
        /// Set preferred LLM provider (openai, anthropic, groq, openrouter, huggingface, custom, auto)
        #[arg(long, value_name = "PROVIDER")]
        llm_provider: Option<String>,
        /// Set specific model name (overrides provider defaults)
        #[arg(long, value_name = "MODEL")]
        llm_model: Option<String>,
        /// Legacy option - Set LLM API key (deprecated, use provider-specific options)
        #[arg(long, value_name = "API_KEY")]
        llm_api: Option<String>,
        /// Enable/disable local RAG system (values: on, off)
        #[arg(long, value_name = "MODE")]
        rag: Option<String>,
        /// Add custom official documentation domain (format: domain.com)
        #[arg(long, value_name = "DOMAIN")]
        add_official_domain: Option<String>,
        /// Set embedding provider for RAG system (hash, onnx:model, ollama:model, openai:model, huggingface:model, custom:url)
        #[arg(long, value_name = "PROVIDER")]
        embedding_provider: Option<String>,
        /// Set embedding API key for API-based providers
        #[arg(long, value_name = "API_KEY")]
        embedding_api_key: Option<String>,
        /// Set embedding model path for local models
        #[arg(long, value_name = "PATH")]
        embedding_model_path: Option<PathBuf>,
        /// Set embedding dimension (default: 384)
        #[arg(long, value_name = "DIMENSION")]
        embedding_dimension: Option<usize>,
    },

    /// 📁 Index local documents or web URLs for RAG search
    ///
    /// INDEXING SOURCES:
    ///   • Local files: manx index ~/docs/api.md
    ///   • Directories: manx index ~/documentation/  
    ///   • Web URLs: manx index https://docs.rust-lang.org/book/ch01-01-installation.html
    ///
    /// SUPPORTED FORMATS:
    ///   • Documents: .md, .txt, .docx, .pdf (with security validation)
    ///   • Web content: HTML pages (auto text extraction)
    ///
    /// SECURITY FEATURES:
    ///   • PDF processing disabled by default (configure to enable)  
    ///   • URL validation (HTTP/HTTPS only)
    ///   • Content sanitization and size limits
    ///
    /// EXAMPLES:
    ///   manx index ~/my-docs/                              # Index directory
    ///   manx index https://docs.python.org --crawl        # Deep crawl documentation site
    ///   manx index https://fastapi.tiangolo.com --crawl --max-depth 2  # Limited depth crawl
    ///   manx index api.pdf --alias "API Reference"        # Index with custom alias
    Index {
        /// Path to document/directory or URL to index
        #[arg(value_name = "PATH_OR_URL")]
        path: String,
        /// Optional alias for the indexed source
        #[arg(long, value_name = "ALIAS")]
        id: Option<String>,
        /// Enable deep crawling for URLs (follows links to discover more pages)
        #[arg(long)]
        crawl: bool,
        /// Maximum crawl depth for deep crawling (default: 3)
        #[arg(long, value_name = "DEPTH")]
        max_depth: Option<u32>,
        /// Maximum number of pages to crawl (default: no limit)
        #[arg(long, value_name = "PAGES")]
        max_pages: Option<u32>,
    },

    /// 📂 Manage indexed document sources
    Sources {
        #[command(subcommand)]
        command: SourceCommands,
    },

    /// 🔗 Open a specific documentation section by ID
    Open {
        /// Section ID from previous doc command output
        #[arg(value_name = "SECTION_ID")]
        id: String,
        /// Save opened section to file
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// 🔄 Update Manx to the latest version from GitHub
    Update {
        /// Check for updates without installing
        #[arg(long)]
        check: bool,
        /// Force update even if already on latest version
        #[arg(long)]
        force: bool,
    },

    /// 🧠 Manage embedding models and providers for semantic search
    ///
    /// EMBEDDING PROVIDERS:
    ///   • hash: Hash-based embeddings (default, fast, lightweight)
    ///   • onnx:model: Local ONNX models (requires download)
    ///   • ollama:model: Ollama API (requires Ollama server)
    ///   • openai:model: OpenAI embeddings API (requires API key)
    ///   • huggingface:model: HuggingFace embeddings API (requires API key)
    ///   • custom:url: Custom endpoint API
    ///
    /// EXAMPLES:
    ///   manx embedding status                     # Show current provider and models
    ///   manx embedding set hash                   # Use hash-based (default)
    ///   manx embedding set onnx:all-MiniLM-L6-v2 # Use local ONNX model
    ///   manx embedding set ollama:nomic-embed-text # Use Ollama model
    ///   manx embedding download all-MiniLM-L6-v2  # Download ONNX model
    ///   manx embedding test "sample query"        # Test current embedding setup
    Embedding {
        #[command(subcommand)]
        command: EmbeddingCommands,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Remove all cached documentation and free up disk space
    Clear,
    /// Display cache size, file count, and storage statistics  
    Stats,
    /// Show all currently cached libraries and their sizes
    List,
}

#[derive(Subcommand)]
pub enum SourceCommands {
    /// List all indexed document sources
    List,
    /// Add a document source to the index
    Add {
        /// Path to document or directory
        path: PathBuf,
        /// Optional alias for the source
        #[arg(long)]
        id: Option<String>,
    },
    /// Clear all indexed documents
    Clear,
}

#[derive(Subcommand)]
pub enum EmbeddingCommands {
    /// Show current embedding provider status and configuration
    Status,
    /// Set embedding provider (hash, onnx:model, ollama:model, openai:model, huggingface:model, custom:url)
    Set {
        /// Provider specification
        #[arg(value_name = "PROVIDER")]
        provider: String,
        /// API key for API-based providers
        #[arg(long, value_name = "API_KEY")]
        api_key: Option<String>,
        /// Custom endpoint URL (for custom provider)
        #[arg(long, value_name = "URL")]
        endpoint: Option<String>,
        /// Embedding dimension (default: 384)
        #[arg(long, value_name = "DIMENSION")]
        dimension: Option<usize>,
    },
    /// Download and install a local ONNX model
    Download {
        /// Model name to download (e.g., 'all-MiniLM-L6-v2')
        #[arg(value_name = "MODEL_NAME")]
        model: String,
        /// Force redownload if model already exists
        #[arg(long)]
        force: bool,
    },
    /// List available models for download or installed models
    List {
        /// List available models for download instead of installed models
        #[arg(long)]
        available: bool,
    },
    /// Test current embedding setup with a sample query
    Test {
        /// Query text to test embeddings with
        #[arg(value_name = "QUERY")]
        query: String,
        /// Show detailed embedding vector information
        #[arg(long)]
        verbose: bool,
    },
    /// Remove downloaded local models
    Remove {
        /// Model name to remove
        #[arg(value_name = "MODEL_NAME")]
        model: String,
    },
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

fn get_version_info() -> &'static str {
    concat!(
        "\n",
        "__| |__________________________________________________________________________| |__\n",
        "__   __________________________________________________________________________   __\n",
        "  | |                                                                          | |  \n",
        "  | |       ███        ██████   ██████   █████████   ██████   █████ █████ █████| |  \n",
        "  | |      ░░░██      ░░██████ ██████   ███░░░░░███ ░░██████ ░░███ ░░███ ░░███ | |  \n",
        "  | | ██     ░░██      ░███░█████░███  ░███    ░███  ░███░███ ░███  ░░███ ███  | |  \n",
        "  | |░░       ░░███    ░███░░███ ░███  ░███████████  ░███░░███░███   ░░█████   | |  \n",
        "  | |          ██░     ░███ ░░░  ░███  ░███░░░░░███  ░███ ░░██████    ███░███  | |  \n",
        "  | |         ██       ░███      ░███  ░███    ░███  ░███  ░░█████   ███ ░░███ | |  \n",
        "  | | ██    ███        █████     █████ █████   █████ █████  ░░█████ █████ █████| |  \n",
        "  | |░░    ░░░        ░░░░░     ░░░░░ ░░░░░   ░░░░░ ░░░░░    ░░░░░ ░░░░░ ░░░░░ | |  \n",
        "__| |__________________________________________________________________________| |__\n",
        "__   __________________________________________________________________________   __\n",
        "  | |                                                                          | |  \n",
        "\n",
        "  v",
        env!("CARGO_PKG_VERSION"),
        " • blazing-fast docs finder\n"
    )
}
