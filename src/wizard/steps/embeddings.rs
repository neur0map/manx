use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use spinoff::{spinners, Color, Spinner};

use crate::config::Config;
use crate::rag::EmbeddingProvider;
use crate::wizard::navigation::WizardAction;

pub async fn setup(config: &mut Config, theme: &ColorfulTheme) -> Result<WizardAction> {
    println!();
    println!(
        "🔍 Choose your {} - this affects how well manx understands your searches:",
        style("search engine").bold()
    );
    println!();

    println!("{}:", style("Hash Search (Default)").cyan());
    println!("  ✓ Works immediately, no setup required");
    println!("  ✓ Fast and reliable");
    println!("  ✓ Perfect for exact keyword matching");
    println!();

    println!("{}:", style("Neural Search (Recommended)").green());
    println!("  ✓ Understands meaning: \"auth\" finds \"authentication\"");
    println!("  ✓ Better results for complex queries");
    println!("  ✓ Small download (~22MB), runs locally");
    println!();

    let choices = vec![
        "Hash Search (fast, no download)",
        "Neural Search (download small model ~22MB)",
        "Keep current setting",
        "── Navigation ──",
        "← Back to previous step",
        "✕ Quit setup",
    ];

    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("Select search engine")
        .items(&choices)
        .default(1) // Default to neural
        .interact()?;

    match selection {
        0 => {
            // Hash
            config.rag.embedding.provider = EmbeddingProvider::Hash;
            config.rag.embedding.dimension = 384;
            println!();
            println!("{}", style("✓ Hash search configured!").green().bold());
            println!("{}", style("  Fast keyword search ready to use.").dim());
            Ok(WizardAction::Next)
        }
        1 => {
            // Neural - show model selection
            match select_neural_model(config, theme).await? {
                WizardAction::Next => Ok(WizardAction::Next),
                WizardAction::Back => Ok(WizardAction::Skip), // Go back to main menu
                WizardAction::Quit => Ok(WizardAction::Quit),
                _ => Ok(WizardAction::Next),
            }
        }
        2 => {
            // Keep current
            println!();
            println!("{}", style("Keeping current search settings.").dim());
            Ok(WizardAction::Next)
        }
        3 => {
            // Separator - should not be selectable
            Ok(WizardAction::Next)
        }
        4 => {
            // Back
            Ok(WizardAction::Back)
        }
        5 => {
            // Quit
            Ok(WizardAction::Quit)
        }
        _ => Ok(WizardAction::Next),
    }
}

async fn select_neural_model(config: &mut Config, theme: &ColorfulTheme) -> Result<WizardAction> {
    use crate::rag::providers::onnx::OnnxProvider;

    loop {
        println!();
        println!("{}", style("🧠 Choose Neural Search Model").cyan().bold());
        println!();
        println!("Available models (runs locally, no data sent to external servers):");
        println!();

        // Get available models from the ONNX provider
        let available_models = OnnxProvider::list_available_models();

        // Create model choices with descriptions
        let model_descriptions = vec![
            (
                "sentence-transformers/all-MiniLM-L6-v2",
                "Recommended - Fast, small (~22MB), great quality",
            ),
            (
                "sentence-transformers/all-mpnet-base-v2",
                "Higher quality - Larger (~120MB), best results",
            ),
            (
                "sentence-transformers/multi-qa-MiniLM-L6-cos-v1",
                "Q&A optimized - Great for questions (~22MB)",
            ),
            (
                "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2",
                "Multilingual - Supports many languages (~110MB)",
            ),
            (
                "BAAI/bge-small-en-v1.5",
                "BGE Small - Fast Chinese/English model (~33MB)",
            ),
            (
                "BAAI/bge-base-en-v1.5",
                "BGE Base - Balanced performance (~110MB)",
            ),
            (
                "BAAI/bge-large-en-v1.5",
                "BGE Large - Best quality, slower (~330MB)",
            ),
        ];

        let mut choices = Vec::new();
        for (model, desc) in &model_descriptions {
            if available_models.contains(model) {
                choices.push(format!(
                    "{} - {}",
                    model.split('/').next_back().unwrap_or(model),
                    desc
                ));
            }
        }

        // Add navigation options
        choices.push("── Navigation ──".to_string());
        choices.push("← Back to search engine selection".to_string());
        choices.push("✕ Quit setup".to_string());

        let selection = dialoguer::Select::with_theme(theme)
            .with_prompt("Select neural search model")
            .items(&choices)
            .default(0) // Default to all-MiniLM-L6-v2 (recommended)
            .interact()?;

        let model_count = model_descriptions.len();

        match selection {
            i if i < model_count => {
                // User selected a model
                let (selected_model, _) = model_descriptions[i];

                println!();
                println!("Selected: {}", style(selected_model).yellow().bold());

                // Show confirmation with options
                let confirm_choices = vec![
                    "Download and configure this model",
                    "Choose a different model",
                    "← Back to search engine selection",
                ];

                let confirm_selection = dialoguer::Select::with_theme(theme)
                    .with_prompt("Proceed with this model?")
                    .items(&confirm_choices)
                    .default(0)
                    .interact()?;

                match confirm_selection {
                    0 => {
                        // Download and configure
                        download_and_configure_model(config, selected_model).await?;
                        return Ok(WizardAction::Next);
                    }
                    1 => {
                        // Choose different model - continue loop
                        continue;
                    }
                    2 => {
                        // Back to search engine selection
                        return Ok(WizardAction::Back);
                    }
                    _ => continue,
                }
            }
            i if i == model_count => {
                // Separator - should not be selectable
                continue;
            }
            i if i == model_count + 1 => {
                // Back to search engine selection
                return Ok(WizardAction::Back);
            }
            i if i == model_count + 2 => {
                // Quit
                return Ok(WizardAction::Quit);
            }
            _ => continue,
        }
    }
}

async fn download_and_configure_model(config: &mut Config, model_name: &str) -> Result<()> {
    println!();
    println!(
        "{}",
        style("📥 Preparing to download neural search model")
            .cyan()
            .bold()
    );

    // Get the appropriate size description for the model
    let size_desc = get_model_size_description(model_name);

    println!();
    println!(
        "  {} {}",
        style("Model:").dim(),
        style(model_name).yellow().bold()
    );
    println!("  {} {}", style("Size:").dim(), style(size_desc).yellow());
    println!(
        "  {} Downloaded to local storage (no data sent to external servers)",
        style("Privacy:").dim()
    );
    println!();

    // Show what files will be downloaded
    println!("{}", style("Downloading 3 files from HuggingFace:").cyan());
    println!("  {} model.onnx (main neural network)", style("1.").dim());
    println!("  {} tokenizer.json (text processing)", style("2.").dim());
    println!("  {} config.json (model configuration)", style("3.").dim());
    println!();

    let mut spinner = Spinner::new(
        spinners::Dots12,
        format!("Downloading {} {}...", model_name, size_desc),
        Color::Cyan,
    );

    // Use the ONNX provider to download
    use crate::rag::providers::onnx::OnnxProvider;

    match OnnxProvider::download_model(model_name, false).await {
        Ok(()) => {
            spinner.success(&format!(
                "{} Neural search model installed successfully!",
                style("✓").green().bold()
            ));

            // Configure the provider
            config.rag.embedding.provider = EmbeddingProvider::Onnx(model_name.to_string());

            // Get the model path from metadata
            use crate::rag::model_metadata::ModelMetadataManager;
            if let Ok(manager) = ModelMetadataManager::new() {
                if let Some(metadata) = manager.get_model(model_name) {
                    config.rag.embedding.model_path = metadata.model_path.clone();

                    // Show installation details
                    println!();
                    println!("{}", style("📊 Installation Summary:").green().bold());
                    println!("  {} {}", style("Model:").dim(), style(model_name).cyan());
                    println!(
                        "  {} {}D",
                        style("Embedding dimension:").dim(),
                        style(metadata.dimension.to_string()).cyan()
                    );
                    println!(
                        "  {} {:.1} MB",
                        style("Downloaded size:").dim(),
                        style(metadata.size_mb.to_string()).cyan()
                    );
                    if let Some(path) = &metadata.model_path {
                        println!(
                            "  {} {}",
                            style("Location:").dim(),
                            style(path.display().to_string()).dim()
                        );
                    }
                }
            }

            // Detect and update dimension
            if let Err(e) = config.rag.embedding.detect_and_update_dimension().await {
                println!();
                println!(
                    "{}",
                    style(format!(
                        "⚠️  Warning: Could not detect model dimension: {}",
                        e
                    ))
                    .yellow()
                );
                println!("{}", style("   Using default dimension (384)").dim());
                config.rag.embedding.dimension = 384;
            }

            println!();
            println!("{}", style("🎉 Neural search is ready!").green().bold());
            println!(
                "{}",
                style("   Your searches will now understand context and meaning.").dim()
            );
            println!(
                "{}",
                style("   Try: manx search \"authentication patterns\"").dim()
            );
        }
        Err(e) => {
            spinner.fail(&format!(
                "{} Download failed: {}",
                style("✗").red().bold(),
                e
            ));

            println!();
            println!(
                "{}",
                style("⚠️  Download unsuccessful - falling back to hash search")
                    .yellow()
                    .bold()
            );
            println!();
            println!(
                "{}",
                style("Hash search will still work great for exact matches!").dim()
            );
            println!(
                "{}",
                style("You can try downloading the neural model later with:").dim()
            );
            println!(
                "  {}",
                style(format!("manx embedding download {}", model_name))
                    .cyan()
                    .bold()
            );

            config.rag.embedding.provider = EmbeddingProvider::Hash;
            config.rag.embedding.dimension = 384;
        }
    }

    Ok(())
}

fn get_model_size_description(model_name: &str) -> &'static str {
    match model_name {
        "sentence-transformers/all-MiniLM-L6-v2" => "(~22MB)",
        "sentence-transformers/all-mpnet-base-v2" => "(~120MB)",
        "sentence-transformers/multi-qa-MiniLM-L6-cos-v1" => "(~22MB)",
        "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2" => "(~110MB)",
        "BAAI/bge-small-en-v1.5" => "(~33MB)",
        "BAAI/bge-base-en-v1.5" => "(~110MB)",
        "BAAI/bge-large-en-v1.5" => "(~330MB)",
        _ => "(downloading...)", // Fallback for unknown models
    }
}
