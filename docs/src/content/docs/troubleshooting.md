---
title: Troubleshooting
description: Common issues and solutions for Manx
---

## Installation Issues

### "Command not found: manx"

**Cause**: Binary not in PATH or not executable.

**Solution**:
```bash
# Check if manx exists
which manx

# If not found, check common locations
ls -la /usr/local/bin/manx
ls -la ~/.local/bin/manx

# Add to PATH if needed
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Make executable if needed
chmod +x /usr/local/bin/manx
```

### "Permission denied"

**Cause**: Binary lacks execute permissions.

**Solution**:
```bash
# Fix permissions
chmod +x /usr/local/bin/manx

# Or install to user directory
mkdir -p ~/.local/bin
mv manx ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### macOS "Unidentified Developer" Warning

**Cause**: Binary not signed with Apple developer certificate.

**Solution**:
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/manx

# Or allow in System Preferences
# System Preferences → Security & Privacy → Allow
```

## Search Issues

### "No results found"

**Possible causes and solutions**:

1. **Library name typo**:
   ```bash
   # Wrong
   manx fast-api
   
   # Correct
   manx fastapi
   ```

2. **Library not in Context7 database**:
   ```bash
   # Check if library exists
   manx --debug library-name
   
   # Try variations
   manx python-library
   manx python_library
   ```

3. **Cache corruption**:
   ```bash
   # Clear and retry
   manx cache clear
   manx fastapi
   ```

### "Rate limit exceeded"

**Cause**: Too many requests without API key.

**Solution**:
```bash
# Get API key from https://context7.com/dashboard
manx config --api-key sk-your-key-here

# Or wait and retry
sleep 60
manx fastapi
```

### Slow search responses

**Troubleshooting steps**:

1. **Check network**:
   ```bash
   ping mcp.context7.com
   curl -I https://mcp.context7.com/mcp
   ```

2. **Enable debug mode**:
   ```bash
   manx --debug fastapi
   ```

3. **Use cached results**:
   ```bash
   manx --offline fastapi
   ```

4. **Clear cache**:
   ```bash
   manx cache clear
   ```

## Cache Issues

### "Permission denied" on cache

**Cause**: Incorrect cache directory permissions.

**Solution**:
```bash
# Check cache directory
ls -la ~/.cache/manx

# Fix permissions
chmod 755 ~/.cache/manx
chmod -R 644 ~/.cache/manx/*

# Or use custom cache directory
manx config --cache-dir ~/my-cache
```

### Cache taking too much disk space

**Solution**:
```bash
# Check cache size
manx cache stats

# Clear old entries
manx cache clear

# Adjust cache settings
manx config --max-cache-size 50  # 50MB limit
```

### Cache corruption

**Symptoms**: Garbled output, parsing errors

**Solution**:
```bash
# Remove corrupted cache
rm -rf ~/.cache/manx

# Recreate cache directory
mkdir -p ~/.cache/manx

# Test with fresh search
manx fastapi
```

## Network Issues

### "Connection timeout"

**Troubleshooting**:

1. **Check internet connection**:
   ```bash
   ping 8.8.8.8
   curl https://google.com
   ```

2. **Check Context7 status**:
   ```bash
   curl -I https://mcp.context7.com/mcp
   ```

3. **Use offline mode**:
   ```bash
   manx --offline fastapi
   ```

4. **Configure proxy** (if behind corporate firewall):
   ```bash
   export HTTP_PROXY=http://proxy.company.com:8080
   export HTTPS_PROXY=http://proxy.company.com:8080
   manx fastapi
   ```

### "SSL/TLS certificate error"

**Solution**:
```bash
# Update certificates (macOS)
brew install ca-certificates

# Update certificates (Ubuntu/Debian)
sudo apt-get update && sudo apt-get install ca-certificates

# Update certificates (Fedora/RHEL)
sudo dnf update ca-certificates
```

## Configuration Issues

### "Config file not found"

**Solution**:
```bash
# Create config directory
mkdir -p ~/.config/manx

# Run manx to generate default config
manx --version

# Verify config created
cat ~/.config/manx/config.json
```

### "Invalid API key"

**Solution**:
```bash
# Check current API key
manx config --show

# Remove invalid key
manx config --api-key ""

# Get new key from https://context7.com/dashboard
manx config --api-key sk-new-valid-key
```

### Config changes not taking effect

**Solution**:
```bash
# Environment variables override config
unset MANX_API_KEY
unset MANX_CACHE_DIR

# Verify config is used
manx config --show
```

## Output Issues

### Garbled or corrupted text

**Cause**: Terminal encoding issues.

**Solution**:
```bash
# Set UTF-8 encoding
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Disable colors if needed
export NO_COLOR=1
manx fastapi
```

### "No color output"

**Solution**:
```bash
# Enable colors
unset NO_COLOR
manx config --color-output true
```

### JSON output malformed

**Solution**:
```bash
# Use quiet mode for clean JSON
manx --quiet --json fastapi | jq .

# Clear cache if corrupted
manx cache clear
manx --json fastapi
```

## Platform-Specific Issues

### Windows Issues

#### "The system cannot find the path specified"

```bash
# Add to PATH
setx PATH "%PATH%;C:\path\to\manx"

# Restart terminal
```

#### PowerShell execution policy

```powershell
# If needed, allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux Issues

#### "libssl.so not found"

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# Fedora/RHEL
sudo dnf install openssl-devel

# Arch
sudo pacman -S openssl
```

#### "GLIBC version mismatch"

```bash
# Check GLIBC version
ldd --version

# If too old, install from source
cargo install manx-cli
```

## Performance Issues

### Very slow startup

**Troubleshooting**:

1. **Check disk space**:
   ```bash
   df -h ~/.cache
   ```

2. **Check cache size**:
   ```bash
   manx cache stats
   ```

3. **Clear cache**:
   ```bash
   manx cache clear
   ```

4. **Disable cache temporarily**:
   ```bash
   manx --no-cache fastapi
   ```

### High memory usage

**Solution**:
```bash
# Reduce cache size
manx config --max-cache-size 25

# Clear cache
manx cache clear

# Monitor memory
ps aux | grep manx
```

## Debug Mode

### Enable comprehensive debugging

```bash
# Method 1: Command flag
manx --debug fastapi

# Method 2: Environment variable
export MANX_DEBUG=1
manx fastapi

# Method 3: Maximum verbosity
RUST_LOG=debug manx fastapi
```

### Debug output explanation

```
[DEBUG] Config loaded from: /home/user/.config/manx/config.json
[DEBUG] Cache directory: /home/user/.cache/manx
[DEBUG] API endpoint: https://api.context7.com/v1
[DEBUG] Request: GET /search?q=fastapi
[DEBUG] Response: 200 OK (456ms)
[DEBUG] Cache write: /home/user/.cache/manx/fastapi.json
[DEBUG] Results rendered: 10 items
```

## Getting Help

### Collect diagnostic information

```bash
# Create diagnostic report
cat > debug-report.txt << EOF
# Manx Debug Report
Version: $(manx --version)
OS: $(uname -a)
Config: $(manx config --show)
Cache: $(manx cache stats)
EOF

# Include debug output
manx --debug fastapi >> debug-report.txt 2>&1
```

### Where to get help

1. **Check this troubleshooting guide**
2. **Search existing issues**: [GitHub Issues](https://github.com/neur0map/manx/issues)
3. **Create new issue** with debug report
4. **Join discussion**: [GitHub Discussions](https://github.com/neur0map/manx/discussions)

### Reporting bugs

Include this information:
- Manx version (`manx --version`)
- Operating system
- Command that failed
- Full error message
- Debug output (`manx --debug ...`)
- Config file content (redact API key)

## Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `command not found: manx` | Not in PATH | Add to PATH or reinstall |
| `permission denied` | No execute permission | `chmod +x manx` |
| `no results found` | Library not found | Check spelling, clear cache |
| `rate limit exceeded` | Too many requests | Set API key |
| `connection timeout` | Network issue | Check internet, use offline |
| `invalid json response` | Cache corruption | Clear cache |
| `config file error` | Malformed config | Delete and recreate |

## Prevention Tips

1. **Set up API key** to avoid rate limits
2. **Regular cache maintenance** with `manx cache stats`
3. **Keep manx updated** with latest releases
4. **Monitor disk space** for cache directory
5. **Backup config** before major changes