#!/usr/bin/env bash
# VibeAnvil Install Script
# Usage: curl -fsSL https://raw.githubusercontent.com/OWNER/vibeanvil/main/install.sh | bash

set -euo pipefail

REPO="OWNER/vibeanvil"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}→${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1" >&2
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Darwin)  os="macos" ;;
        Linux)   os="linux" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *)       error "Unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x64" ;;
        aarch64|arm64) arch="arm64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${os}-${arch}"
}

# Get the download URL for the specified version
get_download_url() {
    local platform="$1"
    local asset_name="vibeanvil-${platform}"
    
    if [[ "$platform" == *"windows"* ]]; then
        asset_name="${asset_name}.exe"
    fi

    if [ "$VERSION" = "latest" ]; then
        echo "https://github.com/${REPO}/releases/latest/download/${asset_name}"
    else
        echo "https://github.com/${REPO}/releases/download/${VERSION}/${asset_name}"
    fi
}

# Get checksums URL
get_checksums_url() {
    if [ "$VERSION" = "latest" ]; then
        echo "https://github.com/${REPO}/releases/latest/download/checksums.txt"
    else
        echo "https://github.com/${REPO}/releases/download/${VERSION}/checksums.txt"
    fi
}

# Verify checksum
verify_checksum() {
    local file="$1"
    local checksums_url="$2"
    local filename
    filename=$(basename "$file")

    info "Verifying checksum..."
    
    local checksums
    checksums=$(curl -fsSL "$checksums_url" 2>/dev/null) || {
        warn "Could not fetch checksums, skipping verification"
        return 0
    }

    local expected
    expected=$(echo "$checksums" | grep "$filename" | awk '{print $1}')
    
    if [ -z "$expected" ]; then
        warn "Checksum not found for $filename, skipping verification"
        return 0
    fi

    local actual
    if command -v sha256sum &> /dev/null; then
        actual=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum &> /dev/null; then
        actual=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        warn "No sha256 tool found, skipping verification"
        return 0
    fi

    if [ "$actual" != "$expected" ]; then
        error "Checksum verification failed!\nExpected: $expected\nActual:   $actual"
    fi

    success "Checksum verified"
}

# Main installation
main() {
    echo ""
    echo "🔨 VibeAnvil Installer"
    echo "   Contract-first vibe coding with evidence, audit, and repo-brain harvesting"
    echo ""

    # Detect platform
    local platform
    platform=$(detect_platform)
    info "Detected platform: $platform"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    # Download binary
    local download_url
    download_url=$(get_download_url "$platform")
    local temp_file
    temp_file=$(mktemp)

    info "Downloading from: $download_url"
    
    if ! curl -fsSL "$download_url" -o "$temp_file"; then
        error "Failed to download binary. Check if the release exists."
    fi

    # Verify checksum
    local checksums_url
    checksums_url=$(get_checksums_url)
    verify_checksum "$temp_file" "$checksums_url"

    # Install binary
    local binary_name="vibeanvil"
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="vibeanvil.exe"
    fi

    local install_path="$INSTALL_DIR/$binary_name"
    mv "$temp_file" "$install_path"
    chmod +x "$install_path"

    success "Installed to: $install_path"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo ""
        warn "Add this to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo ""
        echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
    fi

    # Verify installation
    echo ""
    if command -v vibeanvil &> /dev/null; then
        success "Installation complete!"
        echo ""
        vibeanvil --version 2>/dev/null || echo "vibeanvil installed successfully"
    else
        success "Binary installed. Restart your shell or run:"
        echo ""
        echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
        echo "    vibeanvil --version"
    fi

    echo ""
    echo "Get started:"
    echo "    vibeanvil init"
    echo "    vibeanvil --help"
    echo ""
}

main "$@"
