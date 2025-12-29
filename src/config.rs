use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::rag::{
    llm::{LlmConfig, LlmProvider},
    RagConfig,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // Context7 MCP settings (existing)
    pub api_key: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub default_limit: usize,
    pub offline_mode: bool,
    pub color_output: bool,
    pub auto_cache_enabled: bool,
    pub cache_ttl_hours: u64,
    pub max_cache_size_mb: u64,

    // Local RAG settings
    pub rag: RagConfig,

    // LLM integration settings
    pub llm: LlmConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Context7 MCP defaults (existing)
            api_key: None,
            cache_dir: None,
            default_limit: 10,
            offline_mode: false,
            color_output: true,
            auto_cache_enabled: true,
            cache_ttl_hours: 24,
            max_cache_size_mb: 100,

            // RAG defaults
            rag: RagConfig::default(),

            // LLM defaults
            llm: LlmConfig::default(),
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

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config =
            serde_json::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        Ok(ProjectDirs::from("", "", "manx")
            .context("Failed to determine config directory")?
            .config_dir()
            .join("config.json"))
    }

    pub fn merge_with_cli(
        &mut self,
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

        // Context7 MCP Settings
        output.push_str("Context7 MCP:\n");
        output.push_str(&format!(
            "  API Key: {}\n",
            self.api_key
                .as_ref()
                .map(|k| {
                    if k.len() > 8 {
                        format!("{}...{}", &k[..4], &k[k.len() - 4..])
                    } else {
                        "***".to_string()
                    }
                })
                .unwrap_or_else(|| "Not set".to_string())
        ));

        output.push_str(&format!(
            "  Cache Directory: {}\n",
            self.cache_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Default (~/.cache/manx)".to_string())
        ));

        output.push_str(&format!("  Default Search Limit: {}\n", self.default_limit));
        output.push_str(&format!("  Offline Mode: {}\n", self.offline_mode));
        output.push_str(&format!("  Color Output: {}\n", self.color_output));
        output.push_str(&format!(
            "  Auto Cache Enabled: {}\n",
            self.auto_cache_enabled
        ));
        output.push_str(&format!("  Cache TTL (hours): {}\n", self.cache_ttl_hours));
        output.push_str(&format!(
            "  Max Cache Size (MB): {}\n",
            self.max_cache_size_mb
        ));

        // Local RAG Settings
        output.push_str("\nLocal RAG:\n");
        output.push_str(&format!("  Enabled: {}\n", self.rag.enabled));
        output.push_str(&format!(
            "  Index Path: {}\n",
            self.rag.index_path.display()
        ));
        output.push_str(&format!("  Max Results: {}\n", self.rag.max_results));
        output.push_str(&format!(
            "  PDF Processing: {} (Security Setting)\n",
            if self.rag.allow_pdf_processing {
                "Enabled"
            } else {
                "Disabled"
            }
        ));

        // Embedding Settings
        output.push_str(&format!(
            "  Embedding Provider: {:?}\n",
            self.rag.embedding.provider
        ));
        output.push_str(&format!(
            "  Embedding Dimension: {}\n",
            self.rag.embedding.dimension
        ));
        if let Some(model_path) = &self.rag.embedding.model_path {
            output.push_str(&format!("  Model Path: {}\n", model_path.display()));
        }
        if self.rag.embedding.api_key.is_some() {
            output.push_str("  API Key: ****\n");
        }
        if let Some(endpoint) = &self.rag.embedding.endpoint {
            output.push_str(&format!("  Custom Endpoint: {}\n", endpoint));
        }

        // LLM Settings
        output.push_str("\nLLM Integration:\n");
        let llm_status = if self.has_llm_configured() {
            "Available"
        } else {
            "Not configured"
        };
        output.push_str(&format!("  Status: {}\n", llm_status));

        if let Some(key) = &self.llm.openai_api_key {
            output.push_str(&format!(
                "  OpenAI API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(key) = &self.llm.anthropic_api_key {
            output.push_str(&format!(
                "  Anthropic API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(key) = &self.llm.groq_api_key {
            output.push_str(&format!(
                "  Groq API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(key) = &self.llm.openrouter_api_key {
            output.push_str(&format!(
                "  OpenRouter API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(key) = &self.llm.huggingface_api_key {
            output.push_str(&format!(
                "  HuggingFace API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(key) = &self.llm.zai_api_key {
            output.push_str(&format!(
                "  Z.AI API Key: {}...{}\n",
                &key[..4],
                &key[key.len() - 4..]
            ));
        }

        if let Some(endpoint) = &self.llm.custom_endpoint {
            output.push_str(&format!("  Custom Endpoint: {}\n", endpoint));
        }

        output.push_str(&format!("  Provider: {:?}\n", self.llm.preferred_provider));

        if let Some(model) = &self.llm.model_name {
            output.push_str(&format!("  Model: {}\n", model));
        }

        output
    }

    /// Check if LLM functionality should be used
    pub fn should_use_llm(&self, no_llm_flag: bool) -> bool {
        if no_llm_flag {
            return false;
        }
        self.has_llm_configured()
    }

    /// Check if any LLM provider is configured
    pub fn has_llm_configured(&self) -> bool {
        self.llm.openai_api_key.is_some()
            || self.llm.anthropic_api_key.is_some()
            || self.llm.groq_api_key.is_some()
            || self.llm.openrouter_api_key.is_some()
            || self.llm.huggingface_api_key.is_some()
            || self.llm.zai_api_key.is_some()
            || self.llm.custom_endpoint.is_some()
    }

    /// Set LLM API key (auto-detect provider)
    pub fn set_llm_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            // Clear all API keys
            self.llm.openai_api_key = None;
            self.llm.anthropic_api_key = None;
            return Ok(());
        }

        // Auto-detect provider based on key format
        if key.starts_with("sk-") {
            self.llm.openai_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::OpenAI;
        } else if key.starts_with("sk-ant-") {
            self.llm.anthropic_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::Anthropic;
        } else {
            // Default to OpenAI format
            self.llm.openai_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::OpenAI;
        }

        self.save()
    }

    /// Set OpenAI API key
    pub fn set_openai_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.llm.openai_api_key = None;
        } else {
            self.llm.openai_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::OpenAI;
        }
        self.save()
    }

    /// Set Anthropic API key
    pub fn set_anthropic_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.llm.anthropic_api_key = None;
        } else {
            self.llm.anthropic_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::Anthropic;
        }
        self.save()
    }

    /// Set Groq API key
    pub fn set_groq_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.llm.groq_api_key = None;
        } else {
            self.llm.groq_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::Groq;
        }
        self.save()
    }

    /// Set OpenRouter API key
    pub fn set_openrouter_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.llm.openrouter_api_key = None;
        } else {
            self.llm.openrouter_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::OpenRouter;
        }
        self.save()
    }

    /// Set HuggingFace API key
    pub fn set_huggingface_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.llm.huggingface_api_key = None;
        } else {
            self.llm.huggingface_api_key = Some(key);
            self.llm.preferred_provider = LlmProvider::HuggingFace;
        }
        self.save()
    }

    /// Set custom endpoint
    pub fn set_custom_endpoint(&mut self, endpoint: String) -> Result<()> {
        if endpoint.is_empty() {
            self.llm.custom_endpoint = None;
        } else {
            self.llm.custom_endpoint = Some(endpoint);
            self.llm.preferred_provider = LlmProvider::Custom;
        }
        self.save()
    }

    /// Set LLM provider preference
    pub fn set_llm_provider(&mut self, provider: String) -> Result<()> {
        match provider.to_lowercase().as_str() {
            "openai" => self.llm.preferred_provider = LlmProvider::OpenAI,
            "anthropic" => self.llm.preferred_provider = LlmProvider::Anthropic,
            "groq" => self.llm.preferred_provider = LlmProvider::Groq,
            "openrouter" => self.llm.preferred_provider = LlmProvider::OpenRouter,
            "huggingface" => self.llm.preferred_provider = LlmProvider::HuggingFace,
            "zai" => self.llm.preferred_provider = LlmProvider::Zai,
            "custom" => self.llm.preferred_provider = LlmProvider::Custom,
            "auto" => self.llm.preferred_provider = LlmProvider::Auto,
            _ => anyhow::bail!("Invalid provider '{}'. Use: openai, anthropic, groq, openrouter, huggingface, zai, custom, auto", provider),
        }
        self.save()
    }

    /// Set specific LLM model
    pub fn set_llm_model(&mut self, model: String) -> Result<()> {
        if model.is_empty() {
            self.llm.model_name = None;
        } else {
            self.llm.model_name = Some(model);
        }
        self.save()
    }

    /// Enable/disable local RAG
    pub fn set_rag_enabled(&mut self, enabled: bool) -> Result<()> {
        self.rag.enabled = enabled;
        self.save()
    }

    /// Set embedding provider (dimension will be detected dynamically)
    pub fn set_embedding_provider(&mut self, provider_str: &str) -> Result<()> {
        use crate::rag::EmbeddingProvider;

        let provider = match provider_str.to_lowercase().as_str() {
            "hash" => EmbeddingProvider::Hash,
            _ if provider_str.starts_with("onnx:") => {
                let model_name = provider_str.strip_prefix("onnx:").unwrap_or("").to_string();
                if model_name.is_empty() {
                    anyhow::bail!("ONNX provider requires model name: onnx:model_name");
                }
                EmbeddingProvider::Onnx(model_name)
            },
            _ if provider_str.starts_with("ollama:") => {
                let model_name = provider_str.strip_prefix("ollama:").unwrap_or("").to_string();
                if model_name.is_empty() {
                    anyhow::bail!("Ollama provider requires model name: ollama:model_name");
                }
                EmbeddingProvider::Ollama(model_name)
            },
            _ if provider_str.starts_with("openai:") => {
                let model_name = provider_str.strip_prefix("openai:").unwrap_or("text-embedding-3-small").to_string();
                EmbeddingProvider::OpenAI(model_name)
            },
            _ if provider_str.starts_with("huggingface:") => {
                let model_name = provider_str.strip_prefix("huggingface:").unwrap_or("").to_string();
                if model_name.is_empty() {
                    anyhow::bail!("HuggingFace provider requires model name: huggingface:model_name");
                }
                EmbeddingProvider::HuggingFace(model_name)
            },
            _ if provider_str.starts_with("custom:") => {
                let endpoint = provider_str.strip_prefix("custom:").unwrap_or("").to_string();
                if endpoint.is_empty() {
                    anyhow::bail!("Custom provider requires endpoint URL: custom:http://...");
                }
                EmbeddingProvider::Custom(endpoint)
            },
            _ => anyhow::bail!(
                "Invalid embedding provider '{}'. Use: hash, onnx:model, ollama:model, openai:model, huggingface:model, custom:url",
                provider_str
            ),
        };

        // Set provider (dimension will be detected on first use)
        self.rag.embedding.provider = provider;

        self.save()
    }

    /// Set embedding API key (for API providers)
    pub fn set_embedding_api_key(&mut self, key: String) -> Result<()> {
        if key.is_empty() {
            self.rag.embedding.api_key = None;
        } else {
            self.rag.embedding.api_key = Some(key);
        }
        self.save()
    }

    /// Set embedding model path (for local models)
    pub fn set_embedding_model_path(&mut self, path: std::path::PathBuf) -> Result<()> {
        self.rag.embedding.model_path = Some(path);
        self.save()
    }

    /// Set embedding dimension
    pub fn set_embedding_dimension(&mut self, dimension: usize) -> Result<()> {
        if dimension == 0 {
            anyhow::bail!("Embedding dimension must be greater than 0");
        }
        self.rag.embedding.dimension = dimension;
        self.save()
    }
}
