#!/bin/bash

# MAME Frontend Uninstall Script
# This script removes the MAME frontend installation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to confirm action
confirm() {
    echo -n "$1 (y/N): "
    read -r response
    case "$response" in
        [yY][eE][sS]|[yY])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

print_status "MAME Frontend Uninstaller"
echo "=============================="
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_error "This script should not be run as root"
    exit 1
fi

# Find the installation directory
INSTALL_DIR=""
if [ -L ~/.local/bin/mame-frontend ]; then
    INSTALL_DIR=$(readlink -f ~/.local/bin/mame-frontend)
    INSTALL_DIR=$(dirname "$INSTALL_DIR")
    print_status "Found installation at: $INSTALL_DIR"
elif [ -d ~/mame-frontend ]; then
    INSTALL_DIR=~/mame-frontend
    print_status "Found installation at: $INSTALL_DIR"
else
    print_warning "Could not find MAME Frontend installation"
    print_status "Checking common locations..."
    
    # Check common build directories
    for dir in ~/mame-frontend ~/Downloads/mame-frontend ~/src/mame-frontend; do
        if [ -d "$dir" ] && [ -f "$dir/target/release/mame-frontend" ]; then
            INSTALL_DIR="$dir"
            print_status "Found installation at: $INSTALL_DIR"
            break
        fi
    done
fi

if [ -z "$INSTALL_DIR" ]; then
    print_error "MAME Frontend installation not found"
    print_status "Please manually remove the installation directory"
    exit 1
fi

# Confirm uninstallation
if ! confirm "Do you want to uninstall MAME Frontend from $INSTALL_DIR?"; then
    print_status "Uninstallation cancelled"
    exit 0
fi

# Remove desktop entry
if [ -f ~/.local/share/applications/mame-frontend.desktop ]; then
    print_status "Removing desktop entry..."
    rm ~/.local/share/applications/mame-frontend.desktop
    print_success "Desktop entry removed"
fi

# Remove application icons
print_status "Removing application icons..."

# Remove all icon sizes
for size in scalable 16x16 32x32 48x48 64x64 128x128 256x256; do
    if [ -f ~/.local/share/icons/hicolor/$size/apps/mame-frontend.png ]; then
        rm ~/.local/share/icons/hicolor/$size/apps/mame-frontend.png
        print_success "$size PNG icon removed"
    fi
    if [ -f ~/.local/share/icons/hicolor/$size/apps/mame-frontend.svg ]; then
        rm ~/.local/share/icons/hicolor/$size/apps/mame-frontend.svg
        print_success "$size SVG icon removed"
    fi
done

# Remove symlink
if [ -L ~/.local/bin/mame-frontend ]; then
    print_status "Removing command-line symlink..."
    rm ~/.local/bin/mame-frontend
    print_success "Symlink removed"
fi

# Remove installation directory
if [ -d "$INSTALL_DIR" ]; then
    if confirm "Do you want to remove the entire installation directory ($INSTALL_DIR)?"; then
        print_status "Removing installation directory..."
        rm -rf "$INSTALL_DIR"
        print_success "Installation directory removed"
    else
        print_status "Keeping installation directory"
    fi
fi

# Check for configuration files
CONFIG_DIRS=(
    ~/.config/mame-frontend
    ~/.local/share/mame-frontend
)

for config_dir in "${CONFIG_DIRS[@]}"; do
    if [ -d "$config_dir" ]; then
        if confirm "Do you want to remove configuration files in $config_dir?"; then
            print_status "Removing configuration files..."
            rm -rf "$config_dir"
            print_success "Configuration files removed"
        else
            print_status "Keeping configuration files"
        fi
    fi
done

print_success "Uninstallation completed!"
echo ""
print_warning "Note: System dependencies (Rust, MAME, etc.) were not removed"
print_status "To remove system dependencies, use your package manager:"
echo "  - Debian/Ubuntu: sudo apt remove mame"
echo "  - Fedora/RHEL: sudo dnf remove mame"
echo "  - Arch: sudo pacman -R mame"
echo "  - Rust: rustup self uninstall" 