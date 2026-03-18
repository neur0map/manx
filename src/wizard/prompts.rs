use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::time::Duration;
use tokio::time::sleep;

/// A simple spinner handle that prints start/success/fail messages.
pub struct SpinnerHandle {
    message: String,
}

impl SpinnerHandle {
    pub fn success(&self, msg: &str) {
        println!("[OK] {}", msg);
    }

    pub fn fail(&self, msg: &str) {
        println!("[FAIL] {}", msg);
    }
}

/// Show a simple spinner (just prints the message).
pub fn show_spinner(message: &str) -> SpinnerHandle {
    println!("... {}", message);
    SpinnerHandle {
        message: message.to_string(),
    }
}

/// Run an async test with a spinner-like output.
pub async fn test_with_spinner<F, Fut>(message: &str, test_fn: F) -> Result<bool>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<bool>>,
{
    let spinner = show_spinner(message);
    sleep(Duration::from_millis(500)).await;

    match test_fn().await {
        Ok(success) => {
            if success {
                spinner.success(&format!("[OK] {}", message));
                Ok(true)
            } else {
                spinner.fail(&format!("[FAIL] {}", message));
                Ok(false)
            }
        }
        Err(e) => {
            spinner.fail(&format!("[FAIL] {}: {}", message, e));
            Ok(false)
        }
    }
}

/// Prompt for an API key. Returns None if the user enters an empty string.
pub fn prompt_for_api_key(provider: &str) -> Result<Option<String>> {
    println!();
    let key = get_input(&format!("Enter your {} API key", provider));
    if key.is_empty() {
        Ok(None)
    } else {
        Ok(Some(key))
    }
}

/// Confirm an action with a y/n prompt. Returns the default if the user presses Enter.
pub fn confirm_action(message: &str, default: bool) -> Result<bool> {
    Ok(confirm(message, default))
}

/// Print a numbered list of options, read a number from stdin, return the index.
pub fn select_option(prompt: &str, options: &[&str]) -> usize {
    loop {
        println!();
        for (i, option) in options.iter().enumerate() {
            println!("  [{}] {}", i + 1, option);
        }
        print!("{} [1-{}]: ", prompt, options.len());
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().lock().read_line(&mut input).is_err() {
            continue;
        }
        let trimmed = input.trim();

        // Default to first option on empty input
        if trimmed.is_empty() {
            return 0;
        }

        if let Ok(n) = trimmed.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return n - 1;
            }
        }
        println!("Invalid selection, please try again.");
    }
}

/// Print a numbered list of owned String options, read a number from stdin, return the index.
pub fn select_option_owned(prompt: &str, options: &[String]) -> usize {
    loop {
        println!();
        for (i, option) in options.iter().enumerate() {
            println!("  [{}] {}", i + 1, option);
        }
        print!("{} [1-{}]: ", prompt, options.len());
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().lock().read_line(&mut input).is_err() {
            continue;
        }
        let trimmed = input.trim();

        // Default to first option on empty input
        if trimmed.is_empty() {
            return 0;
        }

        if let Ok(n) = trimmed.parse::<usize>() {
            if n >= 1 && n <= options.len() {
                return n - 1;
            }
        }
        println!("Invalid selection, please try again.");
    }
}

/// Print a prompt and read a line from stdin.
pub fn get_input(prompt: &str) -> String {
    print!("{}: ", prompt);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap_or(0);
    input.trim().to_string()
}

/// Print a "prompt [Y/n]:" or "[y/N]:" and read y/n from stdin.
pub fn confirm(prompt: &str, default: bool) -> bool {
    let hint = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {}: ", prompt, hint);
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().lock().read_line(&mut input).unwrap_or(0);
    let trimmed = input.trim().to_lowercase();

    if trimmed.is_empty() {
        return default;
    }

    matches!(trimmed.as_str(), "y" | "yes")
}
