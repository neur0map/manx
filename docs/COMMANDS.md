# 📖 Commands Reference

Complete reference for all manx commands with examples and options.

## 🔍 Search Commands

### `manx snippet <library> [query]`
Find code snippets and examples from official documentation.

```bash
# Basic usage
manx snippet react "useState"
manx snippet python "async functions"
manx snippet fastapi "middleware"

# Advanced patterns
manx snippet react "custom hooks patterns"
manx snippet python "error handling decorators"
manx snippet rust "lifetime annotations"
```

**Options:**
- `--limit <N>` - Limit results (default: 12)
- `--save-all` - Export all results to file
- `--rag` - Search only indexed documents
- `--no-llm` - Disable AI analysis
- `--llm` - Force AI analysis

### `manx search <query>`
Search official documentation across multiple sources.

```bash
# Documentation search
manx search "authentication best practices"
manx search "rust async programming"
manx search "react performance optimization"

# With RAG mode
manx search "team coding standards" --rag
manx search "deployment process" --rag
```

**Options:**
- `--rag` - Search indexed documents only
- `--limit <N>` - Limit results
- `--no-llm` - Disable AI synthesis
- `--offline` - Use only cached results

### `manx doc <library> [topic]`
Browse comprehensive documentation sections.

```bash
# Browse documentation
manx doc react "hooks"
manx doc fastapi "security"
manx doc django "models"

# Explore library overview
manx doc svelte
manx doc pytorch
```

### `manx get <id>`
Retrieve specific results by ID from previous searches.

```bash
# Get specific result
manx get doc-3
manx get snippet-7

# Export to file
manx get doc-3 -o documentation.md
manx get snippet-7 -o example.py
```

## 📁 Document Management

### `manx index <path>`
Index local documents or web URLs for RAG search.

```bash
# Index local files
manx index ~/dev-notes/
manx index ~/team-docs/important-guide.md

# Index web documentation
manx index https://docs.fastapi.tiangolo.com
manx index https://docs.rust-lang.org/book --crawl --max-depth 3
```

**Options:**
- `--id <alias>` - Custom alias for indexed source
- `--crawl` - Deep crawl for URLs (follows links)
- `--max-depth <N>` - Maximum crawl depth (default: 3)
- `--max-pages <N>` - Maximum pages to crawl

**Supported formats:**
- Text: `.md`, `.txt`, `.rst`
- Documents: `.docx`
- Web: Any HTTP/HTTPS URL

### `manx sources`
Manage indexed document sources.

```bash
# List indexed sources
manx sources list

# Add a document source to the index
manx sources add <path> [--id <alias>]

# Clear all indexed documents
manx sources clear
```

## 🧠 Embedding Management

### `manx embedding`
Manage neural embedding models for semantic search.

```bash
# Check current configuration
manx embedding status

# List available models
manx embedding list

# Download a model
manx embedding download all-MiniLM-L6-v2

# Test embedding generation
manx embedding test "your test query"
```

**Available models:**
- `sentence-transformers/all-MiniLM-L6-v2` (87MB, fast)
- `sentence-transformers/all-mpnet-base-v2` (400MB, high quality)
- `sentence-transformers/multi-qa-MiniLM-L6-cos-v1` (87MB, QA focused)
- `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` (400MB, multilingual)
- `BAAI/bge-small-en-v1.5` (134MB, retrieval optimized)
- `BAAI/bge-base-en-v1.5` (438MB, balanced)
- `BAAI/bge-large-en-v1.5` (1.34GB, best quality)

## ⚙️ Configuration

### `manx config`
Configure manx settings and providers.

```bash
# View current settings
manx config --show

# Embedding provider
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# LLM configuration
manx config --openai-api "sk-your-key"
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-5-sonnet"

# Context7 API key
manx config --api-key "sk-your-context7-key"
```

## 🗂️ Cache Management

### `manx cache`
Manage local documentation cache.

```bash
# Clear all cached data
manx cache clear

# View cache statistics
manx cache stats

# Show all currently cached libraries
manx cache list
```

## 🔗 Utility Commands

### `manx open <id>`
Open documentation section by ID in browser.

```bash
manx open doc-5
manx open snippet-12
```

### `manx update`
Update manx to latest version from GitHub.

```bash
manx update
manx update --force
```

## 🎯 Command Examples by Use Case

### **Learning New Framework**
```bash
manx doc svelte                           # Overview
manx snippet svelte "component props"     # Code examples
manx search "svelte vs react comparison"  # Detailed comparison
```

### **Debugging Issues**
```bash
manx search "rust borrow checker errors"
manx snippet python "exception handling"
manx search "memory leaks javascript" --rag  # Check team notes
```

### **Team Collaboration**
```bash
manx index ~/team-handbook/               # One-time setup
manx search "deployment checklist" --rag  # Daily usage
manx snippet "code review process" --rag  # Team procedures
```

### **Research & Analysis**
```bash
manx config --openai-api "sk-key"        # Enable AI
manx search "microservices patterns"      # Comprehensive analysis
manx doc kubernetes "ingress"             # Technical deep dive
```

## 🚀 Global Options

Available for all commands:

- `--debug` - Show detailed debug information
- `--offline` - Work offline using cached results only
- `--api-key <key>` - Override API key for this session
- `--cache-dir <dir>` - Override cache directory
- `--clear-cache` - Clear cache before command
- `--auto-cache-on/off` - Enable/disable automatic caching

## 💡 Tips & Tricks

### **Efficient Querying**
```bash
# Use specific terms for better results
manx snippet react "useState with objects"  # Better than "state"

# Combine with library names for precision
manx search "fastapi dependency injection"  # Better than "dependency injection"
```

### **Progressive Enhancement**
```bash
# Start with defaults
manx snippet python "functions"

# Add semantic search for better results
manx embedding download all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Index your docs for private search
manx index ~/dev-notes/

# Add AI for comprehensive answers
manx config --openai-api "sk-key"
```

### **Result Management**
```bash
# Save interesting results
manx snippet react hooks --save-all

# Reference specific results later
manx get snippet-5 -o my-reference.md

# Build your knowledge base
manx index ./saved-examples/
```