# 🚀 Manx - Lightning-Fast Documentation Finder

> *Find code snippets, documentation, and answers instantly from your terminal*

<div align="center">

![GitHub Release](https://img.shields.io/github/v/release/neur0map/manx)
![Crates.io Version](https://img.shields.io/crates/v/manx-cli)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)

**📚 [Setup Guide](docs/SETUP_GUIDE.md) • 🔍 [Commands](docs/COMMANDS.md) • ⚙️ [Configuration](docs/CONFIGURATION.md) • 🧠 [AI Features](docs/AI_FEATURES.md)**

</div>

## 🎥 See Manx in Action

<div align="center">

[![Manx Demo Video](https://img.youtube.com/vi/3gINTsmHnYA/0.jpg)](https://www.youtube.com/watch?v=3gINTsmHnYA)

*Click to watch: Complete walkthrough of Manx features and capabilities*

</div>

## ⚡ Quick Start

```bash
# Install and run setup wizard
cargo install manx-cli
manx init  # Interactive setup wizard

# Or install and start immediately
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash

# Find code snippets instantly
manx snippet react "useState hook"
manx snippet python "async functions"

# Search documentation and crawl sites
manx search "rust error handling"
manx doc fastapi "middleware"
manx index https://docs.rs/ --crawl
```

**Works immediately!** No setup required, enhanced with the `manx init` wizard.

## ✨ What is Manx?

Manx helps developers **find answers fast** with four powerful modes:

| Mode | Setup | Description |
|------|-------|-------------|
| **🚀 Default** | None | Official docs + keyword search (works instantly) |
| **🧠 Enhanced** | 1 command | Neural search + semantic understanding |
| **📁 RAG** | Index docs | Search your private documentation |
| **🤖 AI** | Add API key | Full synthesis with explanations + citations |

### Progressive Enhancement
Start simple → Add semantic search → Index your docs → Enable AI

## 🎯 Core Features

### **Code Snippet Search**
```bash
manx snippet react "custom hooks"
manx snippet python "decorators"
manx snippet rust "error handling"
```
*Get working code examples with explanations from official documentation*

### **Documentation Search**
```bash
manx search "authentication best practices"
manx doc fastapi "dependency injection"
```
*Search official docs across hundreds of frameworks and languages*

### **Personal Knowledge Base**
```bash
# Index local documentation or crawl websites
manx index ~/dev-notes/
manx index https://docs.python.org --crawl-depth 2
manx index https://react.dev --crawl-all

# Search with semantic understanding
manx search "team coding standards" --rag
```
*Index local files or crawl documentation sites for private search*

### **AI-Powered Analysis** *(Optional)*
```bash
manx init  # Setup wizard includes AI configuration
manx snippet react hooks  # Now includes AI explanations
```
*Get comprehensive answers with code examples, explanations, and citations*

## 🚀 Why Manx?

- **⚡ Instant**: Works immediately after installation
- **🎯 Accurate**: Searches official documentation, not forums
- **🧠 Smart**: Optional semantic search understands intent
- **🔒 Private**: Your documents never leave your machine
- **⚙️ Flexible**: Choose your level of enhancement
- **🚀 Fast**: Optimized Rust performance with embedded ONNX Runtime

## 📚 Learn More

- **🔧 [Setup Guide](docs/SETUP_GUIDE.md)** - Complete installation and configuration
- **📖 [Commands Reference](docs/COMMANDS.md)** - All commands with examples
- **⚙️ [Configuration](docs/CONFIGURATION.md)** - Customize settings and providers
- **🧠 [AI Features](docs/AI_FEATURES.md)** - LLM integration and capabilities
- **📁 [RAG Mode](docs/RAG_GUIDE.md)** - Index and search personal documentation
- **🔍 [Neural Search](docs/NEURAL_SEARCH.md)** - Enhanced semantic understanding

## 📦 Installation

### Quick Install Script
```bash
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```
*Automatically detects your platform and installs the latest release*

### Cargo (Alternative)
```bash
cargo install manx-cli
```

### Manual Download
- **Binary**: Download from [GitHub Releases](https://github.com/neur0map/manx/releases)

## 🆘 Getting Help

- **📖 Documentation**: Check the guides linked above
- **🐛 Issues**: [GitHub Issues](https://github.com/neur0map/manx/issues)

## 🙏 Shoutouts

Huge thanks to the amazing open source community and projects that make Manx possible:
- **[Anthropic](https://anthropic.com)** - For Claude and the incredible Claude Code IDE
- **[Context7](https://context7.com/)** - For providing the documentation API that powers default search
- **[Hugging Face](https://huggingface.co)** - For the neural embedding models and infrastructure
- **[ONNX Runtime](https://onnxruntime.ai)** - For fast, local neural inference
- **[Rust Community](https://rust-lang.org)** - For the amazing ecosystem and libraries

## 💡 Built with AI

This tool was fully built through "vibe coding" with **[Claude Code](https://claude.ai/code)** 🤖

I'm not a programmer - just a cybersecurity student learning the basics and building tools for my own use under [prowl.sh](https://prowl.sh). If people find these tools useful, I'm more than happy to continue working on them and improving the experience!

## 📋 Roadmap

### Release Binaries
The install script currently falls back to cargo compilation when pre-built binaries aren't available. Future releases should include binaries for:

- ✅ `x86_64-apple-darwin` (Intel Mac)
- ✅ `aarch64-apple-darwin` (Apple Silicon Mac)
- ✅ `x86_64-pc-windows-msvc` (Windows x64)
- ⏳ `aarch64-unknown-linux-gnu` (ARM64 Linux - Raspberry Pi, ARM servers)

### GitHub Repository Search
Future enhancement to search directly within GitHub repositories for code examples and implementation patterns:

- ⏳ **GitHub Access**: Search repositories, issues, and discussions from the CLI
- ⏳ **Code Search**: Look through repo code with the option to add extra context using embeddings
- ⏳ **Issue Tracking**: Pull in issues and make them easier to reference alongside other results
- ⏳ **Docs Indexing**: Treat READMEs and repo docs as part of the searchable database
- ⏳ **Extra Context**: When needed, let an LLM help summarize or clarify what the search finds

**Integration with Manx Intelligence:**
- 🧠 **Embedding-Enhanced Code Search**: Neural embeddings understand code similarity and patterns
- 🤖 **LLM Code Analysis**: Synthesize solutions from multiple repos with explanations
- 🎯 **Smart Query Routing**: Framework detection automatically searches relevant repositories  
- 📊 **Hybrid Results**: Combine web docs + GitHub code + issue discussions in unified answers
- 🔍 **Semantic Issue Search**: Find related problems even with different terminology

Example: `manx search "Tauri tables"` would search official docs AND `tauri-apps/tauri` repo for real implementations.

## 📄 License

GPL-3 © [neur0map](https://github.com/neur0map)

---

*Happy coding! 🚀*
