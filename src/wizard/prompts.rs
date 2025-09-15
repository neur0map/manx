use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use spinoff::{spinners, Color, Spinner};
use std::time::Duration;
use tokio::time::sleep;

pub fn show_spinner(message: &str) -> Spinner {
    Spinner::new(spinners::Dots, message.to_string(), Color::Cyan)
}

pub async fn test_with_spinner<F, Fut>(message: &str, test_fn: F) -> Result<bool>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    let mut spinner = show_spinner(message);
    sleep(Duration::from_millis(500)).await;

    match test_fn().await {
        Ok(success) => {
            if success {
                spinner.success(&format!("{} {}", style("✓").green().bold(), message));
                Ok(true)
            } else {
                spinner.fail(&format!("{} {}", style("✗").red().bold(), message));
                Ok(false)
            }
        }
        Err(e) => {
            spinner.fail(&format!("{} {}: {}", style("✗").red().bold(), message, e));
            Ok(false)
        }
    }
}

pub fn prompt_for_api_key(theme: &ColorfulTheme, provider: &str) -> Result<Option<String>> {
    println!();
    let api_key: String = dialoguer::Input::with_theme(theme)
        .with_prompt(format!("Enter your {} API key", provider))
        .allow_empty(true)
        .interact_text()?;

    if api_key.is_empty() {
        Ok(None)
    } else {
        Ok(Some(api_key))
    }
}

pub fn confirm_action(theme: &ColorfulTheme, message: &str, default: bool) -> Result<bool> {
    dialoguer::Confirm::with_theme(theme)
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(Into::into)
}
