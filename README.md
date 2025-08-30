# 📚 Manx

> *A blazing-fast CLI documentation finder that brings Context7 MCP docs right to your terminal - no IDE required*

<div align="center">

![GitHub Release](https://img.shields.io/github/v/release/neur0map/manx)
![Crates.io Version](https://img.shields.io/crates/v/manx-cli)
![GitHub Downloads](https://img.shields.io/github/downloads/neur0map/manx/total?label=github%20downloads)
![Crates.io Downloads](https://img.shields.io/crates/d/manx-cli?label=crates.io%20downloads)
![Crates.io Recent Downloads](https://img.shields.io/crates/dr/manx-cli?label=recent%20downloads)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Binary Size](https://img.shields.io/badge/binary-2.9MB-blue.svg)

</div>

## What is Manx?

Manx is a command-line interface documentation finder designed for developers who prefer working in the terminal. It uses **Context7 MCP** (Model Context Protocol) as its primary backend to provide **real-time, version-specific** documentation snippets without leaving your development environment.

### Why Manx?

- ⚡ **< 1 second** search results
- 🎯 **Version-specific** docs (e.g., React 18 vs 17)
- 📦 **Single 2.9MB binary** - no dependencies  
- 🔌 **Context7 MCP integration** - always up-to-date
- 💾 **Smart caching** - works offline after first use
- 🌈 **Beautiful terminal output** with syntax highlighting
- 📊 **Smart result limiting** - shows 10 results by default, customizable
- 🚀 **Export to Markdown/JSON** for documentation

## Quick Install

```bash
# Option 1: Using Cargo (if you have Rust installed)
cargo install manx-cli

# Option 2: Shell script installer
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash

# Option 3: Using wget
wget -qO- https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```

## Quick Start

```bash
# Search for any library
manx fastapi

# Search with a query
manx fastapi middleware

# Version-specific search  
manx react@18 hooks

# Get full documentation
manx doc fastapi authentication

# Limit results (default: 10, use 0 for unlimited)
manx fastapi --limit 5
manx react hooks --limit 0

# Save results to file
manx fastapi --save 1,3,5
manx react hooks --save-all --json
```

---

<details>
<summary><strong>📦 Installation Options</strong></summary>

### Manual Installation

1. **Download the latest release** for your platform:
   - [Linux x86_64](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-unknown-linux-gnu)
   - [Linux ARM64](https://github.com/neur0map/manx/releases/latest/download/manx-aarch64-unknown-linux-gnu)  
   - [macOS x86_64](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-apple-darwin)
   - [macOS ARM64](https://github.com/neur0map/manx/releases/latest/download/manx-aarch64-apple-darwin)
   - [Windows](https://github.com/neur0map/manx/releases/latest/download/manx-x86_64-pc-windows-msvc.exe)

2. **Make executable and move to PATH**:
   ```bash
   chmod +x manx-*
   sudo mv manx-* /usr/local/bin/manx
   ```

3. **Verify installation**:
   ```bash
   manx --version
   ```

### From Cargo (Recommended for Rust Users)

```bash
# Install from crates.io
cargo install manx-cli

# Verify installation
manx --version
```

### From Source

```bash
# Build from source
git clone https://github.com/neur0map/manx.git
cd manx
cargo build --release
sudo cp target/release/manx /usr/local/bin/
```

</details>

<details>
<summary><strong>📖 Complete Usage Guide</strong></summary>

## Basic Search
```bash
manx <library>              # Search library docs
manx <library> <query>      # Search library for specific query
manx <library>@<version>    # Version-specific search

# Examples
manx fastapi                # All FastAPI docs
manx fastapi cors           # FastAPI CORS documentation  
manx react@18              # React v18 documentation
manx vue@3 composition     # Vue 3 Composition API
```

## Full Documentation
```bash
manx doc <library> <query>  # Get comprehensive documentation

# Examples  
manx doc fastapi middleware    # Complete FastAPI middleware guide
manx doc react hooks          # Full React Hooks documentation
manx doc django orm           # Django ORM complete guide
```

## Result Limiting
```bash
manx fastapi --limit 5         # Show only first 5 results
manx react hooks --limit 0     # Show all results (unlimited)
manx vue --limit 15            # Show first 15 results
# Default limit is 10 results
```

## Export Options
```bash
manx fastapi --save 1,3,7     # Save specific results as markdown
manx fastapi --save 1,3,7 --json  # Save as JSON
manx react --save-all         # Save all results
manx doc react -o react.md    # Export documentation
```

## Cache Management
```bash
manx cache stats           # Show cache statistics
manx cache list            # List cached libraries
manx cache clear           # Clear all cached data
manx --clear-cache         # Quick cache clear (global flag)
```

## Other Options
```bash
manx --limit 5                 # Limit number of results (default: 10)
manx --offline                 # Use cache only (no network)
manx --quiet                   # JSON output (for scripts)
manx --debug                   # Enable debug logging
```

</details>

<details>
<summary><strong>🔑 Context7 API Key Setup</strong></summary>

**Important:** Without an API key, Manx uses Context7's shared MCP endpoint which has strict rate limits. Users often experience rate limiting after just a few searches. Setting up an API key provides dedicated access with much higher limits.

### Why You Need an API Key

- **Without API Key:** Uses shared MCP endpoint (`mcp.context7.com/mcp`) with very low rate limits
- **With API Key:** Uses dedicated API endpoint with high rate limits
- ✅ **Faster responses** and better reliability
- ✅ **Premium features** access

### Getting Your API Key

1. Visit the [Context7 Dashboard](https://context7.com/dashboard)
2. Create a free account or log in
3. Generate your API key (starts with `sk-`)
4. Set it up in manx:

```bash
# Method 1: Using config command (recommended)
manx config --api-key sk-your-context7-key-here

# Method 2: Environment variable
export MANX_API_KEY=sk-your-context7-key-here

# Method 3: Direct config file edit (~/.config/manx/config.json)
{
  "api_key": "sk-your-context7-key-here"
}
```

### Verifying Your Setup

```bash
# Check current configuration
manx config --show

# Test with your API key (should be much faster)
manx fastapi
```

### Removing Your API Key

```bash
# Remove API key (switches back to shared rate limits)
manx config --api-key ""

# Or unset environment variable
unset MANX_API_KEY
```

**Note:** The API key only affects rate limiting and endpoint selection. All documentation content remains the same.

</details>

<details>
<summary><strong>⚙️ Configuration</strong></summary>

Manx stores configuration in `~/.config/manx/config.json`:

```json
{
  "api_key": "sk-your-context7-key",
  "cache_dir": null,
  "default_limit": 10,
  "offline_mode": false,
  "color_output": true,
  "auto_cache_enabled": true,
  "cache_ttl_hours": 24,
  "max_cache_size_mb": 100
}
```

### Configuration Commands

```bash
manx config --show                    # Show current settings
manx config --api-key YOUR_KEY       # Set Context7 API key
manx config --cache-dir /path/cache  # Set cache directory
manx config --auto-cache on          # Enable auto-caching
manx config --auto-cache off         # Disable auto-caching

# Note: Default result limit (10) is configurable in config.json
```

### Environment Variables

```bash
export NO_COLOR=1              # Disable colored output
export MANX_CACHE_DIR=~/cache  # Custom cache directory  
export MANX_API_KEY=sk-xxx     # API key (overrides config)
export MANX_DEBUG=1            # Enable debug logging
```

</details>

<details>
<summary><strong>🚀 Performance & Benchmarks</strong></summary>

Manx is designed for speed and efficiency:

| Metric | Value | Notes |
|--------|--------|-------|
| **Binary Size** | 2.9MB | Single static binary |
| **Startup Time** | < 50ms | Near-instantaneous |
| **Search Speed** | < 1s | Including network + parsing |
| **Memory Usage** | < 10MB | Minimal RAM footprint |
| **Cache Size** | 100MB max | Auto-managed, configurable |
| **Offline Mode** | ✅ | Full functionality with cache |

### Benchmarks
```bash
# Cold search (first time)
$ time manx fastapi
Real: 0.8s  User: 0.1s  Sys: 0.05s

# Warm search (cached)  
$ time manx --offline fastapi
Real: 0.1s  User: 0.08s  Sys: 0.02s

# Export benchmark
$ time manx fastapi -o docs.md
Real: 0.9s  User: 0.15s  Sys: 0.08s
```

</details>

<details>
<summary><strong>🛠️ Troubleshooting</strong></summary>

### Common Issues

#### "No results found" 
```bash
# Check if library name is correct
manx config --show                    # Verify settings
manx fastapi                          # Try exact library name
manx python                           # Try broader search

# Clear cache and retry
manx cache clear
manx fastapi
```

#### Network/Connectivity Issues
```bash
# Test with debug mode
manx --debug fastapi

# Use offline mode if you have cache
manx --offline fastapi

# Check Context7 status
curl -I https://mcp.context7.com/mcp
```

#### Cache Issues
```bash
# Check cache stats
manx cache stats

# Clear and rebuild cache
manx cache clear
manx fastapi                          # Rebuild cache

# Use custom cache location
manx config --cache-dir ~/my-cache
```

#### Permission Issues
```bash
# Fix binary permissions
chmod +x /usr/local/bin/manx

# Alternative install location (no sudo)
mkdir -p ~/.local/bin
mv manx ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Logs and Debug Info

```bash
# Enable debug mode
manx --debug fastapi 2>&1 | tee debug.log

# Check cache directory
ls -la ~/.cache/manx/

# View config file
cat ~/.config/manx/config.json
```

</details>

<details>
<summary><strong>🗑️ Uninstall</strong></summary>

Remove Manx completely:

```bash
# Remove binary
sudo rm /usr/local/bin/manx

# Remove config and cache
rm -rf ~/.config/manx
rm -rf ~/.cache/manx

# Or use the installer  
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash -s -- --uninstall
```

</details>

## Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature-name`
3. **Make changes** and add tests
4. **Run tests**: `cargo test`  
5. **Check formatting**: `cargo fmt --check`
6. **Run linter**: `cargo clippy`
7. **Submit a Pull Request**

### Development Setup

```bash
git clone https://github.com/neur0map/manx.git
cd manx
cargo build
cargo test
./target/debug/manx --help
```

## 📊 Stats & Activity

<div align="center">

![GitHub Stars](https://img.shields.io/github/stars/neur0map/manx?style=social)
![GitHub Forks](https://img.shields.io/github/forks/neur0map/manx?style=social)
![GitHub Issues](https://img.shields.io/github/issues/neur0map/manx)
![GitHub Last Commit](https://img.shields.io/github/last-commit/neur0map/manx)

</div>

## License

MIT License - see [LICENSE](LICENSE) for details

## Acknowledgments

- **Context7** for the excellent MCP documentation API
- **Rust community** for amazing crates and tooling
- **Clap** for beautiful CLI parsing
- **All contributors** who make Manx better

---

Built with ❤️ for developers who live in the terminal.

**[⬆️ Back to Top](#-manx)**