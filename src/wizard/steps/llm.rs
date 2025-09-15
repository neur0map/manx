use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;

use crate::config::Config;
use crate::rag::llm::LlmProvider;
use crate::wizard::{navigation::WizardAction, validators};

pub async fn setup(config: &mut Config, theme: &ColorfulTheme) -> Result<WizardAction> {
    println!();
    println!(
        "🤖 Enable {} for comprehensive answers with explanations?",
        style("AI features").bold()
    );
    println!();
    println!("{}:", style("AI features provide").green());
    println!("  • Detailed explanations with code examples");
    println!("  • Answers synthesized from multiple sources");
    println!("  • Source citations for verification");
    println!();
    println!("{}:", style("Without AI").dim());
    println!("  • Raw documentation snippets (still very useful!)");
    println!("  • Faster responses, no API costs");
    println!();

    let choices = vec![
        "Skip AI features (use basic search only)",
        "OpenAI (GPT models) - most popular",
        "Anthropic (Claude models) - this tool's creator",
        "Groq (fastest inference)",
        "I'll set this up later",
        "── Navigation ──",
        "← Back to previous step",
        "✕ Quit setup",
    ];

    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("Add AI features?")
        .items(&choices)
        .default(0) // Default to skip, don't assume users want AI
        .interact()?;

    match selection {
        0 => {
            // Skip AI features
            println!();
            println!(
                "{}",
                style("AI features skipped - manx will work great without them!").dim()
            );
            println!(
                "{}",
                style("💡 You can enable AI later with: manx config --openai-api <key>").dim()
            );
            Ok(WizardAction::Next)
        }
        1 => {
            // OpenAI
            if setup_openai(config, theme)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip) // User cancelled API key input
            }
        }
        2 => {
            // Anthropic
            if setup_anthropic(config, theme)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip)
            }
        }
        3 => {
            // Groq
            if setup_groq(config, theme)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip)
            }
        }
        4 => {
            // Skip for now
            println!();
            println!("{}", style("AI setup deferred.").dim());
            println!(
                "{}",
                style("Use 'manx config' to set up AI providers later.").dim()
            );
            Ok(WizardAction::Next)
        }
        5 => {
            // Separator - should not be selectable
            Ok(WizardAction::Next)
        }
        6 => {
            // Back
            Ok(WizardAction::Back)
        }
        7 => {
            // Quit
            Ok(WizardAction::Quit)
        }
        _ => Ok(WizardAction::Next),
    }
}

fn setup_openai(config: &mut Config, theme: &ColorfulTheme) -> Result<bool> {
    println!();
    println!("{}", style("Setting up OpenAI...").cyan());
    println!(
        "{}",
        style("Get your API key from: https://platform.openai.com/api-keys").dim()
    );
    println!();

    let api_key: String = dialoguer::Input::with_theme(theme)
        .with_prompt("Enter your OpenAI API key (or press Enter to skip)")
        .allow_empty(true)
        .validate_with(|input: &String| {
            if input.is_empty() {
                Ok(())
            } else if !validators::validate_api_key(input, "OpenAI") {
                Err("Invalid API key format - should start with 'sk-'")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    if api_key.is_empty() {
        println!("{}", style("OpenAI setup skipped.").dim());
        return Ok(false);
    }

    // Select OpenAI model
    let model = select_openai_model(theme)?;

    config.llm.openai_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::OpenAI;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("{}", style("✓ OpenAI configured!").green().bold());
    println!("{}", style(format!("  Using {}", model)).dim());

    Ok(true)
}

fn setup_anthropic(config: &mut Config, theme: &ColorfulTheme) -> Result<bool> {
    println!();
    println!("{}", style("Setting up Anthropic...").cyan());
    println!(
        "{}",
        style("Get your API key from: https://console.anthropic.com/").dim()
    );
    println!();

    let api_key: String = dialoguer::Input::with_theme(theme)
        .with_prompt("Enter your Anthropic API key (or press Enter to skip)")
        .allow_empty(true)
        .validate_with(|input: &String| {
            if input.is_empty() {
                Ok(())
            } else if !validators::validate_api_key(input, "Anthropic") {
                Err("Invalid API key format - should start with 'sk-ant-'")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    if api_key.is_empty() {
        println!("{}", style("Anthropic setup skipped.").dim());
        return Ok(false);
    }

    // Select Anthropic model
    let model = select_anthropic_model(theme)?;

    config.llm.anthropic_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::Anthropic;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("{}", style("✓ Anthropic configured!").green().bold());
    println!("{}", style(format!("  Using {}", model)).dim());

    Ok(true)
}

fn setup_groq(config: &mut Config, theme: &ColorfulTheme) -> Result<bool> {
    println!();
    println!("{}", style("Setting up Groq...").cyan());
    println!(
        "{}",
        style("Get your API key from: https://console.groq.com/").dim()
    );
    println!();

    let api_key: String = dialoguer::Input::with_theme(theme)
        .with_prompt("Enter your Groq API key (or press Enter to skip)")
        .allow_empty(true)
        .validate_with(|input: &String| {
            if input.is_empty() {
                Ok(())
            } else if !validators::validate_api_key(input, "Groq") {
                Err("Invalid API key format - should start with 'gsk_'")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    if api_key.is_empty() {
        println!("{}", style("Groq setup skipped.").dim());
        return Ok(false);
    }

    // Select Groq model
    let model = select_groq_model(theme)?;

    config.llm.groq_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::Groq;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("{}", style("✓ Groq configured!").green().bold());
    println!("{}", style(format!("  Using {}", model)).dim());

    Ok(true)
}

fn select_openai_model(theme: &ColorfulTheme) -> Result<String> {
    println!();
    println!("{}", style("🤖 Choose OpenAI Model").cyan().bold());
    println!();

    let models = [
        ("gpt-4o", "Most capable - Best for complex tasks"),
        (
            "gpt-4o-mini",
            "Recommended - Fast, cost-effective, great quality",
        ),
        ("gpt-4-turbo", "Previous generation - Still very capable"),
        ("gpt-3.5-turbo", "Budget option - Fast and cheap"),
    ];

    let choices: Vec<String> = models
        .iter()
        .map(|(model, desc)| format!("{} - {}", model, desc))
        .collect();

    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("Select OpenAI model")
        .items(&choices)
        .default(1) // Default to gpt-4o-mini (recommended)
        .interact()?;

    Ok(models[selection].0.to_string())
}

fn select_anthropic_model(theme: &ColorfulTheme) -> Result<String> {
    println!();
    println!("{}", style("🤖 Choose Anthropic Model").cyan().bold());
    println!();

    let models = [
        (
            "claude-3-5-sonnet-20241022",
            "Most capable - Best reasoning and code",
        ),
        (
            "claude-3-haiku-20240307",
            "Recommended - Fast, cost-effective",
        ),
        (
            "claude-3-sonnet-20240229",
            "Balanced - Good quality and speed",
        ),
        (
            "claude-3-opus-20240229",
            "Premium - Highest quality (expensive)",
        ),
    ];

    let choices: Vec<String> = models
        .iter()
        .map(|(model, desc)| format!("{} - {}", model, desc))
        .collect();

    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("Select Anthropic model")
        .items(&choices)
        .default(1) // Default to haiku (recommended)
        .interact()?;

    Ok(models[selection].0.to_string())
}

fn select_groq_model(theme: &ColorfulTheme) -> Result<String> {
    println!();
    println!("{}", style("🤖 Choose Groq Model").cyan().bold());
    println!();

    let models = [
        (
            "llama-3.1-8b-instant",
            "Recommended - Lightning fast, good quality",
        ),
        (
            "llama-3.1-70b-versatile",
            "More capable - Slower but better reasoning",
        ),
        (
            "llama-3.2-11b-vision-preview",
            "Vision capable - Can analyze images",
        ),
        ("mixtral-8x7b-32768", "Mixtral - Good for longer contexts"),
        (
            "qwen/qwen-2.5-72b-instruct",
            "Qwen - Excellent for coding tasks",
        ),
        (
            "qwen/qwen-2.5-32b-instruct",
            "Qwen 32B - Balanced performance",
        ),
    ];

    let choices: Vec<String> = models
        .iter()
        .map(|(model, desc)| format!("{} - {}", model, desc))
        .collect();

    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("Select Groq model")
        .items(&choices)
        .default(0) // Default to llama-3.1-8b-instant (recommended)
        .interact()?;

    Ok(models[selection].0.to_string())
}
