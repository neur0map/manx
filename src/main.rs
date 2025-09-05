mod cache;
mod cli;
mod client;
mod config;
mod export;
mod render;
mod search;
mod update;

use anyhow::Result;
use colored::control;
use std::process;

use crate::cache::CacheManager;
use crate::cli::{CacheCommands, Cli, Commands};
use crate::client::Context7Client;
use crate::config::Config;
use crate::export::Exporter;
use crate::render::Renderer;
use crate::search::SearchEngine;
use crate::update::SelfUpdater;

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
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }

    // Load configuration
    let mut config = Config::load().unwrap_or_default();

    // Config is loaded as-is, no CLI arguments to merge

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
        Some(Commands::Config {
            show,
            api_key,
            cache_dir,
            auto_cache,
            cache_ttl,
            max_cache_size,
        }) => {
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
                            println!(
                                "  [{:<10}] {} ({:.1} KB)",
                                item.category, item.name, item.size_kb
                            );
                        }
                    }
                }
            }
        }

        Some(Commands::Doc {
            library,
            query,
            output,
            limit,
        }) => {
            handle_doc_command(
                &library,
                &query,
                output.as_ref(),
                &config,
                &renderer,
                false,
                limit,
            )
            .await?;
        }

        Some(Commands::Snippet {
            library,
            query,
            output,
            offline,
            save,
            save_all,
            json,
            limit,
        }) => {
            let query_str = query.unwrap_or_default();
            handle_search_command(
                &library,
                &query_str,
                output.as_ref(),
                &config,
                &renderer,
                offline,
                save.as_ref(),
                save_all,
                json,
                limit,
            )
            .await?;
        }

        Some(Commands::Get { id, output }) => {
            handle_get_command(&id, output.as_ref(), &config, &renderer, false).await?;
        }


        Some(Commands::Open { id, output }) => {
            handle_open_command(&id, output.as_ref(), &config, &renderer).await?;
        }

        Some(Commands::Update { check, force }) => {
            let updater = SelfUpdater::new(renderer)?;

            if check {
                let update_info = updater.check_for_updates().await?;

                if update_info.update_available {
                    println!("🎉 Update available!");
                    println!("Current version: v{}", update_info.current_version);
                    println!("Latest version:  v{}", update_info.latest_version);

                    if !update_info.release_notes.trim().is_empty() {
                        println!("\n📝 Release Notes:");
                        println!("{}", update_info.release_notes);
                    }

                    println!("\nRun 'manx update' to install the latest version.");
                } else {
                    println!(
                        "✅ You're already on the latest version (v{})",
                        update_info.current_version
                    );
                }
            } else {
                updater.perform_update(force).await?;
            }
        }

        None => {
            // This should never be reached due to arg_required_else_help = true
            // But just in case, show a simple message
            println!("Use 'manx --help' for usage information");
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn handle_search_command(
    library: &str,
    query: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    offline: bool,
    save_numbers: Option<&String>,
    save_all: bool,
    json_format: bool,
    limit: Option<usize>,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };

    let cache_key = format!("{}_{}", library, query);

    // Try cache first if offline mode
    if offline || config.offline_mode {
        if let Some(results) = cache_manager
            .get::<Vec<crate::client::SearchResult>>("search", &cache_key)
            .await?
        {
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

    let (results, library_title, library_id) = search_engine
        .search(library, query, Some(config.default_limit))
        .await?;

    pb.finish_and_clear();

    // Cache results only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("search", &cache_key, &results).await.ok();
    }

    // Render results with library information and limit
    renderer.render_search_results_with_library(
        &results,
        Some((&library_title, &library_id)),
        limit,
    )?;

    // Export if requested
    if let Some(path) = output {
        Exporter::export_search_results(&results, path)?;
        renderer.print_success(&format!("Results exported to {:?}", path));
    }

    // Handle batch save operations
    if save_all || save_numbers.is_some() {
        let filename = if let Some(path) = output {
            path.clone()
        } else {
            // Generate smart default filename
            let extension = if json_format { "json" } else { "md" };
            let prefix = if save_all { "all" } else { "snippets" };
            std::path::PathBuf::from(format!("{}-{}.{}", library, prefix, extension))
        };

        if save_all {
            // Save all results
            Exporter::export_batch_snippets(
                &results,
                &filename,
                json_format,
                library,
                &cache_manager,
            )
            .await?;
            renderer.print_success(&format!(
                "All {} results saved to {:?}",
                results.len(),
                filename
            ));
        } else if let Some(numbers_str) = save_numbers {
            // Parse and save specific results
            let numbers: Result<Vec<usize>, _> = numbers_str
                .split(',')
                .map(|s| s.trim().parse::<usize>())
                .collect();

            match numbers {
                Ok(indices) => {
                    let selected_results: Vec<crate::client::SearchResult> = indices
                        .iter()
                        .filter_map(|&i| results.get(i.saturating_sub(1)).cloned()) // Convert 1-based to 0-based and clone
                        .collect();

                    if selected_results.is_empty() {
                        renderer.print_error("No valid result numbers specified");
                    } else {
                        Exporter::export_batch_snippets(
                            &selected_results,
                            &filename,
                            json_format,
                            library,
                            &cache_manager,
                        )
                        .await?;
                        renderer.print_success(&format!(
                            "{} snippets saved to {:?}",
                            selected_results.len(),
                            filename
                        ));
                    }
                }
                Err(_) => {
                    renderer.print_error(
                        "Invalid format for --save. Use comma-separated numbers like: --save 1,3,7",
                    );
                }
            }
        }
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
    limit: Option<usize>,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };

    let cache_key = format!("{}_{}", library, query);

    // Try cache first if offline mode
    if offline || config.offline_mode {
        if let Some(doc) = cache_manager
            .get::<crate::client::Documentation>("docs", &cache_key)
            .await?
        {
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

    let doc_text = search_engine
        .get_documentation(library, if query.is_empty() { None } else { Some(query) })
        .await?;

    pb.finish_and_clear();

    // Cache documentation only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("docs", &cache_key, &doc_text).await.ok();
    }

    // Render documentation using the new Context7 parser
    renderer.render_context7_documentation_with_limit(library, &doc_text, limit)?;

    // Export if requested
    if let Some(path) = output {
        // For now, just write the raw text - we can improve this later
        std::fs::write(path, &doc_text)?;
        renderer.print_success(&format!("Documentation exported to {:?}", path));
    }

    Ok(())
}

async fn handle_snippet_command(
    id: &str,
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

    // We need to find which library this snippet belongs to by scanning the cache
    // Since we don't know which library was searched, we'll look through all cached snippets

    let pb = renderer.show_progress(&format!("Looking for snippet {}...", id));

    let mut found_snippet: Option<String> = None;
    let mut library_name = String::new();

    // Get the snippets cache directory and scan for the snippet ID
    let dummy_path = cache_manager.cache_key("snippets", "dummy");
    let snippets_cache_dir = dummy_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get snippets cache directory"))?;

    if snippets_cache_dir.exists() {
        // Read all cached snippet files and find the most recent one with the matching ID
        let mut matching_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(snippets_cache_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                if let Some(filename_str) = filename.to_str() {
                    // Snippet files are named like "libraryname_doc-1.json"
                    if filename_str.ends_with(&format!("{}.json", id)) {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                matching_files.push((filename_str.to_string(), modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time (most recent first)
        matching_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Try to load the most recent matching snippet
        for (filename, _) in matching_files {
            if let Some(underscore_pos) = filename.rfind('_') {
                library_name = filename[..underscore_pos].to_string();
                let snippet_key = format!("{}_{}", library_name, id);

                if let Ok(Some(content)) =
                    cache_manager.get::<String>("snippets", &snippet_key).await
                {
                    found_snippet = Some(content);
                    break;
                }
            }
        }
    }

    pb.finish_and_clear();

    match found_snippet {
        Some(content) => {
            // Render the snippet using the Context7 documentation parser
            renderer
                .render_context7_documentation(&format!("{} - {}", library_name, id), &content)?;

            // Export if requested
            if let Some(path) = output {
                std::fs::write(path, &content)?;
                renderer.print_success(&format!("Snippet exported to {:?}", path));
            }
        }
        None => {
            renderer.print_error(&format!(
                "Snippet '{}' not found in cache. Snippets are references to recent search results.", id
            ));
            renderer.print_success("Try running a search first, then use the ID from the results:");
            renderer.print_success("  manx fastapi          # Search FastAPI docs");
            renderer.print_success("  manx snippet doc-1    # Expand first result");

            if !offline && !config.offline_mode {
                renderer.print_success("You can also try searching again to refresh the cache.");
            }
        }
    }

    Ok(())
}

async fn handle_open_command(
    id: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };

    let pb = renderer.show_progress(&format!("Looking for section {}...", id));

    let mut found_section: Option<String> = None;
    let mut library_name = String::new();

    // Get the doc_sections cache directory and scan for the section ID
    let dummy_path = cache_manager.cache_key("doc_sections", "dummy");
    let sections_cache_dir = dummy_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to get doc_sections cache directory"))?;

    if sections_cache_dir.exists() {
        // Read all cached section files and find the most recent one with the matching ID
        let mut matching_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(sections_cache_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name();
                if let Some(filename_str) = filename.to_str() {
                    // Section files are named like "libraryname_doc-1.json"
                    if filename_str.ends_with(&format!("{}.json", id)) {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                matching_files.push((filename_str.to_string(), modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time (most recent first)
        matching_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Try to load the most recent matching section
        for (filename, _) in matching_files {
            if let Some(underscore_pos) = filename.rfind('_') {
                library_name = filename[..underscore_pos].to_string();
                let section_key = format!("{}_{}", library_name, id);

                if let Ok(Some(content)) = cache_manager
                    .get::<String>("doc_sections", &section_key)
                    .await
                {
                    found_section = Some(content);
                    break;
                }
            }
        }
    }

    pb.finish_and_clear();

    match found_section {
        Some(content) => {
            // Render the specific section
            renderer.render_open_section(&format!("{} - {}", library_name, id), &content)?;

            // Export if requested
            if let Some(path) = output {
                std::fs::write(path, &content)?;
                renderer.print_success(&format!("Section exported to {:?}", path));
            }
        }
        None => {
            renderer.print_error(&format!(
                "Section '{}' not found in cache. Doc sections are cached from recent 'doc' commands.", id
            ));
            renderer.print_success("Try running a doc command first, then use the section ID:");
            renderer.print_success("  manx doc react           # Get React documentation");
            renderer.print_success("  manx open doc-1          # Open first section");
        }
    }

    Ok(())
}

async fn handle_get_command(
    id: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    _offline: bool,
) -> Result<()> {
    let cache_manager = if let Some(dir) = &config.cache_dir {
        CacheManager::with_custom_dir(dir.clone())?
    } else {
        CacheManager::new()?
    };

    let pb = renderer.show_progress(&format!("Looking for item {}...", id));

    let mut found_content: Option<String> = None;
    let mut library_name = String::new();
    let mut content_type = String::new();

    // First, try to find in snippets cache (for doc-N IDs)
    if id.starts_with("doc-") {
        content_type = "snippet".to_string();
        let dummy_path = cache_manager.cache_key("snippets", "dummy");
        if let Some(snippets_cache_dir) = dummy_path.parent() {
            if snippets_cache_dir.exists() {
                let mut matching_files = Vec::new();

                if let Ok(entries) = std::fs::read_dir(snippets_cache_dir) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name();
                        if let Some(filename_str) = filename.to_str() {
                            if filename_str.ends_with(&format!("{}.json", id)) {
                                if let Ok(metadata) = entry.metadata() {
                                    if let Ok(modified) = metadata.modified() {
                                        matching_files.push((filename_str.to_string(), modified));
                                    }
                                }
                            }
                        }
                    }
                }

                matching_files.sort_by(|a, b| b.1.cmp(&a.1));

                for (filename, _) in matching_files {
                    if let Some(underscore_pos) = filename.rfind('_') {
                        library_name = filename[..underscore_pos].to_string();
                        let snippet_key = format!("{}_{}", library_name, id);

                        if let Ok(Some(content)) =
                            cache_manager.get::<String>("snippets", &snippet_key).await
                        {
                            found_content = Some(content);
                            break;
                        }
                    }
                }
            }
        }
    }

    // If not found in snippets, try doc_sections cache (for section-N IDs or doc-N fallback)
    if found_content.is_none() {
        content_type = "doc_section".to_string();
        let dummy_path = cache_manager.cache_key("doc_sections", "dummy");
        if let Some(sections_cache_dir) = dummy_path.parent() {
            if sections_cache_dir.exists() {
                let mut matching_files = Vec::new();

                if let Ok(entries) = std::fs::read_dir(sections_cache_dir) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name();
                        if let Some(filename_str) = filename.to_str() {
                            // Try both the original ID and with section- prefix
                            let id_variants = if id.starts_with("doc-") {
                                vec![id.to_string(), id.replace("doc-", "section-")]
                            } else if id.starts_with("section-") {
                                vec![id.to_string()]
                            } else {
                                vec![format!("section-{}", id)]
                            };

                            for variant in id_variants {
                                if filename_str.ends_with(&format!("{}.json", variant)) {
                                    if let Ok(metadata) = entry.metadata() {
                                        if let Ok(modified) = metadata.modified() {
                                            matching_files.push((filename_str.to_string(), modified, variant));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                matching_files.sort_by(|a, b| b.1.cmp(&a.1));

                for (filename, _, variant) in matching_files {
                    if let Some(underscore_pos) = filename.rfind('_') {
                        library_name = filename[..underscore_pos].to_string();
                        let section_key = format!("{}_{}", library_name, variant);

                        if let Ok(Some(content)) = cache_manager
                            .get::<String>("doc_sections", &section_key)
                            .await
                        {
                            found_content = Some(content);
                            break;
                        }
                    }
                }
            }
        }
    }

    pb.finish_and_clear();

    match found_content {
        Some(content) => {
            let title = format!("{} - {}", library_name, id);
            
            // Render based on content type
            if content_type == "snippet" {
                renderer.render_context7_documentation(&title, &content)?;
            } else {
                renderer.render_open_section(&title, &content)?;
            }

            // Export if requested
            if let Some(path) = output {
                std::fs::write(path, &content)?;
                renderer.print_success(&format!("Item exported to {:?}", path));
            }
        }
        None => {
            renderer.print_error(&format!(
                "Item '{}' not found in cache.", id
            ));
            renderer.print_success("💡 Available item types:");
            renderer.print_success("  • doc-N: Search result snippets (from 'manx snippet' commands)");
            renderer.print_success("  • section-N: Documentation sections (from 'manx doc' commands)");
            renderer.print_success("");
            renderer.print_success("📖 How to get items:");
            renderer.print_success("  manx snippet fastapi        # Search for snippets");
            renderer.print_success("  manx get doc-3               # Get snippet result");
            renderer.print_success("  manx doc react              # Browse documentation"); 
            renderer.print_success("  manx get section-5           # Get doc section");
        }
    }

    Ok(())
}
