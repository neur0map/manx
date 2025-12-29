use anyhow::Result;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;

mod navigation;
mod prompts;
mod steps;
mod themes;
mod validators;

use crate::config::Config;

pub struct SetupWizard {
    term: Term,
    theme: ColorfulTheme,
    config: Config,
}

impl SetupWizard {
    pub fn new() -> Result<Self> {
        let term = Term::stdout();
        let theme = themes::create_theme();
        let config = Config::load().unwrap_or_default();

        Ok(Self {
            term,
            theme,
            config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        use navigation::WizardStep;

        // Clear screen and show welcome
        self.term.clear_screen()?;
        steps::welcome::show(&self.term)?;

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
                    let action = steps::context7::setup(&mut self.config, &self.theme).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Embedding => {
                    navigation::show_step_header(&current_step);
                    let action = steps::embeddings::setup(&mut self.config, &self.theme).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Llm => {
                    navigation::show_step_header(&current_step);
                    let action = steps::llm::setup(&mut self.config, &self.theme).await?;
                    current_step = self.handle_navigation_action(action, &current_step)?;
                    if current_step == WizardStep::Complete {
                        break;
                    }
                }
                WizardStep::Summary => {
                    navigation::show_step_header(&current_step);
                    let action = steps::summary::show_and_test(&self.config, &self.theme).await?;
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
            println!(
                "{}",
                style("Existing configuration detected!").yellow().bold()
            );
            println!("Would you like to reconfigure manx?");
            println!();

            let reconfigure =
                crate::wizard::prompts::confirm_action(&self.theme, "Reconfigure manx?", false)?;

            if !reconfigure {
                println!();
                println!("{}", style("Setup cancelled.").dim());
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn show_completion_message(&self) -> Result<()> {
        println!();
        println!("{}", style("─".repeat(50)).dim());
        println!();
        println!("{} manx is ready!", style("Setup complete!").green().bold());
        println!();
        println!("Get started with these commands:");

        if self.config.api_key.is_some() {
            println!("  {} snippet react hooks", style("manx").bold().blue());
            println!(
                "  {} search \"authentication patterns\"",
                style("manx").bold().blue()
            );
            println!("  {} doc fastapi middleware", style("manx").bold().blue());
        } else {
            println!(
                "  {} search \"rust error handling\"",
                style("manx").bold().blue()
            );
            println!(
                "  {} config --api-key <key>  {} {}",
                style("manx").bold().blue(),
                style("# Add Context7 later").dim(),
                style("(optional)").dim()
            );
        }

        println!();
        println!(
            "{} {}",
            style("Need help? Try:").dim(),
            style("manx --help").bold()
        );
        println!();

        Ok(())
    }
}
