#!/usr/bin/env bash
set -euo pipefail

# Manx Installation Script
# Installs the latest release of Manx CLI from GitHub

readonly REPO="neur0map/manx"
readonly BINARY_NAME="manx"
readonly INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
readonly CONFIG_DIR="${HOME}/.config/manx"
readonly CACHE_DIR="${HOME}/.cache/manx"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Platform detection
detect_platform() {
    local platform
    case "$(uname -s)" in
        Linux*)
            case "$(uname -m)" in
                x86_64) platform="x86_64-unknown-linux-gnu" ;;
                aarch64|arm64) platform="aarch64-unknown-linux-gnu" ;;
                *) echo "❌ Unsupported architecture: $(uname -m)"; exit 1 ;;
            esac
            ;;
        Darwin*)
            case "$(uname -m)" in
                x86_64) platform="x86_64-apple-darwin" ;;
                arm64) platform="aarch64-apple-darwin" ;;
                *) echo "❌ Unsupported architecture: $(uname -m)"; exit 1 ;;
            esac
            ;;
        CYGWIN*|MINGW*|MSYS*)
            platform="x86_64-pc-windows-msvc"
            ;;
        *)
            echo "❌ Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
    echo "$platform"
}

# Logging functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file with progress
download_file() {
    local url="$1"
    local output="$2"
    
    if command_exists curl; then
        curl -fsSL --progress-bar "$url" -o "$output"
    elif command_exists wget; then
        wget -q --show-progress "$url" -O "$output"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Get latest release version
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO}/releases/latest"
    
    if command_exists curl; then
        curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command_exists wget; then
        wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        log_error "Cannot fetch latest version. Please install curl or wget."
        exit 1
    fi
}

# Verify installation
verify_installation() {
    local install_path="$1"
    
    if [[ -x "$install_path" ]]; then
        local version
        version=$("$install_path" --version 2>/dev/null || echo "unknown")
        log_success "Manx installed successfully: $version"
        log_info "Location: $install_path"
        return 0
    else
        log_error "Installation verification failed"
        return 1
    fi
}

# Uninstall function
uninstall_manx() {
    log_info "Uninstalling Manx..."
    
    # Remove binary
    local binary_path="${INSTALL_DIR}/${BINARY_NAME}"
    if [[ -f "$binary_path" ]]; then
        rm -f "$binary_path"
        log_success "Removed binary: $binary_path"
    else
        log_warning "Binary not found at: $binary_path"
    fi
    
    # Ask about config and cache
    echo
    read -p "Remove configuration and cache files? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        [[ -d "$CONFIG_DIR" ]] && rm -rf "$CONFIG_DIR" && log_success "Removed config: $CONFIG_DIR"
        [[ -d "$CACHE_DIR" ]] && rm -rf "$CACHE_DIR" && log_success "Removed cache: $CACHE_DIR"
    else
        log_info "Kept configuration and cache files"
    fi
    
    log_success "Manx uninstalled successfully"
    exit 0
}

# Main installation function
install_manx() {
    log_info "Installing Manx CLI..."
    
    # Detect platform
    local platform
    platform=$(detect_platform)
    log_info "Detected platform: $platform"
    
    # Get latest version
    local version
    version=$(get_latest_version)
    if [[ -z "$version" ]]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi
    log_info "Latest version: $version"
    
    # Construct download URL
    local binary_file="${BINARY_NAME}-${platform}"
    local download_url="https://github.com/${REPO}/releases/download/${version}/${binary_file}"
    
    # Add .exe extension for Windows
    if [[ "$platform" == *"windows"* ]]; then
        binary_file="${binary_file}.exe"
        download_url="${download_url}.exe"
    fi
    
    # Create temporary directory
    local temp_dir
    temp_dir=$(mktemp -d)
    trap "rm -rf '$temp_dir'" EXIT
    
    local temp_file="${temp_dir}/${binary_file}"
    local install_path="${INSTALL_DIR}/${BINARY_NAME}"
    
    # Download binary
    log_info "Downloading from: $download_url"
    if ! download_file "$download_url" "$temp_file"; then
        log_error "Failed to download Manx binary"
        log_info "Binary not available for your platform. Trying cargo install..."
        
        # Fallback to cargo install if available
        if command_exists cargo; then
            log_info "Installing via cargo..."
            if cargo install manx-cli; then
                log_success "🎉 Manx installed successfully via cargo!"
                echo
                echo "Quick start:"
                echo "  manx fastapi              # Search FastAPI docs"
                echo "  manx react@18 hooks       # Search React 18 hooks"
                echo "  manx config --api-key KEY # Set Context7 API key"
                echo "  manx --help               # Show all options"
                echo
                log_info "For full documentation: https://github.com/${REPO}#readme"
                exit 0
            else
                log_error "Cargo install failed"
                exit 1
            fi
        else
            log_error "Please install Rust and Cargo, then run: cargo install manx-cli"
            log_info "Visit https://rustup.rs/ to install Rust"
            exit 1
        fi
    fi
    
    # Make executable
    chmod +x "$temp_file"
    
    # Check if we need sudo for installation
    local use_sudo=false
    if [[ ! -w "$(dirname "$install_path")" ]]; then
        use_sudo=true
        log_warning "Need sudo for installation to $INSTALL_DIR"
    fi
    
    # Install binary
    if [[ "$use_sudo" == true ]]; then
        if ! command_exists sudo; then
            log_error "sudo required but not found. Please install to a writable directory."
            log_info "Try: INSTALL_DIR=\"\$HOME/.local/bin\" curl -fsSL ... | bash"
            exit 1
        fi
        sudo mv "$temp_file" "$install_path"
    else
        # Create directory if it doesn't exist
        mkdir -p "$(dirname "$install_path")"
        mv "$temp_file" "$install_path"
    fi
    
    # Verify installation
    if verify_installation "$install_path"; then
        echo
        log_success "🎉 Manx is now installed and ready to use!"
        echo
        echo "Quick start:"
        echo "  manx fastapi              # Search FastAPI docs"
        echo "  manx react@18 hooks       # Search React 18 hooks"
        echo "  manx config --api-key KEY # Set Context7 API key"
        echo "  manx --help               # Show all options"
        echo
        log_info "For full documentation: https://github.com/${REPO}#readme"
    else
        exit 1
    fi
}

# Show usage
show_usage() {
    cat << EOF
Manx CLI Installer

USAGE:
    install.sh [OPTIONS]

OPTIONS:
    -h, --help        Show this help message
    --uninstall       Uninstall Manx
    --version         Show installer version

ENVIRONMENT VARIABLES:
    INSTALL_DIR       Installation directory (default: /usr/local/bin)

EXAMPLES:
    # Default installation
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash
    
    # Install to custom directory
    INSTALL_DIR="\$HOME/.local/bin" curl -fsSL ... | bash
    
    # Uninstall
    curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | bash -s -- --uninstall

For more information, visit: https://github.com/${REPO}
EOF
}

# Main script
main() {
    case "${1:-}" in
        -h|--help)
            show_usage
            exit 0
            ;;
        --uninstall)
            uninstall_manx
            ;;
        --version)
            echo "Manx Installer v1.0.0"
            exit 0
            ;;
        "")
            install_manx
            ;;
        *)
            log_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
}

# Check if running as root (warn but allow)
if [[ $EUID -eq 0 ]]; then
    log_warning "Running as root. Consider running as normal user with INSTALL_DIR=\$HOME/.local/bin"
fi

# Run main function
main "$@"