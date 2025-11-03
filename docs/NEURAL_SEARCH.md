# Neural Search Guide

Guide to manx's neural embedding capabilities for semantic search and enhanced documentation discovery.

## What is Neural Search?

Neural Search replaces simple keyword matching with semantic understanding:

- Understands meaning: "database" matches "data storage", "persistence layer"
- Intent recognition: Knows what you're looking for
- Better ranking: Most relevant results first, not just keyword frequency
- Context awareness: Understands relationships between concepts
- Works locally: All processing happens on your machine

## Quick Start

```bash
# 1. Download a neural model (one-time setup)
manx embedding download all-MiniLM-L6-v2

# 2. Enable neural search
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# 3. Test enhanced search
manx snippet react "state management"  # Much smarter results!
```

## Available Models

### Lightweight & Fast (384D)
```bash
# MiniLM - Best balance of speed and quality
manx embedding download all-MiniLM-L6-v2
# Size: 87MB | Speed: Fastest | Quality: Good | Use: General purpose

# BGE Small - Optimized for technical content
manx embedding download bge-small-en-v1.5
# Size: 128MB | Speed: Fast | Quality: Very Good | Use: Code, docs, retrieval

# Multi-QA - Question-answer focused
manx embedding download multi-qa-MiniLM-L6-cos-v1
# Size: 87MB | Speed: Fast | Quality: Good | Use: FAQ, troubleshooting
```

### High Quality (768D+)
```bash
# MPNet - Superior semantic understanding
manx embedding download all-mpnet-base-v2
# Size: 416MB | Speed: Slower | Quality: Best | Use: Research, precision

# BGE Base - Excellent for technical content
manx embedding download bge-base-en-v1.5
# Size: 440MB | Speed: Slower | Quality: Best | Use: Professional, technical

# BGE Large - Highest quality available
manx embedding download bge-large-en-v1.5
# Size: 1.2GB | Speed: Slowest | Quality: Supreme | Use: Critical accuracy
```

### API-Based Providers (No Download Required)
```bash
# OpenAI embeddings
manx config --embedding-provider openai:text-embedding-3-small
manx config --embedding-api-key "sk-your-openai-key"

# HuggingFace embeddings  
manx config --embedding-provider huggingface:all-MiniLM-L6-v2
manx config --embedding-api-key "hf_your-token"

# Ollama (self-hosted)
manx config --embedding-provider ollama:nomic-embed-text
# Requires: ollama serve + ollama pull nomic-embed-text

# Custom endpoint
manx config --embedding-provider custom:http://localhost:8080/embeddings
# For self-hosted embedding services
```

### Multilingual Models
```bash
# Multilingual MiniLM - Supports 50+ languages
manx embedding download sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2
# Size: 278MB | Speed: Slower | Quality: Good | Use: Non-English content
```

## Model Installation & Management

### Download Models
```bash
# Check available models
manx embedding list

# Download specific model
manx embedding download sentence-transformers/all-MiniLM-L6-v2

# Force re-download (if corrupted)
manx embedding download sentence-transformers/all-MiniLM-L6-v2 --force
```

### Configure Active Model
```bash
# Set active embedding provider
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Verify configuration
manx embedding status
```

### Switch Between Models
```bash
# Switch to high-quality model for research
manx config --embedding-provider onnx:all-mpnet-base-v2

# Switch to fast model for daily use
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Back to hash embeddings (disable neural search)
manx config --embedding-provider hash
```

### Test Model Performance
```bash
# Test embedding generation
manx embedding test "machine learning algorithms"

# Note: Benchmark functionality not currently implemented
# Use embedding test for basic verification
```

## Semantic Search Examples

### Before vs After Neural Search

**Without Neural Embeddings (Keyword Matching):**
```bash
manx search "database connections"
# Finds: Only docs containing exact words "database" AND "connections"
```

**With Neural Embeddings (Semantic Matching):**
```bash
manx search "database connections" 
# Finds: Database connections, DB connectivity, data persistence, 
#        connection pooling, database drivers, data access layers
```

### Real-World Comparisons

**Learning Framework Patterns:**
```bash
# Query: "component lifecycle"
# Keyword search: Only exact matches
# Neural search: Lifecycle hooks, mounting/unmounting, component states,
#                initialization, cleanup, lifecycle methods
```

**Debugging Issues:**
```bash  
# Query: "memory leaks"
# Keyword search: Documents with "memory" AND "leaks" 
# Neural search: Memory management, garbage collection, heap issues,
#                memory profiling, resource cleanup, memory optimization
```

**Architecture Patterns:**
```bash
# Query: "microservices communication"
# Keyword search: Exact phrase matches only
# Neural search: Service mesh, API gateways, event-driven architecture,
#                inter-service communication, distributed systems
```

## How Neural Search Works

### Embedding Generation Process
1. **Text Input**: "React state management"
2. **Tokenization**: Split into subword tokens
3. **Neural Processing**: 384-768 dimensional vector generation  
4. **Normalization**: L2 normalization for cosine similarity
5. **Storage**: Vector stored for semantic similarity search

### Search Process
1. **Query Embedding**: Convert search query to vector
2. **Similarity Calculation**: Cosine similarity with all documents
3. **Ranking**: Sort by semantic relevance score
4. **Results**: Return most semantically similar content

### Technical Details
- **Model Architecture**: Sentence Transformers (BERT/RoBERTa based)
- **Inference Engine**: ONNX Runtime with optimizations
- **Vector Dimensions**: 384 (fast) to 1024 (high quality)
- **Similarity Metric**: Cosine similarity
- **Normalization**: L2 normalization for consistent scoring

## Performance Tuning

### Model Selection by Use Case

**Daily Development (Speed Priority):**
```bash
manx config --embedding-provider onnx:all-MiniLM-L6-v2
# 87MB, 384D, ~50ms inference time
```

**Research & Analysis (Quality Priority):**
```bash
manx config --embedding-provider onnx:all-mpnet-base-v2
# 400MB, 768D, ~150ms inference time
```

**Specialized Retrieval:**
```bash
manx config --embedding-provider onnx:BAAI/bge-small-en-v1.5
# 134MB, 384D, optimized for document retrieval
```

### System Requirements
- **RAM**: 2GB+ (models load into memory)
- **Storage**: 87MB - 1.34GB per model
- **CPU**: Any modern processor (benefits from multiple cores)
- **GPU**: Not required (CPU-optimized inference)

### Performance Optimization
```bash
# Reduce memory usage
manx config --embedding-cache-size 1000

# Increase inference speed
manx config --embedding-threads 4

# Batch processing for large document sets
manx config --embedding-batch-size 16
```

## Model Comparison

### Speed Comparison
| Model | Size | Dimensions | Inference Time | Use Case |
|-------|------|------------|----------------|----------|
| MiniLM-L6-v2 | 87MB | 384 | ~50ms | Daily use |
| MPNet-base-v2 | 400MB | 768 | ~150ms | High quality |
| BGE-small | 134MB | 384 | ~60ms | Retrieval |
| BGE-base | 438MB | 768 | ~180ms | Production |
| BGE-large | 1.34GB | 1024 | ~400ms | Research |

### Quality Comparison
Based on semantic search benchmarks:

1. **MPNet-base-v2**: Highest quality, best for complex queries
2. **BGE-large**: Maximum capability, research-grade
3. **BGE-base**: Excellent balance, production-ready  
4. **BGE-small**: Good quality, very efficient
5. **MiniLM-L6-v2**: Great quality, fastest inference

### Specialization Comparison
- **General Purpose**: MiniLM-L6-v2, MPNet-base-v2
- **Document Retrieval**: BGE series
- **Question Answering**: multi-qa-MiniLM-L6-cos-v1
- **Multilingual**: paraphrase-multilingual-MiniLM-L12-v2

## Advanced Usage

### Combining with RAG
```bash
# 1. Enable neural search
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# 2. Index your documents (uses neural embeddings)
manx index ~/team-docs/

# 3. Semantic search through your docs
manx search "authentication patterns" --rag
```

### Hybrid Search Strategies
```bash
# Semantic search in your docs + keyword search in official docs
manx search "react performance optimization"

# Pure semantic search (RAG only)
manx search "deployment strategies" --rag

# Keyword fallback (disable neural search)
manx search "exact phrase match" --embedding-provider hash
```

### Model Experimentation
```bash
# Compare models on the same query
manx config --embedding-provider onnx:all-MiniLM-L6-v2
manx search "microservices architecture" > miniLM_results.txt

manx config --embedding-provider onnx:BAAI/bge-base-en-v1.5  
manx search "microservices architecture" > bge_results.txt

# Compare results and choose your preferred model
```

## Search Quality Tips

### Effective Query Patterns
```bash
# Good: Specific concepts
manx search "react useEffect cleanup"

# Better: Include context
manx search "react useEffect cleanup memory leaks"

# Best: Natural language intent
manx search "how to prevent memory leaks in react useEffect"
```

### Query Optimization
```bash
# Use domain-specific terms
manx search "kubernetes pod scheduling"  # Good
manx search "container orchestration"     # More general

# Include implementation details
manx search "postgres connection pooling" # Specific
manx search "database optimization"       # Too broad
```

### Troubleshooting Search Quality
```bash
# If results aren't relevant:
# 1. Try different models
manx config --embedding-provider onnx:all-mpnet-base-v2

# 2. Use more specific queries
manx search "specific error message here"

# 3. Check if content is actually indexed
manx sources list
```

## Integration Examples

### Development Workflow
```bash
# Morning: Check new patterns in your domain
manx search "recent react patterns" --rag

# Coding: Find implementation examples
manx snippet python "async database patterns"

# Debugging: Semantic search for similar issues
manx search "authentication token expiration handling"

# Evening: Research new technologies  
manx search "rust vs go for microservices"
```

### Team Knowledge Sharing
```bash
# Setup semantic search for team docs
manx embedding download sentence-transformers/all-MiniLM-L6-v2
manx config --embedding-provider onnx:all-MiniLM-L6-v2
manx index ~/team-handbook/

# Team members can now find information semantically
manx search "onboarding process" --rag  # Finds: new hire docs, setup guides
manx search "incident response" --rag   # Finds: runbooks, escalation procedures
```

### Research & Learning
```bash
# Use high-quality model for learning
manx config --embedding-provider onnx:all-mpnet-base-v2

# Comprehensive topic exploration
manx search "distributed systems consensus algorithms"
manx search "machine learning model deployment strategies" 
manx search "frontend architecture patterns"
```

## Troubleshooting

### Model Download Issues
```bash
# Check internet connection
ping huggingface.co

# Retry download with force
manx embedding download sentence-transformers/all-MiniLM-L6-v2 --force

# Check available disk space
df -h ~/.cache/manx/models/
```

### Performance Issues
```bash
# Use smaller model
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Reduce inference threads
manx config --embedding-threads 2

# Clear embedding cache
manx cache clear --embeddings-only
```

### Quality Issues
```bash
# Try higher-quality model
manx config --embedding-provider onnx:all-mpnet-base-v2

# Use more specific queries
manx search "react functional component state management"

# Check if neural search is actually enabled
manx embedding status
```

### Memory Issues
```bash
# Monitor memory usage
manx embedding status --memory

# Use smaller model
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Reduce cache size
manx config --embedding-cache-size 500
```

## Best Practices

### Model Selection
- **Start with MiniLM-L6-v2**: Great quality, fast, small download
- **Upgrade to MPNet-base-v2**: When you need highest quality
- **Use BGE models**: For specialized document retrieval tasks
- **Try different models**: Find what works best for your use case

### Query Optimization
- **Be specific**: Include relevant technical terms
- **Use natural language**: Neural models understand intent
- **Include context**: Add framework names, error messages, etc.
- **Iterate queries**: Refine based on results

### System Optimization
- **Monitor performance**: Use `manx embedding status` regularly
- **Choose appropriate model**: Balance speed vs quality for your needs
- **Cache management**: Clear caches if experiencing issues
- **Regular updates**: Keep models updated for best performance

## Future Features

Planned neural search enhancements:
- **Custom model support**: Upload your own fine-tuned models
- **Model auto-selection**: Automatically choose best model for query type
- **Multi-model ensemble**: Combine multiple models for better results
- **Performance analytics**: Detailed search quality metrics
- **Model fine-tuning**: Adapt models to your specific domain