#!/bin/bash
# VibeAnvil Uninstall Script
# Usage: curl -fsSL https://raw.githubusercontent.com/ThanhNguyxn/vibeanvil/main/uninstall.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}→${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warning() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

echo ""
echo -e "${RED}╔═══════════════════════════════════════════╗${NC}"
echo -e "${RED}║        🗑️  VibeAnvil Uninstaller           ║${NC}"
echo -e "${RED}╚═══════════════════════════════════════════╝${NC}"
echo ""

# Possible install locations
LOCATIONS=(
    "$HOME/.local/bin/vibeanvil"
    "/usr/local/bin/vibeanvil"
    "$HOME/.cargo/bin/vibeanvil"
)

removed=0

for loc in "${LOCATIONS[@]}"; do
    if [ -f "$loc" ]; then
        rm -f "$loc"
        success "Removed: $loc"
        removed=1
    fi
    # Remove backup if exists
    if [ -f "${loc}.bak" ]; then
        rm -f "${loc}.bak"
        success "Removed backup: ${loc}.bak"
    fi
done

if [ $removed -eq 0 ]; then
    warning "No vibeanvil binary found in standard locations"
fi

# Remove workspace (optional)
if [ -d ".vibeanvil" ]; then
    echo ""
    read -p "Remove workspace directory (.vibeanvil)? [y/N] " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
        rm -rf .vibeanvil
        success "Removed workspace: .vibeanvil"
    else
        info "Keeping workspace"
    fi
fi

# Remove global data
if [ -d "$HOME/.vibeanvil" ]; then
    echo ""
    read -p "Remove global data (~/.vibeanvil)? [y/N] " confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
        rm -rf "$HOME/.vibeanvil"
        success "Removed global data: $HOME/.vibeanvil"
    else
        info "Keeping global data (BrainPack, etc.)"
    fi
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✅ VibeAnvil uninstalled successfully  ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════╝${NC}"
echo ""
echo "Thank you for using VibeAnvil! 👋"
echo ""
