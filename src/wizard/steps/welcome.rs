use anyhow::Result;
use console::{style, Term};

pub fn show(_term: &Term) -> Result<()> {
    // Display ASCII art banner with dynamic version
    println!();
    let banner_with_version = BANNER.replace("{}", env!("CARGO_PKG_VERSION"));
    println!("{}", style(banner_with_version).cyan().bold());
    println!();

    // Welcome message
    println!("{}", style("Welcome to Manx Setup Wizard!").bold().cyan());
    println!("{}", style("━".repeat(50)).dim());
    println!();

    println!("Let's configure manx for optimal documentation search.");
    println!("This wizard will help you set up:");
    println!();
    println!("  {} Context7 API for official docs", style("•").cyan());
    println!("  {} Neural search models", style("•").cyan());
    println!("  {} AI providers (optional)", style("•").cyan());
    println!();

    println!("{}", style("Press Enter to begin setup...").dim().italic());

    // Wait for user input
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}

const BANNER: &str = r#"
    ███        ██████   ██████   █████████   ██████   █████ █████ █████
   ░░░██      ░░██████ ██████   ███░░░░░███ ░░██████ ░░███ ░░███ ░░███
      ░██      ░███░█████░███  ░███    ░███  ░███░███ ░███  ░░███ ███
      ░░███    ░███░░███ ░███  ░███████████  ░███░░███░███   ░░█████
       ██░     ░███ ░░░  ░███  ░███░░░░░███  ░███ ░░██████    ███░███
      ██       ░███      ░███  ░███    ░███  ░███  ░░█████   ███ ░░███
     ███       █████     █████ █████   █████ █████  ░░█████ █████ █████
    ░░░       ░░░░░     ░░░░░ ░░░░░   ░░░░░ ░░░░░    ░░░░░ ░░░░░ ░░░░░
                           Setup Wizard v{}
"#;
