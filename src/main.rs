mod cli;
mod client;
mod search;
mod render;
mod cache;
mod export;
mod config;

use anyhow::Result;
use colored::control;
use std::process;

use crate::cli::{Cli, Commands, CacheCommands};
use crate::client::Context7Client;
use crate::search::SearchEngine;
use crate::render::Renderer;
use crate::cache::CacheManager;
use crate::export::Exporter;
use crate::config::Config;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Parse CLI arguments
    let args = Cli::parse_args();
    
    // Initialize logging if debug mode
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
    }
    
    // Load configuration
    let mut config = Config::load().unwrap_or_default();
    
    // Handle NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() || !config.color_output {
        control::set_override(false);
    }
    
    // Create renderer
    let renderer = Renderer::new(args.quiet);
    
    // Handle global flags first
    if args.clear_cache {
        let cache_manager = if let Some(dir) = &config.cache_dir {
            CacheManager::with_custom_dir(dir.clone())?
        } else {
            CacheManager::new()?
        };
        
        let pb = renderer.show_progress("Clearing cache...");
        cache_manager.clear().await?;
        pb.finish_and_clear();
        renderer.print_success("Cache cleared successfully");
        return Ok(());
    }
    
    if args.auto_cache_on {
        config.auto_cache_enabled = true;
        config.save()?;
        renderer.print_success("Auto-caching enabled");
        return Ok(());
    }
    
    if args.auto_cache_off {
        config.auto_cache_enabled = false;
        config.save()?;
        renderer.print_success("Auto-caching disabled");
        return Ok(());
    }

    // Handle commands
    match args.command {
        Some(Commands::Config { show, api_key, cache_dir, auto_cache, cache_ttl, max_cache_size }) => {
            if show {
                println!("{}", config.display());
            } else {
                let mut updated = false;
                
                if let Some(key) = api_key {
                    config.api_key = Some(key);
                    renderer.print_success("API key updated");
                    updated = true;
                }
                
                if let Some(dir) = cache_dir {
                    config.cache_dir = Some(dir);
                    renderer.print_success("Cache directory updated");
                    updated = true;
                }
                
                if let Some(auto_cache_setting) = auto_cache {
                    match auto_cache_setting.to_lowercase().as_str() {
                        "on" | "true" | "1" => {
                            config.auto_cache_enabled = true;
                            renderer.print_success("Auto-caching enabled");
                            updated = true;
                        }
                        "off" | "false" | "0" => {
                            config.auto_cache_enabled = false;
                            renderer.print_success("Auto-caching disabled");
                            updated = true;
                        }
                        _ => {
                            renderer.print_error("Invalid auto-cache value. Use 'on' or 'off'");
                        }
                    }
                }
                
                if let Some(ttl) = cache_ttl {
                    config.cache_ttl_hours = ttl;
                    renderer.print_success(&format!("Cache TTL set to {} hours", ttl));
                    updated = true;
                }
                
                if let Some(size) = max_cache_size {
                    config.max_cache_size_mb = size;
                    renderer.print_success(&format!("Max cache size set to {} MB", size));
                    updated = true;
                }
                
                if updated {
                    config.save()?;
                } else {
                    println!("{}", config.display());
                }
            }
        }
        
        Some(Commands::Cache { command }) => {
            let cache_manager = if let Some(dir) = &config.cache_dir {
                CacheManager::with_custom_dir(dir.clone())?
            } else {
                CacheManager::new()?
            };
            
            match command {
                CacheCommands::Clear => {
                    let pb = renderer.show_progress("Clearing cache...");
                    cache_manager.clear().await?;
                    pb.finish_and_clear();
                    renderer.print_success("Cache cleared successfully");
                }
                CacheCommands::Stats => {
                    let stats = cache_manager.stats().await?;
                    println!("Cache Statistics:");
                    println!("  Total size: {:.2} MB", stats.total_size_mb);
                    println!("  Files: {}", stats.file_count);
                    println!("  Categories: {}", stats.categories.join(", "));
                }
                CacheCommands::List => {
                    let items = cache_manager.list_cached().await?;
                    if items.is_empty() {
                        println!("No cached items found");
                    } else {
                        println!("Cached items:");
                        for item in items {
                            println!("  [{:<10}] {} ({:.1} KB)", 
                                item.category, item.name, item.size_kb);
                        }
                    }
                }
            }
        }
        
        Some(Commands::Doc { library, query, output }) => {
            handle_doc_command(
                &library,
                &query,
                output.as_ref(),
                &config,
                &renderer,
                args.offline
            ).await?;
        }
        
        Some(Commands::Snippet { id, output }) => {
            handle_snippet_command(
                &id,
                output.as_ref(),
                &config,
                &renderer,
                args.offline
            ).await?;
        }
        
        None => {
            // Default search command
            if let Some(library) = args.library {
                let query = args.query.unwrap_or_default();
                handle_search_command(
                    &library,
                    &query,
                    args.output.as_ref(),
                    &config,
                    &renderer,
                    args.offline
                ).await?;
            } else {
                // Show help if no arguments
                println!("manx - A blazing-fast CLI documentation finder\n");
                println!("Usage:");
                println!("  manx <library> [query]        Search documentation");
                println!("  manx doc <library> <query>    Get full documentation");
                println!("  manx snippet <id>             Expand a search result");
                println!("  manx cache <command>          Manage local cache");
                println!("  manx config                   Configure settings");
                println!("\nExamples:");
                println!("  manx fastapi                  Search FastAPI docs");
                println!("  manx react@18 hooks           Search React v18 for 'hooks'");
                println!("  manx doc fastapi middleware   Get FastAPI middleware docs");
                println!("\nFor more help: manx --help");
            }
        }
    }
    
    Ok(())
}

async fn handle_search_command(
    library: &str,
    query: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    offline: bool,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };
    
    let cache_key = format!("{}_{}", library, query);
    
    // Try cache first if offline mode
    if offline || config.offline_mode {
        if let Some(results) = cache_manager.get::<Vec<crate::client::SearchResult>>(
            "search", 
            &cache_key
        ).await? {
            renderer.render_search_results(&results)?;
            if let Some(path) = output {
                Exporter::export_search_results(&results, path)?;
                renderer.print_success(&format!("Results exported to {:?}", path));
            }
            return Ok(());
        } else if offline || config.offline_mode {
            anyhow::bail!("No cached results available in offline mode");
        }
    }
    
    // Perform online search
    let pb = renderer.show_progress(&format!("Searching {} for '{}'...", library, query));
    
    let client = Context7Client::new(config.api_key.clone())?;
    let search_engine = SearchEngine::new(client);
    
    let results = search_engine.search(
        library,
        query,
        Some(config.default_limit)
    ).await?;
    
    pb.finish_and_clear();
    
    // Cache results only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("search", &cache_key, &results).await.ok();
    }
    
    // Render results
    renderer.render_search_results(&results)?;
    
    // Export if requested
    if let Some(path) = output {
        Exporter::export_search_results(&results, path)?;
        renderer.print_success(&format!("Results exported to {:?}", path));
    }
    
    Ok(())
}

async fn handle_doc_command(
    library: &str,
    query: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    offline: bool,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };
    
    let cache_key = format!("{}_{}", library, query);
    
    // Try cache first if offline mode
    if offline || config.offline_mode {
        if let Some(doc) = cache_manager.get::<crate::client::Documentation>(
            "docs", 
            &cache_key
        ).await? {
            renderer.render_documentation(&doc)?;
            if let Some(path) = output {
                Exporter::export_documentation(&doc, path)?;
                renderer.print_success(&format!("Documentation exported to {:?}", path));
            }
            return Ok(());
        } else if offline || config.offline_mode {
            anyhow::bail!("No cached documentation available in offline mode");
        }
    }
    
    // Fetch documentation online
    let pb = renderer.show_progress(&format!("Fetching {} documentation...", library));
    
    let client = Context7Client::new(config.api_key.clone())?;
    let search_engine = SearchEngine::new(client);
    
    let doc = search_engine.get_documentation(library, Some(query)).await?;
    
    pb.finish_and_clear();
    
    // Cache documentation only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("docs", &cache_key, &doc).await.ok();
    }
    
    // Render documentation
    renderer.render_documentation(&doc)?;
    
    // Export if requested
    if let Some(path) = output {
        Exporter::export_documentation(&doc, path)?;
        renderer.print_success(&format!("Documentation exported to {:?}", path));
    }
    
    Ok(())
}

async fn handle_snippet_command(
    id: &str,
    _output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    offline: bool,
) -> Result<()> {
    // For snippets, we need to fetch from cache or make a new request
    // This is a simplified version - in production, we'd store snippet metadata
    
    if offline || config.offline_mode {
        anyhow::bail!("Snippet expansion requires online mode");
    }
    
    let pb = renderer.show_progress(&format!("Fetching snippet {}...", id));
    
    // In a real implementation, we'd have a method to fetch specific snippets
    // For now, we'll return an error message
    pb.finish_and_clear();
    
    renderer.print_error("Snippet expansion not yet implemented. Use 'manx doc' instead.");
    
    Ok(())
}