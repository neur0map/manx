# 🤖 AI Features Guide

Complete guide to manx's AI-powered features for enhanced documentation search and analysis.

## ✨ What AI Adds to Manx

AI integration transforms manx from a search tool into an **intelligent documentation assistant** that:

- **🧠 Synthesizes information** from multiple sources
- **📝 Provides explanations** with working code examples  
- **🔗 Includes citations** to original documentation
- **🎯 Understands context** and intent behind queries
- **💡 Suggests related topics** and best practices

## 🚀 Quick Start

```bash
# 1. Configure LLM provider
manx config --openai-api "sk-your-openai-key"

# 2. Test AI features
manx snippet react hooks  # Now includes AI explanations
manx search "authentication patterns"  # Comprehensive analysis
```

## 🔧 Supported LLM Providers

### OpenAI
```bash
manx config --llm-provider "openai"
manx config --openai-api "sk-your-openai-key"
manx config --llm-model "gpt-4o"              # Default, most capable
manx config --llm-model "gpt-4o-mini"         # Faster, cheaper
```

### Anthropic (Claude)
```bash
manx config --llm-provider "anthropic"
manx config --anthropic-api "your-anthropic-key"
manx config --llm-model "claude-3-5-sonnet-20241022"  # Default, excellent reasoning
manx config --llm-model "claude-3-haiku-20240307"     # Fast, cost-effective
```

### Groq (Ultra-Fast Inference)
```bash
manx config --llm-provider "groq"
manx config --groq-api "gsk_your-groq-key"
manx config --llm-model "llama-3.1-8b-instant"       # Lightning fast
manx config --llm-model "llama-3.1-70b-versatile"    # More capable
```

### Google Gemini
```bash
manx config --llm-provider "google"
manx config --google-api "your-google-key"
manx config --llm-model "gemini-1.5-flash"           # Fast and capable
manx config --llm-model "gemini-1.5-pro"             # Maximum capability
```

### Azure OpenAI
```bash
manx config --llm-provider "azure"
manx config --azure-api "your-azure-key"
manx config --azure-endpoint "https://your-resource.openai.azure.com/"
manx config --llm-model "gpt-4o"
```

### Ollama (Local/Private)
```bash
manx config --llm-provider "ollama"
manx config --ollama-endpoint "http://localhost:11434"
manx config --llm-model "llama3.1:8b"                # Privacy-focused
```

## 🎯 AI-Enhanced Commands

### Snippet Search with AI
```bash
# Without AI: Just code examples
manx snippet react hooks --no-llm

# With AI: Code + explanations + best practices
manx snippet react hooks
```

**AI Enhancement Includes:**
- **Code explanations** line-by-line
- **Best practices** and common pitfalls
- **Related patterns** and alternatives  
- **Performance considerations**
- **Testing strategies**

### Intelligent Documentation Search
```bash
# Complex technical queries
manx search "microservices communication patterns"
manx search "react performance optimization strategies" 
manx search "rust memory safety guarantees"
```

**AI Analysis Provides:**
- **Comprehensive overview** of the topic
- **Pros/cons comparison** of different approaches
- **Real-world examples** and use cases
- **Implementation guidance** step-by-step
- **Related technologies** and alternatives

### RAG + AI = Powerful Insights
```bash
# Index your team documentation
manx index ~/team-docs/
manx index ~/project-architecture/

# Get AI insights from your private docs
manx search "deployment architecture decisions" --rag
manx snippet "authentication implementation" --rag
```

**Combined Power:**
- **Your knowledge** + **Official docs** + **AI synthesis**
- **Context-aware** responses using your team's decisions
- **Consistent** with your coding standards and practices

## 🎛️ AI Control Options

### Per-Command Control
```bash
# Force AI analysis
manx snippet python --llm

# Disable AI for this query  
manx snippet python --no-llm

# Default behavior (respects global config)
manx snippet python
```

### Global AI Settings
```bash
# Always use AI when available
manx config --ai-default true

# Never use AI unless explicitly requested
manx config --ai-default false

# Use AI only for complex queries (smart mode)
manx config --ai-mode "smart"
```

### AI Response Formatting
```bash
# Detailed explanations
manx config --ai-verbosity "detailed"

# Concise responses
manx config --ai-verbosity "concise"

# Code-focused responses
manx config --ai-format "code-first"
```

## 💡 AI Use Cases

### Learning New Technologies
```bash
# Get comprehensive introduction
manx search "getting started with kubernetes"

# Understand complex concepts
manx search "rust ownership and borrowing explained"

# Compare technologies
manx search "react vs vue vs svelte comparison"
```

### Debugging & Problem Solving
```bash
# Debug specific issues
manx search "react useEffect infinite loop solutions"

# Understand error messages
manx search "typescript type 'string' is not assignable to type"

# Performance troubleshooting
manx search "python asyncio performance bottlenecks"
```

### Architecture & Design Decisions
```bash
# Design patterns
manx search "microservices vs monolith trade-offs"

# Best practices
manx search "database design patterns for scalability"

# Technology selection
manx search "choosing between postgresql and mongodb"
```

### Team Knowledge Management
```bash
# Index team documentation
manx index ~/team-handbook/
manx index ~/architecture-decisions/

# Get AI insights on team practices
manx search "our deployment process best practices" --rag
manx search "coding standards for new developers" --rag
```

## 🔒 Privacy & Security

### Data Handling
- **Search queries** are sent to configured LLM providers for analysis
- **Documentation content** is sent to provide context for responses
- **Your indexed documents** are only included if using `--rag` flag
- **API keys** are stored securely in local config files

### Privacy Options
```bash
# Use local models only (no external APIs)
manx config --llm-provider "ollama"

# Disable AI completely
manx config --llm-provider ""

# Use AI only with your documents (no external docs)
manx search "query" --rag --no-external
```

### Enterprise Security
- **SOC2 compliance** varies by provider (check individual provider policies)
- **Data retention policies** differ by LLM provider
- **Zero-data retention** available with some providers
- **Local deployment** possible with Ollama

## 💰 Cost Management

### Cost-Effective Providers
```bash
# Cheapest options
manx config --llm-model "gpt-4o-mini"        # OpenAI
manx config --llm-model "claude-3-haiku"     # Anthropic
manx config --llm-model "llama-3.1-8b"       # Groq (fast + cheap)

# Free local option
manx config --llm-provider "ollama"          # No API costs
```

### Usage Controls
```bash
# Limit AI usage
manx config --ai-max-calls-per-day 50
manx config --ai-token-limit 4000

# Smart usage (AI only for complex queries)
manx config --ai-mode "smart"

# Budget tracking
manx config --ai-budget-monthly 25.00  # USD
```

### Cost Monitoring
```bash
# Check usage statistics
manx config --ai-usage-stats

# Estimate costs
manx config --ai-cost-estimate
```

## 🚀 Advanced AI Features

### Custom Prompts
```bash
# Technical focus
manx config --ai-style "technical"

# Beginner-friendly explanations  
manx config --ai-style "beginner"

# Code-heavy responses
manx config --ai-style "code-focused"

# Architecture-focused
manx config --ai-style "architecture"
```

### Multi-Turn Conversations
```bash
# Follow-up questions maintain context
manx search "react state management"
manx search "what about performance implications"  # Continues context
manx search "show me a complex example"           # Still in context
```

### AI + Neural Search Synergy
```bash
# Best quality: Neural embeddings + AI analysis
manx embedding download sentence-transformers/all-mpnet-base-v2
manx config --embedding-provider onnx:sentence-transformers/all-mpnet-base-v2
manx config --openai-api "sk-your-key"

# Now queries get:
# 1. Semantic understanding from neural embeddings
# 2. Intelligent synthesis from AI
# 3. Comprehensive explanations with citations
```

## 🛠️ AI Configuration Examples

### Development Team Setup
```bash
# High-quality AI for architecture decisions
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-5-sonnet"
manx config --ai-style "technical"
manx config --ai-verbosity "detailed"
```

### Individual Developer
```bash
# Balanced cost and capability
manx config --llm-provider "openai"
manx config --llm-model "gpt-4o-mini"
manx config --ai-mode "smart"
manx config --ai-max-calls-per-day 30
```

### Privacy-Conscious Setup
```bash
# Local-only AI (requires Ollama installation)
manx config --llm-provider "ollama"
manx config --ollama-endpoint "http://localhost:11434"
manx config --llm-model "llama3.1:8b"
```

### Learning/Educational Use
```bash
# Detailed explanations for learning
manx config --llm-provider "anthropic"
manx config --llm-model "claude-3-5-sonnet"
manx config --ai-style "beginner"
manx config --ai-verbosity "detailed"
```

## ❓ Troubleshooting AI Features

### Common Issues
```bash
# Test LLM connection
manx config --test-llm

# Check API key configuration
manx config --show

# Debug AI responses
manx --debug search "test query" --llm
```

### Error Resolution
```bash
# Rate limit errors
manx config --ai-rate-limit 1  # Slow down requests

# Token limit errors  
manx config --ai-token-limit 2000  # Reduce context size

# Cost limit errors
manx config --ai-budget-daily 5.00  # Increase budget
```

## 💡 Best Practices

### Effective AI Queries
```bash
# Be specific
manx search "react useEffect cleanup patterns"  # Good
manx search "react hooks"                       # Too broad

# Include context
manx search "fastapi async database connections with sqlalchemy"  # Good
manx search "database connections"                                # Too generic
```

### Combining Features
```bash
# 1. Use neural embeddings for better search
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:sentence-transformers/all-MiniLM-L6-v2

# 2. Index your team docs
manx index ~/team-docs/

# 3. Enable AI for synthesis
manx config --openai-api "sk-key"

# 4. Get comprehensive, contextual answers
manx search "authentication implementation" --rag
```

### Cost Optimization
- Start with cheaper models (`gpt-4o-mini`, `claude-3-haiku`)
- Use `--ai-mode "smart"` to enable AI only for complex queries
- Set daily/monthly budgets to prevent overuse
- Consider local models (Ollama) for high-volume usage