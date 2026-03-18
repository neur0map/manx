use anyhow::Result;

pub fn show() -> Result<()> {
    // Display ASCII art banner with dynamic version
    println!();
    let banner_with_version = BANNER.replace("{}", env!("CARGO_PKG_VERSION"));
    println!("{}", banner_with_version);
    println!();

    // Welcome message
    println!("Welcome to Manx Setup Wizard!");
    println!("{}", "=".repeat(50));
    println!();

    println!("Let's configure manx for optimal documentation search.");
    println!("This wizard will help you set up:");
    println!();
    println!("  * Context7 API for official docs");
    println!("  * Neural search models");
    println!("  * AI providers (optional)");
    println!();

    println!("Press Enter to begin setup...");

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
