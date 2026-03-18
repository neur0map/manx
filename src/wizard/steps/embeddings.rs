use anyhow::Result;

use crate::config::Config;
use crate::rag::EmbeddingProvider;
use crate::wizard::navigation::WizardAction;
use crate::wizard::prompts;

pub async fn setup(config: &mut Config) -> Result<WizardAction> {
    println!();
    println!("Choose your search engine - this affects how well manx understands your searches:");
    println!();

    println!("Hash Search (Default):");
    println!("  - Works immediately, no setup required");
    println!("  - Fast and reliable");
    println!("  - Perfect for exact keyword matching");
    println!();

    println!("Neural Search (Recommended):");
    println!("  - Understands meaning: \"auth\" finds \"authentication\"");
    println!("  - Better results for complex queries");
    println!("  - Small download (~22MB), runs locally");
    println!();

    let choices = vec![
        "Hash Search (fast, no download)",
        "Neural Search (download small model ~22MB)",
        "Keep current setting",
        "-- Navigation --",
        "<- Back to previous step",
        "x  Quit setup",
    ];

    let selection = prompts::select_option("Select search engine", &choices);

    match selection {
        0 => {
            // Hash
            config.rag.embedding.provider = EmbeddingProvider::Hash;
            config.rag.embedding.dimension = 384;
            println!();
            println!("Hash search configured!");
            println!("  Fast keyword search ready to use.");
            Ok(WizardAction::Next)
        }
        1 => {
            // Neural - show model selection
            match select_neural_model(config).await? {
                WizardAction::Next => Ok(WizardAction::Next),
                WizardAction::Back => Ok(WizardAction::Skip), // Go back to main menu
                WizardAction::Quit => Ok(WizardAction::Quit),
                _ => Ok(WizardAction::Next),
            }
        }
        2 => {
            // Keep current
            println!();
            println!("Keeping current search settings.");
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

async fn select_neural_model(config: &mut Config) -> Result<WizardAction> {
    use crate::rag::providers::onnx::OnnxProvider;

    loop {
        println!();
        println!("Choose Neural Search Model");
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
        choices.push("-- Navigation --".to_string());
        choices.push("<- Back to search engine selection".to_string());
        choices.push("x  Quit setup".to_string());

        let selection = prompts::select_option_owned("Select neural search model", &choices);

        let model_count = model_descriptions.len();

        match selection {
            i if i < model_count => {
                // User selected a model
                let (selected_model, _) = model_descriptions[i];

                println!();
                println!("Selected: {}", selected_model);

                // Show confirmation with options
                let confirm_choices = vec![
                    "Download and configure this model",
                    "Choose a different model",
                    "<- Back to search engine selection",
                ];

                let confirm_selection =
                    prompts::select_option("Proceed with this model?", &confirm_choices);

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
    println!("Preparing to download neural search model");

    // Get the appropriate size description for the model
    let size_desc = get_model_size_description(model_name);

    println!();
    println!("  Model: {}", model_name);
    println!("  Size: {}", size_desc);
    println!("  Privacy: Downloaded to local storage (no data sent to external servers)");
    println!();

    // Show what files will be downloaded
    println!("Downloading 3 files from HuggingFace:");
    println!("  1. model.onnx (main neural network)");
    println!("  2. tokenizer.json (text processing)");
    println!("  3. config.json (model configuration)");
    println!();

    let spinner = prompts::show_spinner(&format!(
        "Downloading {} {}...",
        model_name, size_desc
    ));

    // Use the ONNX provider to download
    use crate::rag::providers::onnx::OnnxProvider;

    match OnnxProvider::download_model(model_name, false).await {
        Ok(()) => {
            spinner.success(&format!(
                "[OK] Neural search model installed successfully!"
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
                    println!("Installation Summary:");
                    println!("  Model: {}", model_name);
                    println!("  Embedding dimension: {}D", metadata.dimension);
                    println!("  Downloaded size: {:.1} MB", metadata.size_mb);
                    if let Some(path) = &metadata.model_path {
                        println!("  Location: {}", path.display());
                    }
                }
            }

            // Detect and update dimension
            if let Err(e) = config.rag.embedding.detect_and_update_dimension().await {
                println!();
                println!(
                    "Warning: Could not detect model dimension: {}",
                    e
                );
                println!("   Using default dimension (384)");
                config.rag.embedding.dimension = 384;
            }

            println!();
            println!("Neural search is ready!");
            println!("   Your searches will now understand context and meaning.");
            println!("   Try: manx search \"authentication patterns\"");
        }
        Err(e) => {
            spinner.fail(&format!("[FAIL] Download failed: {}", e));

            println!();
            println!("Download unsuccessful - falling back to hash search");
            println!();
            println!("Hash search will still work great for exact matches!");
            println!("You can try downloading the neural model later with:");
            println!("  manx embedding download {}", model_name);

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
