---
title: Contributing
description: How to contribute to Manx development
---

## Welcome Contributors! 🚀

We're thrilled you're interested in contributing to Manx! This guide will help you get started.

## Ways to Contribute

### 1. 🐛 Bug Reports

[Create an issue](https://github.com/neur0map/manx/issues/new) if you find:
- Incorrect search results
- Performance issues
- Crashes or errors
- Installation problems

### 2. ✨ Feature Requests

Suggest new features:
- New documentation sources
- Better search algorithms  
- UI/UX improvements
- Integration possibilities

### 3. 📝 Documentation

- Fix typos and unclear sections
- Add examples and use cases
- Improve installation guides
- Translate documentation

### 4. 🛠️ Code Contributions

- Fix bugs
- Implement new features
- Performance optimizations
- Test improvements

## Development Setup

### Prerequisites

- **Rust** 1.70+ (`rustup` recommended)
- **Git** for version control
- **Text editor** (VS Code, vim, etc.)

### Fork & Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/manx.git
cd manx

# Add upstream remote
git remote add upstream https://github.com/neur0map/manx.git
```

### Build & Run

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with debug info
cargo run -- --help

# Install development version
cargo install --path .
```

### Development Workflow

```bash
# Create feature branch
git checkout -b feature/amazing-feature

# Make changes and test
cargo test
cargo run -- fastapi

# Format code
cargo fmt

# Run linter
cargo clippy

# Commit changes
git commit -m "Add amazing feature"

# Push to your fork
git push origin feature/amazing-feature
```

## Project Structure

```
manx/
├── src/
│   ├── main.rs         # Entry point
│   ├── cli.rs          # Command-line interface
│   ├── client.rs       # HTTP client for Context7
│   ├── search.rs       # Search functionality
│   ├── cache.rs        # Caching system
│   ├── config.rs       # Configuration management
│   ├── render.rs       # Output rendering
│   ├── export.rs       # Result export
│   └── update.rs       # Auto-update system
├── tests/           # Integration tests
├── docs/            # Documentation site
├── Cargo.toml       # Rust manifest
├── README.md        # Project readme
└── install.sh       # Installation script
```

## Code Style

### Rust Guidelines

1. **Use `rustfmt`** for consistent formatting:
   ```bash
   cargo fmt
   ```

2. **Follow Clippy suggestions**:
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Write descriptive variable names**:
   ```rust
   // Good
   let search_results = client.search(library, query).await?;
   
   // Avoid
   let r = c.s(l, q).await?;
   ```

4. **Add documentation for public APIs**:
   ```rust
   /// Searches for documentation in the specified library
   /// 
   /// # Arguments
   /// 
   /// * `library` - The library name to search in
   /// * `query` - The search query string
   pub async fn search(library: &str, query: &str) -> Result<Vec<SearchResult>> {
       // implementation
   }
   ```

### Error Handling

Use `anyhow` for application errors and `thiserror` for library errors:

```rust
use anyhow::{Context, Result};

fn parse_config() -> Result<Config> {
    let config_str = fs::read_to_string("config.json")
        .context("Failed to read config file")?;
    
    serde_json::from_str(&config_str)
        .context("Failed to parse config JSON")
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_search_functionality

# Run integration tests
cargo test --test integration
```

### Writing Tests

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_library_name() {
        assert_eq!(parse_library_name("fastapi"), "fastapi");
        assert_eq!(parse_library_name("react@18"), "react");
    }

    #[tokio::test]
    async fn test_search_api() {
        let client = Client::new();
        let results = client.search("fastapi", "middleware").await.unwrap();
        assert!(!results.is_empty());
    }
}
```

#### Integration Tests

```rust
// tests/integration.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_basic_search() {
    let mut cmd = Command::cargo_bin("manx").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
```

## Documentation

### Code Documentation

```rust
/// Configuration for the Manx application
/// 
/// This struct holds all configuration options that can be
/// set via config file, environment variables, or CLI flags.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// Context7 API key for authenticated requests
    pub api_key: Option<String>,
    
    /// Custom cache directory path
    pub cache_dir: Option<PathBuf>,
}
```

### User Documentation

When adding features, update:
- `README.md` if it affects basic usage
- Documentation site in `docs/` folder
- Help text in `cli.rs`
- Examples in code comments

## Pull Request Process

### Before Submitting

1. **Update from upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run full test suite**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy -- -D warnings
   ```

3. **Test manually**:
   ```bash
   cargo build --release
   ./target/release/manx fastapi
   ```

4. **Update documentation** if needed

### Submitting

1. **Create descriptive PR title**:
   - `feat: add GitHub integration for documentation`
   - `fix: handle network timeout errors gracefully`
   - `docs: update installation instructions`

2. **Write clear PR description**:
   ```markdown
   ## Changes
   - Added support for GitHub repository documentation
   - Fixed timeout handling in network requests
   
   ## Testing
   - [x] Manual testing with various repositories
   - [x] Unit tests for new functionality
   - [x] Integration tests pass
   
   ## Breaking Changes
   None
   ```

3. **Link related issues**: `Fixes #123`

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Testing** on different platforms if needed
4. **Documentation** review
5. **Merge** after approval

## Release Process

### Versioning

We follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

### Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. Tag release: `git tag v1.2.3`
5. GitHub Actions handles:
   - Building binaries
   - Publishing to crates.io
   - Creating GitHub release

## Community Guidelines

### Code of Conduct

- **Be respectful** and inclusive
- **Help newcomers** get started
- **Give constructive feedback**
- **Assume positive intent**

### Communication

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, general discussion
- **Pull Requests**: Code contributions

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- GitHub contributor graphs
- Release notes for significant contributions

## Getting Help

### For Development

- **Setup issues**: Check existing issues or create new one
- **Rust questions**: [Rust Discord](https://discord.gg/rust-lang) or [r/rust](https://reddit.com/r/rust)
- **Project questions**: [GitHub Discussions](https://github.com/neur0map/manx/discussions)

### Good First Issues

Look for issues labeled:
- `good first issue`: Perfect for newcomers
- `help wanted`: Maintainers would appreciate help
- `documentation`: Improve docs
- `bug`: Fix reported bugs

## Example Contributions

### Adding New Feature

```rust
// src/features/new_feature.rs
pub struct NewFeature {
    config: Config,
}

impl NewFeature {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    pub async fn execute(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_new_feature() {
        let config = Config::default();
        let feature = NewFeature::new(config);
        assert!(feature.execute().await.is_ok());
    }
}
```

### Fixing a Bug

```rust
// Before (buggy)
fn parse_version(input: &str) -> String {
    input.split('@').collect::<Vec<_>>()[1].to_string() // Panics if no '@'
}

// After (fixed)
fn parse_version(input: &str) -> Option<String> {
    input.split('@').nth(1).map(|s| s.to_string())
}
```

## Thank You!

Every contribution, big or small, helps make Manx better for everyone. We appreciate your time and effort! 🙏

---

Have questions? Don't hesitate to ask in [GitHub Discussions](https://github.com/neur0map/manx/discussions)!