---
title: Getting Started
description: Get up and running with Manx in under a minute
---

## Welcome to Manx!

Manx is a blazing-fast CLI tool that brings documentation right to your terminal. This guide will help you get started in just a few minutes.

## Prerequisites

- **Operating System**: Linux, macOS, or Windows
- **Terminal**: Any modern terminal emulator
- **Optional**: Rust toolchain (for building from source)

## Quick Install

Choose your preferred installation method:

### Option 1: Using Cargo (Recommended for Rust developers)

```bash
cargo install manx-cli
```

### Option 2: Shell Script Installer

```bash
curl -fsSL https://raw.githubusercontent.com/neur0map/manx/main/install.sh | bash
```

### Option 3: Manual Binary Download

1. Download the binary for your platform from the [releases page](https://github.com/neur0map/manx/releases)
2. Make it executable: `chmod +x manx-*`
3. Move to your PATH: `sudo mv manx-* /usr/local/bin/manx`

## Verify Installation

```bash
manx --version
# Output: manx 0.3.5
```

## Your First Search

Let's search for FastAPI documentation:

```bash
manx fastapi
```

You should see results in under a second! 🚀

## Basic Usage

### Search a library
```bash
manx react
```

### Search with a query
```bash
manx react hooks
```

### Version-specific search
```bash
manx react@18 useEffect
```

### Get full documentation
```bash
manx doc react hooks
```

## Setting Up Context7 API Key (Recommended)

While Manx works without an API key, you'll get better performance with one:

1. Get your free API key from [Context7 Dashboard](https://context7.com/dashboard)
2. Configure Manx:

```bash
manx config --api-key sk-your-key-here
```

## What's Next?

- Learn about all [available commands](/manx/commands/)
- Configure Manx to your needs in [Configuration](/manx/configuration/)
- See [real-world examples](/manx/examples/)
- Troubleshoot issues in our [FAQ](/manx/troubleshooting/)

## Quick Tips

- 💡 Use `--limit 0` to see all results
- 💡 Use `--save` to export results to files
- 💡 Use `--offline` to work without internet
- 💡 Use `manx cache clear` to free up space

## Need Help?

Check out our [troubleshooting guide](/troubleshooting/) or [open an issue](https://github.com/neur0map/manx/issues) on GitHub.