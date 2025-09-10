# ⚙️ Configuration Guide

Complete guide to configuring manx for your workflow and preferences.

## 🔧 Quick Configuration

### View Current Settings
```bash
manx config --show
```

### Reset to Defaults
```bash
manx config --reset
```

## 🧠 Embedding Configuration

### Set Embedding Provider
```bash
# Use neural models for semantic search
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

# Use built-in hash embeddings (default)
manx config --embedding-provider hash
```

### Download and Configure Models
```bash
# 1. Download model
manx embedding download sentence-transformers/all-MiniLM-L6-v2

# 2. Set as active provider
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

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

### Google (Gemini)
```bash
manx config --llm-provider "google"
manx config --google-api "your-google-key"
manx config --llm-model "gemini-1.5-flash"           # Default
manx config --llm-model "gemini-1.5-pro"             # More capable
```

### Azure OpenAI
```bash
manx config --llm-provider "azure"
manx config --azure-api "your-azure-key"
manx config --azure-endpoint "https://your-resource.openai.azure.com/"
manx config --llm-model "gpt-4o"
```

### Ollama (Local Models)
```bash
manx config --llm-provider "ollama"
manx config --ollama-endpoint "http://localhost:11434"
manx config --llm-model "llama3.1:8b"
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
manx config --test-llm       # Test LLM connection
manx config --test-context7  # Test Context7 connection
```

## 📁 RAG Configuration

### Enable RAG Mode
```bash
manx config --rag-enabled
```

### Default RAG Behavior
```bash
# Always use RAG for searches
manx config --rag-default

# Use hybrid search (RAG + official docs)
manx config --rag-hybrid
```

## 🎛️ Advanced Settings

### Cache Configuration
```bash
manx config --cache-dir "~/custom-cache/"
manx config --cache-max-size "5GB"
manx config --cache-ttl "7d"  # Time to live
```

### Performance Tuning
```bash
manx config --max-results 20
manx config --timeout 30s
manx config --concurrent-requests 5
```

### Output Formatting
```bash
manx config --output-format "markdown"  # or "json", "plain"
manx config --color-output true
manx config --compact-results false
```

## 🔧 Environment Variables

You can also configure manx using environment variables:

```bash
export MANX_API_KEY="sk-your-context7-key"
export MANX_OPENAI_API="sk-your-openai-key"
export MANX_ANTHROPIC_API="your-anthropic-key"
export MANX_CACHE_DIR="~/custom-cache"
export MANX_LLM_PROVIDER="openai"
export MANX_LLM_MODEL="gpt-4o"
export MANX_EMBEDDING_PROVIDER="onnx:sentence-transformers/all-MiniLM-L6-v2"
```

## 📁 Configuration File

Manx stores configuration in `~/.config/manx/config.toml`:

```toml
[api]
context7_key = "sk-your-context7-key"
openai_key = "sk-your-openai-key"

[llm]
provider = "openai"
model = "gpt-4o"

[embeddings]
provider = "onnx:sentence-transformers/all-MiniLM-L6-v2"

[cache]
directory = "~/.cache/manx"
max_size = "2GB"
ttl = "7d"

[rag]
enabled = true
default_mode = false
hybrid_search = true

[output]
format = "markdown"
color = true
compact = false
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
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2
```

### Team Collaboration
```bash
# Semantic search + team docs
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2
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
- Keys are stored in `~/.config/manx/config.toml`
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
cp ~/.config/manx/config.toml ~/manx-config-backup.toml
```

### Restore Configuration
```bash
cp ~/manx-config-backup.toml ~/.config/manx/config.toml
```

### Export Settings
```bash
manx config --export > my-manx-settings.json
```

### Import Settings
```bash
manx config --import my-manx-settings.json
```

## ❓ Configuration Troubleshooting

### Check Configuration Status
```bash
manx config --show      # View all settings
manx config --validate  # Check for issues
```

### Fix Common Issues
```bash
# Reset corrupted config
manx config --reset

# Clear cache if embedding issues
manx cache clear --embeddings-only

# Test connections
manx config --test-llm
manx config --test-context7
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
# Different configs for different projects
cd ~/work-project/
export MANX_CONFIG_DIR="./.manx"
manx config --llm-provider "anthropic"  # Use Claude for work

cd ~/personal-project/
export MANX_CONFIG_DIR="./.manx"  
manx config --llm-provider "openai"     # Use OpenAI for personal
```

### Performance Optimization
```bash
# For slower machines
manx config --embedding-provider "onnx:sentence-transformers/all-MiniLM-L6-v2"

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