use anyhow::Result;

use crate::config::Config;
use crate::rag::llm::LlmProvider;
use crate::wizard::{navigation::WizardAction, prompts, validators};

pub async fn setup(config: &mut Config) -> Result<WizardAction> {
    println!();
    println!("Enable AI features for comprehensive answers with explanations?");
    println!();
    println!("AI features provide:");
    println!("  * Detailed explanations with code examples");
    println!("  * Answers synthesized from multiple sources");
    println!("  * Source citations for verification");
    println!();
    println!("Without AI:");
    println!("  * Raw documentation snippets (still very useful!)");
    println!("  * Faster responses, no API costs");
    println!();

    let choices = vec![
        "Skip AI features (use basic search only)",
        "OpenAI (GPT models) - most popular",
        "Anthropic (Claude models) - this tool's creator",
        "Groq (fastest inference)",
        "Z.AI (GLM Coding Plan) - affordable, code-optimized",
        "I'll set this up later",
        "-- Navigation --",
        "<- Back to previous step",
        "x  Quit setup",
    ];

    let selection = prompts::select_option("Add AI features?", &choices);

    match selection {
        0 => {
            // Skip AI features
            println!();
            println!("AI features skipped - manx will work great without them!");
            println!("You can enable AI later with: manx config --openai-api <key>");
            Ok(WizardAction::Next)
        }
        1 => {
            // OpenAI
            if setup_openai(config)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip) // User cancelled API key input
            }
        }
        2 => {
            // Anthropic
            if setup_anthropic(config)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip)
            }
        }
        3 => {
            // Groq
            if setup_groq(config)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip)
            }
        }
        4 => {
            // Zai
            if setup_zai(config)? {
                Ok(WizardAction::Next)
            } else {
                Ok(WizardAction::Skip)
            }
        }
        5 => {
            // Skip for now
            println!();
            println!("AI setup deferred.");
            println!("Use 'manx config' to set up AI providers later.");
            Ok(WizardAction::Next)
        }
        6 => {
            // Separator - should not be selectable
            Ok(WizardAction::Next)
        }
        7 => {
            // Back
            Ok(WizardAction::Back)
        }
        8 => {
            // Quit
            Ok(WizardAction::Quit)
        }
        _ => Ok(WizardAction::Next),
    }
}

fn setup_openai(config: &mut Config) -> Result<bool> {
    println!();
    println!("Setting up OpenAI...");
    println!("Get your API key from: https://platform.openai.com/api-keys");
    println!();

    let api_key =
        prompts::get_input("Enter your OpenAI API key (or press Enter to skip)");

    if api_key.is_empty() {
        println!("OpenAI setup skipped.");
        return Ok(false);
    }

    if !validators::validate_api_key(&api_key, "OpenAI") {
        println!("Invalid API key format - should start with 'sk-'");
        return Ok(false);
    }

    // Select OpenAI model
    let model = select_openai_model()?;

    config.llm.openai_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::OpenAI;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("OpenAI configured!");
    println!("  Using {}", model);

    Ok(true)
}

fn setup_anthropic(config: &mut Config) -> Result<bool> {
    println!();
    println!("Setting up Anthropic...");
    println!("Get your API key from: https://console.anthropic.com/");
    println!();

    let api_key =
        prompts::get_input("Enter your Anthropic API key (or press Enter to skip)");

    if api_key.is_empty() {
        println!("Anthropic setup skipped.");
        return Ok(false);
    }

    if !validators::validate_api_key(&api_key, "Anthropic") {
        println!("Invalid API key format - should start with 'sk-ant-'");
        return Ok(false);
    }

    // Select Anthropic model
    let model = select_anthropic_model()?;

    config.llm.anthropic_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::Anthropic;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("Anthropic configured!");
    println!("  Using {}", model);

    Ok(true)
}

fn setup_groq(config: &mut Config) -> Result<bool> {
    println!();
    println!("Setting up Groq...");
    println!("Get your API key from: https://console.groq.com/");
    println!();

    let api_key =
        prompts::get_input("Enter your Groq API key (or press Enter to skip)");

    if api_key.is_empty() {
        println!("Groq setup skipped.");
        return Ok(false);
    }

    if !validators::validate_api_key(&api_key, "Groq") {
        println!("Invalid API key format - should start with 'gsk_'");
        return Ok(false);
    }

    // Select Groq model
    let model = select_groq_model()?;

    config.llm.groq_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::Groq;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("Groq configured!");
    println!("  Using {}", model);

    Ok(true)
}

fn select_openai_model() -> Result<String> {
    println!();
    println!("Choose OpenAI Model");
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

    let selection = prompts::select_option_owned("Select OpenAI model", &choices);

    Ok(models[selection].0.to_string())
}

fn select_anthropic_model() -> Result<String> {
    println!();
    println!("Choose Anthropic Model");
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

    let selection = prompts::select_option_owned("Select Anthropic model", &choices);

    Ok(models[selection].0.to_string())
}

fn select_groq_model() -> Result<String> {
    println!();
    println!("Choose Groq Model");
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

    let selection = prompts::select_option_owned("Select Groq model", &choices);

    Ok(models[selection].0.to_string())
}

fn setup_zai(config: &mut Config) -> Result<bool> {
    println!();
    println!("Setting up Z.AI GLM Coding Plan...");
    println!("Get your API key from: https://z.ai/model-api");
    println!();

    let api_key =
        prompts::get_input("Enter your Z.AI API key (or press Enter to skip)");

    if api_key.is_empty() {
        println!("Z.AI setup skipped.");
        return Ok(false);
    }

    if api_key.len() < 10 {
        println!("API key too short - please enter a valid key");
        return Ok(false);
    }

    // Select Zai model
    let model = select_zai_model()?;

    config.llm.zai_api_key = Some(api_key);
    config.llm.preferred_provider = LlmProvider::Zai;
    config.llm.model_name = Some(model.clone());

    println!();
    println!("Z.AI configured!");
    println!("  Using {}", model);

    Ok(true)
}

fn select_zai_model() -> Result<String> {
    println!();
    println!("Choose Z.AI Model");
    println!();

    let models = [
        ("glm-4.7", "Recommended - Best quality, optimized for code"),
        (
            "glm-4.5-air",
            "Lightweight - Faster response, good for quick tasks",
        ),
        (
            "glm-4-flash",
            "Ultra-fast - Minimal latency for simple queries",
        ),
    ];

    let choices: Vec<String> = models
        .iter()
        .map(|(model, desc)| format!("{} - {}", model, desc))
        .collect();

    let selection = prompts::select_option_owned("Select Z.AI model", &choices);

    Ok(models[selection].0.to_string())
}
