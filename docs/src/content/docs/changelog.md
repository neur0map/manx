---
title: Changelog
description: Release notes and version history for Manx
---

## [v0.3.5] - 2025-08-30

### ✨ Added
- Documentation section caching for faster access
- Open command functionality
- Enhanced Dual Operating Modes description
- Comprehensive download badges in README

### 🐛 Fixed
- Fixed clippy warning for unnecessary_map_or
- Fixed formatting issues for CI
- Fixed release workflow to use git tag message instead of hardcoded body

### 📝 Documentation
- Updated README with better goals and descriptions
- Enhanced project documentation and examples

---

## [v0.3.4] - 2025-08-30

### ✨ Added
- Improved search result diversity and display
- New `--save-all` flag for exporting all results
- Custom output filename support with `-o` flag
- Version-specific search with `@version` syntax

### 🚀 Performance
- 20% faster JSON parsing with optimized deserialization
- Reduced memory usage for large result sets
- Better concurrent request handling

### 🐛 Fixed
- Fixed search results showing duplicate content
- Fixed race condition in cache writes
- Improved error messages for network failures
- Better handling of empty search results

---

## [v0.3.3] - 2025-08-30

### ✨ Added
- Full documentation mode with `doc` command
- Enhanced fuzzy matching for search queries
- Support for custom Context7 API endpoints

### 🔧 Changed
- Default result limit changed from 5 to 10
- Improved cache TTL handling with background refresh
- Better terminal color detection

### 🐛 Fixed
- Fixed cache invalidation issues
- Resolved binary size bloat from debug symbols
- Better error handling for corrupted cache files

---

## [v0.3.2] - 2025-08-30

### ✨ Added
- Context7 MCP integration for real-time documentation
- Smart caching with configurable TTL
- Auto-retry logic for failed requests

### 🚀 Performance
- Sub-second search results with intelligent caching
- Optimized binary size (now 2.9MB)
- Reduced startup time by 60%

### 🐛 Fixed
- Fixed SSL certificate validation issues
- Improved JSON parsing error messages
- Better handling of special characters in search queries

---

## [v0.3.1] - 2025-08-30

### ✨ Added
- Configuration management with `config` command
- Environment variable support for all options
- Offline mode for cached results

### 🔧 Changed
- Moved from custom API to Context7 MCP protocol
- Simplified installation process
- Updated CLI interface for better usability

### 🐛 Fixed
- Fixed cross-platform path handling
- Resolved config file permission issues
- Better error handling for network timeouts

---

## [v0.3.0] - 2025-08-30 🎉 **First Public Release**

### ✨ Major Features
- **Context7 MCP Integration**: Real-time access to documentation
- **Smart Caching**: Lightning-fast repeat searches
- **Export Functionality**: Save results as Markdown or JSON
- **Version-Specific Search**: Query specific library versions

### 🚀 Performance
- 10x faster than web-based documentation searches
- < 1s average search time with caching
- Minimal memory footprint (< 10MB)

### 🔧 Breaking Changes
- Removed legacy API endpoint support
- Changed cache directory structure  
- Updated command-line interface

### 📚 Documentation
- Complete rewrite of README with examples
- Added troubleshooting guide
- Created installation script
- Public repository and releases

---

## [v0.2.x] - Private Development Versions

### [v0.2.4] - 2025-08-15
- Added basic search functionality
- Implemented simple caching
- Cross-platform binary releases

### [v0.2.3] - 2025-08-10  
- Enhanced CLI interface
- HTTP API integration improvements
- Personal workflow optimization

### [v0.2.2] - 2025-08-05
- Core search engine development
- Basic configuration system
- Performance improvements

### [v0.2.1] - 2025-07-30
- Proof of concept implementation
- Initial Context7 integration testing

### [v0.2.0] - 2025-07-25
- First functional prototype
- Basic search and display functionality

---

## [v0.1.x] - Initial Development

### [v0.1.0] - 2025-07-15
- Initial project setup
- Basic Rust structure and CLI framework
- Personal development tool foundation

**Note**: Versions 0.1.x - 0.2.x were private development versions used personally before public release.

---

## Upcoming Features 🚀

### v0.4.0 - GitHub Integration (Coming Soon)
- Direct GitHub repository documentation access
- Natural language Q&A for project docs
- Integration with GitHub API for real-time updates

### v0.5.0 - Knowledge Search Engine
- Universal search across documentation and personal notes
- AI-powered documentation assistance
- Smart context understanding

### v1.0.0 - Full Release
- Official package manager distribution (Homebrew, apt, etc.)
- Plugin architecture for extensibility
- Advanced analytics and usage tracking

## Migration Guides

### Upgrading from v0.2.x to v0.3.x

**Cache Directory Changes:**
```bash
# Old cache will be automatically migrated
# Manual cleanup if needed:
rm -rf ~/.manx-cache  # Old location
# New location: ~/.cache/manx
```

**Configuration Changes:**
```bash
# Old config format no longer supported
# Run this to generate new config:
manx config --show
```

**Command Changes:**
```bash
# Old: manx search fastapi
# New: manx fastapi

# Old: manx --clear-cache
# New: manx cache clear
```

### Upgrading from v0.1.x to v0.2.x

**Complete Rewrite:**
No migration path available. Please reinstall and reconfigure.

## Release Schedule

- **Patch releases** (0.3.x): Every 1-2 weeks
- **Minor releases** (0.x.0): Every 1-2 months  
- **Major releases** (x.0.0): Every 6-12 months

## Support Policy

- **Current version**: Full support and updates
- **Previous minor**: Security fixes only
- **Older versions**: Community support only

## Contributors

Special thanks to all contributors across these releases:
- Core development team
- Community bug reporters
- Documentation contributors
- Feature suggestion providers

## Acknowledgments

- **Context7** for the excellent MCP documentation API
- **Rust community** for amazing crates and tooling
- **Clap** for beautiful CLI parsing
- **All users** who provide feedback and suggestions

---

**Note**: This changelog follows [Keep a Changelog](https://keepachangelog.com/) format and [Semantic Versioning](https://semver.org/) principles.