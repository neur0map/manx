# 📚 Manx

> *A blazing-fast CLI documentation finder that brings Context7 MCP docs right to your terminal - no IDE required*

<div align="center">

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/language-Rust-orange.svg)
![Status](https://img.shields.io/badge/status-Production%20Ready-brightgreen.svg)
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
- 🚀 **Export to Markdown/JSON** for documentation

## Features

- 🔍 **Fast search**: `manx fastapi` → instant results
- 📚 **Version support**: `manx react@18 hooks` → React 18-specific hooks
- 🖥️ **Native CLI experience** - no IDE required
- 📝 **Full documentation**: `manx doc fastapi middleware` → complete guides
- 💾 **Intelligent caching** with TTL and auto-cleanup
- 📤 **Export functionality** to Markdown and JSON formats
- ⚙️ **Configurable settings** and API key support
- 🌐 **Offline mode** support with local cache
- 🎨 **Colored output** (respects NO_COLOR)

## Installation

### Quick Install (Recommended)

```bash
# Linux/macOS - Install to /usr/local/bin
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash

# Or with wget
wget -qO- https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```

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

### From Source (Cargo)

```bash
# Install from crates.io (coming soon)
cargo install manx

# Or build from source
git clone https://github.com/neur0map/manx.git
cd manx
cargo build --release
sudo cp target/release/manx /usr/local/bin/
```

## Usage

### Quick Start

```bash
# Search for any library
manx fastapi

# Search with a query
manx fastapi middleware

# Version-specific search  
manx react@18 hooks
manx vue@3 composition

# Get full documentation
manx doc fastapi authentication
manx doc django models

# Export results
manx fastapi -o results.md
manx doc react hooks -o react-hooks.json
```

### Command Reference

#### Basic Search
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

#### Full Documentation
```bash
manx doc <library> <query>  # Get comprehensive documentation

# Examples  
manx doc fastapi middleware    # Complete FastAPI middleware guide
manx doc react hooks          # Full React Hooks documentation
manx doc django orm           # Django ORM complete guide
```

#### Cache Management
```bash
manx cache stats           # Show cache statistics
manx cache list            # List cached libraries
manx cache clear           # Clear all cached data
manx --clear-cache         # Quick cache clear (global flag)
```

#### Configuration
```bash
manx config --show                    # Show current settings
manx config --api-key YOUR_KEY       # Set Context7 API key
manx config --cache-dir /path/cache  # Set cache directory
manx config --auto-cache on          # Enable auto-caching
manx config --auto-cache off         # Disable auto-caching
```

#### Export Options
```bash
manx fastapi -o results.md     # Export as Markdown
manx fastapi -o results.json   # Export as JSON
manx doc react -o react.md     # Export documentation

# Format auto-detected by file extension
```

#### Other Options
```bash
manx --offline                 # Use cache only (no network)
manx --quiet                   # JSON output (for scripts)
manx --debug                   # Enable debug logging
manx --auto-cache-on           # Enable auto-caching
manx --auto-cache-off          # Disable auto-caching
```

### Advanced Usage

#### Scripting with Quiet Mode
```bash
# Get JSON output for scripts
manx react hooks --quiet | jq '.[] | .title'

# Export and process
manx fastapi -o /tmp/docs.json --quiet
cat /tmp/docs.json | jq '.[] | select(.relevance_score > 0.8)'
```

#### Working Offline
```bash
# First, cache some libraries online
manx react hooks
manx fastapi middleware
manx django models

# Then work offline
manx --offline react hooks     # Uses cached results
manx --offline fastapi        # Works from cache
```

#### Configuration Management  
```bash
# Set up API key for better rate limits
manx config --api-key sk-your-context7-api-key

# Custom cache location
manx config --cache-dir ~/Documents/manx-cache

# Check current settings
manx config --show
```

## Configuration

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

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `api_key` | `null` | Context7 API key (optional, for rate limits) |
| `cache_dir` | `~/.cache/manx` | Local cache directory |
| `default_limit` | `10` | Default number of search results |
| `offline_mode` | `false` | Always use cache only |
| `color_output` | `true` | Enable colored terminal output |
| `auto_cache_enabled` | `true` | Auto-cache search results |
| `cache_ttl_hours` | `24` | Cache expiration time |
| `max_cache_size_mb` | `100` | Maximum cache size |

## Context7 API Key

While Manx works without an API key, setting one up provides:
- ✅ **Higher rate limits**
- ✅ **Priority access** to Context7 servers  
- ✅ **Premium features** (coming soon)

Get your free API key at [context7.com](https://context7.com) and set it:
```bash
manx config --api-key sk-your-key-here
```

## Performance

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

## Troubleshooting

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

#### API Key Issues
```bash
# Verify API key format
manx config --show

# Test without API key (should still work)
manx config --api-key ""
manx fastapi

# Get new API key from context7.com
```

### Environment Variables

Manx respects these environment variables:

```bash
export NO_COLOR=1              # Disable colored output
export MANX_CACHE_DIR=~/cache  # Custom cache directory  
export MANX_API_KEY=sk-xxx     # API key (overrides config)
export MANX_DEBUG=1            # Enable debug logging
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

## Uninstall

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

### Architecture

- `src/main.rs` - Entry point and command dispatch
- `src/cli.rs` - Command-line argument parsing  
- `src/client.rs` - Context7 MCP API client
- `src/search.rs` - Search logic and fuzzy matching
- `src/render.rs` - Terminal output and formatting
- `src/cache.rs` - Local caching system
- `src/export.rs` - File export functionality
- `src/config.rs` - Configuration management

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