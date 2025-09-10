//! Multi-provider LLM integration for answer synthesis
//!
//! Supports OpenAI GPT, Anthropic Claude, Groq, OpenRouter, HuggingFace, and custom endpoints
//! with automatic failover and comprehensive error handling.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::rag::RagSearchResult;

/// Configuration for LLM integration supporting multiple providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub groq_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub huggingface_api_key: Option<String>,
    pub custom_endpoint: Option<String>,
    pub preferred_provider: LlmProvider,
    pub fallback_providers: Vec<LlmProvider>,
    pub timeout_seconds: u64,
    pub max_tokens: u32,
    pub temperature: f32,
    pub model_name: Option<String>,
    pub streaming: bool,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            anthropic_api_key: None,
            groq_api_key: None,
            openrouter_api_key: None,
            huggingface_api_key: None,
            custom_endpoint: None,
            preferred_provider: LlmProvider::Auto,
            fallback_providers: vec![
                LlmProvider::OpenAI,
                LlmProvider::Anthropic,
                LlmProvider::Groq,
                LlmProvider::OpenRouter,
            ],
            timeout_seconds: 30,
            max_tokens: 1000,
            temperature: 0.1,
            model_name: None,
            streaming: false,
        }
    }
}

/// Available LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmProvider {
    Auto,
    OpenAI,
    Anthropic,
    Groq,
    OpenRouter,
    HuggingFace,
    Custom,
}

/// LLM response with comprehensive metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub answer: String,
    pub sources_used: Vec<String>,
    pub confidence: Option<f32>,
    pub provider_used: LlmProvider,
    pub model_used: String,
    pub tokens_used: Option<u32>,
    pub response_time_ms: u64,
    pub finish_reason: Option<String>,
    pub citations: Vec<Citation>,
}

/// Citation information linking to source documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source_id: String,
    pub source_title: String,
    pub source_url: Option<String>,
    pub relevance_score: f32,
    pub excerpt: String,
}

/// Multi-provider LLM client with automatic failover
pub struct LlmClient {
    pub(crate) config: LlmConfig,
    pub(crate) http_client: reqwest::Client,
}

impl LlmClient {
    /// Create a new LLM client with configuration
    pub fn new(config: LlmConfig) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Check if any LLM provider is available
    pub fn is_available(&self) -> bool {
        self.has_openai_key()
            || self.has_anthropic_key()
            || self.has_groq_key()
            || self.has_openrouter_key()
            || self.has_huggingface_key()
            || self.config.custom_endpoint.is_some()
    }

    /// Check availability of specific providers
    pub fn has_openai_key(&self) -> bool {
        self.config
            .openai_api_key
            .as_ref()
            .is_some_and(|key| !key.is_empty())
    }

    pub fn has_anthropic_key(&self) -> bool {
        self.config
            .anthropic_api_key
            .as_ref()
            .is_some_and(|key| !key.is_empty())
    }

    pub fn has_groq_key(&self) -> bool {
        self.config
            .groq_api_key
            .as_ref()
            .is_some_and(|key| !key.is_empty())
    }

    pub fn has_openrouter_key(&self) -> bool {
        self.config
            .openrouter_api_key
            .as_ref()
            .is_some_and(|key| !key.is_empty())
    }

    pub fn has_huggingface_key(&self) -> bool {
        self.config
            .huggingface_api_key
            .as_ref()
            .is_some_and(|key| !key.is_empty())
    }

    /// Get the best available provider based on configuration and API key availability
    pub fn get_best_provider(&self) -> Option<LlmProvider> {
        if self.config.preferred_provider != LlmProvider::Auto {
            // Check if preferred provider is available
            if self.is_provider_available(&self.config.preferred_provider) {
                return Some(self.config.preferred_provider.clone());
            }
        }

        // Try fallback providers in order
        for provider in &self.config.fallback_providers {
            if self.is_provider_available(provider) {
                return Some(provider.clone());
            }
        }

        None
    }

    /// Check if a specific provider is available
    pub fn is_provider_available(&self, provider: &LlmProvider) -> bool {
        match provider {
            LlmProvider::OpenAI => self.has_openai_key(),
            LlmProvider::Anthropic => self.has_anthropic_key(),
            LlmProvider::Groq => self.has_groq_key(),
            LlmProvider::OpenRouter => self.has_openrouter_key(),
            LlmProvider::HuggingFace => self.has_huggingface_key(),
            LlmProvider::Custom => self.config.custom_endpoint.is_some(),
            LlmProvider::Auto => false, // Auto is not a real provider
        }
    }

    /// Synthesize an answer from search results using the best available provider
    pub async fn synthesize_answer(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let provider = self
            .get_best_provider()
            .ok_or_else(|| anyhow!("No LLM provider available"))?;

        let start_time = std::time::Instant::now();

        let response = match provider {
            LlmProvider::OpenAI => self.synthesize_with_openai(query, results).await,
            LlmProvider::Anthropic => self.synthesize_with_anthropic(query, results).await,
            LlmProvider::Groq => self.synthesize_with_groq(query, results).await,
            LlmProvider::OpenRouter => self.synthesize_with_openrouter(query, results).await,
            LlmProvider::HuggingFace => self.synthesize_with_huggingface(query, results).await,
            LlmProvider::Custom => self.synthesize_with_custom(query, results).await,
            LlmProvider::Auto => unreachable!(),
        };

        // If primary provider fails, try fallback providers
        match response {
            Ok(mut resp) => {
                resp.response_time_ms = start_time.elapsed().as_millis() as u64;
                Ok(resp)
            }
            Err(e) => {
                log::warn!("Primary provider {:?} failed: {}", provider, e);
                self.try_fallback_providers(query, results, &provider).await
            }
        }
    }

    /// Try fallback providers if primary fails
    async fn try_fallback_providers(
        &self,
        query: &str,
        results: &[RagSearchResult],
        failed_provider: &LlmProvider,
    ) -> Result<LlmResponse> {
        for provider in &self.config.fallback_providers {
            if provider != failed_provider && self.is_provider_available(provider) {
                log::info!("Trying fallback provider: {:?}", provider);

                let start_time = std::time::Instant::now();
                let response = match provider {
                    LlmProvider::OpenAI => self.synthesize_with_openai(query, results).await,
                    LlmProvider::Anthropic => self.synthesize_with_anthropic(query, results).await,
                    LlmProvider::Groq => self.synthesize_with_groq(query, results).await,
                    LlmProvider::OpenRouter => {
                        self.synthesize_with_openrouter(query, results).await
                    }
                    LlmProvider::HuggingFace => {
                        self.synthesize_with_huggingface(query, results).await
                    }
                    LlmProvider::Custom => self.synthesize_with_custom(query, results).await,
                    LlmProvider::Auto => continue,
                };

                if let Ok(mut resp) = response {
                    resp.response_time_ms = start_time.elapsed().as_millis() as u64;
                    return Ok(resp);
                }
            }
        }

        Err(anyhow!("All LLM providers failed"))
    }

    /// Get the appropriate model name for a provider
    fn get_model_name(&self, provider: &LlmProvider) -> String {
        if let Some(model) = &self.config.model_name {
            return model.clone();
        }

        match provider {
            LlmProvider::OpenAI => "gpt-4o-mini".to_string(),
            LlmProvider::Anthropic => "claude-3-haiku-20240307".to_string(),
            LlmProvider::Groq => "llama-3.1-8b-instant".to_string(),
            LlmProvider::OpenRouter => "openai/gpt-3.5-turbo".to_string(),
            LlmProvider::HuggingFace => "microsoft/DialoGPT-medium".to_string(),
            LlmProvider::Custom => "custom-model".to_string(),
            LlmProvider::Auto => "auto".to_string(),
        }
    }

    /// Create concise system prompt focused on clean, scannable output
    fn create_system_prompt(&self) -> String {
        r#"You are a concise technical documentation assistant. Provide clear, scannable answers based ONLY on the provided search results.

RESPONSE FORMAT:
1. **Quick Answer** (1-2 sentences max)
2. **Key Points** (bullet points, max 4 items)  
3. **Code Example** (if available - keep it short and practical)

RULES:
- Be extremely concise and scannable
- Use bullet points and short paragraphs
- Only include essential information
- Cite sources as [Source N] 
- Never add information not in the sources
- Focus on what developers need to know immediately

STYLE:
- Write for busy developers who want quick answers
- Use clear, simple language
- Keep code examples minimal but complete
- Prioritize readability over completeness"#.to_string()
    }

    /// Create user prompt with query and search results
    fn create_user_prompt(&self, query: &str, results: &[RagSearchResult]) -> String {
        let mut prompt = format!("Question: {}\n\nSearch Results:\n\n", query);

        for (i, result) in results.iter().enumerate() {
            prompt.push_str(&format!(
                "[Source {}] {}\nURL: {}\nContent: {}\n\n",
                i + 1,
                result.title.as_ref().unwrap_or(&"Untitled".to_string()),
                result.source_path.to_string_lossy(),
                result.content.chars().take(1000).collect::<String>()
            ));
        }

        prompt.push_str("\nPlease provide a comprehensive answer based on these search results.");
        prompt
    }

    /// Extract the actual answer from responses that may contain thinking content
    fn extract_final_answer(&self, response_text: &str) -> String {
        // Handle models with thinking capabilities - check for both <thinking> and <think> tags
        if response_text.contains("<thinking>") && response_text.contains("</thinking>") {
            // Find the end of the thinking section
            if let Some(thinking_end) = response_text.find("</thinking>") {
                let after_thinking = &response_text[thinking_end + "</thinking>".len()..];
                return after_thinking.trim().to_string();
            }
        }

        // Handle models that use <think> tags instead of <thinking>
        if response_text.contains("<think>") && response_text.contains("</think>") {
            // Find the end of the think section
            if let Some(think_end) = response_text.find("</think>") {
                let after_think = &response_text[think_end + "</think>".len()..];
                return after_think.trim().to_string();
            }
        }

        // Handle models that might use other thinking patterns
        // Some models use patterns like "Let me think about this..." followed by the actual answer
        if response_text.starts_with("Let me think") || response_text.starts_with("I need to think")
        {
            // Look for common transition phrases that indicate the start of the actual answer
            let transition_phrases = [
                "Here's my answer:",
                "My answer is:",
                "To answer your question:",
                "Based on the search results:",
                "The answer is:",
                "\n\n**", // Common formatting transition
                "\n\nQuick Answer:",
                "\n\n##", // Markdown heading transition
            ];

            for phrase in &transition_phrases {
                if let Some(pos) = response_text.find(phrase) {
                    let answer_start = if phrase.starts_with('\n') {
                        pos + 2 // Skip the newlines
                    } else {
                        pos + phrase.len()
                    };
                    return response_text[answer_start..].trim().to_string();
                }
            }
        }

        // For other models or no thinking pattern detected, return the full response
        response_text.to_string()
    }

    /// Extract citations from LLM response
    fn extract_citations(&self, response_text: &str, results: &[RagSearchResult]) -> Vec<Citation> {
        let mut citations = Vec::new();

        // Simple citation extraction - look for [Source N] patterns
        for (i, result) in results.iter().enumerate() {
            let source_ref = format!("[Source {}]", i + 1);
            if response_text.contains(&source_ref) {
                citations.push(Citation {
                    source_id: result.id.clone(),
                    source_title: result
                        .title
                        .clone()
                        .unwrap_or_else(|| "Untitled".to_string()),
                    source_url: Some(result.source_path.to_string_lossy().to_string()),
                    relevance_score: result.score,
                    excerpt: result.content.chars().take(200).collect(),
                });
            }
        }

        citations
    }

    /// OpenAI GPT integration with streaming support
    async fn synthesize_with_openai(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let api_key = self
            .config
            .openai_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("OpenAI API key not configured"))?;

        let model = self.get_model_name(&LlmProvider::OpenAI);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": self.config.streaming
        });

        let response = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI response format"))?;
        let answer = self.extract_final_answer(raw_answer);

        let usage = &response_json["usage"];
        let tokens_used = usage["total_tokens"].as_u64().map(|t| t as u32);
        let finish_reason = response_json["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.9), // OpenAI typically high confidence
            provider_used: LlmProvider::OpenAI,
            model_used: model,
            tokens_used,
            response_time_ms: 0, // Will be set by caller
            finish_reason,
            citations,
        })
    }

    /// Anthropic Claude integration with function calling support
    async fn synthesize_with_anthropic(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let api_key = self
            .config
            .anthropic_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Anthropic API key not configured"))?;

        let model = self.get_model_name(&LlmProvider::Anthropic);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        let payload = serde_json::json!({
            "model": model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "system": system_prompt,
            "messages": [
                {
                    "role": "user",
                    "content": user_prompt
                }
            ]
        });

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Anthropic response format"))?;
        let answer = self.extract_final_answer(raw_answer);

        let usage = &response_json["usage"];
        let tokens_used = usage["output_tokens"].as_u64().map(|t| t as u32);
        let finish_reason = response_json["stop_reason"].as_str().map(|s| s.to_string());

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.85), // Claude typically good confidence
            provider_used: LlmProvider::Anthropic,
            model_used: model,
            tokens_used,
            response_time_ms: 0,
            finish_reason,
            citations,
        })
    }

    /// Groq fast inference integration for ultra-fast responses
    async fn synthesize_with_groq(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let api_key = self
            .config
            .groq_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Groq API key not configured"))?;

        let model = self.get_model_name(&LlmProvider::Groq);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": false
        });

        let response = self
            .http_client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            log::error!(
                "Groq API error - Status: {}, Response: {}",
                status,
                error_text
            );
            return Err(anyhow!("Groq API error ({}): {}", status, error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Groq response format"))?;
        let answer = self.extract_final_answer(raw_answer);

        let usage = &response_json["usage"];
        let tokens_used = usage["total_tokens"].as_u64().map(|t| t as u32);
        let finish_reason = response_json["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.8), // Groq usually good quality
            provider_used: LlmProvider::Groq,
            model_used: model,
            tokens_used,
            response_time_ms: 0,
            finish_reason,
            citations,
        })
    }

    /// OpenRouter multi-model gateway for access to multiple providers
    async fn synthesize_with_openrouter(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let api_key = self
            .config
            .openrouter_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("OpenRouter API key not configured"))?;

        let model = self.get_model_name(&LlmProvider::OpenRouter);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": self.config.streaming
        });

        let response = self
            .http_client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/neur0map/manx")
            .header("X-Title", "Manx Documentation Finder")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenRouter API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenRouter response format"))?;
        let answer = self.extract_final_answer(raw_answer);

        let usage = &response_json["usage"];
        let tokens_used = usage["total_tokens"].as_u64().map(|t| t as u32);
        let finish_reason = response_json["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.82), // Varies by underlying model
            provider_used: LlmProvider::OpenRouter,
            model_used: model,
            tokens_used,
            response_time_ms: 0,
            finish_reason,
            citations,
        })
    }

    /// HuggingFace Router API for open-source models
    async fn synthesize_with_huggingface(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let api_key = self
            .config
            .huggingface_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("HuggingFace API key not configured"))?;

        let model = self.get_model_name(&LlmProvider::HuggingFace);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        // Use OpenAI-compatible chat completions format
        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature
        });

        let response = self
            .http_client
            .post("https://router.huggingface.co/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("HuggingFace API error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = if let Some(choices) = response_json["choices"].as_array() {
            if let Some(first_choice) = choices.first() {
                if let Some(message) = first_choice["message"].as_object() {
                    message["content"].as_str().unwrap_or("")
                } else {
                    return Err(anyhow!(
                        "Invalid HuggingFace response format: missing message"
                    ));
                }
            } else {
                return Err(anyhow!(
                    "Invalid HuggingFace response format: empty choices"
                ));
            }
        } else {
            return Err(anyhow!(
                "Invalid HuggingFace response format: missing choices"
            ));
        };

        let answer = self.extract_final_answer(raw_answer);

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.75), // Open source models vary
            provider_used: LlmProvider::HuggingFace,
            model_used: model,
            tokens_used: response_json["usage"]["total_tokens"]
                .as_u64()
                .map(|t| t as u32),
            response_time_ms: 0,
            finish_reason: response_json["choices"][0]["finish_reason"]
                .as_str()
                .map(|s| s.to_string()),
            citations,
        })
    }

    /// Custom endpoint integration for self-hosted models
    async fn synthesize_with_custom(
        &self,
        query: &str,
        results: &[RagSearchResult],
    ) -> Result<LlmResponse> {
        let endpoint = self
            .config
            .custom_endpoint
            .as_ref()
            .ok_or_else(|| anyhow!("Custom endpoint not configured"))?;

        let model = self.get_model_name(&LlmProvider::Custom);
        let system_prompt = self.create_system_prompt();
        let user_prompt = self.create_user_prompt(query, results);

        // Use OpenAI-compatible format for custom endpoints
        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "stream": self.config.streaming
        });

        let response = self
            .http_client
            .post(format!("{}/v1/chat/completions", endpoint))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Custom endpoint error: {}", error_text));
        }

        let response_json: serde_json::Value = response.json().await?;

        let raw_answer = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid custom endpoint response format"))?;
        let answer = self.extract_final_answer(raw_answer);

        let usage = &response_json["usage"];
        let tokens_used = usage
            .get("total_tokens")
            .and_then(|t| t.as_u64())
            .map(|t| t as u32);
        let finish_reason = response_json["choices"][0]
            .get("finish_reason")
            .and_then(|r| r.as_str())
            .map(|s| s.to_string());

        let citations = self.extract_citations(&answer, results);

        Ok(LlmResponse {
            answer,
            sources_used: results.iter().map(|r| r.id.clone()).collect(),
            confidence: Some(0.8), // Assume reasonable confidence for custom
            provider_used: LlmProvider::Custom,
            model_used: model,
            tokens_used,
            response_time_ms: 0,
            finish_reason,
            citations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_final_answer_with_thinking_tags() {
        let client = LlmClient::new(LlmConfig::default()).unwrap();

        let response_with_thinking = r#"<thinking>
Let me analyze this query about Rust error handling.

The user is asking about Result types and how to handle errors properly.
I should explain the basics of Result<T, E> and common patterns.
</thinking>

**Quick Answer**
Rust uses `Result<T, E>` for error handling, where `T` is the success type and `E` is the error type.

**Key Points**
- Use `?` operator for error propagation
- `unwrap()` panics on error, avoid in production
- `expect()` provides custom panic message
- Pattern match with `match` for comprehensive handling"#;

        let extracted = client.extract_final_answer(response_with_thinking);

        assert!(!extracted.contains("<thinking>"));
        assert!(!extracted.contains("</thinking>"));
        assert!(extracted.contains("**Quick Answer**"));
        assert!(extracted.contains("Result<T, E>"));
    }

    #[test]
    fn test_extract_final_answer_with_think_tags() {
        let client = LlmClient::new(LlmConfig::default()).unwrap();

        let response_with_think = r#"<think>
This question is about JavaScript async/await patterns.

The user wants to understand how to handle asynchronous operations.
I should provide clear examples and best practices.
</think>

**Quick Answer**
Use `async/await` for handling asynchronous operations in JavaScript.

**Key Points**
- `async` functions return Promises
- `await` pauses execution until Promise resolves
- Use try/catch for error handling
- Avoid callback hell with Promise chains"#;

        let extracted = client.extract_final_answer(response_with_think);

        assert!(!extracted.contains("<think>"));
        assert!(!extracted.contains("</think>"));
        assert!(extracted.contains("**Quick Answer**"));
        assert!(extracted.contains("async/await"));
    }

    #[test]
    fn test_extract_final_answer_without_thinking() {
        let client = LlmClient::new(LlmConfig::default()).unwrap();

        let normal_response = r#"**Quick Answer**
This is a normal response without thinking tags.

**Key Points**
- Point 1
- Point 2"#;

        let extracted = client.extract_final_answer(normal_response);

        assert_eq!(extracted, normal_response);
    }

    #[test]
    fn test_extract_final_answer_with_thinking_prefix() {
        let client = LlmClient::new(LlmConfig::default()).unwrap();

        let response_with_prefix = r#"Let me think about this question carefully...

I need to consider the different aspects of the query.

Based on the search results:

**Quick Answer**
Here is the actual answer after thinking.

**Key Points**
- Important point 1
- Important point 2"#;

        let extracted = client.extract_final_answer(response_with_prefix);

        assert!(!extracted.contains("Let me think"));
        assert!(extracted.contains("**Quick Answer**"));
        assert!(extracted.contains("Here is the actual answer"));
    }
}
