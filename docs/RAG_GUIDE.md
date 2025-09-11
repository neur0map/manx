# 📁 RAG Mode Guide

Complete guide to manx's RAG (Retrieval-Augmented Generation) capabilities for indexing and searching your personal documentation.

## ✨ What is RAG Mode?

RAG Mode transforms manx into a **personal knowledge base** that can:

- 🔒 **Index your private documents** (never leaves your machine)
- 🎯 **Semantic search** through your own documentation  
- 🧠 **AI-powered insights** from your team's knowledge
- 📁 **Multi-format support** for various document types
- 🔍 **Hybrid search** combining your docs with official documentation

## 🚀 Quick Start

```bash
# 1. Index your documentation
manx index ~/dev-notes/
manx index ~/team-handbook/

# 2. Search your indexed content
manx search "authentication setup" --rag
manx snippet python "team coding standards" --rag

# 3. View what's indexed
manx sources list
```

## 📁 Indexing Documents

### Local Files & Directories
```bash
# Index entire directories
manx index ~/documentation/
manx index ~/project-notes/
manx index ~/team-handbook/

# Index specific files
manx index ~/important-guide.md
manx index ~/api-documentation.docx
manx index ~/deployment-runbook.txt
```

### Supported File Formats
- **Markdown**: `.md`, `.markdown`
- **Text files**: `.txt`, `.rst`  
- **Documents**: `.docx`
- **Web content**: Any HTTP/HTTPS URL

### Web Documentation
```bash
# Index single web pages
manx index https://docs.fastapi.tiangolo.com/tutorial/first-steps/
manx index https://your-company-wiki.com/deployment-guide

# Deep crawl documentation sites  
manx index https://docs.fastapi.tiangolo.com --crawl
manx index https://docs.rust-lang.org/book --crawl --max-depth 3
```

### Indexing Options
```bash
# Custom alias for indexed source
manx index ~/team-docs/ --id "team-handbook"

# Control crawling behavior
manx index https://docs.site.com --crawl --max-depth 2 --max-pages 100

# Verbose output during indexing
manx index ~/docs/ --verbose
```

## 🔍 Searching Indexed Content

### RAG-Only Search
```bash
# Search only your indexed documents
manx search "deployment process" --rag
manx snippet python "team utilities" --rag
manx doc "api guidelines" --rag
```

### Hybrid Search (Default)
```bash
# Search both indexed docs AND official documentation
manx search "react hooks patterns"
# Returns: Official React docs + your team's React notes
```

### Search Examples
```bash
# Find team-specific implementations
manx search "authentication middleware" --rag

# Locate code examples from your projects
manx snippet "custom logging decorator" --rag

# Get team procedures and runbooks
manx search "incident response checklist" --rag

# Find architectural decisions
manx search "why we chose postgresql over mongodb" --rag
```

## 🗂️ Managing Indexed Sources

### List All Sources
```bash
manx sources list
```
**Example output:**
```
📁 Indexed Sources:
1. team-handbook (~/team-docs/) - 45 chunks, 2.3MB
2. dev-notes (~/dev-notes/) - 23 chunks, 1.1MB  
3. fastapi-docs (https://docs.fastapi.tiangolo.com) - 156 chunks, 8.7MB
```

### Add Document Sources
```bash
# Add a document source to the index
manx sources add ~/team-docs/ --id "team-handbook"
manx sources add ~/important-guide.md
```

### Clear All Sources
```bash
manx sources clear
```

### Update Indexed Content
```bash
# Re-index to pick up changes
manx index ~/team-docs/ --force
manx index https://docs.updated-site.com --force
```

## 🧠 RAG + Neural Search

### Enable Semantic Search
```bash
# Download a neural model for better semantic understanding
manx embedding download all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2
```

### Why Neural + RAG is Powerful
- **Semantic matching**: "database" matches "data storage" in your docs
- **Intent understanding**: Finds relevant docs even with different wording
- **Context awareness**: Understands relationships between concepts
- **Better ranking**: Most relevant results first

### Example Comparison
```bash
# Without neural embeddings (keyword matching)
manx search "auth middleware" --rag
# Finds: Documents containing exact words "auth" and "middleware"

# With neural embeddings (semantic matching)  
manx search "auth middleware" --rag
# Finds: Authentication, authorization, middleware, guards, interceptors, etc.
```

## 🤖 RAG + AI Integration

### Setup AI with RAG
```bash
# 1. Configure AI provider
manx config --openai-api "sk-your-openai-key"

# 2. Now RAG searches include AI analysis
manx search "deployment strategies" --rag
```

### AI Enhancement with Your Docs
When AI is enabled, RAG searches provide:
- **Synthesized insights** from multiple documents
- **Team-specific recommendations** based on your indexed decisions
- **Context-aware explanations** using your terminology
- **Consistent guidance** aligned with your team's practices

### Example AI + RAG Response
```bash
manx search "microservices communication" --rag
```
**Response includes:**
- **Your team's service architecture** (from indexed docs)
- **Chosen communication patterns** (from your decision records)  
- **Implementation examples** (from your codebases)
- **Lessons learned** (from your postmortem docs)
- **AI synthesis** connecting everything with best practices

## 🎯 RAG Use Cases

### Team Knowledge Base
```bash
# Setup
manx index ~/team-handbook/
manx index ~/coding-standards/
manx index ~/architecture-decisions/
manx index ~/postmortems/

# Daily usage
manx search "onboarding new developers" --rag
manx search "incident response procedures" --rag  
manx snippet "deployment checklist" --rag
```

### Project Documentation
```bash
# Index project-specific docs
manx index ~/project/docs/
manx index ~/project/README.md
manx index ~/project/architecture.md

# Find project information quickly
manx search "database schema changes" --rag
manx search "API endpoint documentation" --rag
```

### Learning & Research
```bash
# Index learning materials
manx index ~/learning-notes/
manx index ~/conference-talks/
manx index ~/course-materials/

# Retrieve insights
manx search "advanced react patterns" --rag
manx search "system design principles" --rag
```

### Troubleshooting Knowledge
```bash
# Index troubleshooting docs
manx index ~/debugging-guides/
manx index ~/known-issues/
manx index ~/solution-database/

# Quick problem resolution
manx search "memory leak investigation" --rag
manx search "deployment rollback procedure" --rag
```

## ⚙️ RAG Configuration

### RAG Configuration
```bash
# Enable RAG mode
manx config --rag on

# Disable RAG mode  
manx config --rag off

# Set embedding provider for better semantic search
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Configure embedding dimensions
manx config --embedding-dimension 384
```

### RAG Storage
```bash
# Note: Advanced RAG configuration options like chunk size,
# compression, and custom storage directories are not currently
# implemented. RAG uses default settings optimized for most use cases.
```

## 🔒 Privacy & Security

### Data Handling
- **All documents stay local** - never uploaded to external services
- **Embeddings generated locally** using your configured model
- **Search happens offline** (except for hybrid mode with official docs)
- **AI analysis** only if you explicitly enable LLM integration

### Team Security
- **No data leakage** to external documentation services
- **Full control** over what gets indexed
- **Audit trail** of all indexed sources
- **Easy removal** of sensitive documents

### Compliance
- **GDPR friendly** - all data processing happens locally
- **SOC2 compatible** - no unauthorized data transmission  
- **Enterprise ready** - suitable for confidential documentation

## 🚀 Advanced RAG Features

### Advanced Features
```bash
# Note: Metadata filtering with --filter flags is not currently
# implemented. Use specific search terms instead:
manx search "deployment runbook" --rag
manx search "security updates 2024" --rag
```

### Re-indexing
```bash
# Re-index by running the index command again
manx index ~/team-docs/ --id "team-handbook"

# Note: Auto re-indexing, watch mode, and incremental indexing
# are not currently implemented. Re-run index command manually
# when documents change.
```

### RAG Management
```bash
# View indexed sources
manx sources list

# Note: Advanced analytics like popular content and performance
# metrics are not currently implemented.
```

## ❓ Troubleshooting RAG

### Common Issues

**"No results found"**
```bash
# Check if content is indexed
manx sources list

# Verify search syntax
manx search "exact phrase" --rag

# Try broader search terms
manx search "deploy" --rag  # Instead of "deployment-procedure-v2"
```

**"Indexing failed"**
```bash
# Check file permissions
ls -la ~/docs/

# Verify file format support
file ~/docs/document.pdf  # PDF format not supported

# Try force re-indexing
manx index ~/docs/ --force
```

**"Poor search results"**
```bash
# Use neural embeddings for better semantic search
manx embedding download all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Check chunk size settings
manx config --rag-chunk-size 1024  # Smaller chunks for precise matching
```

### Performance Issues
```bash
# Large document collections
manx config --rag-batch-size 100
manx config --rag-parallel-processing 4

# Memory constraints
manx config --rag-memory-limit "2GB"
manx config --rag-disk-cache enabled
```

## 💡 Best Practices

### Effective Document Organization
```bash
# Organize by purpose
manx index ~/team-docs/procedures/ --id "procedures"
manx index ~/team-docs/architecture/ --id "architecture"  
manx index ~/team-docs/runbooks/ --id "runbooks"

# Use descriptive aliases
manx index ~/project-alpha-docs/ --id "project-alpha"
manx index ~/legacy-system-docs/ --id "legacy-docs"
```

### Optimal Search Strategies
```bash
# Start broad, then narrow
manx search "authentication" --rag
manx search "oauth2 implementation" --rag
manx search "oauth2 middleware bug fix" --rag

# Use domain-specific terms
manx search "kubernetes pod restart" --rag  # Good
manx search "container restart" --rag       # Less specific
```

### Content Curation
- **Keep docs up-to-date**: Regularly re-index changed content
- **Remove outdated content**: Clean up obsolete documentation  
- **Use consistent terminology**: Helps with search accuracy
- **Add metadata**: Use frontmatter in Markdown files for better filtering

### Integration Workflows
```bash
# Morning standup prep
manx search "yesterday's deployment issues" --rag

# Code review insights
manx search "coding standards for this component" --rag

# Architecture discussions  
manx search "previous decisions about microservices" --rag

# Incident response
manx search "similar incidents" --rag
manx search "rollback procedure" --rag
```