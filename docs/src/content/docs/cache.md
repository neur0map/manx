---
title: Cache Management
description: Understanding and optimizing Manx's caching system
---

## How Caching Works

Manx uses intelligent caching to provide lightning-fast repeat searches:

1. **First search**: Fetches from Context7 MCP (~800ms)
2. **Cached search**: Loads from disk (~100ms)
3. **Auto-refresh**: Updates cache when TTL expires

## Cache Location

### Default Locations

- **Linux/macOS**: `~/.cache/manx/`
- **Windows**: `%LOCALAPPDATA%\manx\cache\`

### Custom Location

```bash
# Set custom cache directory
manx config --cache-dir ~/my-docs-cache

# Or use environment variable
export MANX_CACHE_DIR=~/my-cache
```

## Cache Structure

```
~/.cache/manx/
├── libraries/
│   ├── fastapi/
│   │   ├── metadata.json     # Library metadata
│   │   ├── search_results/   # Search results
│   │   │   ├── middleware.json
│   │   │   └── authentication.json
│   │   └── docs/            # Full documentation
│   └── react/
│       ├── metadata.json
│       └── search_results/
└── stats.json            # Cache statistics
```

## Cache Commands

### View Cache Statistics

```bash
manx cache stats
```

Output:
```
Cache Statistics:
  Location: /home/user/.cache/manx
  Total Size: 67.2 MB
  Libraries: 43
  Search Results: 312
  Oldest Entry: 2024-01-10 (15 days ago)
  Newest Entry: 2024-01-25 (today)
  Hit Rate: 87.3% (last 100 requests)
  Average Response Time: 0.12s
```

### List Cached Libraries

```bash
manx cache list
```

Output:
```
Cached Libraries:
  fastapi      (15.2 MB, 67 searches, updated 2h ago)
  react        (12.8 MB, 43 searches, updated 1d ago)
  django       (8.9 MB,  31 searches, updated 3d ago)
  vue          (6.1 MB,  24 searches, updated 5d ago)
  angular      (4.3 MB,  18 searches, updated 1w ago)
```

### Clear Cache

```bash
# Clear all cache
manx cache clear

# Clear specific library
manx cache clear fastapi

# Clear old entries (older than 1 week)
manx cache clean --older-than 7d
```

## Cache Configuration

### Auto-Caching

```bash
# Enable automatic caching (default)
manx config --auto-cache on

# Disable automatic caching
manx config --auto-cache off
```

### Cache TTL (Time To Live)

```bash
# Set cache TTL to 48 hours
echo '{
  "cache_ttl_hours": 48
}' > ~/.config/manx/config.json
```

### Maximum Cache Size

```bash
# Limit cache to 50MB
echo '{
  "max_cache_size_mb": 50
}' > ~/.config/manx/config.json
```

## Cache Strategies

### For Fast Development

```json
{
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 200
}
```

**Benefits:**
- Quick repeat searches
- Large cache for many libraries
- Daily updates for fresh content

### For Limited Storage

```json
{
  "auto_cache_enabled": true,
  "cache_ttl_hours": 6,
  "max_cache_size_mb": 25
}
```

**Benefits:**
- Minimal disk usage
- Frequent updates
- Still provides speed benefits

### For Offline Work

```json
{
  "auto_cache_enabled": true,
  "cache_ttl_hours": 168,
  "max_cache_size_mb": 500
}
```

**Benefits:**
- Week-long cache retention
- Large cache for many libraries
- Extended offline capability

## Pre-warming Cache

Speed up your workflow by pre-caching commonly used libraries:

```bash
# Cache common web frameworks
for lib in react vue angular svelte; do
  echo "Caching $lib..."
  manx $lib --limit 20 > /dev/null
done

# Cache Python libraries
for lib in fastapi django flask pandas numpy; do
  echo "Caching $lib..."
  manx $lib --limit 20 > /dev/null
done
```

### Team Cache Warming

```bash
#!/bin/bash
# team-cache-warmup.sh

# Define your team's commonly used libraries
LIBRARIES=(
  "fastapi" "pydantic" "sqlalchemy"
  "react" "typescript" "tailwindcss"
  "docker" "kubernetes" "terraform"
)

echo "Warming cache for team libraries..."
for lib in "${LIBRARIES[@]}"; do
  echo "  Caching $lib..."
  manx $lib getting-started --limit 10 > /dev/null
  manx $lib best-practices --limit 10 > /dev/null
done

echo "Cache warming complete!"
manx cache stats
```

## Cache Performance

### Performance Comparison

| Scenario | No Cache | With Cache | Improvement |
|----------|----------|------------|-----------|
| First search | 800ms | 800ms | - |
| Repeat search | 800ms | 100ms | **8x faster** |
| Offline search | ❌ Fails | 100ms | **Works!** |
| Network down | ❌ Fails | 100ms | **Works!** |

### Cache Hit Rates

- **Typical development**: 85-95% hit rate
- **Learning new library**: 60-80% hit rate  
- **Production support**: 95%+ hit rate

## Working Offline

### Offline Mode

```bash
# Use only cached results
manx --offline fastapi middleware

# Enable offline mode globally
manx config --offline-mode on
```

### Check Offline Capability

```bash
# See what's available offline
manx cache list

# Test offline search
manx --offline react hooks
```

### Preparing for Offline Work

```bash
# Before going offline, cache what you need
manx fastapi authentication
manx fastapi middleware  
manx fastapi testing
manx react hooks
manx react context

# Then work offline
manx config --offline-mode on
```

## Cache Maintenance

### Regular Maintenance

```bash
# Weekly maintenance script
#!/bin/bash

# Show current cache status
echo "Current cache status:"
manx cache stats

# Clean old entries
echo "Cleaning old entries..."
manx cache clean --older-than 14d

# Show updated status
echo "After cleanup:"
manx cache stats
```

### Automatic Cleanup

Manx automatically manages cache size:

1. **Size limit reached**: Removes oldest entries
2. **TTL expired**: Refreshes on next access
3. **Corrupted entries**: Automatically re-fetched

### Manual Cleanup

```bash
# Remove everything older than 1 week
find ~/.cache/manx -mtime +7 -delete

# Remove specific library
rm -rf ~/.cache/manx/libraries/old-library

# Full cache reset
rm -rf ~/.cache/manx
mkdir -p ~/.cache/manx
```

## Troubleshooting Cache

### Cache Corruption

**Symptoms:**
- Garbled output
- JSON parsing errors
- Inconsistent results

**Solution:**
```bash
# Clear corrupted cache
manx cache clear

# Re-run search
manx fastapi middleware
```

### Permission Issues

**Symptoms:**
- "Permission denied" errors
- Cache not updating

**Solution:**
```bash
# Fix permissions
chmod 755 ~/.cache/manx
chmod -R 644 ~/.cache/manx/*

# Or use different location
manx config --cache-dir ~/my-cache
```

### Disk Space Issues

**Symptoms:**
- "No space left on device"
- Cache not growing

**Solution:**
```bash
# Check disk space
df -h ~/.cache

# Reduce cache size
manx config --max-cache-size 50

# Clean old entries
manx cache clean --older-than 7d
```

### Slow Cache Access

**Possible causes:**
- Very large cache
- Slow disk (HDD vs SSD)
- File system fragmentation

**Solutions:**
```bash
# Move cache to faster storage
manx config --cache-dir /path/to/ssd/cache

# Reduce cache size
manx config --max-cache-size 100

# Clean old entries
manx cache clean
```

## Advanced Cache Usage

### Cache Inspection

```bash
# Look at specific cache entry
cat ~/.cache/manx/libraries/fastapi/search_results/middleware.json | jq .

# Check modification times
ls -la ~/.cache/manx/libraries/fastapi/search_results/

# Find largest cache entries
du -sh ~/.cache/manx/libraries/* | sort -hr | head -10
```

### Cache Synchronization

```bash
# Export cache for sharing
tar -czf manx-cache-backup.tar.gz -C ~/.cache manx

# Import cache on another machine
tar -xzf manx-cache-backup.tar.gz -C ~/.cache
```

### CI/CD Cache

```yaml
# GitHub Actions cache example
- name: Cache Manx Documentation
  uses: actions/cache@v3
  with:
    path: ~/.cache/manx
    key: manx-cache-${{ hashFiles('docs/libraries.txt') }}
    restore-keys: manx-cache-
```

The cache system is designed to be transparent and maintenance-free, but understanding how it works can help you optimize your workflow and troubleshoot issues when they arise.