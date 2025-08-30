use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub default_limit: usize,
    pub offline_mode: bool,
    pub color_output: bool,
    pub auto_cache_enabled: bool,
    pub cache_ttl_hours: u64,
    pub max_cache_size_mb: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            cache_dir: None,
            default_limit: 10,
            offline_mode: false,
            color_output: true,
            auto_cache_enabled: true,
            cache_ttl_hours: 24,
            max_cache_size_mb: 100,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: Config = serde_json::from_str(&content)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        Ok(ProjectDirs::from("", "", "manx")
            .context("Failed to determine config directory")?
            .config_dir()
            .join("config.json"))
    }
    
    pub fn merge_with_cli(&mut self, 
        api_key: Option<String>,
        cache_dir: Option<PathBuf>,
        offline: bool,
    ) {
        if api_key.is_some() {
            self.api_key = api_key;
        }
        if cache_dir.is_some() {
            self.cache_dir = cache_dir;
        }
        if offline {
            self.offline_mode = true;
        }
        
        // Check NO_COLOR environment variable
        if std::env::var("NO_COLOR").is_ok() {
            self.color_output = false;
        }
    }
    
    pub fn display(&self) -> String {
        let mut output = String::new();
        output.push_str("Current Configuration:\n");
        output.push_str("=====================\n\n");
        
        output.push_str(&format!("API Key: {}\n", 
            self.api_key.as_ref()
                .map(|k| {
                    if k.len() > 8 {
                        format!("{}...{}", &k[..4], &k[k.len()-4..])
                    } else {
                        "***".to_string()
                    }
                })
                .unwrap_or_else(|| "Not set".to_string())
        ));
        
        output.push_str(&format!("Cache Directory: {}\n",
            self.cache_dir.as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Default (~/.cache/manx)".to_string())
        ));
        
        output.push_str(&format!("Default Search Limit: {}\n", self.default_limit));
        output.push_str(&format!("Offline Mode: {}\n", self.offline_mode));
        output.push_str(&format!("Color Output: {}\n", self.color_output));
        output.push_str(&format!("Auto Cache Enabled: {}\n", self.auto_cache_enabled));
        output.push_str(&format!("Cache TTL (hours): {}\n", self.cache_ttl_hours));
        output.push_str(&format!("Max Cache Size (MB): {}\n", self.max_cache_size_mb));
        
        output
    }
}