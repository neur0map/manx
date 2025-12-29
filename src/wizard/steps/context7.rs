use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;

use crate::config::Config;
use crate::wizard::{navigation::WizardAction, prompts, validators};

pub async fn setup(config: &mut Config, theme: &ColorfulTheme) -> Result<WizardAction> {
    loop {
        println!();
        println!(
            "{} provides access to official documentation from thousands of projects.",
            style("Context7").bold()
        );
        println!("It's optional but highly recommended for the best search experience.");
        println!();
        println!("{}:", style("Without Context7").dim());
        println!("  - Limited to hash-based search");
        println!("  - No official documentation access");
        println!("{}:", style("With Context7").green());
        println!("  - Search official docs from React, Python, Rust, etc.");
        println!("  - Access to latest documentation");
        println!("  - Better search results");
        println!();

        let mut choices = vec![
            "Skip (I'll set it up later)",
            "Get a free API key (opens browser)",
            "I have an API key",
        ];

        // Add navigation options
        choices.push("── Navigation ──");
        choices.push("← Back to previous step");
        choices.push("✕ Quit setup");

        let selection = dialoguer::Select::with_theme(theme)
            .with_prompt("How would you like to proceed?")
            .items(&choices)
            .default(1) // Default to getting a key
            .interact()?;

        match selection {
            0 => {
                // Skip
                println!();
                println!("{}", style("Context7 setup skipped.").dim());
                println!(
                    "{}",
                    style("You can set this up later with: manx config --api-key <key>").dim()
                );
                config.api_key = None;
                return Ok(WizardAction::Next);
            }
            1 => {
                // Get API key
                match handle_browser_api_setup(config, theme).await? {
                    WizardAction::Next => return Ok(WizardAction::Next),
                    WizardAction::Back => continue, // Go back to main menu
                    WizardAction::Quit => return Ok(WizardAction::Quit),
                    _ => continue,
                }
            }
            2 => {
                // Enter existing key
                match handle_manual_api_setup(config, theme).await? {
                    WizardAction::Next => return Ok(WizardAction::Next),
                    WizardAction::Back => continue, // Go back to main menu
                    WizardAction::Quit => return Ok(WizardAction::Quit),
                    _ => continue,
                }
            }
            3 => {
                // Separator - should not be selectable, but just in case
                continue;
            }
            4 => {
                // Back - but this is the first step, so just continue
                return Ok(WizardAction::Back);
            }
            5 => {
                // Quit
                return Ok(WizardAction::Quit);
            }
            _ => continue,
        }
    }
}

async fn handle_browser_api_setup(
    config: &mut Config,
    theme: &ColorfulTheme,
) -> Result<WizardAction> {
    println!();
    println!("{}", style("Opening Context7 in your browser...").cyan());

    if webbrowser::open("https://context7.com/").is_err() {
        println!(
            "{}",
            style("Could not open browser automatically.").yellow()
        );
        println!(
            "Please visit: {}",
            style("https://context7.com/").cyan().underlined()
        );
    }

    println!();
    println!("After signing up and getting your API key:");

    loop {
        println!(
            "{}",
            style("Press Enter with empty input to go back to main menu").dim()
        );

        if let Some(api_key) = prompts::prompt_for_api_key(theme, "Context7")? {
            if !validators::validate_api_key(&api_key, "Context7") {
                println!("{}", style("Invalid API key format").red());
                continue;
            }

            // Show confirmation with options
            println!();
            println!("{}", style("API key looks valid!").green());

            let choices = vec![
                "Save and continue",
                "Try a different API key",
                "Go back to main menu",
            ];

            let confirm_selection = dialoguer::Select::with_theme(theme)
                .with_prompt("What would you like to do?")
                .items(&choices)
                .default(0)
                .interact()?;

            match confirm_selection {
                0 => {
                    // Save and continue
                    config.api_key = Some(api_key);
                    println!();
                    println!("{}", style("Context7 API configured!").green().bold());
                    return Ok(WizardAction::Next);
                }
                1 => {
                    // Try different key - loop continues
                    println!("{}", style("Let's try again...").dim());
                    continue;
                }
                2 => {
                    // Go back
                    return Ok(WizardAction::Back);
                }
                _ => continue,
            }
        } else {
            // Empty input - go back to main menu
            println!("{}", style("Going back to main menu...").dim());
            return Ok(WizardAction::Back);
        }
    }
}

async fn handle_manual_api_setup(
    config: &mut Config,
    theme: &ColorfulTheme,
) -> Result<WizardAction> {
    println!();

    loop {
        println!(
            "{}",
            style("Press Enter with empty input to go back to main menu").dim()
        );

        let api_key: String = dialoguer::Input::with_theme(theme)
            .with_prompt("Enter your Context7 API key (or leave empty to go back)")
            .allow_empty(true)
            .validate_with(|input: &String| {
                if input.is_empty() {
                    Ok(()) // Allow empty to go back
                } else if !validators::validate_api_key(input, "Context7") {
                    Err("Invalid API key format")
                } else {
                    Ok(())
                }
            })
            .interact_text()?;

        if api_key.is_empty() {
            println!("{}", style("Going back to main menu...").dim());
            return Ok(WizardAction::Back); // Go back to main menu
        }

        // Show confirmation with options
        println!();
        println!("{}", style("API key looks valid!").green());

        let choices = vec![
            "Save and continue",
            "Try a different API key",
            "Go back to main menu",
        ];

        let confirm_selection = dialoguer::Select::with_theme(theme)
            .with_prompt("What would you like to do?")
            .items(&choices)
            .default(0)
            .interact()?;

        match confirm_selection {
            0 => {
                // Save and continue
                config.api_key = Some(api_key);
                println!();
                println!("{}", style("Context7 API configured!").green().bold());
                return Ok(WizardAction::Next);
            }
            1 => {
                // Try different key - loop continues
                println!("{}", style("Let's try again...").dim());
                continue;
            }
            2 => {
                // Go back
                return Ok(WizardAction::Back);
            }
            _ => continue,
        }
    }
}
