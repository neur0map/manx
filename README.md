# 🤖 Manx - AI-Powered Documentation Assistant

> *Blazing-fast CLI documentation finder with AI synthesis, local RAG, and multi-provider LLM support*

<div align="center">

![GitHub Release](https://img.shields.io/github/v/release/neur0map/manx)
![Crates.io Version](https://img.shields.io/crates/v/manx-cli)
![GitHub Downloads](https://img.shields.io/github/downloads/neur0map/manx/total?label=github%20downloads)
![Crates.io Downloads](https://img.shields.io/crates/d/manx-cli?label=crates.io%20downloads)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Binary Size](https://img.shields.io/badge/binary-2.9MB-blue.svg)

**🚀 [Quick Start](#-quick-start) • 🤖 [AI Setup](#-ai-integration)**

</div>

## ✨ What Makes Manx Special?

Manx transforms from a simple documentation finder into your **personal AI development assistant** by combining:

- **🔍 Official Documentation** - Context7 MCP integration for real-time docs
- **📁 Your Personal Knowledge** - Index local docs, notes, and wikis
- **🤖 AI Synthesis** - Multi-provider LLM integration (OpenAI, Anthropic, Groq, etc.)
- **⚡ Lightning Fast** - sub-second snippet searches, sub-2 second web searches
- **🎨 Beautiful UX** - Colorized, scannable terminal output

---

## 🌟 **NEW: Complete AI Integration**

### 🤖 **AI-Powered Answers for ALL Commands**

Every command now provides intelligent AI synthesis with beautiful, colorized output (this can be disabled with `--no-llm` or removing the API key from the config file):

<table>
<tr>
<td width="50%">

**🔍 Web Search + AI**
```bash
manx search "rust async programming"
```
*Returns: Web docs + AI synthesis with citations*

**📚 Documentation + AI**  
```bash
manx doc python "async functions"
```
*Returns: Official docs + AI summary with examples*

</td>
<td width="50%">

**💡 Code Snippets + AI**
```bash
manx snippet react "useEffect cleanup"
```
*Returns: Code examples + comprehensive AI explanation*

**📁 Local Docs + AI**
```bash
manx snippet "authentication setup"
```
*Returns: Official docs + your notes + AI synthesis*

</td>
</tr>
</table>

### 🎨 **Beautiful Terminal Experience**

Every AI response features:
- **❯ Quick Answer** - Immediate, concise response
- **❯ Key Points** - Essential information in scannable bullets
- **❯ Code Example** - Syntax-highlighted, runnable code
- **📖 Sources** - Clear citations with relevance scores

### 🔧 **Multi-Provider LLM Support**

Choose your AI provider with simple configuration:

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
# 🔍 Search web documentation with AI synthesis
manx search "docker compose production setup"

# 📚 Browse official documentation with AI summaries
manx doc fastapi "authentication middleware"

# 💡 Find code snippets with AI explanations  
manx snippet react "custom hooks patterns"

# 📁 Index your personal documentation
manx index ~/dev-notes/
manx index https://your-team-wiki.com/docs
```

### 3. **AI Configuration (Optional)**

```bash
# Configure your preferred AI provider
manx config --openai-api "sk-your-key-here"

# Verify AI is working
manx snippet python "list comprehensions"
# Now includes beautiful AI synthesis!

# Control AI usage per command
manx search "topic" --no-llm    # Disable AI for this search
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
manx search "react hooks patterns" --no-llm
manx search "python async" --limit 5
```

**Documentation Browser**
```bash  
manx doc fastapi "authentication"
manx doc react@18 "useState patterns"
manx doc python "async functions" --no-llm
```

</td>
<td width="50%">

**Code Snippets**
```bash
manx snippet react "useEffect cleanup"  
manx snippet fastapi "middleware setup"
manx snippet python "decorators" --no-llm
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

# AI Provider Configuration
manx config --openai-api "sk-key"       # OpenAI
manx config --anthropic-api "sk-key"    # Anthropic  
manx config --groq-api "gsk-key"        # Groq
manx config --llm-provider "groq"       # Set preferred provider
manx config --llm-model "llama-3.1-8b"  # Set specific model

# Context7 API (for official docs)
manx config --api-key "sk-context7-key"

# Other Settings
manx config --cache-dir ~/my-cache      # Custom cache location
manx config --auto-cache off            # Disable auto-caching
```

---

## 🧠 **Local RAG Integration**

Transform Manx into your personal knowledge assistant:

### **📚 Index Your Knowledge**

```bash
# Personal development notes
manx index ~/coding-notes/
manx index ~/project-documentation/

# Team knowledge base  
manx index ~/company-wiki/
manx index ~/internal-procedures/

# Web documentation
manx index https://your-team-docs.com
manx index https://internal-api-docs.example.com
```

### **🔍 Unified Search Experience**

```bash
manx snippet "authentication setup"
```

**Returns:**
- 🌐 **Official docs** (FastAPI, OAuth, JWT guides)
- 📁 **Your notes** (team auth procedures, troubleshooting)  
- 🤖 **AI synthesis** (comprehensive answer with citations)

### **🛡️ Security Features**

- **PDF Security**: Validates PDFs for malicious content
- **Content Sanitization**: Cleans and validates all indexed content
- **Local Processing**: RAG runs entirely locally
- **Privacy Control**: Use `--no-llm` for zero external API calls

### **💾 Supported Formats**

- **Documents**: `.md`, `.txt`, `.docx`, `.pdf`
- **Web Content**: HTML pages with automatic text extraction
- **Code Files**: Syntax-aware indexing
- **URLs**: Automatic content fetching and cleaning

---

## 🤖 **AI Integration Deep Dive**

### **🎯 Smart Answer Generation**

Every AI response follows a consistent, scannable format:

```
┌─────────────────────────────────────────┐
│ 🤖 AI Summary                           │
└─────────────────────────────────────────┘
  ❯ Quick Answer
  React hooks allow you to use state and lifecycle 
  features in functional components.

  ❯ Key Points  
  • useState manages component state [Source 1]
  • useEffect handles side effects [Source 2]
  • Custom hooks enable logic reuse [Source 3]

  ❯ Code Example
  ```javascript
  const [count, setCount] = useState(0);
  useEffect(() => {
    document.title = `Count: ${count}`;
  }, [count]);
  ```

  📖 Sources used:
  • React Hooks Introduction
  • useState Hook Documentation
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
manx search "topic"              # Use configured AI
manx search "topic" --no-llm     # Force retrieval-only
manx snippet react hooks        # AI synthesis enabled
manx snippet react --no-llm     # Raw results only
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
- **Binary Size**: 2.9MB single file
- **Memory Usage**: < 10MB RAM
- **Startup Time**: < 50ms
- **Cache Support**: Smart auto-caching

</td>
<td width="50%">

**🔧 Technical Features**
- **Multi-threading**: Parallel search processing
- **BERT Embeddings**: Semantic search understanding  
- **Vector Storage**: Local file-based RAG system
- **HTTP/2**: Modern API communication
- **Cross-platform**: Linux, macOS, Windows

</td>
</tr>
</table>

---

## 🎯 **Real-World Use Cases**

### **👨‍💻 Individual Developer**

```bash
# Morning workflow: Check React patterns
manx snippet react "performance optimization"
# Returns: Official React docs + your optimization notes + AI guide

# Debug session: Memory leak investigation  
manx search "javascript memory leaks"
# Returns: MDN docs + Stack Overflow + your debugging notes + AI analysis

# Learning: New framework exploration
manx doc svelte "component lifecycle"  
# Returns: Official Svelte docs + AI explanation with examples
```

### **👥 Development Team**

```bash
# Onboard new developer
manx index ~/team-handbook/
manx index ~/coding-standards/
manx snippet "deployment process"
# Returns: Official CI/CD docs + team procedures + AI walkthrough

# Solve production issue
manx search "kubernetes pod restart loops"
# Returns: K8s docs + team runbooks + AI troubleshooting steps
```

### **🔒 Privacy-Focused Usage**

```bash
# Index sensitive documentation locally
manx index ~/classified-procedures/
manx snippet "security protocols" --no-llm
# Pure local search - no external API calls

# Team knowledge without AI
manx config --llm-provider ""  # Disable AI globally
manx snippet "internal processes"
# Uses only local knowledge + official docs
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

**No AI Synthesis Appearing**
```bash
# Check if LLM provider is configured
manx config --show

# Set up a provider
manx config --openai-api "sk-your-key"

# Test AI functionality
manx snippet python "functions"
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

- **🔌 New AI Providers** - Add support for additional LLM APIs
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
- **OpenAI, Anthropic, Groq** - Amazing LLM capabilities  
- **Rust Community** - Outstanding ecosystem and tooling
- **Contributors** - Making Manx better every day

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

**Built with ❤️ for developers who live in the terminal**

**[⬆️ Back to Top](#-manx---ai-powered-documentation-assistant)**

![Manx Demo](https://via.placeholder.com/600x300/1a1a1a/00d4aa?text=🤖+AI+Synthesis+Demo)

*Beautiful, intelligent documentation assistance - right in your terminal*

</div>