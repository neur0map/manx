# About Manx - The Intelligent Documentation Finder

**Manx** is a blazing-fast, Rust-powered CLI tool that revolutionizes how developers find and access documentation. Built for speed and privacy, manx provides instant access to official documentation, code snippets, and your own knowledge base through advanced semantic search and optional AI synthesis.

## 🎯 Core Mission

Manx solves the documentation discovery problem that every developer faces:
- **Time waste**: Hunting through search engines for accurate, up-to-date documentation
- **Information overload**: Sifting through outdated tutorials and Stack Overflow answers
- **Knowledge scatter**: Your team's documentation spread across wikis, PDFs, and internal tools
- **Context switching**: Leaving the terminal to find answers

**Solution**: One command, instant results, right in your terminal.

## 🏗️ Architecture & Design Philosophy

### Design Principles
1. **Privacy First**: Core functionality works entirely offline, your data stays local
2. **Performance Obsessed**: Sub-second search results, optimized neural inference
3. **Progressive Enhancement**: Works great out-of-the-box, gets better with configuration
4. **Developer Focused**: Terminal-native, scriptable, automation-friendly
5. **Security Conscious**: Content sanitization, secure defaults, local processing

### Technical Foundation
- **Language**: Rust (for memory safety, performance, and zero-cost abstractions)
- **Binary Size**: 25MB single executable (includes ONNX Runtime for neural embeddings)
- **Neural Models**: Downloaded separately (87MB-1.3GB each, user choice)
- **Memory Usage**: <50MB RAM during operation (varies with model size)
- **Startup Time**: <50ms (hash mode), <200ms (neural model loading)
- **Supported Platforms**: Linux, macOS, Windows (x86_64, ARM64)

## 🚀 Four Capability Levels

Manx operates in four progressive capability levels, each building on the previous:

### 1. **Default Mode** (No Setup Required)
- **Hash-based embeddings**: Fast, lightweight algorithm (0ms processing, 0MB storage)
- **Official documentation**: Context7 API integration for real-time access
- **Keyword search**: Excellent for exact phrase matching
- **Offline capable**: Works without internet once cached

### 2. **Enhanced Mode** (1-Command Setup)
- **Neural embeddings**: HuggingFace transformer models (87-400MB download)
- **Semantic understanding**: "database connection" matches "data storage"
- **Intent matching**: Superior relevance ranking
- **Easy installation**: `manx embedding download sentence-transformers/all-MiniLM-L6-v2`

### 3. **RAG Mode** (Local Knowledge Base)
- **Private document indexing**: Your markdown, PDFs, code, and web documentation
- **Deep crawling**: Automatically discovers interconnected documentation pages
- **Local vector storage**: File-based system, no external dependencies
- **Multi-format support**: `.md`, `.txt`, `.docx`, `.pdf`, HTML pages

### 4. **AI Mode** (Optional LLM Integration)
- **Multi-provider support**: OpenAI, Anthropic, Groq, OpenRouter, HuggingFace
- **Comprehensive synthesis**: Code examples + explanations + citations
- **Fine-grained control**: Per-command AI toggle
- **Cost optimization**: Uses AI only when beneficial

## 🔧 Core Features & Capabilities

### Lightning-Fast Search Commands

#### `snippet` - Code Examples & Quick References
```bash
manx snippet react "useEffect cleanup"
manx snippet fastapi "middleware setup"  
manx snippet python "async functions"
```
- Finds working code examples with clear explanations
- Supports version-specific queries (`react@18`, `django@4.2`)
- Semantic matching finds relevant patterns with different terminology

#### `search` - Official Documentation Discovery
```bash
manx search "kubernetes deployment strategies"
manx search "rust async programming patterns"
```
- Prioritizes official documentation sources (10x relevance boost)
- Falls back to trusted community sources with clear notifications
- Uses semantic embeddings for intelligent result ranking

#### `doc` - Comprehensive Documentation Browser
```bash
manx doc fastapi "authentication middleware"
manx doc python "async functions"
```
- Browses structured documentation sections
- Real-time access to official documentation
- Context-aware topic discovery

#### `get` - Result Retrieval & Export
```bash
manx get doc-3
manx get snippet-7 -o implementation.md
```
- Retrieve specific search results by ID
- Export to various formats (Markdown, JSON)
- Scriptable for automation workflows

### Advanced Knowledge Management

#### Local Document Indexing
```bash
# Index individual files or directories
manx index ~/documentation/
manx index ./API-Guide.pdf
manx index https://docs.example.com

# Deep crawl entire documentation sites
manx index https://docs.rust-lang.org --crawl
manx index https://fastapi.tiangolo.com --crawl --max-depth 3
```

#### Source Management
```bash
manx sources list           # View all indexed sources
manx sources clear          # Clear all indexed documents
```

#### Cache System
```bash
manx cache stats            # Show cache usage and statistics
manx cache clear            # Clear all cached results
```

## 🧠 Embedding System Architecture

Manx features a flexible, pluggable embedding system that automatically selects the best search method:

### Supported Providers

#### Hash Provider (Default)
- **Speed**: 0ms processing time
- **Storage**: 0MB disk usage
- **Quality**: Excellent for exact keyword matching
- **Privacy**: 100% offline operation

#### ONNX Provider (Local Neural Models)
- **Models**: HuggingFace transformer models
- **Speed**: 0ms after initial loading
- **Storage**: 87-400MB per model
- **Quality**: Superior semantic understanding
- **Privacy**: 100% local processing

#### API Providers
- **OpenAI**: `text-embedding-3-small`, `text-embedding-3-large`
- **HuggingFace**: Thousands of embedding models
- **Ollama**: Local model server integration
- **Custom**: Self-hosted embedding endpoints

### Configuration & Management
```bash
# View current embedding setup
manx embedding status

# List available models for download
manx embedding list --available

# Download and install neural models
manx embedding download sentence-transformers/all-MiniLM-L6-v2

# Switch between providers instantly
manx config --embedding-provider hash
manx config --embedding-provider onnx:all-MiniLM-L6-v2
manx config --embedding-provider openai:text-embedding-3-small

# Test embedding quality
manx embedding test "your sample query"
```

## 🗂️ RAG System (Retrieval-Augmented Generation)

### Local Vector Storage
- **File-based system**: No external vector database required
- **JSON storage**: Human-readable, debuggable vector storage
- **Incremental updates**: Add documents without rebuilding entire index
- **Security**: Content sanitization and PDF validation

### Document Processing Pipeline
1. **Content Extraction**: Text from documents, code syntax awareness
2. **Chunking**: Intelligent text segmentation preserving context
3. **Embedding Generation**: Vector representations using configured provider
4. **Storage**: Local JSON files with metadata and vectors
5. **Indexing**: File-based search optimization

### Supported Formats & Sources
- **Local Documents**: `.md`, `.txt`, `.docx`, `.pdf`
- **Code Files**: Language-aware processing and syntax highlighting
- **Web Content**: Single pages with automatic text extraction
- **Deep Crawling**: Automatic discovery of interconnected documentation
- **URL Validation**: Security checks for web content processing

### Deep Crawling Capabilities
```bash
# Discover all linked pages automatically
manx index https://docs.rust-lang.org --crawl

# Control crawl depth and scope  
manx index https://fastapi.tiangolo.com --crawl --max-depth 2 --max-pages 50
```

## 🤖 AI Integration System

### Multi-Provider Architecture
Manx supports multiple AI providers with automatic failover and load balancing:

#### OpenAI Integration
- **Models**: GPT-4, GPT-3.5-turbo, GPT-4-turbo
- **Features**: Function calling, streaming responses
- **Strengths**: High-quality synthesis, broad knowledge

#### Anthropic Integration  
- **Models**: Claude 3.5 Sonnet, Claude 3 Haiku
- **Features**: Large context windows (200K tokens)
- **Strengths**: Code understanding, safety-focused responses

#### Groq Integration
- **Models**: Llama 3.1, Mixtral, Gemma
- **Features**: Ultra-fast inference (<100ms)
- **Strengths**: Cost-effective, low latency

#### Other Providers
- **OpenRouter**: Multi-model access, pay-per-use
- **HuggingFace**: Open-source models, research access
- **Custom**: Self-hosted model endpoints

### AI Enhancement Features
- **Documentation Synthesis**: Combines multiple sources with AI analysis
- **Code Generation**: Working examples with explanations
- **Citation Tracking**: Links to original documentation sources  
- **Context Awareness**: Uses search results as knowledge base
- **Quality Control**: Verification and authenticity checking

### Configuration & Control
```bash
# Configure AI providers
manx config --openai-api "sk-your-openai-key"
manx config --anthropic-api "sk-ant-your-anthropic-key"
manx config --groq-api "gsk-your-groq-key"

# Set preferred provider and model
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-sonnet"

# Per-command AI control
manx snippet react hooks              # Uses AI if configured
manx snippet react hooks --no-llm     # Forces retrieval-only mode

# Disable AI entirely
manx config --llm-provider ""
```

## ⚙️ Configuration System

### Configuration File Location
```
~/.config/manx/config.json
```

### Complete Configuration Structure
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
    "enabled": true,
    "index_path": "~/.cache/manx/rag_index",
    "max_results": 10,
    "similarity_threshold": 0.6,
    "allow_pdf_processing": false,
    "embedding": {
      "provider": "Hash",
      "dimension": 384,
      "model_path": null,
      "api_key": null,
      "endpoint": null,
      "timeout_seconds": 30,
      "batch_size": 32
    }
  },
  "llm": {
    "openai_api_key": null,
    "anthropic_api_key": null,
    "groq_api_key": null,
    "openrouter_api_key": null,
    "huggingface_api_key": null,
    "custom_endpoint": null,
    "preferred_provider": null,
    "model_name": null
  }
}
```

### Environment Variables
```bash
export NO_COLOR=1                    # Disable colored output
export MANX_CACHE_DIR=~/custom-cache # Custom cache directory
export MANX_API_KEY=sk-xxx           # Context7 API key
export MANX_DEBUG=1                  # Enable debug logging
```

## 📊 Performance Characteristics

### Speed Benchmarks
- **Hash embeddings**: 0ms processing time
- **ONNX embeddings**: 0ms after model loading (~500ms initial load)
- **API embeddings**: ~100ms network latency
- **Search results**: <1 second for snippets, <2 seconds for web search
- **Cache retrieval**: <50ms for cached results

### Resource Usage
- **Binary size**: 25MB (single executable with ONNX Runtime)
- **Memory usage**: <50MB RAM during operation (includes neural model loading)
- **Disk usage**: Configurable cache size (default: 100MB max)
- **Network**: Only for API calls and web search (optional)

### Scalability Limits
- **Local index**: Tested with 100k+ documents
- **Concurrent searches**: Multi-threaded processing
- **Cache size**: Automatically managed with LRU eviction
- **API rate limits**: Handled with exponential backoff

## 🔐 Security & Privacy Features

### Data Privacy
- **Local processing**: Core functionality works entirely offline
- **No telemetry**: Zero data collection or usage tracking
- **Secure defaults**: PDF processing disabled by default
- **API isolation**: LLM providers only receive search context, not personal data

### Content Security
- **PDF validation**: Malicious document detection
- **Content sanitization**: XSS and injection prevention
- **URL validation**: HTTPS enforcement, domain filtering
- **File system protection**: Sandboxed document processing

### Network Security
- **TLS enforcement**: All external connections use HTTPS
- **Certificate validation**: Full SSL/TLS certificate checking
- **Rate limiting**: Built-in protection against API abuse
- **Timeout handling**: Network request timeout protection

## 🎯 Use Cases & Workflows

### Individual Developer Workflows

#### Morning Coding Session
```bash
# Check React patterns for current project
manx snippet react "performance optimization"
# Result: Official React docs + your optimization notes

# Debug memory leak issue
manx search "javascript memory leaks debugging"
# Result: MDN docs + Stack Overflow + your debugging notes
```

#### Learning New Technology
```bash
# Explore framework documentation
manx doc svelte "component lifecycle"
# Result: Official Svelte docs with clear examples

# Find implementation patterns
manx snippet svelte "state management"
# Result: Working code examples with explanations
```

### Team Development Workflows

#### Developer Onboarding
```bash
# Index team documentation
manx index ~/team-handbook/
manx index ~/coding-standards/
manx index https://company-wiki.internal.com --crawl

# New developer searches
manx snippet "deployment process"
# Result: Official CI/CD docs + team-specific procedures

manx search "code review guidelines"
# Result: Team standards + industry best practices
```

#### Production Troubleshooting
```bash
# Quick reference during incidents
manx snippet kubernetes "pod restart debugging"
# Result: Official K8s docs + team runbooks

# Historical knowledge search
manx search "payment processing timeout" --rag
# Result: Previous incident reports + solution documentation
```

### Enterprise Knowledge Management

#### Documentation Centralization
```bash
# Index various documentation sources
manx index ~/enterprise-docs/
manx index https://internal-api-docs.company.com --crawl
manx index ~/compliance-procedures/

# Unified search across all sources
manx search "security compliance procedures"
# Result: Policy docs + implementation guides + audit checklists
```

#### Training & Compliance
```bash
# Onboarding material access
manx snippet "security protocols"
# Result: Company security guidelines + implementation examples

# Regulatory compliance queries
manx doc "gdpr data handling procedures" --rag
# Result: Legal requirements + company implementation guides
```

### Research & Documentation Workflows

#### Technical Research
```bash
# Gather information on emerging technologies
manx search "rust async performance patterns"
# Result: Official docs + research papers + community best practices

# Compare implementation approaches
manx snippet "database connection pooling comparison"
# Result: Multiple language examples + performance considerations
```

#### Documentation Creation
```bash
# Research before writing
manx search "api documentation best practices"
# Result: Industry standards + tool recommendations

# Find code examples for documentation
manx snippet python "type hints advanced usage"
# Result: Official examples + real-world patterns
```

## 🛠️ Command Reference

### Search Commands
```bash
# Basic searches
manx search "query"                    # Web documentation search
manx snippet library "pattern"        # Code snippet search  
manx doc library "topic"              # Documentation browser

# Advanced options
manx search "query" --limit 15         # Control result count
manx snippet react "hooks" --no-llm    # Disable AI synthesis
manx search "auth" --rag              # Search only local documents
manx doc fastapi --output api-ref.md  # Export documentation
```

### Knowledge Management
```bash
# Document indexing
manx index /path/to/docs              # Index local directory
manx index file.pdf                   # Index single file
manx index https://docs.example.com   # Index web page
manx index https://docs.site.com --crawl --max-depth 2  # Deep crawl

# Source management  
manx sources list                     # List indexed sources
manx sources clear                    # Clear all sources
```

### Configuration
```bash
# View settings
manx config --show                    # Display current configuration

# Context7 API
manx config --api-key "sk-key"        # Set Context7 API key

# AI providers
manx config --openai-api "sk-key"     # Configure OpenAI
manx config --llm-provider "anthropic" # Set preferred provider
manx config --llm-model "claude-3-sonnet" # Set specific model

# Embedding system
manx config --embedding-provider "hash" # Use hash-based embeddings
manx config --embedding-provider "onnx:all-MiniLM-L6-v2" # Use neural model

# RAG system
manx config --rag on                  # Enable local RAG
manx config --rag off                 # Disable local RAG
```

### Embedding Management
```bash
# Model management
manx embedding list --available       # Show available models
manx embedding download model-name    # Download neural model
manx embedding status                 # Show current setup
manx embedding test "sample query"    # Test embedding quality
```

### Cache & Maintenance
```bash
# Cache operations
manx cache stats                      # Show cache statistics
manx cache clear                      # Clear all cached data
manx cache list                       # List cached libraries

# System maintenance
manx update --check                   # Check for updates
manx update                          # Update to latest version
```

### Advanced Usage
```bash
# Automation & scripting
manx search "query" -q                # JSON output for scripts
manx snippet react hooks --output results.json  # Export results

# Debugging & development
manx --debug search "query"           # Enable debug logging
manx search "query" --offline         # Force offline mode
```

## 🚀 Getting Started Guide

### 1. Installation
```bash
# Using Cargo (recommended)
cargo install manx-cli

# Using shell installer
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash

# Verify installation
manx --version
```

### 2. Basic Usage (Works Immediately)
```bash
# Search for documentation
manx search "docker compose production"

# Find code examples  
manx snippet react "state management"

# Browse official docs
manx doc python "async functions"
```

### 3. Enhanced Setup (Optional)
```bash
# Get Context7 API key for higher rate limits
manx config --api-key "sk-your-context7-key"

# Install neural embeddings for better search
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

# Configure AI provider (optional)
manx config --openai-api "sk-your-openai-key"
```

### 4. Index Your Documentation (Optional)
```bash
# Index local documentation
manx index ~/dev-notes/
manx index ~/project-docs/

# Index web documentation
manx index https://docs.your-framework.com --crawl

# Enable RAG search
manx config --rag on

# Search your indexed content
manx search "team processes" --rag
```

## 🔧 Troubleshooting Guide

### Common Issues & Solutions

#### "No results found"
```bash
# Check Context7 API key configuration
manx config --api-key "sk-your-context7-key"

# Clear cache and retry
manx cache clear
manx snippet fastapi
```

#### Rate Limiting Issues
```bash
# Without Context7 API key, shared limits apply
# Solution: Get dedicated API key
manx config --api-key "sk-your-context7-key"
```

#### Embedding Model Issues
```bash
# Check current embedding setup
manx embedding status

# Test embedding functionality
manx embedding test "sample query"

# Reinstall model if needed
manx embedding download sentence-transformers/all-MiniLM-L6-v2 --force
```

#### RAG System Issues
```bash
# Check indexed sources
manx sources list

# Verify index status
manx cache stats

# Re-index if needed
manx sources clear
manx index ~/your-docs/
```

#### AI Integration Issues
```bash
# Check current LLM configuration
manx config --show

# Test specific provider
manx config --llm-provider "openai"
manx snippet python "functions"  # Should show AI analysis

# Disable AI if problematic
manx config --llm-provider ""
```

### Debug Mode
```bash
# Enable detailed logging
manx --debug snippet react hooks 2>&1 | tee debug.log

# Check system configuration
manx config --show

# View cache and system stats
manx cache stats
```

## 🏆 What Makes Manx Special

### Compared to Traditional Solutions

#### vs. Web Search
- **Speed**: Sub-second results vs. minutes of browsing
- **Accuracy**: Official sources prioritized vs. mixed quality results  
- **Context**: Terminal-native vs. context switching
- **Privacy**: Local processing vs. tracking and ads

#### vs. Documentation Websites
- **Speed**: Instant search vs. slow site navigation
- **Scope**: Cross-library search vs. single-source browsing
- **Offline**: Cached results vs. internet dependency
- **Integration**: CLI automation vs. manual copying

#### vs. AI Coding Assistants
- **Accuracy**: Official documentation vs. hallucination risk
- **Cost**: One-time setup vs. ongoing subscription
- **Privacy**: Local processing vs. cloud dependency
- **Control**: Fine-grained configuration vs. black box

### Unique Advantages

#### Progressive Enhancement
- Works excellently out-of-the-box with zero configuration
- Each enhancement level provides meaningful improvements
- No lock-in to external services or subscriptions

#### Privacy-First Architecture
- Core functionality completely offline
- Your documents never leave your machine
- Optional AI integration with clear data boundaries

#### Developer-Centric Design
- Terminal-native for seamless workflow integration
- Scriptable and automation-friendly
- Respects developer preferences (colors, output format)

#### Performance Engineering
- Rust-powered for memory safety and speed
- Single 25MB binary with embedded ONNX Runtime, no additional dependencies
- Sub-second response times even with large document sets

## 🎯 Future Roadmap

### Planned Features
- **Cost tracking**: Token usage and cost calculation per AI provider
- **Advanced PDF processing**: Improved document parsing and security
- **Plugin system**: Custom embedding providers and output formatters
- **Team collaboration**: Shared knowledge bases and synchronized indexes
- **IDE integrations**: VS Code, Neovim, and Emacs extensions

### Community & Contributions
Manx is open-source and welcomes contributions:
- **Performance improvements**: Make search even faster
- **Document parsers**: Support for more file formats
- **UI enhancements**: Better terminal output and interaction
- **Testing**: Expand test coverage and quality assurance
- **Documentation**: Improve guides and examples

---

**Manx transforms documentation discovery from a time-consuming chore into an instant, powerful capability right in your terminal. Whether you're debugging at 2 AM, onboarding new team members, or researching new technologies, manx delivers the knowledge you need, when you need it, with the privacy and performance you deserve.**