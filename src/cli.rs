use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "manx",
    about = "A blazing-fast CLI documentation finder",
    long_about = "🚀 Manx - Fast documentation search powered by Context7 MCP

Brings real-time, version-specific documentation right to your terminal.
No IDE required, works anywhere!

EXAMPLES:
    manx fastapi                    Search FastAPI documentation
    manx react@18 hooks            Search React v18 for hooks specifically  
    manx doc django models         Get complete Django models guide
    manx --clear-cache             Quick cache cleanup
    manx config --auto-cache off   Disable automatic caching

For more examples: https://github.com/neur0map/manx#usage",
    version,
    author,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Library name to search (examples: 'fastapi', 'react@18', 'vue@3')
    #[arg(value_name = "LIBRARY", help_heading = "ARGUMENTS")]
    pub library: Option<String>,
    
    /// Search query within the library documentation
    #[arg(value_name = "QUERY", help_heading = "ARGUMENTS")]
    pub query: Option<String>,
    
    /// Export results to file (format auto-detected by extension: .md, .json)
    #[arg(short = 'o', long, value_name = "FILE", help_heading = "OUTPUT OPTIONS")]
    pub output: Option<PathBuf>,
    
    /// Output JSON format (useful for scripts and automation)
    #[arg(short = 'q', long, help_heading = "OUTPUT OPTIONS")]
    pub quiet: bool,
    
    /// Work offline using only cached results (no network requests)
    #[arg(long, help_heading = "NETWORK OPTIONS")]
    pub offline: bool,
    
    /// Show detailed debug information and API requests
    #[arg(long, help_heading = "DEBUG OPTIONS")]
    pub debug: bool,
    
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
    /// Get comprehensive documentation with examples and guides
    Doc {
        /// Library name (examples: 'fastapi', 'react@18', 'django')
        #[arg(value_name = "LIBRARY")]
        library: String,
        /// Topic to search for within documentation
        #[arg(value_name = "TOPIC")]
        query: String,
        /// Save documentation to file (auto-detects format)
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    
    /// Expand and view detailed information for a specific result
    Snippet {
        /// Result ID from previous search output
        #[arg(value_name = "RESULT_ID")]
        id: String,
        /// Save snippet details to file
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    
    /// Manage local documentation cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    
    /// Configure Manx settings and preferences
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
    
    /// Update Manx to the latest version from GitHub
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