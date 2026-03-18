use anyhow::Result;

use crate::config::Config;
use crate::wizard::{navigation::WizardAction, prompts, validators};

pub async fn setup(config: &mut Config) -> Result<WizardAction> {
    loop {
        println!();
        println!(
            "Context7 provides access to official documentation from thousands of projects."
        );
        println!("It's optional but highly recommended for the best search experience.");
        println!();
        println!("Without Context7:");
        println!("  - Limited to hash-based search");
        println!("  - No official documentation access");
        println!("With Context7:");
        println!("  - Search official docs from React, Python, Rust, etc.");
        println!("  - Access to latest documentation");
        println!("  - Better search results");
        println!();

        let choices = vec![
            "Skip (I'll set it up later)",
            "Get a free API key (opens browser)",
            "I have an API key",
            "-- Navigation --",
            "<- Back to previous step",
            "x  Quit setup",
        ];

        let selection = prompts::select_option("How would you like to proceed?", &choices);

        match selection {
            0 => {
                // Skip
                println!();
                println!("Context7 setup skipped.");
                println!("You can set this up later with: manx config --api-key <key>");
                config.api_key = None;
                return Ok(WizardAction::Next);
            }
            1 => {
                // Get API key
                match handle_browser_api_setup(config).await? {
                    WizardAction::Next => return Ok(WizardAction::Next),
                    WizardAction::Back => continue, // Go back to main menu
                    WizardAction::Quit => return Ok(WizardAction::Quit),
                    _ => continue,
                }
            }
            2 => {
                // Enter existing key
                match handle_manual_api_setup(config).await? {
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

async fn handle_browser_api_setup(config: &mut Config) -> Result<WizardAction> {
    println!();
    println!("Opening Context7 in your browser...");

    if webbrowser::open("https://context7.com/").is_err() {
        println!("Could not open browser automatically.");
        println!("Please visit: https://context7.com/");
    }

    println!();
    println!("After signing up and getting your API key:");

    loop {
        println!("Press Enter with empty input to go back to main menu");

        if let Some(api_key) = prompts::prompt_for_api_key("Context7")? {
            if !validators::validate_api_key(&api_key, "Context7") {
                println!("Invalid API key format");
                continue;
            }

            // Show confirmation with options
            println!();
            println!("API key looks valid!");

            let choices = vec![
                "Save and continue",
                "Try a different API key",
                "Go back to main menu",
            ];

            let confirm_selection =
                prompts::select_option("What would you like to do?", &choices);

            match confirm_selection {
                0 => {
                    // Save and continue
                    config.api_key = Some(api_key);
                    println!();
                    println!("Context7 API configured!");
                    return Ok(WizardAction::Next);
                }
                1 => {
                    // Try different key - loop continues
                    println!("Let's try again...");
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
            println!("Going back to main menu...");
            return Ok(WizardAction::Back);
        }
    }
}

async fn handle_manual_api_setup(config: &mut Config) -> Result<WizardAction> {
    println!();

    loop {
        println!("Press Enter with empty input to go back to main menu");

        let api_key =
            prompts::get_input("Enter your Context7 API key (or leave empty to go back)");

        if api_key.is_empty() {
            println!("Going back to main menu...");
            return Ok(WizardAction::Back); // Go back to main menu
        }

        if !validators::validate_api_key(&api_key, "Context7") {
            println!("Invalid API key format");
            continue;
        }

        // Show confirmation with options
        println!();
        println!("API key looks valid!");

        let choices = vec![
            "Save and continue",
            "Try a different API key",
            "Go back to main menu",
        ];

        let confirm_selection = prompts::select_option("What would you like to do?", &choices);

        match confirm_selection {
            0 => {
                // Save and continue
                config.api_key = Some(api_key);
                println!();
                println!("Context7 API configured!");
                return Ok(WizardAction::Next);
            }
            1 => {
                // Try different key - loop continues
                println!("Let's try again...");
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
