# 🚀 Manx - Lightning-Fast Documentation Finder

> *Blazing-fast CLI tool for developers to find documentation, code snippets, and answers instantly*

<div align="center">

![GitHub Release](https://img.shields.io/github/v/release/neur0map/manx)
![Crates.io Version](https://img.shields.io/crates/v/manx-cli)
![GitHub Downloads](https://img.shields.io/github/downloads/neur0map/manx/total?label=github%20downloads)
![Crates.io Downloads](https://img.shields.io/crates/d/manx-cli?label=crates.io%20downloads)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Binary Size](https://img.shields.io/badge/binary-5.4MB-blue.svg)

**🚀 [Quick Start](#-quick-start) • 📚 [Documentation](#-complete-command-reference) • ⚙️ [Configuration](#️-configuration)**

</div>

## ✨ What Makes Manx Special?

Manx is the **fastest way to find documentation and code snippets** from your terminal with **three levels of capability**:

<table>
<tr>
<td width="25%" align="center">

### **🚀 Default Mode**
**Works immediately - no setup**

⚡ **Hash Embeddings**  
Built-in algorithm (0ms)

📚 **Official Docs**  
Context7 integration  

🔍 **Keyword Search**  
Great exact matching

💾 **Zero Storage**  
No downloads needed

</td>
<td width="25%" align="center">

### **🧠 Enhanced Mode**
**Better search - 1 command setup**

🤖 **Neural Embeddings**  
HuggingFace models (87-400MB)

🎯 **Semantic Understanding**  
"database" = "data storage"

📊 **Intent Matching**  
Superior result relevance

🔄 **Easy Installation**  
`manx embedding download`

</td>
<td width="25%" align="center">

### **📂 RAG Mode**
**Your docs + AI - local setup**

🔒 **Private Documents**  
Your indexed files only

🎯 **Semantic + AI Search**  
Your knowledge + LLM synthesis

📁 **Multi-format Support**  
PDF, Markdown, DOCX, URLs

🔍 **Use `--rag` flag**  
`manx search "topic" --rag`

</td>
<td width="25%" align="center">

### **🤖 AI Mode**
**Full synthesis - API key setup**

🧠 **Neural + AI Analysis**  
Best of both worlds

💬 **Comprehensive Answers**  
Code + explanations + citations

🌐 **Multi-Provider Support**  
OpenAI, Anthropic, Groq, etc.

🎛️ **Fine-grained Control**  
Per-command AI toggle

</td>
</tr>
</table>

**Start with Default → Upgrade to Enhanced → Index your docs (RAG) → Add AI when needed**

## 🔧 **How Manx Works Under the Hood**

### **📊 Search Architecture Flow**

```mermaid
graph TD
    A[🔍 User Query] --> B{Search Command}
    B --> C[snippet/search/doc]
    C --> D[Query Processing]
    
    D --> E{Embedding Provider}
    E -->|Default| F[🔥 Hash Algorithm]
    E -->|Enhanced| G[🧠 Neural Model]
    E -->|API| H[☁️ OpenAI/HF API]
    
    F --> I[Vector Generation]
    G --> I
    H --> I
    
    I --> J{Data Sources}
    J -->|Official| K[📚 Context7 API]
    J -->|Local| L[📁 Indexed Docs]
    J -->|Cache| M[💾 Local Cache]
    
    K --> N[Semantic Search]
    L --> N  
    M --> N
    
    N --> O[Result Ranking]
    O --> P{AI Enhancement}
    P -->|Disabled| Q[📝 Documentation Results]
    P -->|Enabled| R[🤖 LLM Analysis]
    
    R --> S[🎯 Enhanced Response]
    Q --> T[📱 Terminal Output]
    S --> T
```

### **⚙️ Embedding System Architecture**

```mermaid
graph LR
    A[User Query] --> B{Embedding Config}
    
    B -->|hash| C[🔥 Hash Provider<br/>384D, 0ms, 0MB]
    B -->|onnx:model| D[🧠 ONNX Provider<br/>384-768D, 0ms, 87-400MB]  
    B -->|openai:model| E[☁️ OpenAI Provider<br/>1536-3072D, ~100ms, API]
    B -->|ollama:model| F[🏠 Ollama Provider<br/>Variable, ~50ms, Local]
    
    C --> G[Word Hashing<br/>+ N-gram Features]
    D --> H[Neural Network<br/>Inference]
    E --> I[REST API Call]
    F --> J[Local Model Server]
    
    G --> K[Vector Output]
    H --> K
    I --> K
    J --> K
    
    K --> L[Cosine Similarity<br/>Search]
    L --> M[Ranked Results]
```

### **🔄 Configuration Workflow**

```mermaid
sequenceDiagram
    participant U as User
    participant C as CLI
    participant M as Model Manager
    participant P as Provider
    participant S as Search Engine
    
    Note over U,S: Initial Setup (Optional)
    U->>C: manx embedding list --available
    C->>U: Show HuggingFace models
    
    U->>C: manx embedding download model-name
    C->>M: Download from HuggingFace
    M->>M: Extract dimensions from config.json
    M->>C: Model installed + metadata saved
    
    U->>C: manx config --embedding-provider onnx:model-name
    C->>M: Load model metadata
    M->>C: Dimension: 768, Path: ~/.cache/manx/models/
    C->>C: Update config with detected dimension
    
    Note over U,S: Daily Usage
    U->>C: manx snippet react "hooks"
    C->>P: Initialize provider from config
    P->>P: Load model (onnx) or use algorithm (hash)
    P->>S: Generate embeddings
    S->>U: Search results with semantic ranking
```

### **💾 Data Flow & Storage**

```mermaid
graph TB
    subgraph "🏠 Local Storage"
        A[~/.config/manx/<br/>config.json]
        B[~/.cache/manx/models/<br/>ONNX files + metadata]
        C[~/.cache/manx/rag/<br/>Indexed documents]
        D[~/.cache/manx/cache/<br/>API responses]
    end
    
    subgraph "🌐 External APIs"
        E[Context7<br/>Official Docs]
        F[HuggingFace<br/>Model Downloads]  
        G[OpenAI/Anthropic<br/>AI Synthesis]
        H[Ollama<br/>Local LLM Server]
    end
    
    subgraph "🔧 Core Engine"
        I[Embedding Providers]
        J[Search Algorithm]
        K[Result Processor]
        L[Terminal Renderer]
    end
    
    A --> I
    B --> I
    C --> J
    D --> J
    
    E --> J
    F --> B
    G --> K
    H --> I
    
    I --> J
    J --> K
    K --> L
    L --> M[🖥️ User Terminal]
```

---

## 🌟 **Core Features**

### 🚀 **Lightning-Fast Documentation Search**

Get instant access to documentation and code examples:

<table>
<tr>
<td width="50%">

**🔍 Web Documentation Search**
```bash
manx search "rust async programming"
```
*Returns: Instant access to official docs and tutorials*

**📚 Official Documentation Browser**  
```bash
manx doc python "async functions"
```
*Returns: Real-time official documentation with examples*

</td>
<td width="50%">

**💡 Code Snippet Search**
```bash
manx snippet react "useEffect cleanup"
```
*Returns: Working code examples with clear explanations*

**📁 Local Document Search (RAG)**
```bash
manx search "authentication" --rag
```
*Returns: Semantic search through your indexed documents*

</td>
</tr>
</table>

### 🎨 **Beautiful Terminal Experience**

Every response features:
- **📖 Clear Documentation** - Well-formatted, readable content
- **💡 Code Examples** - Syntax-highlighted, runnable code
- **📊 Quick Results** - Instant access to what you need
- **🔗 Source Links** - Direct links to official documentation

### 🤖 **Optional AI Enhancement**

Add AI analysis when you need deeper insights (completely optional):

```bash
# OpenAI (GPT-4, GPT-3.5)
manx config --openai-api "sk-your-openai-key"

# Anthropic (Claude)
manx config --anthropic-api "sk-ant-your-anthropic-key"  

# Groq (Ultra-fast inference)
manx config --groq-api "gsk-your-groq-key"

# OpenRouter (Multi-model access)
manx config --openrouter-api "sk-or-your-openrouter-key"

# HuggingFace (Open-source models)
manx config --huggingface-api "hf-your-huggingface-key"

# Custom endpoints (Self-hosted models)
manx config --custom-endpoint "http://localhost:8000/v1"
```

### 📂 **Local Document Search (RAG)**

Index and search your own documentation and code files:

```bash
# 1. Index your documents
manx index /path/to/your/docs
manx index /path/to/your/code

# 2. Enable local search
manx config --rag-enabled

# 3. Search your indexed content
manx search "authentication patterns" --rag
manx snippet python "async database" --rag  
manx doc fastapi "middleware setup" --rag
```

**Benefits:**
- 🔒 **Private & Offline** - Your documents never leave your machine
- 🎯 **Semantic Search** - Uses same embedding models as web search
- 🤖 **AI Integration** - Optional LLM synthesis from your own docs
- 📁 **File Formats** - Supports `.md`, `.txt`, `.pdf`, `.docx` + web URLs

---

## 🚀 **Quick Start**

### 1. **Installation**

```bash
# Using Cargo (Recommended)
cargo install manx-cli

# Using shell script
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash

# Manual download from releases
# https://github.com/neur0map/manx/releases/latest
```

### 2. **Core Commands**

```bash
# 🔍 Search web documentation instantly
manx search "docker compose production setup"

# 📚 Browse official documentation
manx doc fastapi "authentication middleware"

# 💡 Find working code snippets
manx snippet react "custom hooks patterns"

# 📁 Index your personal documentation (optional)
manx index ~/dev-notes/                               # Local directory
manx index https://docs.fastapi.tiangolo.com --crawl  # Deep crawl documentation site
manx search "team coding standards" --rag
```

### 3. **Context7 API Configuration (Recommended)**

```bash
# Get higher rate limits for documentation access
manx config --api-key "sk-your-context7-key"

# Test that everything is working
manx snippet python "list comprehensions"

# Optional: Add AI enhancement
manx config --openai-api "sk-your-openai-key"
manx search "topic"  # Now includes AI analysis when helpful
```

---

## 📋 **Complete Command Reference**

### 🔍 **Search Commands**

<table>
<tr>
<td width="50%">

**Web Search**
```bash
manx search "kubernetes deployment"
manx search "react hooks patterns"
manx search "python async" --limit 5
```

**Documentation Browser**
```bash  
manx doc fastapi "authentication"
manx doc react@18 "useState patterns"
manx doc python "async functions"
```

</td>
<td width="50%">

**Code Snippets**
```bash
manx snippet react "useEffect cleanup"  
manx snippet fastapi "middleware setup"
manx snippet python "decorators"
```

**Result Retrieval**
```bash
manx get doc-3                # Get specific result
manx get snippet-7 -o code.md # Export to file
```

</td>
</tr>
</table>

### 📁 **Knowledge Management**

```bash
# Index local documents
manx index ~/documentation/          # Directory
manx index ./README.md               # Single file  
manx index https://docs.example.com  # Web URL

# Deep crawl documentation sites (NEW!)
manx index https://docs.rust-lang.org --crawl                    # Discover all linked pages
manx index https://fastapi.tiangolo.com --crawl --max-depth 2    # Limited depth crawling
manx index https://docs.python.org --crawl --max-pages 10        # Limited page count

# Manage indexed sources
manx sources list                    # View all sources
manx sources clear                   # Clear all indexed docs

# Cache management
manx cache stats                     # Show cache info
manx cache clear                     # Clear cache
```

### ⚙️ **Configuration**

```bash
# View current settings
manx config --show

# Context7 API (for official docs - recommended)
manx config --api-key "sk-context7-key"

# AI Provider Configuration (optional)
manx config --openai-api "sk-key"       # OpenAI
manx config --anthropic-api "sk-key"    # Anthropic  
manx config --groq-api "gsk-key"        # Groq
manx config --llm-provider "groq"       # Set preferred provider
manx config --llm-model "llama-3.1-8b"  # Set specific model

# Switch between models
manx config --llm-provider "openai" --llm-model "gpt-4"
manx config --llm-provider "anthropic" --llm-model "claude-3-sonnet"

# Remove API keys / Disable AI
manx config --openai-api ""             # Remove OpenAI key
manx config --llm-provider ""           # Disable AI entirely
manx config --anthropic-api ""          # Remove Anthropic key

# Other Settings
manx config --cache-dir ~/my-cache      # Custom cache location
manx config --auto-cache off            # Disable auto-caching
```

---

## 🧠 **Personal Knowledge Base**

Index your documentation and notes for instant search:

### **📚 Index Your Knowledge**

```bash
# Personal development notes
manx index ~/coding-notes/
manx index ~/project-documentation/

# Team knowledge base  
manx index ~/company-wiki/
manx index ~/internal-procedures/

# Web documentation (single page)
manx index https://your-team-docs.com
manx index https://internal-api-docs.example.com

# Deep crawl entire documentation sites
manx index https://docs.your-framework.com --crawl              # Discover all pages automatically
manx index https://internal-wiki.company.com --crawl --max-depth 3  # Limit crawl depth
manx index https://team-knowledge.com --crawl --max-pages 50    # Limit total pages crawled
```

### **🔍 Unified Search Experience**

```bash
manx snippet "authentication setup"
```

**Returns:**
- 🌐 **Official docs** (FastAPI, OAuth, JWT guides)
- 📁 **Your notes** (team auth procedures, troubleshooting)  
- 🔗 **Direct links** to source documentation and files

### **🛡️ Security Features**

- **PDF Security**: Validates PDFs for malicious content
- **Content Sanitization**: Cleans and validates all indexed content
- **Local Processing**: RAG runs entirely locally
- **Privacy Control**: Core functionality works entirely offline

### **💾 Supported Formats**

- **Documents**: `.md`, `.txt`, `.docx`, `.pdf`
- **Web Content**: HTML pages with automatic text extraction
- **Code Files**: Syntax-aware indexing
- **URLs**: Single page or deep crawl entire documentation sites
- **Deep Crawling**: Automatically discovers and indexes interconnected documentation pages

---

## 🤖 **Optional AI Features**

### **🎯 Enhanced Analysis (When Enabled)**

When you configure an AI provider, responses include deeper analysis:

```
┌─────────────────────────────────────────┐
│ 📖 Documentation Results                │
└─────────────────────────────────────────┘

1. React Hooks Introduction
   https://reactjs.org/docs/hooks-intro.html
   
2. useState Hook Documentation  
   https://reactjs.org/docs/hooks-state.html
   
┌─────────────────────────────────────────┐
│ 🤖 AI Analysis (Optional)               │
└─────────────────────────────────────────┘
  ❯ Quick Summary
  React hooks allow you to use state and lifecycle 
  features in functional components.

  ❯ Key Insights
  • useState manages component state
  • useEffect handles side effects  
  • Custom hooks enable logic reuse
```

### **🔧 Provider-Specific Features**

<table>
<tr>
<td width="33%">

**OpenAI**
- GPT-4, GPT-3.5-turbo
- Function calling support
- Streaming responses
- High-quality synthesis

</td>
<td width="33%">

**Anthropic** 
- Claude 3.5 Sonnet
- Large context windows
- Excellent code understanding
- Safety-focused responses

</td>
<td width="33%">

**Groq**
- Ultra-fast inference
- Llama 3.1 models  
- Cost-effective
- Low latency

</td>
</tr>
</table>

### **🎛️ Fine-grained Control**

```bash
# Global AI settings
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-sonnet"

# Per-command control
manx search "topic"              # Fast documentation search
manx search "topic" --no-llm     # Force no AI analysis
manx snippet react hooks        # Code examples with optional AI insights
manx snippet react --no-llm     # Just the documentation
```

---

## 🔗 **Context7 Integration**

Access real-time official documentation:

### **⚡ Rate Limiting Solutions**

```bash
# Without API key: Shared rate limits (very restrictive)
manx snippet react hooks
# May hit rate limits after few searches

# With API key: Dedicated access (recommended)
manx config --api-key "sk-your-context7-key"
manx snippet react hooks  # Much higher limits
```

### **🔑 Get Your Context7 API Key**

1. Visit [Context7 Dashboard](https://context7.com/dashboard)
2. Create account or sign in
3. Generate API key (starts with `sk-`)
4. Configure: `manx config --api-key "sk-your-key"`

---

## 📊 **Performance & Features**

<table>
<tr>
<td width="50%">

**⚡ Performance**
- **Search Speed**: < 1 second (snippets), < 2 seconds (web search)
- **Binary Size**: 5.4MB single file
- **Memory Usage**: < 15MB RAM
- **Startup Time**: < 50ms
- **Cache Support**: Smart auto-caching

</td>
<td width="50%">

**🔧 Technical Features**
- **Multi-threading**: Parallel search processing
- **Smart Embeddings**: Hash-based (default) + ONNX neural models  
- **Vector Storage**: Local file-based RAG system
- **HTTP/2**: Modern API communication
- **Cross-platform**: Linux, macOS, Windows

</td>
</tr>
</table>

---

## 🧠 **Semantic Search & Embeddings**

Manx features a **flexible embedding system** that automatically chooses the best search method:

### **🚀 Getting Started (3 Commands)**

```bash
# 1. Works great immediately (no setup)
manx snippet react "state management"

# 2. Optional: Install better search (one-time setup)
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

# 3. Now enjoy superior semantic search
manx snippet react "state management"  # Much smarter results
```

### **📊 Capability Comparison**

| Feature | **Hash (Default)** | **Neural Models** |
|---------|-------------------|-------------------|
| **Setup** | None required | 1 command |
| **Speed** | 0ms (instant) | 0ms (after loading) |
| **Storage** | 0MB | 87-400MB |
| **Understanding** | Keyword matching | Semantic + contextual |
| **Privacy** | 100% offline | 100% local processing |
| **Quality** | Good for exact terms | Excellent for concepts |

### **⚙️ Advanced Configuration**

```bash
# Management commands
manx embedding list --available     # See available models
manx embedding status               # Check current setup
manx embedding test "your query"    # Test search quality

# Provider switching (instant)
manx config --embedding-provider hash                    # Default algorithm
manx config --embedding-provider onnx:all-MiniLM-L6-v2   # Local neural model  
manx config --embedding-provider openai:text-embedding-3 # API-based (requires key)
```

**HuggingFace installation recommended** - best search quality + privacy + no API costs.

---

## 🎯 **Real-World Use Cases**

### **👨‍💻 Individual Developer**

```bash
# Morning workflow: Check React patterns
manx snippet react "performance optimization"
# Returns: Official React docs + your optimization notes

# Debug session: Memory leak investigation  
manx search "javascript memory leaks"
# Returns: MDN docs + Stack Overflow + your debugging notes

# Learning: New framework exploration
manx doc svelte "component lifecycle"  
# Returns: Official Svelte docs with clear examples
```

### **👥 Development Team**

```bash
# Onboard new developer
manx index ~/team-handbook/
manx index ~/coding-standards/
manx snippet "deployment process"
# Returns: Official CI/CD docs + team procedures

# Solve production issue
manx search "kubernetes pod restart loops"
# Returns: K8s docs + team runbooks + troubleshooting guides
```

### **🔒 Privacy-Focused Usage**

```bash
# Index sensitive documentation locally
manx index ~/classified-procedures/
manx snippet "security protocols"
# Pure local search - works completely offline

# Team knowledge stays private
manx snippet "internal processes"
# Uses only local knowledge + official docs (no AI calls)
```

---

## 🛠️ **Installation Options**

<details>
<summary><strong>📦 Detailed Installation Guide</strong></summary>

### Cargo Installation (Recommended)
```bash
cargo install manx-cli
manx --version
```

### Shell Script Installer
```bash
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```

### Manual Binary Download

1. Download for your platform:
   - [Linux x86_64](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-unknown-linux-gnu)
   - [Linux ARM64](https://github.com/neur0map/manx/releases/latest/download/manx-aarch64-unknown-linux-gnu)
   - [macOS x86_64](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-apple-darwin)
   - [macOS ARM64](https://github.com/neur0map/manx/releases/latest/download/manx-aarch64-apple-darwin)
   - [Windows x86_64](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-pc-windows-msvc.exe)

2. Install:
   ```bash
   chmod +x manx-*
   sudo mv manx-* /usr/local/bin/manx
   ```

### From Source
```bash
git clone https://github.com/neur0map/manx.git
cd manx
cargo build --release
sudo cp target/release/manx /usr/local/bin/
```

</details>

<details>
<summary><strong>🔧 Advanced Configuration</strong></summary>

### Configuration File Location
```bash
~/.config/manx/config.json
```

### Full Configuration Example
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
    "allow_pdf_processing": false
  },
  "llm": {
    "openai_api_key": "sk-your-openai-key",
    "anthropic_api_key": "sk-ant-your-anthropic-key",
    "groq_api_key": "gsk-your-groq-key",
    "openrouter_api_key": "sk-or-your-openrouter-key",
    "huggingface_api_key": "hf-your-huggingface-key",
    "custom_endpoint": "http://localhost:8000/v1",
    "preferred_provider": "OpenAI",
    "model_name": "gpt-4"
  }
}
```

### Environment Variables
```bash
export NO_COLOR=1                    # Disable colors
export MANX_CACHE_DIR=~/cache        # Custom cache dir
export MANX_API_KEY=sk-xxx           # Context7 API key
export MANX_DEBUG=1                  # Enable debug logging
```

</details>

<details>
<summary><strong>🛠️ Troubleshooting</strong></summary>

### Common Issues

**Want to Add AI Analysis?**
```bash
# Check current configuration
manx config --show

# Set up an AI provider (optional)
manx config --openai-api "sk-your-key"

# Test enhanced functionality
manx snippet python "functions"
```

**Managing AI Configuration**
```bash
# Switch between providers
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-sonnet"

# Disable AI completely
manx config --llm-provider ""

# Remove specific API keys
manx config --openai-api ""
```

**"No results found"**
```bash
# Check Context7 API key setup
manx config --api-key "sk-your-context7-key"

# Clear cache and retry
manx cache clear
manx snippet fastapi
```

**Rate Limiting Issues**
```bash
# Without Context7 API key, you'll hit shared limits quickly
manx config --api-key "sk-your-context7-key"

# This provides much higher rate limits
```

**Local RAG Not Finding Documents**
```bash
# Check indexed sources
manx sources list

# Re-index if needed
manx sources clear
manx index ~/your-docs/
```

### Debug Mode
```bash
# Enable detailed logging
manx --debug snippet react hooks 2>&1 | tee debug.log

# Check configuration
manx config --show

# View cache stats
manx cache stats
```

</details>

---

## 🤝 **Contributing**

We welcome contributions! Areas where help is needed:

- **⚡ Performance** - Make search even faster
- **📄 Document Parsers** - Support for more file formats  
- **🎨 Terminal UI** - Enhance the visual experience
- **🧪 Testing** - Expand test coverage
- **📖 Documentation** - Improve guides and examples

### Development Setup
```bash
git clone https://github.com/neur0map/manx.git
cd manx
cargo build
cargo test
./target/debug/manx --help
```

---

## 📜 **License**

MIT License - see [LICENSE](LICENSE) for details.

## 🙏 **Acknowledgments**

- **Context7** - Excellent MCP documentation API
- **Rust Community** - Outstanding ecosystem and tooling
- **Contributors** - Making Manx better every day
- **LLM Providers** - Optional AI enhancement capabilities

---

## 🚧 **Roadmap & TODOs**

### 💰 **Cost & Usage Tracking**
- [ ] Add cost calculation functionality to LlmResponse struct
- [ ] Implement per-provider pricing models and cost tracking  
- [ ] Add usage statistics and cost reporting commands
- [ ] Implement token count breakdown (input/output/cached tokens)
- [ ] Implementation of local LLM support

---

<div align="center">

**Built with ❤️ for developers who need answers fast**

**[⬆️ Back to Top](#-manx---ai-powered-documentation-assistant)**

![Manx Demo](https://via.placeholder.com/600x300/1a1a1a/00d4aa?text=🤖+AI+Synthesis+Demo)

*Lightning-fast documentation search - right in your terminal*

</div>