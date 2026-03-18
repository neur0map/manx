use anyhow::Result;

use crate::config::Config;
use crate::wizard::{
    navigation::{WizardAction, WizardStep},
    prompts,
};

pub async fn show_and_test(config: &Config) -> Result<WizardAction> {
    println!();
    println!("Your manx is configured!");
    println!();

    // Show configuration summary
    show_config_summary(config);

    println!();
    println!("{}", "-".repeat(40));

    // Show final options - first ask if they want to test
    let should_test = crate::wizard::prompts::confirm_action(
        "Test configuration before finishing?",
        true,
    )?;

    if should_test {
        // Test configuration
        println!();
        println!("Testing configuration...");
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
        println!("Configuration tests completed!");
    }

    // Show next steps
    show_next_steps(config);

    // Use the navigation function for final navigation
    crate::wizard::navigation::show_navigation_options(&WizardStep::Summary, false)
}

fn show_config_summary(config: &Config) {
    println!("Configuration Summary:");
    println!();

    // Context7 API
    if config.api_key.is_some() {
        println!("  - [Enabled] Context7 API for official documentation");
    } else {
        println!("  - [Disabled] Context7 API (limited search without this)");
    }

    // Search Engine
    match &config.rag.embedding.provider {
        crate::rag::EmbeddingProvider::Hash => {
            println!("  - [Enabled] Hash search engine (fast keyword matching)");
        }
        crate::rag::EmbeddingProvider::Onnx(model) => {
            println!(
                "  - [Enabled] Neural search engine: {} (semantic understanding)",
                model
            );
        }
        _ => {
            println!("  - [Enabled] Custom search engine configured");
        }
    }

    // AI Features
    if config.has_llm_configured() {
        let provider_name = get_llm_provider_name(config);
        println!("  - [Enabled] AI features with {}", provider_name);
    } else {
        println!("  - [Disabled] AI features (raw docs only - still very useful!)");
    }
}

fn show_next_steps(config: &Config) {
    println!();
    println!("Try these commands:");

    if config.api_key.is_some() {
        println!("  manx snippet react hooks");
        println!("  manx search \"python async patterns\"");
        println!("  manx doc fastapi");
    } else {
        println!("  manx embedding download all-MiniLM-L6-v2");
        println!("  manx config --api-key <your-context7-key>");
        println!("  manx search \"local documentation\"");
    }

    println!();
    println!("Your config is saved to ~/.config/manx/config.json");
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
