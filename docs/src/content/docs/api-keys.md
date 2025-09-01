---
title: API Keys
description: Setting up Context7 API keys for optimal performance
---

## Why Use an API Key?

While Manx works without an API key, setting one up provides significant benefits:

### Without API Key
- ❌ Limited to ~10 requests per minute
- ❌ Shared rate limits with all users
- ❌ May experience delays during peak times
- ❌ Basic search functionality only

### With API Key
- ✅ Higher rate limits (100+ requests/minute)
- ✅ Dedicated endpoint for consistent performance
- ✅ Priority support
- ✅ Access to premium features (coming soon)

## Getting Your API Key

1. **Visit Context7 Dashboard**  
   Go to [context7.com/dashboard](https://context7.com/dashboard)

2. **Create Account**  
   Sign up for a free account (no credit card required)

3. **Generate API Key**  
   Click "Generate New Key" and copy the key (starts with `sk-`)

## Setting Up Your API Key

### Method 1: Configuration Command (Recommended)

```bash
manx config --api-key sk-your-context7-key-here
```

### Method 2: Environment Variable

```bash
# Add to ~/.bashrc or ~/.zshrc
export MANX_API_KEY="sk-your-context7-key-here"

# Reload shell
source ~/.bashrc
```

### Method 3: Direct Config File Edit

Edit `~/.config/manx/config.json`:

```json
{
  "api_key": "sk-your-context7-key-here",
  "cache_dir": null,
  "default_limit": 10,
  "offline_mode": false,
  "color_output": true,
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 100
}
```

## Verifying Your Setup

### Check Configuration

```bash
manx config --show
```

Should show:
```
API Key: sk-****** (hidden for security)
```

### Test Performance

```bash
# Clear cache for fresh test
manx cache clear

# Run search - should be noticeably faster
time manx fastapi
```

With API key: ~0.4-0.6s  
Without API key: ~0.8-1.2s

## Managing Your API Key

### View Current Key Status

```bash
manx config --show
```

### Update API Key

```bash
manx config --api-key sk-new-key-here
```

### Remove API Key

```bash
# This will revert to shared endpoint
manx config --api-key ""
```

### Rotate API Key

1. Generate new key in Context7 dashboard
2. Update Manx configuration
3. Delete old key from dashboard

## API Key Security

### Best Practices

1. **Keep it secret**: Never share your API key
2. **Don't commit**: Add to `.gitignore` if storing in project files
3. **Use environment variables**: For CI/CD and shared environments
4. **Rotate regularly**: Generate new keys periodically

### Environment-Specific Setup

#### Development

```bash
# Personal development machine
manx config --api-key sk-dev-key-123
```

#### CI/CD

```yaml
# GitHub Actions
env:
  MANX_API_KEY: ${{ secrets.MANX_API_KEY }}
```

#### Team Shared

```bash
# Use team API key via environment
export MANX_API_KEY="sk-team-key-456"
```

## Troubleshooting API Keys

### "Invalid API key" Error

**Possible causes:**
1. Key was deleted from dashboard
2. Key was mistyped
3. Key has expired (rare)

**Solution:**
```bash
# Check current key
manx config --show

# Generate new key and update
manx config --api-key sk-new-valid-key
```

### "Rate limit exceeded" Despite API Key

**Possible causes:**
1. Key not properly configured
2. Very high usage exceeding even premium limits

**Solution:**
```bash
# Verify key is set
manx config --show

# Enable debug to see which endpoint is used
manx --debug fastapi

# Should show: "Using API endpoint with key: sk-******"
```

### Performance Not Improved

**Check:**
1. API key is correctly set
2. Clear cache to test fresh requests
3. Check network conditions

```bash
# Full performance test
manx cache clear
manx --debug fastapi
# Look for "API endpoint" in debug output
```

## API Usage Monitoring

Context7 dashboard provides usage analytics:

- **Request count**: Total API calls made
- **Rate limit status**: Current usage vs limits  
- **Response times**: Performance metrics
- **Error rates**: Failed request tracking

## FAQ

**Q: Is the API key free?**  
A: Yes, Context7 provides free API keys with generous limits.

**Q: Can I use the same key on multiple machines?**  
A: Yes, but monitor usage to avoid hitting limits.

**Q: What happens if I exceed my rate limit?**  
A: Requests will be delayed or fail. Upgrade to higher tier if needed.

**Q: Can I use Manx without an API key?**  
A: Yes, but with limited performance and functionality.

**Q: How do I know if my API key is working?**  
A: Run `manx --debug` and look for "Using API endpoint" message.

**Q: Is my API key stored securely?**  
A: Yes, it's stored in your local config file with proper permissions.

## Migration from Shared to API Key

If you've been using Manx without an API key:

1. **Get your key** from Context7 dashboard
2. **Configure Manx**: `manx config --api-key sk-your-key`
3. **Clear cache**: `manx cache clear` (for fresh performance test)
4. **Verify improvement**: `time manx fastapi`

You should see immediate performance improvements and fewer rate limit errors.

## Next Steps

Once your API key is configured:
- Explore [advanced configuration options](/manx/configuration/)
- Learn about [cache optimization](/manx/performance/)
- Check out [real-world examples](/manx/examples/)