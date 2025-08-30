use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "manx",
    about = "A blazing-fast CLI documentation finder",
    long_about = "🚀 Manx - Fast documentation finder with two modes:

📝 SEARCH MODE (default): Find specific code snippets and examples
    manx react hooks              Find React hooks code examples
    manx fastapi middleware       Search FastAPI middleware snippets  
    manx django auth --limit 5    Limit search results to 5 entries

📚 DOC MODE: Browse comprehensive guides and documentation
    manx doc react               Browse full React documentation
    manx doc fastapi \"async\"     Get FastAPI async documentation
    manx doc django --limit 3    Show first 3 documentation sections

💾 SAVE & EXPORT:
    manx fastapi --save 1,3,7     Save specific search results
    manx react --save-all --json Export all results as JSON
    manx doc react -o react.md   Export docs to markdown

⚙️  CONFIGURATION:
    manx config --show           Show current settings
    manx cache clear            Clear cached data

Use 'manx <command> --help' for detailed command information.",
    version = get_version_info(),
    author,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Library name to search (examples: 'fastapi', 'react@18', 'vue@3')
    #[arg(value_name = "LIBRARY", help_heading = "ARGUMENTS")]
    pub library: Option<String>,

    /// 📝 Search for specific code snippets and examples
    #[arg(value_name = "QUERY", help_heading = "ARGUMENTS")]
    pub query: Option<String>,

    /// Export results to file (format auto-detected by extension: .md, .json)
    #[arg(
        short = 'o',
        long,
        value_name = "FILE",
        help_heading = "OUTPUT OPTIONS"
    )]
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

    /// Save specific search results by number (e.g., --save 1,3,7)
    #[arg(long, value_name = "NUMBERS", help_heading = "SAVE OPTIONS")]
    pub save: Option<String>,

    /// Save all search results to file
    #[arg(long, help_heading = "SAVE OPTIONS")]
    pub save_all: bool,

    /// Export in JSON format instead of Markdown (use with --save or --save-all)
    #[arg(long, help_heading = "SAVE OPTIONS")]
    pub json: bool,

    /// Limit number of results shown (default: 10, use 0 for unlimited)
    #[arg(
        short = 'l',
        long,
        value_name = "NUMBER",
        help_heading = "OUTPUT OPTIONS"
    )]
    pub limit: Option<usize>,
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

    /// 🔗 Open a specific documentation section by ID (e.g., 'doc-4')
    Open {
        /// Section ID from previous doc command output
        #[arg(value_name = "SECTION_ID")]
        id: String,
        /// Save opened section to file
        #[arg(short = 'o', long, value_name = "FILE")]
        output: Option<PathBuf>,
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

fn get_version_info() -> &'static str {
    concat!(
        "\n",
        "__| |________________________________________________________________________________| |__\n",
        "__   ________________________________________________________________________________   __\n",
        "  | |                                                                                | |  \n",
        "  | | ███                    ██████   ██████   █████████   ██████   █████ █████ █████| |  \n",
        "  | |░░░███                 ░░██████ ██████   ███░░░░░███ ░░██████ ░░███ ░░███ ░░███ | |  \n",
        "  | |  ░░░███                ░███░█████░███  ░███    ░███  ░███░███ ░███  ░░███ ███  | |  \n",
        "  | |    ░░░███              ░███░░███ ░███  ░███████████  ░███░░███░███   ░░█████   | |  \n",
        "  | |     ███░               ░███ ░░░  ░███  ░███░░░░░███  ░███ ░░██████    ███░███  | |  \n",
        "  | |   ███░                 ░███      ░███  ░███    ░███  ░███  ░░█████   ███ ░░███ | |  \n",
        "  | | ███░      █████████    █████     █████ █████   █████ █████  ░░█████ █████ █████| |  \n",
        "  | |░░░       ░░░░░░░░░    ░░░░░     ░░░░░ ░░░░░   ░░░░░ ░░░░░    ░░░░░ ░░░░░ ░░░░░ | |  \n",
        "__| |________________________________________________________________________________| |__\n",
        "__   ________________________________________________________________________________   __\n",
        "  | |                                                                                | |  \n",
        "\n",
        "  v", env!("CARGO_PKG_VERSION"), " • blazing-fast docs finder\n"
    )
}
