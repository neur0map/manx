# 🚀 Manx Setup Guide for Beginners

This guide walks you through setting up Manx step-by-step, from basic installation to advanced AI-powered documentation search.

## 📋 Table of Contents

1. [Installation](#-installation)
2. [Quick Test](#-quick-test)
3. [Enhanced Search (Recommended)](#-enhanced-search-recommended)
4. [Available Models](#-available-embedding-models)
5. [Personal Documentation (RAG)](#-personal-documentation-rag)
6. [AI Integration](#-ai-integration)
7. [Available LLM Providers](#-available-llm-providers)
8. [Common Workflows](#-common-workflows)
9. [Troubleshooting](#-troubleshooting)

## 🔧 Installation

### Option 1: Cargo (Recommended)
```bash
# Install from crates.io
cargo install manx-cli

# Verify installation
manx --version
```

### Option 2: Shell Script
```bash
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```

### Option 3: Manual Download
Download the latest binary from [GitHub Releases](https://github.com/neur0map/manx/releases).

**Binary Details:**
- **Size**: ~25MB (includes ONNX Runtime for neural embeddings)
- **Dependencies**: None - completely self-contained
- **Models**: Downloaded separately as needed (87MB-1.3GB each)

## ⚡ Quick Test

After installation, test that everything works:

```bash
# Search official documentation
manx snippet python "list comprehensions"
manx search "rust error handling"
manx doc react "hooks"
```

**What's happening:** Manx uses built-in hash embeddings and connects to Context7 API for official documentation. No setup required!

## 🧠 Enhanced Search (Recommended)

For much better search quality with semantic understanding:

### Step 1: Download a Model
```bash
# Download a lightweight, fast model (87MB)
manx embedding download sentence-transformers/all-MiniLM-L6-v2
```

### Step 2: Configure Manx to Use It
```bash
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2
```

### Step 3: Test Enhanced Search
```bash
# Try semantic search - should give much better results
manx snippet react "state management"
manx search "database connections"
```

**What's improved:** 
- 🎯 **Semantic understanding**: "database" matches "data storage"
- 📊 **Better ranking**: More relevant results first
- 🧠 **Intent matching**: Understands what you're really looking for

## 📚 Available Embedding Models

Choose the model that fits your needs:

### Lightweight & Fast (Recommended for Most Users)
```bash
# MiniLM - Best balance of speed and quality (87MB, 384 dimensions)
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2
```

### High Quality (For Better Semantic Understanding)
```bash
# MPNet - Superior quality, larger model (400MB, 768 dimensions)
manx embedding download sentence-transformers/all-mpnet-base-v2
manx config --embedding-provider onnx:sentence-transformers/all-mpnet-base-v2
```

### Specialized Models
```bash
# Question-Answer focused (87MB, 384 dimensions)
manx embedding download sentence-transformers/multi-qa-MiniLM-L6-cos-v1

# Multilingual support (400MB, 384 dimensions)
manx embedding download sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2

# BGE models - Optimized for retrieval
manx embedding download BAAI/bge-small-en-v1.5    # 134MB, 384 dimensions
manx embedding download BAAI/bge-base-en-v1.5     # 438MB, 768 dimensions
manx embedding download BAAI/bge-large-en-v1.5    # 1.34GB, 1024 dimensions
```

### Check Available Models
```bash
# List all models you can download
manx embedding list

# Check your current configuration
manx embedding status
```

### Switch Between Models
```bash
# Switch to a different model anytime
manx config --embedding-provider onnx:BAAI/bge-small-en-v1.5

# Verify the switch worked
manx embedding status
```

## 📁 Personal Documentation (RAG)

Index your own documentation for private, semantic search:

### Index Local Files
```bash
# Index a directory
manx index ~/dev-notes/
manx index ~/team-documentation/

# Index specific files
manx index ~/important-guide.md
```

### Index Web Documentation
```bash
# Index a single page
manx index https://docs.rust-lang.org/book/ch01-01-installation.html

# Deep crawl entire documentation sites
manx index https://docs.fastapi.tiangolo.com --crawl --max-depth 3
```

### Search Your Indexed Content
```bash
# Search only your indexed documents
manx search "authentication setup" --rag
manx snippet python "team coding standards" --rag

# View what you've indexed
manx sources list

# Clear all indexed content
manx sources clear
```

### Supported File Formats
- **Text files**: `.md`, `.txt`, `.rst`
- **Documents**: `.docx`
- **Web content**: Any HTTP/HTTPS URL
- **Crawling**: Automatic discovery of linked pages

## 🤖 AI Integration

Add AI-powered synthesis for comprehensive answers:

### Step 1: Configure an LLM Provider
```bash
# OpenAI (most popular)
manx config --openai-api "sk-your-openai-key"

# Or use other providers (see full list below)
manx config --anthropic-api "your-anthropic-key" 
```

### Step 2: Test AI Enhancement
```bash
# Now get AI-powered explanations with citations
manx snippet react hooks
manx search "error handling best practices"
```

### Control AI Usage
```bash
# Force AI analysis
manx snippet python --llm

# Disable AI for this query
manx snippet python --no-llm

# Disable AI globally
manx config --llm-provider ""
```

## 🌐 Available LLM Providers

### OpenAI
```bash
manx config --llm-provider "openai"
manx config --llm-model "gpt-4o"              # Default
manx config --llm-model "gpt-4o-mini"         # Cheaper option
manx config --openai-api "sk-your-key"
```

### Anthropic (Claude)
```bash
manx config --llm-provider "anthropic" 
manx config --llm-model "claude-3-5-sonnet-20241022"  # Default
manx config --llm-model "claude-3-haiku-20240307"     # Cheaper option
manx config --anthropic-api "your-key"
```

### Groq (Fast inference)
```bash
manx config --llm-provider "groq"
manx config --llm-model "llama-3.1-8b-instant"       # Default
manx config --llm-model "llama-3.1-70b-versatile"    # More capable
manx config --groq-api "gsk_your-key"
```

### Google (Gemini)
```bash
manx config --llm-provider "google"
manx config --llm-model "gemini-1.5-flash"           # Default
manx config --llm-model "gemini-1.5-pro"             # More capable
manx config --google-api "your-key"
```

### Azure OpenAI
```bash
manx config --llm-provider "azure"
manx config --llm-model "gpt-4o"
manx config --azure-api "your-key"
manx config --azure-endpoint "https://your-resource.openai.azure.com/"
```

### Ollama (Local models)
```bash
manx config --llm-provider "ollama"
manx config --llm-model "llama3.1:8b"
manx config --ollama-endpoint "http://localhost:11434"
```

### Check Current Configuration
```bash
# View all current settings
manx config --show

# Test LLM connection
manx config --test-llm
```

## 🔄 Common Workflows

### For Developers
```bash
# Morning workflow: Quick React pattern lookup
manx snippet react "performance optimization"

# Debug session: Error investigation  
manx search "javascript memory leaks"

# Learning: New framework exploration
manx doc svelte "component lifecycle"
```

### For Teams
```bash
# Setup team knowledge base
manx index ~/team-handbook/
manx index ~/coding-standards/ 
manx index https://company-docs.internal.com --crawl

# Daily usage
manx search "deployment checklist" --rag
manx snippet "security protocols" --rag
```

### For Research
```bash
# Use high-quality model for better understanding
manx config --embedding-provider onnx:sentence-transformers/all-mpnet-base-v2

# Add AI for comprehensive analysis
manx config --openai-api "sk-your-key"

# Research workflow
manx search "machine learning architectures"
manx doc pytorch "transformers implementation"
```

## 🔧 Troubleshooting

### Installation Issues
```bash
# Update Rust toolchain
rustup update

# Clear cargo cache
cargo clean

# Reinstall
cargo uninstall manx-cli
cargo install manx-cli
```

### Model Download Issues
```bash
# Check internet connection
ping huggingface.co

# Retry with force flag
manx embedding download sentence-transformers/all-MiniLM-L6-v2 --force

# Check available disk space (models are 87MB-1.3GB)
df -h
```

### Rate Limiting Issues
```bash
# Get Context7 API key for higher limits
manx config --api-key "sk-your-context7-key"

# Clear cache if needed
manx cache clear
```

### Performance Issues
```bash
# Use smaller model for faster inference
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

# Check system resources
manx embedding status
```

### Common Error Messages

**"Model not found"**
```bash
# Download the model first
manx embedding download sentence-transformers/all-MiniLM-L6-v2
```

**"ONNX embeddings feature not enabled"**
```bash
# This shouldn't happen with cargo install, but if it does:
cargo install manx-cli --features onnx-embeddings
```

**"Failed to connect to Context7"**
```bash
# Check internet connection or use offline mode
manx search "topic" --offline
```

## 🎯 Next Steps

1. **Start simple**: Use default installation for immediate value
2. **Enhance search**: Download a model for better semantic understanding  
3. **Add your docs**: Index personal documentation with RAG
4. **Optional AI**: Add LLM integration for comprehensive answers

Each step is optional and builds on the previous one. You can stop at any level that meets your needs!

## 📞 Getting Help

- **Documentation**: Full command reference in the main README
- **Issues**: [GitHub Issues](https://github.com/neur0map/manx/issues)
- **Examples**: Check the `examples/` directory in the repository

Happy documenting! 🚀