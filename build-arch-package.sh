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

VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
print_status "Starting MAMEUIx v$VERSION package build..."

# Clean previous builds
print_status "Cleaning previous build artifacts..."
rm -f mameuix-"$VERSION"*.pkg.tar.zst mameuix-"$VERSION"*.pkg.tar.zst.sig 2>/dev/null || true

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

BUILD_TMP_DIR=$(mktemp -d)
trap 'rm -rf "$BUILD_TMP_DIR"' EXIT

print_status "Creating source tarball..."
SOURCE_ROOT="mameuix-$VERSION"
SOURCE_ARCHIVE="mameuix-$VERSION.tar.gz"
mkdir -p "$BUILD_TMP_DIR/source/$SOURCE_ROOT"
tar --exclude='.git' --exclude='.cargo' --exclude='target' --exclude='pkg' \
    --exclude='cfg' --exclude='nvram' --exclude='diff' \
    --exclude='pS_CatVer_277' --exclude='MAMEUI-inifiles' \
    --exclude='memory.md' --exclude='*.log' --exclude='*.tmp' --exclude='*.temp' \
    --exclude='*.deb' --exclude='*.rpm' --exclude='*.pkg.tar.zst' \
    --exclude='*.pkg.tar.zst.sig' \
    --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
    --exclude='*.changes' --exclude='*.dsc' \
    -cf - . | tar -C "$BUILD_TMP_DIR/source/$SOURCE_ROOT" -xf -
tar -C "$BUILD_TMP_DIR/source" -czf "$SOURCE_ARCHIVE" "$SOURCE_ROOT"

ARCH_BUILD_DIR="$BUILD_TMP_DIR/arch"
mkdir -p "$ARCH_BUILD_DIR"
cp PKGBUILD .SRCINFO "$SOURCE_ARCHIVE" "$ARCH_BUILD_DIR/"

# Build the package
print_status "Building MAMEUIX package..."
if (cd "$ARCH_BUILD_DIR" && makepkg -sf --noconfirm); then
    cp "$ARCH_BUILD_DIR"/mameuix-"$VERSION"*.pkg.tar.zst .
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

print_success "MAMEUIx v$VERSION package build completed successfully!"
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
