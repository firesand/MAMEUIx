#!/bin/bash

# MAMEUIx Uninstall Script
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

print_status "MAMEUIx Uninstaller"
echo "=============================="
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    print_error "This script should not be run as root"
    exit 1
fi

# Find the installation directory
INSTALL_DIR=""
if [ -L ~/.local/bin/mameuix ]; then
    INSTALL_DIR=$(readlink -f ~/.local/bin/mameuix)
    INSTALL_DIR=$(dirname "$INSTALL_DIR")
    print_status "Found installation at: $INSTALL_DIR"
elif [ -d ~/mameuix ]; then
    INSTALL_DIR=~/mameuix
    print_status "Found installation at: $INSTALL_DIR"
else
    print_warning "Could not find MAMEUIx installation"
    print_status "Checking common locations..."
    
    # Check common build directories
    for dir in ~/mameuix ~/Downloads/mameuix ~/src/mameuix; do
        if [ -d "$dir" ] && [ -f "$dir/target/release/mameuix" ]; then
            INSTALL_DIR="$dir"
            print_status "Found installation at: $INSTALL_DIR"
            break
        fi
    done
fi

if [ -z "$INSTALL_DIR" ]; then
    print_error "MAMEUIx installation not found"
    print_status "Please manually remove the installation directory"
    exit 1
fi

# Confirm uninstallation
if ! confirm "Are you sure you want to uninstall MAMEUIx?"; then
    print_status "Uninstallation cancelled"
    exit 0
fi

# Remove desktop entry
if [ -f ~/.local/share/applications/mameuix.desktop ]; then
    print_status "Removing desktop entry..."
    rm ~/.local/share/applications/mameuix.desktop
    print_success "Desktop entry removed"
fi

# Remove application icons
print_status "Removing application icons..."

# Remove all icon sizes
for size in 16x16 32x32 48x48 64x64 128x128 256x256; do
    if [ -f ~/.local/share/icons/hicolor/$size/apps/mameuix.png ]; then
        rm ~/.local/share/icons/hicolor/$size/apps/mameuix.png
        print_success "$size icon removed"
    fi
    if [ -f ~/.local/share/icons/hicolor/$size/apps/mameuix.svg ]; then
        rm ~/.local/share/icons/hicolor/$size/apps/mameuix.svg
        print_success "$size SVG icon removed"
    fi
done

# Remove symlink
if [ -L ~/.local/bin/mameuix ]; then
    print_status "Removing symlink..."
    rm ~/.local/bin/mameuix
    print_success "Symlink removed"
fi

# Remove installation directory
if [ -d "$INSTALL_DIR" ]; then
    if confirm "Remove installation directory ($INSTALL_DIR)?"; then
        print_status "Removing installation directory..."
        rm -rf "$INSTALL_DIR"
        print_success "Installation directory removed"
    else
        print_status "Installation directory preserved"
    fi
fi

# Ask about configuration files
print_status "Configuration files are stored in:"
echo "  ~/.config/mameuix"
echo "  ~/.local/share/mameuix"

# Check for configuration files
CONFIG_DIRS=(
    ~/.config/mameuix
    ~/.local/share/mameuix
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