use crate::render::Renderer;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::process::Command;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct CrateData {
    #[serde(default)]
    newest_version: String,
    #[serde(default)]
    max_version: String,
}

#[derive(Debug, Deserialize)]
struct CratesIOResponse {
    #[serde(rename = "crate")]
    crate_data: CrateData,
}

pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
}

pub struct SelfUpdater {
    client: Client,
    renderer: Renderer,
}

impl SelfUpdater {
    pub fn new(renderer: Renderer) -> Result<Self> {
        let client = Client::builder()
            .user_agent(format!("manx/{}", CURRENT_VERSION))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, renderer })
    }

    async fn check_cratesio_version(&self) -> Result<String> {
        let url = "https://crates.io/api/v1/crates/manx-cli";
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to fetch crates.io information")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to check crates.io: {}", response.status());
        }

        let crates_info: CratesIOResponse = response
            .json()
            .await
            .context("Failed to parse crates.io information")?;

        let latest_version = if !crates_info.crate_data.newest_version.is_empty() {
            crates_info.crate_data.newest_version
        } else if !crates_info.crate_data.max_version.is_empty() {
            crates_info.crate_data.max_version
        } else {
            anyhow::bail!("Could not determine latest version from crates.io");
        };

        Ok(latest_version)
    }

    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let pb = self.renderer.show_progress("Checking for updates...");

        let latest_version = self.check_cratesio_version().await?;

        pb.finish_and_clear();

        let current_version = CURRENT_VERSION;
        let update_available = version_compare(&latest_version, current_version)?;

        Ok(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version,
            update_available,
        })
    }

    pub async fn perform_update(&self, force: bool) -> Result<()> {
        let update_info = self.check_for_updates().await?;

        if !update_info.update_available && !force {
            self.renderer
                .print_success("You're already on the latest version!");
            return Ok(());
        }

        self.renderer.print_success(&format!(
            "Updating from v{} to v{}...",
            update_info.current_version, update_info.latest_version
        ));

        self.update_via_cargo().await
    }

    async fn update_via_cargo(&self) -> Result<()> {
        println!("Using cargo to update manx...");

        let pb = self
            .renderer
            .show_progress("Running cargo install manx-cli...");

        let output = Command::new("cargo")
            .args(["install", "manx-cli", "--force"])
            .output()
            .context("Failed to run cargo install. Make sure cargo is installed.")?;

        pb.finish_and_clear();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Cargo install failed: {}", stderr);
        }

        self.renderer
            .print_success("Successfully updated manx via cargo!");
        println!("The update is complete. Run 'manx --version' to verify.");

        Ok(())
    }
}

fn version_compare(latest: &str, current: &str) -> Result<bool> {
    let latest_parts: Vec<u32> = latest.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let current_parts: Vec<u32> = current.split('.').map(|s| s.parse().unwrap_or(0)).collect();

    let max_len = latest_parts.len().max(current_parts.len());
    let mut latest_padded = latest_parts;
    let mut current_padded = current_parts;

    latest_padded.resize(max_len, 0);
    current_padded.resize(max_len, 0);

    Ok(latest_padded > current_padded)
}
