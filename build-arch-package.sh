#!/bin/bash

# MAMEUIX Arch Linux Package Builder
# Builds optimized package for Arch Linux and CachyOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if we're in the right directory
if [ ! -f "PKGBUILD" ]; then
    print_error "PKGBUILD not found. Please run this script from the project root directory."
    exit 1
fi

# Check if we're on Arch Linux or CachyOS
if ! command -v pacman &> /dev/null; then
    print_error "This script is designed for Arch Linux or CachyOS systems."
    exit 1
fi

print_status "Starting MAMEUIX v0.1.4 package build..."

# Clean previous builds
print_status "Cleaning previous build artifacts..."
rm -rf pkg/ src/ *.pkg.tar.zst *.pkg.tar.zst.sig 2>/dev/null || true

# Update system packages (optional)
read -p "Update system packages before building? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Updating system packages..."
    sudo pacman -Syu --noconfirm
fi

# Install build dependencies
print_status "Installing build dependencies..."
sudo pacman -S --needed --noconfirm \
    rust \
    pkgconf \
    zstd \
    git \
    cmake \
    ninja \
    base-devel

# Verify Rust installation
if ! command -v cargo &> /dev/null; then
    print_error "Rust/Cargo not found. Please install Rust first."
    exit 1
fi

print_status "Rust version: $(rustc --version)"
print_status "Cargo version: $(cargo --version)"

# Build the package
print_status "Building MAMEUIX package..."
if makepkg -sf --noconfirm; then
    print_success "Package built successfully!"
else
    print_error "Package build failed!"
    exit 1
fi

# Find the built package
PACKAGE_FILE=$(ls -t mameuix-*.pkg.tar.zst 2>/dev/null | head -n1)

if [ -z "$PACKAGE_FILE" ]; then
    print_error "No package file found after build!"
    exit 1
fi

print_success "MAMEUIX v0.1.4 package build completed successfully!"
print_status "Package file: $PACKAGE_FILE"
print_status "Size: $(du -h "$PACKAGE_FILE" | cut -f1)"

# Show package information
print_status "Package information:"
tar -tf "$PACKAGE_FILE" | head -20
echo "... (showing first 20 files)"

# Optional: Install the package
read -p "Install the package now? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "Installing MAMEUIX package..."
    sudo pacman -U --noconfirm "$PACKAGE_FILE"
    print_success "MAMEUIX installed successfully!"
    
    # Check if MAME is installed
    if ! command -v mame &> /dev/null; then
        print_warning "MAME is not installed. You may want to install it:"
        print_status "sudo pacman -S mame"
    fi
fi

# Optional: Validate package
read -p "Validate package with namcap? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if command -v namcap &> /dev/null; then
        print_status "Validating package..."
        namcap "$PACKAGE_FILE"
    else
        print_warning "namcap not installed. Install it with: sudo pacman -S namcap"
    fi
fi

print_success "Build process completed!"
print_status "Package location: $(pwd)/$PACKAGE_FILE" 