use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;

#[derive(Debug, Clone, PartialEq)]
pub enum WizardAction {
    Next,
    Back,
    Skip,
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WizardStep {
    Welcome,
    Context7,
    Embedding,
    Llm,
    Summary,
    Complete,
}

impl WizardStep {
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::Context7),
            Self::Context7 => Some(Self::Embedding),
            Self::Embedding => Some(Self::Llm),
            Self::Llm => Some(Self::Summary),
            Self::Summary => Some(Self::Complete),
            Self::Complete => None,
        }
    }

    pub fn previous(&self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::Context7 => Some(Self::Welcome),
            Self::Embedding => Some(Self::Context7),
            Self::Llm => Some(Self::Embedding),
            Self::Summary => Some(Self::Llm),
            Self::Complete => Some(Self::Summary),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::Context7 => "Context7 API",
            Self::Embedding => "Search Engine",
            Self::Llm => "AI Features",
            Self::Summary => "Summary",
            Self::Complete => "Complete",
        }
    }

    pub fn number(&self) -> usize {
        match self {
            Self::Welcome => 0,
            Self::Context7 => 1,
            Self::Embedding => 2,
            Self::Llm => 3,
            Self::Summary => 4,
            Self::Complete => 5,
        }
    }

    pub fn total_steps() -> usize {
        4 // Context7, Embedding, LLM, Summary (Welcome and Complete don't count)
    }
}

pub fn show_navigation_options(
    theme: &ColorfulTheme,
    current_step: &WizardStep,
    can_skip: bool,
) -> Result<WizardAction> {
    let mut choices = Vec::new();

    // Always show Next/Continue
    choices.push("Continue");

    // Show Back if not at first step
    if current_step.previous().is_some() {
        choices.push("Back");
    }

    // Show Skip if applicable
    if can_skip {
        choices.push("Skip this step");
    }

    // Always show Quit
    choices.push("Quit setup");

    println!();
    let selection = dialoguer::Select::with_theme(theme)
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()?;

    let mut index = 0;

    // Continue is always first
    if selection == index {
        return Ok(WizardAction::Next);
    }
    index += 1;

    // Back if available
    if current_step.previous().is_some() {
        if selection == index {
            return Ok(WizardAction::Back);
        }
        index += 1;
    }

    // Skip if available
    if can_skip {
        if selection == index {
            return Ok(WizardAction::Skip);
        }
        index += 1;
    }

    // Quit is always last
    if selection == index {
        return Ok(WizardAction::Quit);
    }

    // Should never reach here with valid selection
    Ok(WizardAction::Quit)
}

pub fn show_step_header(step: &WizardStep) {
    if matches!(step, WizardStep::Welcome | WizardStep::Complete) {
        return;
    }

    println!();
    println!(
        "{}",
        style(format!(
            "[Step {}/{}] {}",
            step.number(),
            WizardStep::total_steps(),
            step.name()
        ))
        .cyan()
        .bold()
    );
    println!("{}", style("─".repeat(40)).dim());
}
