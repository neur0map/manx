# ⚙️ Configuration Guide

Complete guide to configuring manx for your workflow and preferences.

## 🔧 Quick Configuration

### View Current Settings
```bash
manx config --show
```

### Reset to Defaults
```bash
# Note: Reset functionality not currently implemented
# To reset, delete the config file:
rm ~/.config/manx/config.json
```

## 🧠 Embedding Configuration

### Set Embedding Provider
```bash
# Use neural models for semantic search
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Use built-in hash embeddings (default)
manx config --embedding-provider hash
```

### Download and Configure Models
```bash
# 1. Download model
manx embedding download all-MiniLM-L6-v2

# 2. Set as active provider
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# 3. Verify configuration
manx embedding status
```

## 🤖 LLM Configuration

### OpenAI
```bash
manx config --llm-provider "openai"
manx config --openai-api "sk-your-openai-key"
manx config --llm-model "gpt-4o"              # Default
manx config --llm-model "gpt-4o-mini"         # Cheaper option
```

### Anthropic (Claude)
```bash
manx config --llm-provider "anthropic"
manx config --anthropic-api "your-anthropic-key"
manx config --llm-model "claude-3-5-sonnet-20241022"  # Default
manx config --llm-model "claude-3-haiku-20240307"     # Faster/cheaper
```

### Groq (Fast Inference)
```bash
manx config --llm-provider "groq"
manx config --groq-api "gsk_your-groq-key"
manx config --llm-model "llama-3.1-8b-instant"       # Default
manx config --llm-model "llama-3.1-70b-versatile"    # More capable
```

### HuggingFace
```bash
manx config --llm-provider "huggingface"
manx config --huggingface-api "your-huggingface-token"
manx config --llm-model "meta-llama/Llama-2-7b-chat-hf"
```

### OpenRouter
```bash
manx config --llm-provider "openrouter"
manx config --openrouter-api "sk-or-your-key"
manx config --llm-model "openai/gpt-4o"
```

### Custom Endpoints
```bash
manx config --llm-provider "custom"
manx config --custom-endpoint "https://your-api-endpoint.com"
manx config --llm-model "your-model-name"
```


### Disable LLM
```bash
manx config --llm-provider ""
```

## 🔑 API Keys

### Context7 (Documentation Access)
```bash
# Recommended: Get higher rate limits
manx config --api-key "sk-your-context7-key"

# Get API key at: https://context7.com/dashboard
```

### Test API Connections
```bash
# Note: API testing functionality not currently implemented
# Test manually by running a simple command:
manx search "test query" --limit 1
```

## 📁 RAG Configuration

### Enable RAG Mode
```bash
manx config --rag on
```

### Disable RAG Mode
```bash
manx config --rag off
```

## 🎛️ Advanced Settings

### Cache Configuration
```bash
manx config --cache-dir "~/custom-cache/"
manx config --max-cache-size 5000  # Size in MB
manx config --cache-ttl 168  # Time to live in hours
manx config --auto-cache on  # Enable auto-caching
```

## 🔧 Environment Variables

Limited environment variable support:

```bash
# Only NO_COLOR is currently supported for disabling color output
export NO_COLOR=1

# Note: MANX_* environment variables are not implemented yet
# Use the config command instead: manx config --show
```

## 📁 Configuration File

Manx stores configuration in `~/.config/manx/config.json`:

```json
{
  "api_key": "sk-your-context7-key",
  "cache_dir": null,
  "default_limit": 10,
  "offline_mode": false,
  "color_output": true,
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 100,
  "rag": {
    "enabled": false,
    "embedding_provider": "hash",
    "embedding_api_key": null,
    "embedding_model_path": null,
    "embedding_dimension": 384
  },
  "llm": {
    "provider": "auto",
    "model": null,
    "openai_api_key": null,
    "anthropic_api_key": null,
    "groq_api_key": null,
    "openrouter_api_key": null,
    "huggingface_api_key": null,
    "custom_endpoint": null
  }
}
```

## 🎯 Configuration Presets

### Minimal Setup (Default)
```bash
# Just install and use - no configuration needed
cargo install manx-cli
manx snippet python "functions"
```

### Enhanced Search
```bash
# Better semantic understanding
manx embedding download all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2
```

### Team Collaboration
```bash
# Semantic search + team docs
manx embedding download all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2
manx config --rag-enabled
manx index ~/team-docs/
```

### Research Mode
```bash
# Full AI-powered analysis
manx embedding download sentence-transformers/all-mpnet-base-v2
manx config --embedding-provider onnx:sentence-transformers/all-mpnet-base-v2
manx config --openai-api "sk-your-key"
manx config --rag-enabled
```

### Privacy-Focused
```bash
# No external API calls
manx config --llm-provider "ollama"
manx config --api-key ""  # Disable Context7
manx config --rag-default  # Use only indexed docs
```

## 🔒 Security Considerations

### API Key Storage
- Keys are stored in `~/.config/manx/config.json`
- File permissions are set to user-only (600)
- Keys are never logged or transmitted except to configured providers

### Data Privacy
- **Local embeddings**: All neural processing happens locally
- **RAG documents**: Never leave your machine
- **API calls**: Only to configured providers (Context7, OpenAI, etc.)

### Network Security
- All API calls use HTTPS
- Certificate validation is enforced
- Timeouts prevent hanging connections

## 🔄 Migration & Backup

### Backup Configuration
```bash
cp ~/.config/manx/config.json ~/manx-config-backup.json
```

### Restore Configuration
```bash
cp ~/manx-config-backup.json ~/.config/manx/config.json
```

### Export/Import Settings
```bash
# Note: Export/import functionality not currently implemented
# Manual backup and restore using cp commands above
```

## ❓ Configuration Troubleshooting

### Check Configuration Status
```bash
manx config --show      # View all settings
```

### Fix Common Issues
```bash
# Reset corrupted config
rm ~/.config/manx/config.json

# Clear cache if embedding issues
manx cache clear

# Test connections manually
manx search "test query" --limit 1
```

### Debug Mode
```bash
# Enable detailed logging
export RUST_LOG=debug
manx --debug search "test query"
```

## 💡 Pro Tips

### Context-Aware Configuration
```bash
# Note: Project-specific configuration not currently implemented
# Use global configuration in ~/.config/manx/config.json
```

### Performance Optimization
```bash
# For slower machines
manx config --embedding-provider "onnx:all-MiniLM-L6-v2"

# For maximum quality
manx config --embedding-provider "onnx:sentence-transformers/all-mpnet-base-v2"

# For retrieval tasks
manx config --embedding-provider "onnx:BAAI/bge-small-en-v1.5"
```

### Cost Optimization
```bash
# Use cheaper models
manx config --llm-model "gpt-4o-mini"
manx config --llm-model "claude-3-haiku"

# Limit API usage
manx config --max-llm-calls-per-hour 10
```