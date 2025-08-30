use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use crate::render::Renderer;

const REPO: &str = "neur0map/manx";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    body: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    content_type: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_notes: String,
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
    
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        let pb = self.renderer.show_progress("Checking for updates...");
        
        let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
        let response = self.client.get(&url).send().await
            .context("Failed to fetch release information")?;
        
        if !response.status().is_success() {
            anyhow::bail!("GitHub API error: {}", response.status());
        }
        
        let release: GitHubRelease = response.json().await
            .context("Failed to parse release information")?;
        
        pb.finish_and_clear();
        
        let latest_version = release.tag_name.trim_start_matches('v');
        let current_version = CURRENT_VERSION;
        
        let update_available = version_compare(latest_version, current_version)?;
        
        Ok(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            update_available,
            release_notes: release.body,
        })
    }
    
    pub async fn perform_update(&self, force: bool) -> Result<()> {
        let update_info = self.check_for_updates().await?;
        
        if !update_info.update_available && !force {
            self.renderer.print_success("You're already on the latest version!");
            return Ok(());
        }
        
        self.renderer.print_success(&format!(
            "Updating from v{} to v{}...",
            update_info.current_version,
            update_info.latest_version
        ));
        
        // Get current executable path
        let current_exe = env::current_exe()
            .context("Failed to get current executable path")?;
        
        // Detect platform
        let platform = detect_platform();
        let binary_name = format!("manx-{}", platform);
        
        // Download new binary
        let pb = self.renderer.show_progress("Downloading latest version...");
        
        let download_url = format!(
            "https://github.com/{}/releases/download/v{}/{}",
            REPO,
            update_info.latest_version,
            binary_name
        );
        
        let response = self.client.get(&download_url).send().await
            .context("Failed to download update")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }
        
        let binary_data = response.bytes().await
            .context("Failed to read binary data")?;
        
        pb.finish_and_clear();
        
        // Create temporary file
        let temp_path = current_exe.with_extension("tmp");
        fs::write(&temp_path, binary_data)
            .context("Failed to write temporary file")?;
        
        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&temp_path, perms)?;
        }
        
        // Replace current binary
        let backup_path = current_exe.with_extension("backup");
        
        // Create backup
        fs::copy(&current_exe, &backup_path)
            .context("Failed to create backup")?;
        
        // Replace with new version
        fs::rename(&temp_path, &current_exe)
            .context("Failed to replace binary")?;
        
        // Remove backup on success
        fs::remove_file(&backup_path).ok();
        
        self.renderer.print_success(&format!(
            "✅ Successfully updated to v{}!",
            update_info.latest_version
        ));
        
        // Show release notes if available
        if !update_info.release_notes.trim().is_empty() {
            println!("\n📝 Release Notes:");
            println!("{}", update_info.release_notes);
        }
        
        println!("\n🚀 Restart your terminal or run 'manx --version' to verify the update.");
        
        Ok(())
    }
}

fn detect_platform() -> String {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    
    match (os, arch) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
        ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
        ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
        ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
        _ => format!("{}-{}", arch, os),
    }
}

fn version_compare(latest: &str, current: &str) -> Result<bool> {
    // Simple semantic version comparison
    let latest_parts: Vec<u32> = latest.split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let current_parts: Vec<u32> = current.split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    
    // Pad to same length
    let max_len = latest_parts.len().max(current_parts.len());
    let mut latest_padded = latest_parts;
    let mut current_padded = current_parts;
    
    latest_padded.resize(max_len, 0);
    current_padded.resize(max_len, 0);
    
    Ok(latest_padded > current_padded)
}