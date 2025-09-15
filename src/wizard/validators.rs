use anyhow::Result;

pub fn validate_api_key(key: &str, provider: &str) -> bool {
    match provider {
        "OpenAI" => key.starts_with("sk-") && key.len() > 20,
        "Anthropic" => key.starts_with("sk-ant-") && key.len() > 20,
        "Groq" => key.starts_with("gsk_") && key.len() > 20,
        "HuggingFace" => key.starts_with("hf_") && key.len() > 20,
        "Context7" => !key.is_empty() && key.len() > 10,
        _ => !key.is_empty(),
    }
}

pub async fn test_context7_api(api_key: &str) -> Result<bool> {
    // Simple validation for now - could make actual API call
    Ok(validate_api_key(api_key, "Context7"))
}

pub async fn test_llm_api(provider: &str, api_key: &str) -> Result<bool> {
    // Validate key format
    Ok(validate_api_key(api_key, provider))
}

pub async fn test_embedding_setup(provider: &crate::rag::EmbeddingProvider) -> Result<bool> {
    use crate::rag::embeddings::EmbeddingModel;
    use crate::rag::EmbeddingConfig;

    let config = EmbeddingConfig {
        provider: provider.clone(),
        dimension: 384,
        model_path: None,
        api_key: None,
        endpoint: None,
        timeout_seconds: 30,
        batch_size: 32,
    };

    // Try to create embedding model and test it
    match EmbeddingModel::new_with_config(config.clone()).await {
        Ok(model) => {
            // Try to get dimension as a simple test
            match model.get_dimension().await {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}
