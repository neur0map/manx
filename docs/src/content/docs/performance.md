---
title: Performance
description: Performance benchmarks, optimization tips, and comparisons
---

## Performance Overview

Manx is designed for speed and efficiency. Here's how it compares to traditional documentation searching methods.

## Benchmarks

### Search Speed Comparison

| Method | Cold Search | Warm Search | Cache Hit |
|--------|-------------|-------------|----------|
| **Manx** | 0.8s | 0.1s | 0.05s |
| Google + Browse | 8-15s | 5-10s | N/A |
| Official Docs | 3-8s | 2-5s | N/A |
| Stack Overflow | 5-12s | 3-8s | N/A |

### Resource Usage

| Metric | Manx | VS Code + Browser | Improvement |
|--------|------|------------------|-------------|
| **Binary Size** | 2.9MB | ~500MB | 99.4% smaller |
| **Memory Usage** | <10MB | 100-500MB | 95% less |
| **Startup Time** | <50ms | 2-5s | 98% faster |
| **CPU Usage** | Minimal | High | 90% less |

## Real-World Performance

### Developer Workflow Impact

```bash
# Traditional workflow:
# 1. Alt+Tab to browser (200ms)
# 2. Search "fastapi middleware" (8s)
# 3. Click result, read docs (15s)
# 4. Alt+Tab back to terminal (200ms)
# Total: ~23s

# Manx workflow:
manx fastapi middleware
# Total: 0.8s (29x faster!)
```

### Cache Performance

#### First Search (Cold)
```
$ time manx fastapi
Real: 0.823s  User: 0.089s  Sys: 0.045s
```

#### Subsequent Searches (Warm)
```
$ time manx --offline fastapi
Real: 0.096s  User: 0.067s  Sys: 0.018s
```

#### Cache Hit Rates
- Typical usage: >90% cache hit rate
- Development workflow: >95% cache hit rate
- CI/CD environments: 100% (with pre-warming)

## Performance Optimization

### 1. API Key Configuration

**Without API Key:**
- Uses shared MCP endpoint
- Rate limited to ~10 requests/minute
- May experience delays during peak times

**With API Key:**
- Dedicated endpoint
- Higher rate limits
- Consistent performance

```bash
# Get 3-5x better performance
manx config --api-key sk-your-key-here
```

### 2. Cache Optimization

#### Optimal Cache Settings

```json
{
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 100
}
```

#### Pre-warm Cache

```bash
# Pre-cache commonly used libraries
for lib in react vue angular fastapi django flask; do
  manx $lib --limit 20
  echo "Cached $lib"
done
```

### 3. Network Optimization

#### For Slow Connections

```bash
# Increase cache TTL
manx config --cache-ttl 168  # 1 week

# Use offline mode when possible
manx --offline react hooks
```

#### For Unreliable Networks

```bash
# Enable auto-retry (built-in)
export MANX_RETRY_COUNT=3
```

## Performance Monitoring

### Built-in Metrics

```bash
# View cache statistics
manx cache stats
```

Output:
```
Cache Statistics:
  Hit Rate: 92.3%
  Total Requests: 1,247
  Cache Hits: 1,151
  Cache Misses: 96
  Average Response: 0.12s
  Cache Size: 67.2 MB
  Entries: 43 libraries
```

### Debug Performance

```bash
# Enable debug mode for timing info
manx --debug fastapi
```

Output:
```
[DEBUG] Cache lookup: 2ms
[DEBUG] Network request: 456ms
[DEBUG] JSON parsing: 12ms
[DEBUG] Rendering: 8ms
[DEBUG] Total: 478ms
```

## Platform-Specific Performance

### macOS
- **M1/M2 Macs**: Exceptional performance (ARM64 optimization)
- **Intel Macs**: Excellent performance
- **Average cold search**: 0.6-0.8s
- **Average warm search**: 0.08-0.12s

### Linux
- **Performance**: Similar to macOS
- **Distribution impact**: Minimal
- **Container performance**: 95% of native

### Windows
- **Performance**: Good (10-15% slower than Unix)
- **WSL2**: Near-native Linux performance
- **PowerShell vs CMD**: No significant difference

## Memory Usage Patterns

### Memory Footprint

```bash
# Check memory usage
ps aux | grep manx
# Typically shows 6-8MB RSS
```

### Memory Over Time

| Operation | Memory Usage |
|-----------|-------------|
| Startup | 3-4MB |
| First search | 6-8MB |
| Cached search | 5-7MB |
| Large result set | 8-12MB |
| After cleanup | 4-6MB |

## Concurrent Usage

### Multiple Sessions

Manx handles concurrent usage efficiently:

```bash
# Terminal 1
manx react hooks &

# Terminal 2
manx fastapi middleware &

# Terminal 3
manx django models &

# All complete in ~1s total
```

### File Locking

- Cache uses file locking for safety
- Multiple processes can read simultaneously
- Writes are properly serialized
- No corruption under concurrent access

## Performance Tips

### 1. Use Specific Queries

```bash
# Slower (broad search)
manx react

# Faster (specific search)
manx react hooks useState
```

### 2. Leverage Cache

```bash
# First time: 0.8s
manx vue composition api

# Subsequent times: 0.1s
manx vue composition api
```

### 3. Batch Operations

```bash
# Instead of multiple single searches
for lib in react vue angular; do
  manx $lib state management
done

# Pre-cache everything first
for lib in react vue angular; do
  manx $lib --limit 1 > /dev/null
done
# Then search quickly
for lib in react vue angular; do
  manx --offline $lib state management
done
```

### 4. Optimize for Workflow

```bash
# Create shell functions for common searches
fr() { manx react "$@"; }  # fr hooks useState
ff() { manx fastapi "$@"; }  # ff middleware
fd() { manx django "$@"; }  # fd models
```

## Performance Troubleshooting

### Slow Searches

1. **Check API key**: `manx config --show`
2. **Clear cache**: `manx cache clear`
3. **Test network**: `ping mcp.context7.com`
4. **Check debug**: `manx --debug library`

### High Memory Usage

1. **Check cache size**: `manx cache stats`
2. **Reduce cache size**: Adjust `max_cache_size_mb`
3. **Clear old cache**: `manx cache clear`

### Cache Issues

```bash
# Clear and rebuild
manx cache clear

# Check permissions
ls -la ~/.cache/manx

# Verify disk space
df -h ~/.cache
```

## Comparison with Alternatives

### vs. Web Browsers

| Aspect | Manx | Browser |
|--------|------|--------|
| Speed | 0.8s | 5-15s |
| Context switching | None | Alt+Tab |
| Resource usage | 8MB | 200MB+ |
| Offline capability | Full | Limited |
| Scriptability | Full | None |

### vs. IDE Extensions

| Aspect | Manx | IDE Extension |
|--------|------|-------------|
| Language agnostic | ✅ | Usually specific |
| Terminal usage | ✅ | Limited |
| Performance | Excellent | Varies |
| Maintenance | Self-updating | Requires updates |

### vs. Local Documentation

| Aspect | Manx | Local Docs |
|--------|------|----------|
| Always updated | ✅ | Manual updates |
| Storage space | 100MB cache | GBs |
| Search quality | Excellent | Varies |
| Multiple versions | ✅ | Complex setup |

Manx provides the best balance of speed, accuracy, and resource efficiency for terminal-based documentation access.