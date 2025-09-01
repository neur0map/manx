---
title: Configuration
description: Configure Manx for optimal performance and customize its behavior
---

## Configuration File

Manx stores its configuration in `~/.config/manx/config.json`.

### Default Configuration

```json
{
  "api_key": null,
  "cache_dir": null,
  "default_limit": 10,
  "offline_mode": false,
  "color_output": true,
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 100
}
```

## Configuration Options

### API Key (`api_key`)

Your Context7 API key for enhanced rate limits.

```bash
# Set via command
manx config --api-key sk-your-key-here

# Set via environment
export MANX_API_KEY=sk-your-key-here
```

**Getting an API key:**
1. Visit [Context7 Dashboard](https://context7.com/dashboard)
2. Create a free account
3. Generate your API key

### Cache Directory (`cache_dir`)

Custom location for cached documentation.

```bash
# Set via command
manx config --cache-dir ~/my-cache

# Set via environment
export MANX_CACHE_DIR=~/my-cache
```

Default: `~/.cache/manx`

### Default Result Limit (`default_limit`)

Number of results to show by default.

```json
{
  "default_limit": 10
}
```

Can be overridden with `--limit` flag.

### Offline Mode (`offline_mode`)

Always use cached results without checking online.

```json
{
  "offline_mode": false
}
```

### Color Output (`color_output`)

Enable/disable colored terminal output.

```json
{
  "color_output": true
}
```

Or use environment variable:
```bash
export NO_COLOR=1  # Disable colors
```

### Auto Cache (`auto_cache_enabled`)

Automatically cache search results.

```bash
manx config --auto-cache on
manx config --auto-cache off
```

### Cache TTL (`cache_ttl_hours`)

How long to keep cached results (in hours).

```json
{
  "cache_ttl_hours": 24
}
```

### Max Cache Size (`max_cache_size_mb`)

Maximum cache size in megabytes.

```json
{
  "max_cache_size_mb": 100
}
```

## Configuration Commands

### View Current Configuration

```bash
manx config --show
```

Output:
```
Current Configuration:
  API Key: sk-****** (hidden)
  Cache Directory: /home/user/.cache/manx
  Default Limit: 10
  Offline Mode: false
  Color Output: true
  Auto Cache: enabled
  Cache TTL: 24 hours
  Max Cache Size: 100 MB
```

### Set Configuration Values

```bash
# Set API key
manx config --api-key sk-your-key

# Set cache directory
manx config --cache-dir /path/to/cache

# Enable/disable auto-cache
manx config --auto-cache on
manx config --auto-cache off
```

### Reset Configuration

To reset to defaults, delete the config file:

```bash
rm ~/.config/manx/config.json
```

## Environment Variables

Environment variables override config file settings.

| Variable | Description | Example |
|----------|-------------|---------||
| `MANX_API_KEY` | Context7 API key | `sk-abc123` |
| `MANX_CACHE_DIR` | Cache directory | `~/my-cache` |
| `MANX_DEBUG` | Enable debug mode | `1` or `true` |
| `NO_COLOR` | Disable colors | `1` or `true` |

### Priority Order

1. Command-line flags (highest priority)
2. Environment variables
3. Config file
4. Default values (lowest priority)

## API Key Configuration

### Without API Key

- Uses shared MCP endpoint
- Limited to ~10 requests per minute
- May experience rate limiting

### With API Key

- Dedicated API endpoint
- Higher rate limits
- Better performance
- Premium features access

### Setting Up API Key

1. **Get your key:**
   ```bash
   # Visit https://context7.com/dashboard
   # Create account and generate key
   ```

2. **Configure Manx:**
   ```bash
   manx config --api-key sk-your-key-here
   ```

3. **Verify setup:**
   ```bash
   manx config --show
   # Should show: API Key: sk-****** (hidden)
   ```

## Cache Configuration

### Cache Location

Default locations by platform:
- Linux/macOS: `~/.cache/manx/`
- Windows: `%LOCALAPPDATA%\manx\cache\`

### Cache Structure

```
~/.cache/manx/
├── libraries/
│   ├── fastapi/
│   │   ├── metadata.json
│   │   └── results/
│   ├── react/
│   └── django/
└── stats.json
```

### Cache Management

```bash
# View cache statistics
manx cache stats

# List cached libraries
manx cache list

# Clear all cache
manx cache clear

# Clear specific library
manx cache clear fastapi
```

### Cache Performance

- First search: ~800ms (network)
- Cached search: ~100ms (local)
- Cache hit rate: >90% typical

## Performance Tuning

### For Slow Connections

```json
{
  "cache_ttl_hours": 168,
  "max_cache_size_mb": 500,
  "auto_cache_enabled": true
}
```

### For Limited Disk Space

```json
{
  "cache_ttl_hours": 6,
  "max_cache_size_mb": 50,
  "auto_cache_enabled": false
}
```

### For Offline Work

```bash
# Pre-cache common libraries
for lib in react vue angular fastapi django flask; do
  manx $lib --limit 20
done

# Work offline
manx config --offline-mode on
```

## Troubleshooting Configuration

### Config file not found

```bash
# Create config directory
mkdir -p ~/.config/manx

# Run any manx command to generate default config
manx --version
```

### Permission issues

```bash
# Fix permissions
chmod 755 ~/.config/manx
chmod 644 ~/.config/manx/config.json
```

### Reset everything

```bash
# Remove all configuration and cache
rm -rf ~/.config/manx
rm -rf ~/.cache/manx

# Start fresh
manx config --show
```

## Best Practices

1. **Always set an API key** for better performance
2. **Enable auto-cache** for frequently used libraries
3. **Adjust cache TTL** based on how often docs change
4. **Monitor cache size** with `manx cache stats`
5. **Use offline mode** when traveling or on slow connections

## Configuration Examples

### Developer Setup

```json
{
  "api_key": "sk-dev-key-123",
  "default_limit": 20,
  "auto_cache_enabled": true,
  "cache_ttl_hours": 48,
  "max_cache_size_mb": 200
}
```

### CI/CD Setup

```json
{
  "offline_mode": true,
  "color_output": false,
  "default_limit": 5
}
```

### Minimal Setup

```json
{
  "api_key": "sk-key-here",
  "auto_cache_enabled": false,
  "max_cache_size_mb": 10
}
```