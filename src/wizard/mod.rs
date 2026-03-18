use anyhow::Result;

mod navigation;
mod prompts;
mod steps;
mod themes;
mod validators;

use crate::config::Config;

pub struct SetupWizard {
    config: Config,
}

impl SetupWizard {
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_default();

        Ok(Self { config })
    }

    pub async fn run(&mut self) -> Result<()> {
        use navigation::WizardStep;

        // Show welcome
        steps::welcome::show()?;

        // Check if reconfiguring
        if self.check_existing_config()? {
            return Ok(());
        }

        // Run setup steps with navigation using the step methods
        let mut current_step = WizardStep::Welcome;

        loop {
            match current_step {
                WizardStep::Welcome => {
                    // Welcome step doesn't need header
                    current_step = WizardStep::Context7;
                }
                WizardStep::Context7 => {
                    navigation::show_step_header(&current_step);
                    let action = steps::context7::setup(&mut self.config).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Embedding => {
                    navigation::show_step_header(&current_step);
                    let action = steps::embeddings::setup(&mut self.config).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Llm => {
                    navigation::show_step_header(&current_step);
                    let action = steps::llm::setup(&mut self.config).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Summary => {
                    navigation::show_step_header(&current_step);
                    let action = steps::summary::show_and_test(&self.config).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Complete => {
                    break;
                }
            }
        }

        // Save configuration
        self.config.save()?;

        // Show success message
        self.show_completion_message()?;

        Ok(())
    }

    fn handle_navigation_action(
        &self,
        action: navigation::WizardAction,
        current_step: &navigation::WizardStep,
    ) -> Result<navigation::WizardStep> {
        use navigation::{WizardAction, WizardStep};

        match action {
            WizardAction::Next => {
                if let Some(next_step) = current_step.next() {
                    Ok(next_step)
                } else {
                    Ok(WizardStep::Complete)
                }
            }
            WizardAction::Back => {
                if let Some(prev_step) = current_step.previous() {
                    Ok(prev_step)
                } else {
                    Ok(current_step.clone()) // Stay at current step if no previous
                }
            }
            WizardAction::Skip => {
                if let Some(next_step) = current_step.next() {
                    Ok(next_step)
                } else {
                    Ok(WizardStep::Complete)
                }
            }
            WizardAction::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn check_existing_config(&self) -> Result<bool> {
        if self.config.api_key.is_some()
            || self.config.has_llm_configured()
            || self.config.rag.embedding.provider != crate::rag::EmbeddingProvider::Hash
        {
            println!();
            println!("Existing configuration detected!");
            println!("Would you like to reconfigure manx?");
            println!();

            let reconfigure =
                crate::wizard::prompts::confirm_action("Reconfigure manx?", false)?;

            if !reconfigure {
                println!();
                println!("Setup cancelled.");
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn show_completion_message(&self) -> Result<()> {
        println!();
        println!("{}", "-".repeat(50));
        println!();
        println!("Setup complete! manx is ready!");
        println!();
        println!("Get started with these commands:");

        if self.config.api_key.is_some() {
            println!("  manx snippet react hooks");
            println!("  manx search \"authentication patterns\"");
            println!("  manx doc fastapi middleware");
        } else {
            println!("  manx search \"rust error handling\"");
            println!("  manx config --api-key <key>  # Add Context7 later (optional)");
        }

        println!();
        println!("Need help? Try: manx --help");
        println!();

        Ok(())
    }
}
