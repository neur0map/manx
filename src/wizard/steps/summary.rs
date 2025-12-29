use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;

use crate::config::Config;
use crate::wizard::{
    navigation::{WizardAction, WizardStep},
    prompts,
};

pub async fn show_and_test(config: &Config, theme: &ColorfulTheme) -> Result<WizardAction> {
    println!();
    println!("{}", style("Your manx is configured!").green().bold());
    println!();

    // Show configuration summary
    show_config_summary(config);

    println!();
    println!("{}", style("─".repeat(40)).dim());

    // Show final options - first ask if they want to test
    let should_test = crate::wizard::prompts::confirm_action(
        theme,
        "Test configuration before finishing?",
        true,
    )?;

    if should_test {
        // Test configuration
        println!();
        println!("{}", style("Testing configuration...").cyan());
        println!();

        // Test Context7 API if configured
        if let Some(api_key) = &config.api_key {
            let _result = prompts::test_with_spinner("Testing Context7 API", || {
                crate::wizard::validators::test_context7_api(api_key)
            })
            .await?;
        }

        // Test Embeddings
        let _result = prompts::test_with_spinner("Testing search engine", || {
            crate::wizard::validators::test_embedding_setup(&config.rag.embedding.provider)
        })
        .await?;

        // Test LLM if configured
        if config.has_llm_configured() {
            let provider_name = get_llm_provider_name(config);
            let api_key = get_llm_api_key(config);

            if let Some(key) = api_key {
                let _result =
                    prompts::test_with_spinner(&format!("Testing {} API", provider_name), || {
                        crate::wizard::validators::test_llm_api(provider_name, key)
                    })
                    .await?;
            }
        }

        println!();
        println!("{}", style("Configuration tests completed!").green().bold());
    }

    // Show next steps
    show_next_steps(config);

    // Use the navigation function for final navigation
    crate::wizard::navigation::show_navigation_options(theme, &WizardStep::Summary, false)
}

fn show_config_summary(config: &Config) {
    println!("{}", style("Configuration Summary:").bold());
    println!();

    // Context7 API
    if config.api_key.is_some() {
        println!(
            "  - {} Context7 API for official documentation",
            style("Enabled").green().bold()
        );
    } else {
        println!(
            "  - {} Context7 API (limited search without this)",
            style("Disabled").dim()
        );
    }

    // Search Engine
    match &config.rag.embedding.provider {
        crate::rag::EmbeddingProvider::Hash => {
            println!(
                "  - {} Hash search engine (fast keyword matching)",
                style("Enabled").green().bold()
            );
        }
        crate::rag::EmbeddingProvider::Onnx(model) => {
            println!(
                "  - {} Neural search engine: {} (semantic understanding)",
                style("Enabled").green().bold(),
                style(model).yellow()
            );
        }
        _ => {
            println!(
                "  - {} Custom search engine configured",
                style("Enabled").green().bold()
            );
        }
    }

    // AI Features
    if config.has_llm_configured() {
        let provider_name = get_llm_provider_name(config);
        println!(
            "  - {} AI features with {}",
            style("Enabled").green().bold(),
            provider_name
        );
    } else {
        println!(
            "  - {} AI features (raw docs only - still very useful!)",
            style("Disabled").dim()
        );
    }
}

fn show_next_steps(config: &Config) {
    println!();
    println!("{}", style("Try these commands:").cyan().bold());

    if config.api_key.is_some() {
        println!("  {} snippet react hooks", style("manx").bold().blue());
        println!(
            "  {} search \"python async patterns\"",
            style("manx").bold().blue()
        );
        println!("  {} doc fastapi", style("manx").bold().blue());
    } else {
        println!(
            "  {} embedding download all-MiniLM-L6-v2",
            style("manx").bold().blue()
        );
        println!(
            "  {} config --api-key <your-context7-key>",
            style("manx").bold().blue()
        );
        println!(
            "  {} search \"local documentation\"",
            style("manx").bold().blue()
        );
    }

    println!();
    println!(
        "{}",
        style("Your config is saved to ~/.config/manx/config.json").dim()
    );
}

fn get_llm_provider_name(config: &Config) -> &'static str {
    match config.llm.preferred_provider {
        crate::rag::llm::LlmProvider::OpenAI => "OpenAI",
        crate::rag::llm::LlmProvider::Anthropic => "Anthropic",
        crate::rag::llm::LlmProvider::Groq => "Groq",
        crate::rag::llm::LlmProvider::OpenRouter => "OpenRouter",
        crate::rag::llm::LlmProvider::HuggingFace => "HuggingFace",
        crate::rag::llm::LlmProvider::Zai => "Z.AI",
        crate::rag::llm::LlmProvider::Custom => "Custom",
        crate::rag::llm::LlmProvider::Auto => "Auto",
    }
}

fn get_llm_api_key(config: &Config) -> Option<&str> {
    match config.llm.preferred_provider {
        crate::rag::llm::LlmProvider::OpenAI => config.llm.openai_api_key.as_deref(),
        crate::rag::llm::LlmProvider::Anthropic => config.llm.anthropic_api_key.as_deref(),
        crate::rag::llm::LlmProvider::Groq => config.llm.groq_api_key.as_deref(),
        crate::rag::llm::LlmProvider::OpenRouter => config.llm.openrouter_api_key.as_deref(),
        crate::rag::llm::LlmProvider::HuggingFace => config.llm.huggingface_api_key.as_deref(),
        crate::rag::llm::LlmProvider::Zai => config.llm.zai_api_key.as_deref(),
        _ => None,
    }
}
