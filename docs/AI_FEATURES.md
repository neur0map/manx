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

### AI Control
```bash
# Note: Global AI settings like --ai-default, --ai-mode, --ai-verbosity
# are not currently implemented. Use per-command flags instead:

# Force disable AI for a command
manx snippet python --no-llm

# AI is automatically used when LLM provider is configured
manx snippet python  # Uses AI if configured
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

# Note: Ollama provider not currently implemented
# Use HuggingFace or custom endpoints for local models
```

### Usage Controls
```bash
# Note: AI usage controls like daily limits, token limits, and budget
# tracking are not currently implemented. Monitor costs through your
# LLM provider's dashboard instead.

# To limit AI usage, use --no-llm flag on individual commands:
manx snippet python --no-llm
```

## 🚀 Advanced AI Features

### AI Behavior
```bash
# Note: Custom prompts, AI styles, and multi-turn conversations
# are not currently implemented. AI responses use default prompts
# optimized for technical documentation.

# AI is stateless - each command is independent
manx search "react state management"
manx search "performance implications"  # Separate query
```

### AI + Neural Search Synergy
```bash
# Best quality: Neural embeddings + AI analysis
manx embedding download all-mpnet-base-v2
manx config --embedding-provider onnx:all-mpnet-base-v2
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