use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "manx",
    about = "A blazing-fast CLI documentation finder",
    long_about = r#"🚀 Fast documentation finder with intelligent semantic search

CORE COMMANDS:
  snippet <lib> [query]          Search code snippets and examples
  doc <lib> [topic]              Browse comprehensive documentation  
  get <id>                       Retrieve specific results by ID

SEMANTIC SEARCH - Use quotes to prioritize exact phrases:
  manx snippet react "useEffect cleanup"   Prioritizes exact phrase matches
  manx snippet tauri "table implementations" Phrases get 10x higher relevance
  manx snippet fastapi middleware          Individual terms search

EXAMPLES:
  manx snippet react hooks                 Search React hooks examples
  manx snippet react "useEffect cleanup"   Prioritize exact phrase match
  manx doc fastapi                         Browse FastAPI documentation  
  manx get doc-3                           Retrieve search result #3

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
}

#[derive(Subcommand)]
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
    },

    /// 🔍 Search for code snippets and examples
    /// 
    /// SEMANTIC SEARCH FEATURES:
    ///   • Use quotes for exact phrases: "useEffect cleanup" 
    ///   • Quoted phrases get 10x higher relevance scores
    ///   • Individual terms: react hooks useState
    ///   • Version-specific: react@18, django@4.2
    ///
    /// EXAMPLES:
    ///   manx snippet react "useEffect cleanup"
    ///   manx snippet fastapi "async middleware" --limit 5
    ///   manx snippet django@4.2 models --save-all
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

    /// ⚙️ Configure Manx settings and preferences
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
        "  v", env!("CARGO_PKG_VERSION"), " • blazing-fast docs finder\n"
    )
}
