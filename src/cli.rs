use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "manx",
    about = "A blazing-fast CLI documentation finder",
    long_about = "Manx brings Context7 MCP docs right to your terminal - no IDE required",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Library to search (e.g., 'fastapi', 'react@18')
    #[arg(value_name = "LIBRARY")]
    pub library: Option<String>,
    
    /// Search query
    #[arg(value_name = "QUERY")]
    pub query: Option<String>,
    
    /// Export output to file (format auto-detected: .md, .json)
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    
    /// Quiet mode (JSON output for scripts)
    #[arg(short = 'q', long)]
    pub quiet: bool,
    
    /// Use offline cache only
    #[arg(long)]
    pub offline: bool,
    
    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,
    
    /// Clear all cached documentation
    #[arg(long)]
    pub clear_cache: bool,
    
    /// Enable automatic caching of search results
    #[arg(long)]
    pub auto_cache_on: bool,
    
    /// Disable automatic caching of search results  
    #[arg(long)]
    pub auto_cache_off: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get full documentation for a library
    Doc {
        /// Library name (e.g., 'fastapi', 'react@18')
        library: String,
        /// Search query within docs
        query: String,
        /// Export output to file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    
    /// Expand a specific snippet by ID
    Snippet {
        /// Snippet ID from search results
        id: String,
        /// Export output to file
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
    },
    
    /// Manage local cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    
    /// Configure manx settings
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Set API key for Context7
        #[arg(long)]
        api_key: Option<String>,
        /// Set cache directory
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        /// Set auto-cache mode (on/off)
        #[arg(long)]
        auto_cache: Option<String>,
        /// Set cache TTL in hours
        #[arg(long)]
        cache_ttl: Option<u64>,
        /// Set maximum cache size in MB
        #[arg(long)]
        max_cache_size: Option<u64>,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Clear all cached data
    Clear,
    /// Show cache statistics
    Stats,
    /// List cached libraries
    List,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}