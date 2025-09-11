# 🧠 Manx Semantic Embeddings Setup Guide

**DEPRECATED**: This manual setup guide is no longer needed. Manx now includes ONNX Runtime and works out of the box.

For modern setup, see [SIMPLE_EMBEDDING_SETUP.md](SIMPLE_EMBEDDING_SETUP.md) instead.

## 📊 Quick Comparison

| Feature | Hash Embeddings (Default) | ONNX Embeddings (Enhanced) |
|---------|---------------------------|------------------------------|
| **Speed** | ⚡ 9,600+ embeddings/sec | 🐌 2,500 embeddings/sec |
| **Quality** | 📊 0.57 semantic score | 🎯 0.87 semantic score |
| **Memory** | 💾 ~5MB | 📂 ~185MB |
| **Setup** | ✅ Works out of box | ⚙️ Requires setup |
| **Dependencies** | ❌ None | 📦 200MB download |
| **Offline** | ✅ Yes | ✅ Yes (after setup) |

## 🚀 Quick Start (Recommended)

**For most users**: The default hash embeddings work great! No setup required.

```bash
# Already works perfectly
manx search "React hooks useState"
manx snippet "database connection"
```

## 🎯 Enhanced Setup (Optional)

Only set this up if you need **superior semantic understanding** and can accept slower performance.

### Modern Setup (Recommended)

**No manual installation needed!** ONNX Runtime is bundled with manx:

```bash
# Simple two-step setup:
# 1. Download a model
manx embedding download all-MiniLM-L6-v2

# 2. Configure manx to use it
manx config --embedding-provider onnx:all-MiniLM-L6-v2

# Test it works
manx embedding test "React hooks useState"
```

### Step 4: Verify Setup

```bash
# Check configuration
manx config

# Should show:
# Embedding Provider: Onnx("sentence-transformers/all-MiniLM-L6-v2")
# Embedding Dimension: 384

# Test semantic search
manx search --rag "database operations"  # Should rank Python guide higher than React guide
```

## 🧪 Testing Semantic Quality

Compare hash vs ONNX embeddings:

```bash
# Test with hash embeddings
manx config --embedding-provider hash
manx search --rag "database operations"
# Note the ranking and scores

# Test with ONNX embeddings  
manx config --embedding-provider onnx:all-MiniLM-L6-v2
manx search --rag "database operations"  
# Note the improved semantic ranking
```

## 🔍 Available Models

### Fast Models (384D)
- `sentence-transformers/all-MiniLM-L6-v2` - Best balance of speed/quality
- `sentence-transformers/multi-qa-MiniLM-L6-cos-v1` - Optimized for Q&A
- `BAAI/bge-small-en-v1.5` - Good multilingual support

### High Quality Models (768D+)  
- `sentence-transformers/all-mpnet-base-v2` - Excellent quality, slower
- `BAAI/bge-base-en-v1.5` - Great for technical content
- `BAAI/bge-large-en-v1.5` - Highest quality, slowest (1024D)

## 🛠 Troubleshooting

### ONNX Runtime Not Found
```bash
# macOS
export DYLD_LIBRARY_PATH="/opt/homebrew/Cellar/onnxruntime/1.22.2_2/lib:$DYLD_LIBRARY_PATH"

# Linux  
export LD_LIBRARY_PATH="/usr/local/lib:$LD_LIBRARY_PATH"

# Find your installation
find /opt/homebrew -name "*onnx*" 2>/dev/null
find /usr -name "*onnx*" 2>/dev/null
```

### Schema Warning Messages
These are non-fatal warnings from ONNX library conflicts:
```
Schema error: Trying to register schema with name...
```
**Solution**: Ignore these warnings. They don't affect functionality.

### Model Download Fails
```bash
# Check network connection
curl -I https://huggingface.co/

# Clear and retry
rm -rf ~/.cache/manx/models/
manx embedding download sentence-transformers/all-MiniLM-L6-v2 --force
```

### Slow Performance
```bash
# Check if ONNX is being used
manx embedding test "test query"

# Should show: "Using provider: ONNX Local Model"
# If showing "Hash-based", ONNX setup failed

# Switch back to hash if needed
manx config --embedding-provider hash
```

## 🎯 Usage Recommendations

### Use Hash Embeddings When:
- **Speed matters** (>100K embeddings/sec needed)
- **Simple keyword matching** is sufficient  
- **Minimal memory** usage required
- **Quick prototyping** or basic search
- **No setup** desired

### Use ONNX Embeddings When:
- **Semantic understanding** is important
- **Search quality** matters more than speed
- You have **200+MB memory** available
- Processing **<10K embeddings/sec** is acceptable
- Building **production semantic search**

### Hybrid Approach:
1. Use **Hash for pre-filtering** large datasets
2. Use **ONNX for final ranking** of top results  
3. Implement **smart caching** for frequently accessed content
4. Allow **user configuration** per use case

## 📈 Performance Benchmarks

Based on testing with 20 text samples:

| Provider | Speed | Quality | Memory | Use Case |
|----------|-------|---------|--------|----------|
| Hash | 9,600 emb/sec | 0.57 | 5MB | Fast keyword search |
| ONNX MiniLM | 2,500 emb/sec | 0.87 | 185MB | Semantic understanding |
| ONNX MPNet | 1,800 emb/sec | 0.91 | 245MB | High quality search |

**Quality Score**: 0.0-1.0 scale measuring semantic similarity accuracy

## 🎉 Success!

You now have enhanced semantic embeddings configured! Your searches will understand context and meaning, not just keywords.

Example improvements:
- "authentication" → finds "login", "credentials", "security"  
- "database operations" → ranks database content higher than unrelated code
- "state management" → understands React hooks, Vuex, Redux connections

Enjoy more intelligent search results! 🚀