mod cache;
mod cli;
mod client;
mod config;
mod export;
mod rag;
mod render;
mod search;
mod update;
mod web_search;

use anyhow::{Context, Result};
use colored::{control, Colorize};
use std::process;

use crate::cache::CacheManager;
use crate::cli::{CacheCommands, Cli, Commands, EmbeddingCommands, SourceCommands};
use crate::client::Context7Client;
use crate::config::Config;
use crate::export::Exporter;
use crate::render::Renderer;
use crate::search::SearchEngine;
use crate::update::SelfUpdater;
use std::path::PathBuf;

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

    // Merge CLI arguments with config
    config.merge_with_cli(args.api_key, args.cache_dir, args.offline);

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
            openai_api,
            anthropic_api,
            groq_api,
            openrouter_api,
            huggingface_api,
            custom_endpoint,
            llm_provider,
            llm_model,
            llm_api,
            rag,
            add_official_domain,
            embedding_provider,
            embedding_api_key,
            embedding_model_path,
            embedding_dimension,
        }) => {
            if show {
                println!("{}", config.display());

                // Also show web search configuration if debug is enabled
                if args.debug {
                    let web_config = crate::web_search::WebSearchConfig::default();
                    let search_system =
                        match crate::web_search::DocumentationSearchSystem::new(web_config, None)
                            .await
                        {
                            Ok(system) => system,
                            Err(_) => {
                                renderer.print_error("Web search system not available");
                                return Ok(());
                            }
                        };

                    renderer.print_success("🔍 Web Search Configuration:");
                    println!("  Available: {}", search_system.is_available());
                    println!("  Configuration: {:?}", search_system.config());

                    // Show official domains for debugging
                    let official_sources =
                        crate::web_search::official_sources::OfficialSourceManager::new();
                    let domains = official_sources.get_official_domains();
                    println!("  Official domains: {} configured", domains.len());
                }
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

                if let Some(llm_key) = llm_api {
                    if llm_key.is_empty() {
                        config.set_llm_api_key("".to_string())?;
                        renderer.print_success("LLM API key cleared");
                    } else {
                        config.set_llm_api_key(llm_key)?;
                        renderer.print_success("LLM API key updated");
                    }
                    updated = true;
                }

                // Provider-specific API key configuration
                if let Some(key) = openai_api {
                    config.set_openai_api_key(key)?;
                    renderer.print_success("OpenAI API key updated");
                    updated = true;
                }

                if let Some(key) = anthropic_api {
                    config.set_anthropic_api_key(key)?;
                    renderer.print_success("Anthropic API key updated");
                    updated = true;
                }

                if let Some(key) = groq_api {
                    config.set_groq_api_key(key)?;
                    renderer.print_success("Groq API key updated");
                    updated = true;
                }

                if let Some(key) = openrouter_api {
                    config.set_openrouter_api_key(key)?;
                    renderer.print_success("OpenRouter API key updated");
                    updated = true;
                }

                if let Some(key) = huggingface_api {
                    config.set_huggingface_api_key(key)?;
                    renderer.print_success("HuggingFace API key updated");
                    updated = true;
                }

                if let Some(endpoint) = custom_endpoint {
                    config.set_custom_endpoint(endpoint)?;
                    renderer.print_success("Custom endpoint updated");
                    updated = true;
                }

                if let Some(provider) = llm_provider {
                    match config.set_llm_provider(provider.clone()) {
                        Ok(_) => {
                            renderer.print_success(&format!("LLM provider set to {}", provider));
                            updated = true;
                        }
                        Err(e) => {
                            renderer.print_error(&e.to_string());
                        }
                    }
                }

                if let Some(model) = llm_model {
                    config.set_llm_model(model.clone())?;
                    renderer.print_success(&format!("LLM model set to {}", model));
                    updated = true;
                }

                if let Some(rag_mode) = rag {
                    match rag_mode.as_str() {
                        "on" | "true" | "enable" => {
                            config.set_rag_enabled(true)?;
                            renderer.print_success("Local RAG system enabled");
                            updated = true;
                        }
                        "off" | "false" | "disable" => {
                            config.set_rag_enabled(false)?;
                            renderer.print_success("Local RAG system disabled");
                            updated = true;
                        }
                        _ => {
                            renderer.print_error(&format!(
                                "Invalid RAG mode '{}'. Use 'on' or 'off'",
                                rag_mode
                            ));
                        }
                    }
                }

                if let Some(domain) = add_official_domain {
                    // Demonstrate adding custom domain to official sources
                    let mut official_sources =
                        crate::web_search::official_sources::OfficialSourceManager::new();
                    official_sources.add_official_domain(
                        domain.clone(),
                        crate::web_search::official_sources::SourceTier::OfficialDocs,
                    );

                    renderer.print_success(&format!(
                        "Custom official domain '{}' added to web search priorities",
                        domain
                    ));
                    renderer.print_success("Domain added to current session");
                    updated = true;
                }

                // Embedding configuration
                if let Some(provider) = embedding_provider {
                    match config.set_embedding_provider(&provider) {
                        Ok(_) => {
                            renderer
                                .print_success(&format!("Embedding provider set to {}", provider));

                            // Detect and update dimension from the new provider
                            if let Err(e) = config.rag.embedding.detect_and_update_dimension().await
                            {
                                log::warn!("Could not detect dimension for new provider: {}", e);
                                println!("   ⚠️  Dimension will be detected on first use");
                            } else {
                                // Save the updated dimension
                                if let Err(e) = config.save() {
                                    log::warn!("Could not save updated dimension: {}", e);
                                } else {
                                    println!(
                                        "   ✅ Detected and updated dimension: {}",
                                        config.rag.embedding.dimension
                                    );
                                }
                            }

                            updated = true;
                        }
                        Err(e) => {
                            renderer.print_error(&e.to_string());
                        }
                    }
                }

                if let Some(key) = embedding_api_key {
                    config.set_embedding_api_key(key)?;
                    renderer.print_success("Embedding API key updated");
                    updated = true;
                }

                if let Some(path) = embedding_model_path {
                    config.set_embedding_model_path(path.clone())?;
                    renderer
                        .print_success(&format!("Embedding model path set to {}", path.display()));
                    updated = true;
                }

                if let Some(dimension) = embedding_dimension {
                    match config.set_embedding_dimension(dimension) {
                        Ok(_) => {
                            renderer.print_success(&format!(
                                "Embedding dimension set to {}",
                                dimension
                            ));
                            updated = true;
                        }
                        Err(e) => {
                            renderer.print_error(&e.to_string());
                        }
                    }
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
            no_llm,
            rag,
        }) => {
            if rag {
                handle_rag_doc_command(
                    &library,
                    &query,
                    output.as_ref(),
                    &config,
                    &renderer,
                    limit.as_ref(),
                    &no_llm,
                )
                .await?;
            } else {
                handle_doc_command(
                    &library,
                    &query,
                    output.as_ref(),
                    &config,
                    &renderer,
                    false,
                    limit,
                    no_llm,
                )
                .await?;
            }
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
            no_llm,
            rag,
        }) => {
            let query_str = query.unwrap_or_default();
            if rag {
                handle_rag_snippet_command(
                    &library,
                    &query_str,
                    output.as_ref(),
                    &config,
                    &renderer,
                    save.as_ref(),
                    &save_all,
                    &json,
                    limit.as_ref(),
                    &no_llm,
                )
                .await?;
            } else {
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
                    no_llm,
                )
                .await?;
            }
        }

        Some(Commands::Search {
            query,
            no_llm,
            output,
            limit,
            rag,
        }) => {
            if rag {
                handle_rag_search_command(
                    &query,
                    &no_llm,
                    output.as_ref(),
                    limit.as_ref(),
                    &config,
                    &renderer,
                )
                .await?;
            } else {
                handle_web_search_command(
                    &query,
                    no_llm,
                    output.as_ref(),
                    limit,
                    &config,
                    &renderer,
                )
                .await?;
            }
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

        Some(Commands::Index {
            path,
            id,
            crawl,
            max_depth,
            max_pages,
        }) => {
            handle_index_command(&path, id, crawl, max_depth, max_pages, &config, &renderer)
                .await?;
        }

        Some(Commands::Sources { command }) => {
            handle_sources_command(command, &config, &renderer).await?;
        }

        Some(Commands::Embedding { command }) => {
            handle_embedding_command(command, &mut config, &renderer).await?;
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
    no_llm: bool,
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

    // Initialize semantically-enhanced search engine for snippets
    let client = Context7Client::new(config.api_key.clone())?;
    let search_engine = match SearchEngine::with_embeddings(client).await {
        Ok(engine) => {
            let search_mode = if engine.has_embeddings() {
                format!(
                    "🧠 Searching {} with semantic matching for '{}'",
                    library, query
                )
            } else {
                format!(
                    "📝 Searching {} with text matching for '{}'",
                    library, query
                )
            };
            let pb = renderer.show_progress(&search_mode);
            pb.finish_and_clear();
            engine
        }
        Err(e) => {
            log::warn!(
                "Semantic embeddings initialization failed, using text-based search: {}",
                e
            );
            let pb =
                renderer.show_progress(&format!("📝 Searching {} for '{}'...", library, query));
            pb.finish_and_clear();
            SearchEngine::new(Context7Client::new(config.api_key.clone())?)
        }
    };

    let (mut results, library_title, library_id) = search_engine
        .search(library, query, Some(config.default_limit))
        .await?;

    // Also search local RAG system if enabled
    if config.rag.enabled {
        match crate::rag::RagSystem::new(config.rag.clone()).await {
            Ok(rag_system) => {
                match rag_system.search(query, Some(5)).await {
                    Ok(rag_results) => {
                        log::info!(
                            "Found {} additional results from local RAG system",
                            rag_results.len()
                        );
                        // Convert RAG results to SearchResult format and append
                        for rag_result in rag_results {
                            let search_result = crate::client::SearchResult {
                                id: format!("rag-{}", rag_result.id),
                                library: "Local".to_string(),
                                title: rag_result
                                    .title
                                    .unwrap_or_else(|| "Local Document".to_string()),
                                excerpt: rag_result.content,
                                url: None,
                                relevance_score: rag_result.score,
                            };
                            results.push(search_result);
                        }
                    }
                    Err(e) => log::warn!("RAG search failed: {}", e),
                }
            }
            Err(e) => log::warn!("Failed to initialize RAG system: {}", e),
        }
    }

    // Cache results only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("search", &cache_key, &results).await.ok();

        // Cache individual snippets for get command
        for result in &results {
            let snippet_key = format!("{}_{}", result.library, result.id);
            cache_manager
                .set("snippets", &snippet_key, &result.excerpt)
                .await
                .ok();
        }
    }

    // Apply LLM synthesis if configured and not disabled
    if config.should_use_llm(no_llm) && !results.is_empty() {
        println!("🤖 Synthesizing answer with AI...");

        // Convert search results to RAG format for LLM synthesis
        let rag_results: Vec<crate::rag::RagSearchResult> = results
            .iter()
            .take(5) // Use top 5 results for synthesis
            .map(|result| {
                use chrono::Utc;
                crate::rag::RagSearchResult {
                    id: result.id.clone(),
                    content: result.excerpt.clone(),
                    title: Some(result.title.clone()),
                    score: result.relevance_score,
                    source_path: std::path::PathBuf::from(&result.library),
                    source_type: crate::rag::SourceType::Curated,
                    section: None,
                    metadata: crate::rag::DocumentMetadata {
                        file_type: "snippet".to_string(),
                        size: result.excerpt.len() as u64,
                        modified: Utc::now(),
                        tags: vec![result.library.clone()],
                        language: None,
                    },
                }
            })
            .collect();

        // Initialize LLM client and synthesize answer
        match crate::rag::llm::LlmClient::new(config.llm.clone()) {
            Ok(llm_client) => {
                match llm_client.synthesize_answer(query, &rag_results).await {
                    Ok(synthesis) => {
                        println!("\n{}", "┌─────────────────────────────────────────┐".cyan());
                        println!(
                            "{} {} {}",
                            "│".cyan(),
                            "🤖 AI Summary".bold().cyan(),
                            "│".cyan()
                        );
                        println!("{}", "└─────────────────────────────────────────┘".cyan());

                        // Clean, colorized AI response
                        for line in synthesis.answer.lines() {
                            if line.trim().is_empty() {
                                println!();
                                continue;
                            }

                            let trimmed = line.trim();
                            if trimmed.starts_with("**Quick Answer**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Quick Answer**",
                                        &format!("{}", "❯ Quick Answer".bold().green())
                                    )
                                );
                            } else if trimmed.starts_with("**Key Points**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Key Points**",
                                        &format!("{}", "❯ Key Points".bold().blue())
                                    )
                                );
                            } else if trimmed.starts_with("**Code Example**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Code Example**",
                                        &format!("{}", "❯ Code Example".bold().magenta())
                                    )
                                );
                            } else if trimmed.starts_with("- ") {
                                // Bullet points in cyan
                                println!("  {}", trimmed.cyan());
                            } else if trimmed.starts_with("```") {
                                // Code blocks in yellow background
                                println!("  {}", trimmed.on_bright_black().yellow());
                            } else if trimmed.contains("[Source") {
                                // Lines with source citations in dim white
                                println!("  {}", trimmed.bright_white());
                            } else {
                                // Regular text in white
                                println!("  {}", trimmed.white());
                            }
                        }

                        if !synthesis.citations.is_empty() && synthesis.citations.len() <= 3 {
                            println!("\n  {} {}", "📖".dimmed(), "Sources used:".dimmed());
                            for citation in synthesis.citations.iter().take(3) {
                                println!("  {} {}", "•".dimmed(), citation.source_title.dimmed());
                            }
                        }
                        println!();
                    }
                    Err(e) => {
                        log::warn!("LLM synthesis failed: {}", e);
                        renderer.print_error(
                            "AI synthesis failed, showing search results only check API status ",
                        );
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to initialize LLM client: {}", e);
                renderer.print_error("Failed to initialize AI client");
            }
        }
    }

    // Add clear separation before search results
    if config.should_use_llm(no_llm) && !results.is_empty() {
        println!("{}", "┌─────────────────────────────────────────┐".blue());
        println!(
            "{} {} {}",
            "│".blue(),
            "📚 Detailed Results".bold().blue(),
            "│".blue()
        );
        println!("{}", "└─────────────────────────────────────────┘".blue());
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

#[allow(clippy::too_many_arguments)]
async fn handle_doc_command(
    library: &str,
    query: &str,
    output: Option<&std::path::PathBuf>,
    config: &Config,
    renderer: &Renderer,
    offline: bool,
    limit: Option<usize>,
    no_llm: bool,
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

    // Initialize semantically-enhanced search engine
    let client = Context7Client::new(config.api_key.clone())?;
    let search_engine = match SearchEngine::with_embeddings(client).await {
        Ok(engine) => {
            let search_mode = if engine.has_embeddings() {
                "🧠 Fetching documentation with semantic processing"
            } else {
                "📝 Fetching documentation (semantic embeddings unavailable)"
            };
            let pb = renderer.show_progress(search_mode);
            pb.finish_and_clear();
            engine
        }
        Err(e) => {
            log::warn!(
                "Semantic embeddings initialization failed, using text-based search: {}",
                e
            );
            let pb = renderer.show_progress(&format!("📝 Fetching {} documentation...", library));
            pb.finish_and_clear();
            SearchEngine::new(Context7Client::new(config.api_key.clone())?)
        }
    };

    let doc_text = search_engine
        .get_documentation(library, if query.is_empty() { None } else { Some(query) })
        .await?;

    // Cache documentation only if auto-caching is enabled
    if config.auto_cache_enabled {
        cache_manager.set("docs", &cache_key, &doc_text).await.ok();
    }

    // Apply LLM synthesis if configured and not disabled
    if config.should_use_llm(no_llm) && !doc_text.trim().is_empty() {
        println!("🤖 Synthesizing documentation with AI...");

        // Convert documentation to RAG format for LLM synthesis
        let doc_sections: Vec<crate::rag::RagSearchResult> = doc_text
            .split("\n\n")
            .filter(|section| !section.trim().is_empty())
            .take(5) // Use first 5 sections
            .enumerate()
            .map(|(i, section)| {
                use chrono::Utc;
                crate::rag::RagSearchResult {
                    id: format!("{}-section-{}", library, i + 1),
                    content: section.to_string(),
                    title: Some(format!("{} - Section {}", library, i + 1)),
                    score: 0.9, // High relevance since it's official documentation
                    source_path: std::path::PathBuf::from(library),
                    source_type: crate::rag::SourceType::Curated,
                    section: Some(format!("Section {}", i + 1)),
                    metadata: crate::rag::DocumentMetadata {
                        file_type: "documentation".to_string(),
                        size: section.len() as u64,
                        modified: Utc::now(),
                        tags: vec![library.to_string(), "documentation".to_string()],
                        language: Some("en".to_string()),
                    },
                }
            })
            .collect();

        let ai_query = if query.is_empty() {
            format!("What is {} and how do I use it?", library)
        } else {
            format!("{} in {}", query, library)
        };

        // Initialize LLM client and synthesize answer
        match crate::rag::llm::LlmClient::new(config.llm.clone()) {
            Ok(llm_client) => {
                match llm_client.synthesize_answer(&ai_query, &doc_sections).await {
                    Ok(synthesis) => {
                        println!("\n{}", "┌─────────────────────────────────────────┐".cyan());
                        println!(
                            "{} {} {}",
                            "│".cyan(),
                            "🤖 AI Summary".bold().cyan(),
                            "│".cyan()
                        );
                        println!("{}", "└─────────────────────────────────────────┘".cyan());

                        // Clean, colorized AI response
                        for line in synthesis.answer.lines() {
                            if line.trim().is_empty() {
                                println!();
                                continue;
                            }

                            let trimmed = line.trim();
                            if trimmed.starts_with("**Quick Answer**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Quick Answer**",
                                        &format!("{}", "❯ Quick Answer".bold().green())
                                    )
                                );
                            } else if trimmed.starts_with("**Key Points**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Key Points**",
                                        &format!("{}", "❯ Key Points".bold().blue())
                                    )
                                );
                            } else if trimmed.starts_with("**Code Example**") {
                                println!(
                                    "  {}",
                                    trimmed.replace(
                                        "**Code Example**",
                                        &format!("{}", "❯ Code Example".bold().magenta())
                                    )
                                );
                            } else if trimmed.starts_with("- ") {
                                println!("  {}", trimmed.cyan());
                            } else if trimmed.starts_with("```") {
                                println!("  {}", trimmed.on_bright_black().yellow());
                            } else if trimmed.contains("[Source") {
                                println!("  {}", trimmed.bright_white());
                            } else {
                                println!("  {}", trimmed.white());
                            }
                        }

                        if !synthesis.citations.is_empty() && synthesis.citations.len() <= 3 {
                            println!("\n  {} {}", "📖".dimmed(), "Sources used:".dimmed());
                            for citation in synthesis.citations.iter().take(3) {
                                println!("  {} {}", "•".dimmed(), citation.source_title.dimmed());
                            }
                        }
                        println!();
                    }
                    Err(e) => {
                        log::warn!("LLM synthesis failed: {}", e);
                        renderer.print_error("AI synthesis failed, showing documentation only");
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to initialize LLM client: {}", e);
                renderer.print_error("Failed to initialize AI client");
            }
        }

        // Add clear separation before documentation
        println!("{}", "┌─────────────────────────────────────────┐".blue());
        println!(
            "{} {} {}",
            "│".blue(),
            "📚 Full Documentation".bold().blue(),
            "│".blue()
        );
        println!("{}", "└─────────────────────────────────────────┘".blue());
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

    // Handle both old format (doc-N) and new format (library-doc-N) snippet IDs
    if id.starts_with("doc-") || id.contains("-doc-") {
        content_type = "snippet".to_string();
        let dummy_path = cache_manager.cache_key("snippets", "dummy");
        if let Some(snippets_cache_dir) = dummy_path.parent() {
            if snippets_cache_dir.exists() {
                let mut matching_files = Vec::new();

                // For new format IDs (library-doc-N), extract the actual doc ID
                let (target_library, doc_id) = if id.contains("-doc-") {
                    let parts: Vec<&str> = id.splitn(2, "-doc-").collect();
                    if parts.len() == 2 {
                        (Some(parts[0]), format!("doc-{}", parts[1]))
                    } else {
                        (None, id.to_string())
                    }
                } else {
                    // Old format (doc-N) - no library specified
                    (None, id.to_string())
                };

                if let Ok(entries) = std::fs::read_dir(snippets_cache_dir) {
                    for entry in entries.flatten() {
                        let filename = entry.file_name();
                        if let Some(filename_str) = filename.to_str() {
                            if filename_str.ends_with(&format!("{}.json", &doc_id)) {
                                // If library is specified, ensure it matches
                                if let Some(target_lib) = target_library {
                                    if !filename_str.starts_with(&format!("{}_", target_lib)) {
                                        continue; // Skip if library doesn't match
                                    }
                                }

                                if let Ok(metadata) = entry.metadata() {
                                    if let Ok(modified) = metadata.modified() {
                                        matching_files.push((filename_str.to_string(), modified));
                                    }
                                }
                            }
                        }
                    }
                }

                // Sort by modification time (most recent first) for old format
                // For new format with specific library, there should be only one match
                matching_files.sort_by(|a, b| b.1.cmp(&a.1));

                for (filename, _) in matching_files {
                    if let Some(underscore_pos) = filename.rfind('_') {
                        library_name = filename[..underscore_pos].to_string();
                        let snippet_key = format!("{}_{}", library_name, &doc_id);

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
                                            matching_files.push((
                                                filename_str.to_string(),
                                                modified,
                                                variant,
                                            ));
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
            // Try fallback: look for recent search results that might contain this ID
            let mut fallback_content: Option<String> = None;
            let mut fallback_title = String::new();

            // Check recent search caches for any results matching this ID
            let dummy_path = cache_manager.cache_key("search", "dummy");
            if let Some(search_cache_dir) = dummy_path.parent() {
                if search_cache_dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(search_cache_dir) {
                        for entry in entries.flatten() {
                            let filename = entry.file_name();
                            if let Some(filename_str) = filename.to_str() {
                                if filename_str.ends_with(".json") {
                                    // Extract cache key from filename
                                    let cache_key = filename_str.trim_end_matches(".json");

                                    // Try to get the cached search results
                                    if let Ok(Some(search_results)) = cache_manager
                                        .get::<Vec<crate::client::SearchResult>>(
                                            "search", cache_key,
                                        )
                                        .await
                                    {
                                        // Look for the matching ID in these results
                                        for result in search_results {
                                            if result.id == id
                                                || result.id.ends_with(&id)
                                                || id.ends_with(&result.id)
                                            {
                                                fallback_content = Some(result.excerpt);
                                                fallback_title = format!(
                                                    "{} - {}",
                                                    result.library, result.title
                                                );
                                                break;
                                            }
                                        }
                                        if fallback_content.is_some() {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(content) = fallback_content {
                // Found in recent search results, display it
                renderer.render_context7_documentation(&fallback_title, &content)?;

                // Also cache it for future use if auto-cache is enabled
                if config.auto_cache_enabled {
                    let snippet_key = format!("fallback_{}", id);
                    cache_manager
                        .set("snippets", &snippet_key, &content)
                        .await
                        .ok();
                }

                // Export if requested
                if let Some(path) = output {
                    std::fs::write(path, &content)?;
                    renderer.print_success(&format!("Item exported to {:?}", path));
                }
            } else {
                // Not found even in fallback, show error and help
                renderer.print_error(&format!("Item '{}' not found in cache.", id));
                renderer.print_success("💡 Available item types:");
                renderer.print_success(
                    "  • doc-N: Search result snippets (from 'manx snippet' commands)",
                );
                renderer.print_success(
                    "  • section-N: Documentation sections (from 'manx doc' commands)",
                );
                renderer.print_success("");
                renderer.print_success("📖 How to get items:");
                renderer.print_success("  manx snippet fastapi        # Search for snippets");
                renderer.print_success("  manx get doc-3               # Get snippet result");
                renderer.print_success("  manx doc react              # Browse documentation");
                renderer.print_success("  manx get section-5           # Get doc section");
            }
        }
    }

    Ok(())
}

/// Handle the index command for RAG document indexing
async fn handle_index_command(
    path_or_url: &str,
    _id: Option<String>,
    crawl: bool,
    max_depth: Option<u32>,
    max_pages: Option<u32>,
    config: &Config,
    renderer: &Renderer,
) -> Result<()> {
    use crate::rag::RagSystem;

    if !config.rag.enabled {
        renderer.print_error("Local RAG is not enabled.");
        renderer.print_success("To enable RAG indexing:");
        renderer.print_success("  1. Enable RAG: manx config --rag on");
        return Ok(());
    }

    // Determine if input is URL or file path
    let is_url = path_or_url.starts_with("http://") || path_or_url.starts_with("https://");

    let pb = if is_url {
        if crawl {
            let crawl_info = match (max_depth, max_pages) {
                (Some(depth), Some(pages)) => {
                    format!(" (max depth: {}, max pages: {})", depth, pages)
                }
                (Some(depth), None) => format!(" (max depth: {})", depth),
                (None, Some(pages)) => format!(" (max pages: {})", pages),
                (None, None) => " (deep crawl)".to_string(),
            };
            renderer.show_progress(&format!(
                "Deep crawling and indexing URL: {}{}",
                path_or_url, crawl_info
            ))
        } else {
            renderer.show_progress(&format!("Fetching and indexing URL: {}", path_or_url))
        }
    } else {
        renderer.show_progress(&format!("Indexing path: {}", path_or_url))
    };

    match RagSystem::new(config.rag.clone()).await {
        Ok(mut rag_system) => {
            let indexed_count = if is_url {
                if crawl {
                    // Index URL content with deep crawling
                    rag_system
                        .index_url_deep(path_or_url, max_depth, max_pages)
                        .await?
                } else {
                    // Index single URL content
                    rag_system.index_url(path_or_url).await?
                }
            } else {
                // Index local file or directory
                let path = std::path::PathBuf::from(path_or_url);
                if path.is_file() {
                    rag_system.index_document(path).await?
                } else if path.is_dir() {
                    rag_system.index_directory(path).await?
                } else {
                    pb.finish_and_clear();
                    return Err(anyhow::anyhow!(
                        "Path does not exist or is not accessible: {}",
                        path_or_url
                    ));
                }
            };

            pb.finish_and_clear();

            if indexed_count == 0 {
                if is_url {
                    renderer.print_error("No content was extracted from the URL. The page might be empty or contain unsupported content.");
                } else {
                    renderer.print_error("No documents were indexed. Make sure the path contains supported files (.md, .txt, .docx, .pdf)");
                }
            } else {
                let source_type = if is_url { "URL" } else { "path" };
                renderer.print_success(&format!(
                    "Successfully indexed {} document chunks from {}: {}",
                    indexed_count, source_type, path_or_url
                ));
            }

            // Show updated stats
            if let Ok(stats) = rag_system.get_stats().await {
                renderer.print_success(&format!(
                    "Total indexed: {} documents, {} chunks ({:.1} MB)",
                    stats.total_documents, stats.total_chunks, stats.index_size_mb
                ));
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("Failed to initialize RAG system: {}", e));
            renderer.print_success("Make sure Qdrant is running:");
            renderer.print_success("  docker run -p 6334:6334 qdrant/qdrant");
        }
    }

    Ok(())
}

/// Handle the sources command for managing RAG document sources
async fn handle_sources_command(
    command: SourceCommands,
    config: &Config,
    renderer: &Renderer,
) -> Result<()> {
    use crate::rag::RagSystem;

    match command {
        SourceCommands::List => {
            if !config.rag.enabled {
                renderer.print_error("Local RAG is not enabled.");
                return Ok(());
            }

            match RagSystem::new(config.rag.clone()).await {
                Ok(rag_system) => {
                    // Run health check first
                    match rag_system.health_check().await {
                        Ok(_) => log::info!("RAG system health check passed"),
                        Err(e) => {
                            renderer.print_error(&format!("RAG system health check failed: {}", e));
                            return Ok(());
                        }
                    }

                    match rag_system.get_stats().await {
                        Ok(stats) => {
                            if stats.total_documents == 0 {
                                renderer.print_success("No documents indexed yet.");
                                renderer.print_success("Use 'manx index <path>' to add documents.");
                            } else {
                                renderer.print_success(&format!(
                                    "📚 Indexed Sources ({} documents, {} chunks):",
                                    stats.total_documents, stats.total_chunks
                                ));

                                // Show sources if available
                                if !stats.sources.is_empty() {
                                    for (i, source) in stats.sources.iter().enumerate() {
                                        println!("  {}. {}", i + 1, source);
                                    }
                                } else {
                                    println!("  (Source details not available)");
                                }

                                println!("\n📊 Index Statistics:");
                                println!("  Size: {:.1} MB", stats.index_size_mb);
                                println!(
                                    "  Last Updated: {}",
                                    stats.last_updated.format("%Y-%m-%d %H:%M:%S")
                                );
                            }
                        }
                        Err(e) => {
                            renderer
                                .print_error(&format!("Failed to get source statistics: {}", e));
                        }
                    }
                }
                Err(e) => {
                    renderer.print_error(&format!("Failed to connect to RAG system: {}", e));
                    renderer.print_success("Make sure Qdrant is running and RAG is enabled.");
                }
            }
        }

        SourceCommands::Add { path, id: _id } => {
            handle_index_command(
                &path.to_string_lossy(),
                None,
                false,
                None,
                None,
                config,
                renderer,
            )
            .await?;
        }

        SourceCommands::Clear => {
            if !config.rag.enabled {
                renderer.print_error("Local RAG is not enabled.");
                return Ok(());
            }

            let pb = renderer.show_progress("Clearing all indexed documents...");

            match RagSystem::new(config.rag.clone()).await {
                Ok(rag_system) => match rag_system.clear_index().await {
                    Ok(_) => {
                        pb.finish_and_clear();
                        renderer.print_success("All indexed documents cleared successfully.");
                    }
                    Err(e) => {
                        pb.finish_and_clear();
                        renderer.print_error(&format!("Failed to clear index: {}", e));
                    }
                },
                Err(e) => {
                    pb.finish_and_clear();
                    renderer.print_error(&format!("Failed to connect to RAG system: {}", e));
                }
            }
        }
    }

    Ok(())
}

/// Smart text truncation with word boundary awareness
fn truncate_text(text: &str, max_length: usize, preserve_sentences: bool) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }

    if preserve_sentences {
        // Try to find a sentence boundary within the limit
        let truncation_point = text[..max_length]
            .rfind(". ")
            .or_else(|| text[..max_length].rfind("! "))
            .or_else(|| text[..max_length].rfind("? "))
            .map(|pos| pos + 1);

        if let Some(pos) = truncation_point {
            if pos > max_length / 2 {
                // Only use sentence boundary if it's not too short
                return format!("{}...", text[..pos].trim());
            }
        }
    }

    // Fallback to word boundary
    let truncation_point = text[..max_length].rfind(' ').unwrap_or(max_length);

    format!("{}...", text[..truncation_point].trim())
}

/// Handle embedding commands for managing semantic search models
async fn handle_embedding_command(
    command: EmbeddingCommands,
    config: &mut Config,
    renderer: &Renderer,
) -> Result<()> {
    use crate::rag::embeddings::EmbeddingModel;

    match command {
        EmbeddingCommands::Status => {
            renderer.print_success("🧠 Embedding Configuration:");
            println!("  Provider: {:?}", config.rag.embedding.provider);
            println!("  Dimension: {}", config.rag.embedding.dimension);

            if let Some(model_path) = &config.rag.embedding.model_path {
                println!("  Model Path: {}", model_path.display());
            }

            if config.rag.embedding.api_key.is_some() {
                println!("  API Key: ****");
            }

            if let Some(endpoint) = &config.rag.embedding.endpoint {
                println!("  Endpoint: {}", endpoint);
            }

            // Test if embedding model loads successfully and detect dimension
            println!("\n🔄 Testing embedding model...");
            match EmbeddingModel::new_with_config(config.rag.embedding.clone()).await {
                Ok(model) => {
                    // Test health check
                    match model.health_check().await {
                        Ok(()) => renderer.print_success("✅ Embedding model loads successfully"),
                        Err(e) => {
                            renderer.print_error(&format!("❌ Health check failed: {}", e));
                            return Ok(());
                        }
                    }

                    // Get actual dimension
                    match model.get_dimension().await {
                        Ok(dim) => {
                            if dim != config.rag.embedding.dimension {
                                println!(
                                    "   ⚠️  Actual dimension: {} (config shows: {})",
                                    dim, config.rag.embedding.dimension
                                );
                            } else {
                                println!("   ✅ Confirmed dimension: {}", dim);
                            }
                        }
                        Err(e) => {
                            println!("   ⚠️  Could not detect dimension: {}", e);
                        }
                    }

                    // Show provider info
                    let provider_info = model.get_provider_info();
                    println!("   Provider: {}", provider_info.name);
                    println!("   Type: {}", provider_info.provider_type);
                    if let Some(model_name) = &provider_info.model_name {
                        println!("   Model: {}", model_name);
                    }
                    if let Some(max_len) = provider_info.max_input_length {
                        println!("   Max input length: {}", max_len);
                    }
                    if !provider_info.description.is_empty() {
                        println!("   Description: {}", provider_info.description);
                    }

                    // Show current config
                    let current_config = model.get_config();
                    if let Some(endpoint) = &current_config.endpoint {
                        println!("   Configured endpoint: {}", endpoint);
                    }
                    if current_config.api_key.is_some() {
                        println!("   API key configured: Yes");
                    }
                }
                Err(e) => {
                    renderer.print_error(&format!("❌ Embedding model failed to load: {}", e));
                    println!("   Tip: Use 'manx embedding download <model>' for ONNX models");
                    println!("   Tip: Check API keys for cloud providers");
                }
            }
        }

        EmbeddingCommands::Set {
            provider,
            api_key,
            endpoint,
            dimension,
        } => {
            // Set provider
            match config.set_embedding_provider(&provider) {
                Ok(_) => {
                    renderer.print_success(&format!("Embedding provider set to: {}", provider));
                }
                Err(e) => {
                    renderer.print_error(&e.to_string());
                    return Ok(());
                }
            }

            // Set API key if provided
            if let Some(key) = api_key {
                config.set_embedding_api_key(key)?;
                renderer.print_success("Embedding API key updated");
            }

            // Set endpoint if provided
            if let Some(url) = endpoint {
                config.rag.embedding.endpoint = Some(url.clone());
                config.save()?;
                renderer.print_success(&format!("Embedding endpoint set to: {}", url));
            }

            // Set dimension if provided
            if let Some(dim) = dimension {
                match config.set_embedding_dimension(dim) {
                    Ok(_) => {
                        renderer.print_success(&format!("Embedding dimension set to: {}", dim));
                    }
                    Err(e) => {
                        renderer.print_error(&e.to_string());
                    }
                }
            }

            // Test new configuration
            println!("\n🔄 Testing new configuration...");
            match EmbeddingModel::new_with_config(config.rag.embedding.clone()).await {
                Ok(_) => {
                    renderer.print_success("✅ New embedding configuration works!");
                }
                Err(e) => {
                    renderer.print_error(&format!("❌ New configuration failed: {}", e));
                }
            }
        }

        EmbeddingCommands::Download { model, force } => {
            use crate::rag::providers::onnx::OnnxProvider;

            let pb = renderer.show_progress(&format!("Downloading model: {}", model));

            match OnnxProvider::download_model(&model, force).await {
                Ok(()) => {
                    pb.finish_and_clear();
                    renderer.print_success(&format!("✅ Successfully downloaded model: {}", model));

                    // Update config to detect and store the actual dimension
                    if let Err(e) = config.rag.embedding.detect_and_update_dimension().await {
                        log::warn!("Could not detect dimension: {}", e);
                    }
                }
                Err(e) => {
                    pb.finish_and_clear();
                    renderer.print_error(&format!("❌ Failed to download model: {}", e));
                }
            }
        }

        EmbeddingCommands::List { available } => {
            if available {
                use crate::rag::providers::onnx::OnnxProvider;

                renderer.print_success("🌐 Available Models for Download:");
                let models = OnnxProvider::list_available_models();
                for model in models {
                    println!("  • {}", model);
                }
                println!("\nUse 'manx embedding download <model-name>' to install.");
            } else {
                use crate::rag::model_metadata::ModelMetadataManager;

                match ModelMetadataManager::new() {
                    Ok(manager) => {
                        let models = manager.list_models();
                        if models.is_empty() {
                            renderer.print_success("💾 No models installed yet.");
                            println!("Use 'manx embedding download <model>' to install models.");
                        } else {
                            renderer
                                .print_success(&format!("💾 Installed Models ({}):", models.len()));
                            for model in models {
                                println!("\n  📦 {}", model.model_name);
                                println!("     Type: {}", model.provider_type);
                                println!("     Dimension: {}", model.dimension);
                                println!("     Size: {:.1} MB", model.size_mb);
                                println!(
                                    "     Installed: {}",
                                    model.installed_date.format("%Y-%m-%d %H:%M:%S")
                                );
                                if let Some(last_used) = &model.last_used {
                                    println!(
                                        "     Last used: {}",
                                        last_used.format("%Y-%m-%d %H:%M:%S")
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        renderer.print_error(&format!("Failed to list models: {}", e));
                    }
                }
            }
        }

        EmbeddingCommands::Test { query, verbose } => {
            println!("🔄 Testing embedding generation with query: '{}'", query);

            let start_time = std::time::Instant::now();

            match EmbeddingModel::new_with_config(config.rag.embedding.clone()).await {
                Ok(model) => {
                    // Show provider info
                    let provider_info = model.get_provider_info();
                    println!("🔧 Using provider: {}", provider_info.name);
                    if let Some(model_name) = &provider_info.model_name {
                        println!("🏷️  Model: {}", model_name);
                    }

                    match model.embed_text(&query).await {
                        Ok(embedding) => {
                            let duration = start_time.elapsed();
                            renderer.print_success(&format!(
                                "✅ Successfully generated embedding with {} dimensions in {:.2}ms",
                                embedding.len(),
                                duration.as_millis()
                            ));

                            // Update config with actual dimension if different
                            if let Ok(actual_dim) = model.get_dimension().await {
                                if actual_dim != config.rag.embedding.dimension {
                                    println!(
                                        "⚠️  Updating dimension: {} -> {}",
                                        config.rag.embedding.dimension, actual_dim
                                    );
                                    config.rag.embedding.dimension = actual_dim;
                                    let _ = config.save();
                                }
                            }

                            if verbose {
                                println!("\n📊 Embedding Vector (first 10 values):");
                                for (i, val) in embedding.iter().take(10).enumerate() {
                                    println!("  [{}]: {:.6}", i, val);
                                }
                                if embedding.len() > 10 {
                                    println!("  ... ({} more values)", embedding.len() - 10);
                                }

                                // Calculate vector statistics
                                let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
                                let mean: f32 =
                                    embedding.iter().sum::<f32>() / embedding.len() as f32;
                                let variance: f32 =
                                    embedding.iter().map(|x| (x - mean).powi(2)).sum::<f32>()
                                        / embedding.len() as f32;
                                let std_dev = variance.sqrt();

                                println!("\n📐 Vector Statistics:");
                                println!("  Norm: {:.6}", norm);
                                println!("  Mean: {:.6}", mean);
                                println!("  Std Dev: {:.6}", std_dev);
                                println!(
                                    "  Min: {:.6}",
                                    embedding.iter().fold(f32::INFINITY, |a, &b| a.min(b))
                                );
                                println!(
                                    "  Max: {:.6}",
                                    embedding.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
                                );
                            }

                            // Test similarity with itself (should be ~1.0)
                            if let Ok(embedding2) = model.embed_text(&query).await {
                                let similarity =
                                    EmbeddingModel::cosine_similarity(&embedding, &embedding2);
                                println!(
                                    "\n🔄 Self-similarity test: {:.6} (should be ~1.0)",
                                    similarity
                                );
                            }
                        }
                        Err(e) => {
                            renderer
                                .print_error(&format!("❌ Failed to generate embedding: {}", e));
                        }
                    }
                }
                Err(e) => {
                    renderer.print_error(&format!("❌ Failed to load embedding model: {}", e));

                    // Provide helpful error messages based on provider type
                    match &config.rag.embedding.provider {
                        crate::rag::EmbeddingProvider::Onnx(model) => {
                            println!("💡 Try: manx embedding download {}", model);
                        }
                        crate::rag::EmbeddingProvider::OpenAI(_) => {
                            println!("💡 Check: manx config --embedding-api-key <your-openai-key>");
                        }
                        crate::rag::EmbeddingProvider::HuggingFace(_) => {
                            println!("💡 Check: manx config --embedding-api-key <your-hf-key>");
                        }
                        crate::rag::EmbeddingProvider::Ollama(_) => {
                            println!("💡 Check: Is Ollama server running? (ollama serve)");
                        }
                        _ => {}
                    }
                }
            }
        }

        EmbeddingCommands::Remove { model } => {
            use crate::rag::model_metadata::ModelMetadataManager;

            match ModelMetadataManager::new() {
                Ok(mut manager) => {
                    if let Some(metadata) = manager.get_model(&model).cloned() {
                        // Confirm deletion
                        println!("⚠️  About to remove model: {}", model);
                        println!("   Type: {}", metadata.provider_type);
                        println!("   Size: {:.1} MB", metadata.size_mb);

                        // Simple confirmation (in a real CLI, you might use a proper prompt)
                        println!("\nAre you sure? Type 'yes' to confirm:");
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_ok() {
                            if input.trim().to_lowercase() == "yes" {
                                // Remove model files if they exist
                                if let Some(model_path) = &metadata.model_path {
                                    if model_path.exists() {
                                        if let Err(e) = std::fs::remove_dir_all(model_path) {
                                            renderer.print_error(&format!(
                                                "Failed to remove model files: {}",
                                                e
                                            ));
                                            return Ok(());
                                        }
                                    }
                                }

                                // Remove from metadata
                                if let Err(e) = manager.remove_model(&model) {
                                    renderer
                                        .print_error(&format!("Failed to update metadata: {}", e));
                                    return Ok(());
                                }

                                renderer.print_success(&format!(
                                    "✅ Successfully removed model: {}",
                                    model
                                ));
                            } else {
                                println!("❌ Cancelled.");
                            }
                        }
                    } else {
                        renderer.print_error(&format!("Model '{}' not found.", model));
                    }
                }
                Err(e) => {
                    renderer.print_error(&format!("Failed to access model metadata: {}", e));
                }
            }
        }
    }

    Ok(())
}

/// Handle web search command for official documentation
async fn handle_web_search_command(
    query: &str,
    no_llm: bool,
    output: Option<&PathBuf>,
    limit: Option<usize>,
    config: &Config,
    renderer: &render::Renderer,
) -> Result<()> {
    if query.trim().is_empty() {
        renderer.print_error("Search query cannot be empty");
        return Ok(());
    }

    // Initialize LLM config - auto-detect if API is configured
    // Only use LLM if: 1) API key is configured AND 2) user hasn't explicitly disabled it
    let llm_config = if config.should_use_llm(no_llm) {
        Some(config.llm.clone())
    } else {
        None
    };
    let will_use_llm = llm_config.is_some();

    // Show appropriate progress message based on LLM availability
    let search_mode = if will_use_llm {
        "🔍 Searching with AI verification"
    } else if config.has_llm_configured() {
        "🔍 Searching (LLM disabled by --no-llm)"
    } else {
        "🔍 Searching with semantic matching"
    };

    let pb = renderer.show_progress(&format!("{} for '{}'", search_mode, query));

    // Initialize web search configuration
    let mut web_search_config = web_search::WebSearchConfig::default();
    if let Some(max_results) = limit {
        web_search_config.max_results = max_results.min(20); // Cap at 20 for performance
    }
    let max_display_results = web_search_config.max_results;

    // Create web search system
    let mut search_system =
        match web_search::DocumentationSearchSystem::new(web_search_config, llm_config).await {
            Ok(system) => system,
            Err(e) => {
                pb.finish_and_clear();
                renderer.print_error(&format!("Failed to initialize search system: {}", e));
                return Ok(());
            }
        };

    // Perform search
    match search_system.search(query).await {
        Ok(response) => {
            pb.finish_and_clear();

            // Display search info
            if response.used_fallback {
                renderer.print_error(&format!(
                    "⚠️ Limited official results found ({}), expanded to trusted community sources",
                    response.official_results_count
                ));
            } else {
                renderer.print_success(&format!(
                    "✓ Found {} results from official documentation sources",
                    response.official_results_count
                ));
            }

            // Display results
            if response.results.is_empty() {
                renderer.print_error("No relevant documentation found");
                return Ok(());
            }

            // Apply LLM synthesis if configured and not disabled
            if config.should_use_llm(no_llm) && !response.results.is_empty() {
                println!("🤖 Synthesizing answer with AI...");

                // Convert search results to RAG format for LLM synthesis
                let rag_results: Vec<crate::rag::RagSearchResult> = response
                    .results
                    .iter()
                    .take(5) // Use top 5 results for synthesis
                    .map(|result| {
                        use chrono::Utc;
                        crate::rag::RagSearchResult {
                            id: result.url.clone(),
                            content: result.snippet.clone(),
                            title: Some(result.title.clone()),
                            score: result.similarity_score,
                            source_path: std::path::PathBuf::from(&result.url),
                            source_type: if result.is_official {
                                crate::rag::SourceType::Curated
                            } else {
                                crate::rag::SourceType::Remote
                            },
                            section: None,
                            metadata: crate::rag::DocumentMetadata {
                                file_type: "web".to_string(),
                                size: result.snippet.len() as u64,
                                modified: Utc::now(),
                                tags: vec![if result.is_official {
                                    "official".to_string()
                                } else {
                                    "community".to_string()
                                }],
                                language: Some("en".to_string()),
                            },
                        }
                    })
                    .collect();

                // Initialize LLM client and synthesize answer
                match crate::rag::llm::LlmClient::new(config.llm.clone()) {
                    Ok(llm_client) => {
                        match llm_client.synthesize_answer(query, &rag_results).await {
                            Ok(synthesis) => {
                                println!(
                                    "\n{}",
                                    "┌─────────────────────────────────────────┐".cyan()
                                );
                                println!(
                                    "{} {} {}",
                                    "│".cyan(),
                                    "🤖 AI Summary".bold().cyan(),
                                    "│".cyan()
                                );
                                println!(
                                    "{}",
                                    "└─────────────────────────────────────────┘".cyan()
                                );

                                // Clean, colorized AI response
                                for line in synthesis.answer.lines() {
                                    if line.trim().is_empty() {
                                        println!();
                                        continue;
                                    }

                                    let trimmed = line.trim();
                                    if trimmed.starts_with("**Quick Answer**") {
                                        println!(
                                            "  {}",
                                            trimmed.replace(
                                                "**Quick Answer**",
                                                &format!("{}", "❯ Quick Answer".bold().green())
                                            )
                                        );
                                    } else if trimmed.starts_with("**Key Points**") {
                                        println!(
                                            "  {}",
                                            trimmed.replace(
                                                "**Key Points**",
                                                &format!("{}", "❯ Key Points".bold().blue())
                                            )
                                        );
                                    } else if trimmed.starts_with("**Code Example**") {
                                        println!(
                                            "  {}",
                                            trimmed.replace(
                                                "**Code Example**",
                                                &format!("{}", "❯ Code Example".bold().magenta())
                                            )
                                        );
                                    } else if trimmed.starts_with("- ") {
                                        println!("  {}", trimmed.cyan());
                                    } else if trimmed.starts_with("```") {
                                        println!("  {}", trimmed.on_bright_black().yellow());
                                    } else if trimmed.contains("[Source") {
                                        println!("  {}", trimmed.bright_white());
                                    } else {
                                        println!("  {}", trimmed.white());
                                    }
                                }

                                if !synthesis.citations.is_empty() && synthesis.citations.len() <= 3
                                {
                                    println!("\n  {} {}", "📖".dimmed(), "Sources used:".dimmed());
                                    for citation in synthesis.citations.iter().take(3) {
                                        println!(
                                            "  {} {}",
                                            "•".dimmed(),
                                            citation.source_title.dimmed()
                                        );
                                    }
                                }
                                println!();
                            }
                            Err(e) => {
                                log::warn!("LLM synthesis failed: {}", e);
                                renderer.print_error(
                                    "AI synthesis failed, showing search results only check API status",
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to initialize LLM client: {}", e);
                        renderer.print_error("Failed to initialize AI client");
                    }
                }
            }

            // Add clear separation before search results
            if config.should_use_llm(no_llm) && !response.results.is_empty() {
                println!("{}", "┌─────────────────────────────────────────┐".blue());
                println!(
                    "{} {} {}",
                    "│".blue(),
                    "📚 Detailed Results".bold().blue(),
                    "│".blue()
                );
                println!("{}", "└─────────────────────────────────────────┘".blue());
            }

            // Show summary (truncated)
            println!("\n📝 Summary:");
            let summary = truncate_text(&response.summary, 150, true);
            println!("{}", summary);

            // Show top results
            println!("\n📚 Documentation Results:");
            for (i, result) in response
                .results
                .iter()
                .enumerate()
                .take(max_display_results)
            {
                // Truncate title if too long
                let title = truncate_text(&result.title, 80, false);
                println!("\n{}. {}", i + 1, title);
                println!("   🔗 {}", result.url);

                let source_indicator = if result.is_official {
                    "📋 Official Documentation"
                } else {
                    "🌐 Community Source"
                };
                println!(
                    "   {} • Relevance: {:.1}%",
                    source_indicator,
                    result.similarity_score * 100.0
                );

                // Show snippet (smart truncated)
                let snippet = truncate_text(&result.snippet, 120, true);
                println!("   {}", snippet);
            }

            // Show search stats
            println!("\n📊 Search Statistics:");
            println!("• Total found: {}", response.total_found);
            println!("• Official sources: {}", response.official_results_count);
            println!("• Search time: {}ms", response.search_time_ms);
            println!("• Sources: {}", response.sources.join(", "));

            // Export if requested
            if let Some(output_path) = output {
                // Convert response to JSON format for export
                let export_content = serde_json::to_string_pretty(&response)?;
                std::fs::write(output_path, export_content)
                    .context("Failed to write export file")?;
                renderer.print_success(&format!("Results exported to: {}", output_path.display()));
            }
        }

        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("Search failed: {}", e));

            // Provide helpful suggestions
            println!("💡 Search Tips:");
            println!("• Try more specific terms: 'react hooks useEffect' instead of 'react'");
            println!("• Check your internet connection");
            println!("• Use quotes for exact phrases: '\"memory management\"'");
        }
    }

    Ok(())
}

/// Handle RAG search command for searching locally indexed documents
async fn handle_rag_search_command(
    query: &str,
    no_llm: &bool,
    output: Option<&PathBuf>,
    limit: Option<&usize>,
    config: &Config,
    renderer: &render::Renderer,
) -> Result<()> {
    if query.trim().is_empty() {
        renderer.print_error("Search query cannot be empty");
        return Ok(());
    }

    // Check if RAG is enabled
    if !config.rag.enabled {
        renderer.print_error("RAG (local document search) is not enabled.");
        println!("💡 Enable with: manx config --rag-enabled");
        return Ok(());
    }

    let search_mode = if config.should_use_llm(*no_llm) {
        "🔍 Searching indexed documents with AI synthesis"
    } else {
        "🔍 Searching indexed documents with semantic matching"
    };

    let pb = renderer.show_progress(&format!("{} for '{}'", search_mode, query));

    // Initialize RAG system
    let rag_system = match crate::rag::RagSystem::new(config.rag.clone()).await {
        Ok(system) => system,
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("Failed to initialize RAG system: {}", e));
            println!("💡 Try indexing documents first: manx index /path/to/docs");
            return Ok(());
        }
    };

    // Perform search
    let max_results = limit.copied().unwrap_or(10);
    match rag_system.search(query, Some(max_results)).await {
        Ok(results) => {
            pb.finish_and_clear();

            if results.is_empty() {
                renderer.print_error("No relevant documents found in local index");
                println!("💡 Index more documents with: manx index /path/to/docs");
                return Ok(());
            }

            renderer.print_success(&format!(
                "✓ Found {} results from indexed documents",
                results.len()
            ));

            // Apply LLM synthesis if configured
            if config.should_use_llm(*no_llm) && !results.is_empty() {
                match synthesize_rag_results(query, &results, config, renderer).await {
                    Ok(synthesis) => {
                        println!("\n🤖 AI Analysis:");
                        println!("{}", synthesis.answer);

                        if !synthesis.citations.is_empty() {
                            println!("\n📖 Sources:");
                            for citation in synthesis.citations.iter().take(5) {
                                println!("  • {}", citation.source_title);
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("LLM synthesis failed: {}", e);
                        renderer.print_error(
                            "AI synthesis failed, showing search results only check API status",
                        );
                    }
                }
            }

            // Display results
            display_rag_results(&results, renderer);

            // Handle output if specified
            if let Some(output_path) = output {
                export_rag_results(&results, output_path, renderer)?;
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("RAG search failed: {}", e));
            println!("💡 Try: manx config --rag-enabled or index documents with: manx index /path");
        }
    }

    Ok(())
}

/// Handle RAG snippet command for searching locally indexed documents with library focus
#[allow(clippy::too_many_arguments)]
async fn handle_rag_snippet_command(
    library: &str,
    query: &str,
    output: Option<&PathBuf>,
    config: &Config,
    renderer: &render::Renderer,
    save: Option<&String>,
    save_all: &bool,
    json: &bool,
    limit: Option<&usize>,
    no_llm: &bool,
) -> Result<()> {
    if query.trim().is_empty() {
        renderer.print_error("Search query cannot be empty");
        return Ok(());
    }

    // Check if RAG is enabled
    if !config.rag.enabled {
        renderer.print_error("RAG (local document search) is not enabled.");
        println!("💡 Enable with: manx config --rag-enabled");
        return Ok(());
    }

    // Combine library and query for focused search
    let focused_query = if !library.is_empty() {
        format!("{} {}", library, query)
    } else {
        query.to_string()
    };

    let search_mode = if config.should_use_llm(*no_llm) {
        "🔍 Finding code snippets with AI understanding"
    } else {
        "🔍 Finding code snippets with semantic matching"
    };

    let pb = renderer.show_progress(&format!("{} for '{}'", search_mode, focused_query));

    // Initialize RAG system
    let rag_system = match crate::rag::RagSystem::new(config.rag.clone()).await {
        Ok(system) => system,
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("Failed to initialize RAG system: {}", e));
            return Ok(());
        }
    };

    // Perform search
    let max_results = limit.copied().unwrap_or(10);
    match rag_system.search(&focused_query, Some(max_results)).await {
        Ok(results) => {
            pb.finish_and_clear();

            if results.is_empty() {
                renderer.print_error(&format!(
                    "No code snippets found for '{}' in indexed documents",
                    library
                ));
                println!("💡 Try: manx index /path/to/code");
                return Ok(());
            }

            renderer.print_success(&format!(
                "✓ Found {} snippets for {}",
                results.len(),
                library
            ));

            // Apply LLM synthesis if configured
            if config.should_use_llm(*no_llm) && !results.is_empty() {
                match synthesize_rag_results(&focused_query, &results, config, renderer).await {
                    Ok(synthesis) => {
                        println!("\n🤖 Code Analysis:");
                        println!("{}", synthesis.answer);
                    }
                    Err(e) => {
                        log::warn!("LLM synthesis failed: {}", e);
                    }
                }
            }

            // Display snippet results
            display_rag_snippet_results(&results, library, renderer);

            // Handle saving and export
            handle_snippet_save_and_export(&results, save, save_all, json, output, renderer)?;
        }
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("RAG snippet search failed: {}", e));
        }
    }

    Ok(())
}

/// Handle RAG doc command for searching locally indexed documents
async fn handle_rag_doc_command(
    library: &str,
    query: &str,
    output: Option<&PathBuf>,
    config: &Config,
    renderer: &render::Renderer,
    limit: Option<&usize>,
    no_llm: &bool,
) -> Result<()> {
    // Check if RAG is enabled
    if !config.rag.enabled {
        renderer.print_error("RAG (local document search) is not enabled.");
        println!("💡 Enable with: manx config --rag-enabled");
        return Ok(());
    }

    let search_query = if !query.is_empty() {
        format!("{} {}", library, query)
    } else {
        library.to_string()
    };

    let search_mode = if config.should_use_llm(*no_llm) {
        "📚 Finding documentation with AI understanding"
    } else {
        "📚 Finding documentation with semantic matching"
    };

    let pb = renderer.show_progress(&format!("{} for '{}'", search_mode, search_query));

    // Initialize RAG system
    let rag_system = match crate::rag::RagSystem::new(config.rag.clone()).await {
        Ok(system) => system,
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("Failed to initialize RAG system: {}", e));
            return Ok(());
        }
    };

    // Perform search
    let max_results = limit.copied().unwrap_or(10);
    match rag_system.search(&search_query, Some(max_results)).await {
        Ok(results) => {
            pb.finish_and_clear();

            if results.is_empty() {
                renderer.print_error(&format!(
                    "No documentation found for '{}' in indexed documents",
                    library
                ));
                return Ok(());
            }

            renderer.print_success(&format!(
                "✓ Found {} documentation sections for {}",
                results.len(),
                library
            ));

            // Apply LLM synthesis if configured
            if config.should_use_llm(*no_llm) && !results.is_empty() {
                match synthesize_rag_results(&search_query, &results, config, renderer).await {
                    Ok(synthesis) => {
                        println!("\n🤖 Documentation Summary:");
                        println!("{}", synthesis.answer);
                    }
                    Err(e) => {
                        log::warn!("LLM synthesis failed: {}", e);
                    }
                }
            }

            // Display documentation results
            display_rag_doc_results(&results, library, renderer);

            // Handle output if specified
            if let Some(output_path) = output {
                export_rag_results(&results, output_path, renderer)?;
            }
        }
        Err(e) => {
            pb.finish_and_clear();
            renderer.print_error(&format!("RAG doc search failed: {}", e));
        }
    }

    Ok(())
}

/// Synthesize RAG results using LLM
async fn synthesize_rag_results(
    query: &str,
    results: &[crate::rag::RagSearchResult],
    config: &Config,
    _renderer: &render::Renderer,
) -> Result<crate::rag::llm::LlmResponse> {
    let llm_client = crate::rag::llm::LlmClient::new(config.llm.clone())?;
    llm_client.synthesize_answer(query, results).await
}

/// Display RAG search results
fn display_rag_results(results: &[crate::rag::RagSearchResult], _renderer: &render::Renderer) {
    println!("\n📄 Local Document Results:");
    for (i, result) in results.iter().enumerate() {
        println!(
            "\n{}. {} (Score: {:.2})",
            i + 1,
            result.title.as_deref().unwrap_or("Untitled"),
            result.score
        );
        println!("   📁 {}", result.source_path.display());

        let preview = if result.content.len() > 150 {
            format!("{}...", &result.content[..150])
        } else {
            result.content.clone()
        };
        println!("   {}", preview);
    }
}

/// Display RAG snippet results with code focus
fn display_rag_snippet_results(
    results: &[crate::rag::RagSearchResult],
    library: &str,
    _renderer: &render::Renderer,
) {
    println!("\n💻 Code Snippets for {}:", library);
    for (i, result) in results.iter().enumerate() {
        println!(
            "\n{}. {} (Score: {:.2})",
            i + 1,
            result.title.as_deref().unwrap_or("Code Snippet"),
            result.score
        );
        println!("   📁 {}", result.source_path.display());

        // Show code content with some formatting
        println!("   ```");
        let lines: Vec<&str> = result.content.lines().take(8).collect();
        for line in lines {
            println!("   {}", line);
        }
        if result.content.lines().count() > 8 {
            println!("   ... (truncated)");
        }
        println!("   ```");
    }
}

/// Display RAG documentation results
fn display_rag_doc_results(
    results: &[crate::rag::RagSearchResult],
    library: &str,
    _renderer: &render::Renderer,
) {
    println!("\n📖 Documentation for {}:", library);
    for (i, result) in results.iter().enumerate() {
        println!(
            "\n{}. {} (Score: {:.2})",
            i + 1,
            result.title.as_deref().unwrap_or("Documentation"),
            result.score
        );
        println!("   📁 {}", result.source_path.display());

        let preview = if result.content.len() > 200 {
            format!("{}...", &result.content[..200])
        } else {
            result.content.clone()
        };
        println!("   {}", preview);
    }
}

/// Export RAG results to file
fn export_rag_results(
    results: &[crate::rag::RagSearchResult],
    output_path: &PathBuf,
    renderer: &render::Renderer,
) -> Result<()> {
    let export_content = if output_path.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::to_string_pretty(results)?
    } else {
        // Export as markdown
        let mut content = String::new();
        content.push_str("# Local Document Search Results\n\n");

        for (i, result) in results.iter().enumerate() {
            content.push_str(&format!(
                "## {}. {}\n\n",
                i + 1,
                result.title.as_deref().unwrap_or("Untitled")
            ));
            content.push_str(&format!("**Source:** `{}`\n", result.source_path.display()));
            content.push_str(&format!("**Score:** {:.3}\n\n", result.score));
            content.push_str(&result.content);
            content.push_str("\n\n---\n\n");
        }
        content
    };

    std::fs::write(output_path, export_content).context("Failed to write export file")?;
    renderer.print_success(&format!("Results exported to: {}", output_path.display()));

    Ok(())
}

/// Handle snippet saving and export functionality
fn handle_snippet_save_and_export(
    results: &[crate::rag::RagSearchResult],
    save: Option<&String>,
    save_all: &bool,
    json: &bool,
    output: Option<&PathBuf>,
    renderer: &render::Renderer,
) -> Result<()> {
    // Handle individual saves
    if let Some(save_indices) = save {
        let indices: Result<Vec<usize>, _> = save_indices
            .split(',')
            .map(|s| s.trim().parse::<usize>().map(|i| i.saturating_sub(1)))
            .collect();

        match indices {
            Ok(indices) => {
                for &idx in &indices {
                    if let Some(result) = results.get(idx) {
                        let filename = format!(
                            "snippet_{}_{}.{}",
                            idx + 1,
                            result
                                .source_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("snippet"),
                            if *json { "json" } else { "md" }
                        );

                        let content = if *json {
                            serde_json::to_string_pretty(result)?
                        } else {
                            format!(
                                "# {}\n\n**Source:** `{}`\n\n{}",
                                result.title.as_deref().unwrap_or("Code Snippet"),
                                result.source_path.display(),
                                result.content
                            )
                        };

                        std::fs::write(&filename, content)?;
                        renderer.print_success(&format!("Saved snippet to: {}", filename));
                    }
                }
            }
            Err(e) => {
                renderer.print_error(&format!("Invalid save indices: {}", e));
            }
        }
    }

    // Handle save all
    if *save_all {
        let filename = format!("all_snippets.{}", if *json { "json" } else { "md" });
        let content = if *json {
            serde_json::to_string_pretty(results)?
        } else {
            let mut content = String::new();
            content.push_str("# All Code Snippets\n\n");
            for (i, result) in results.iter().enumerate() {
                content.push_str(&format!(
                    "## {}. {}\n\n",
                    i + 1,
                    result.title.as_deref().unwrap_or("Code Snippet")
                ));
                content.push_str(&format!(
                    "**Source:** `{}`\n\n",
                    result.source_path.display()
                ));
                content.push_str(&result.content);
                content.push_str("\n\n---\n\n");
            }
            content
        };

        std::fs::write(&filename, content)?;
        renderer.print_success(&format!("Saved all snippets to: {}", filename));
    }

    // Handle general output
    if let Some(output_path) = output {
        export_rag_results(results, output_path, renderer)?;
    }

    Ok(())
}
