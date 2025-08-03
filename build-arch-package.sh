#!/bin/bash

# MAMEUIX Arch/CachyOS Package Builder
# Builds and validates the MAMEUIX package for Arch Linux and CachyOS

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
    print_error "PKGBUILD not found. Please run this script from the MAMEUIX project directory."
    exit 1
fi

print_status "Starting MAMEUIX v0.1.4 package build..."

# Clean previous builds
print_status "Cleaning previous builds..."
rm -rf pkg/ src/ *.pkg.tar.zst *.pkg.tar.zst.sig 2>/dev/null || true

# Update .SRCINFO
print_status "Updating .SRCINFO..."
makepkg --printsrcinfo > .SRCINFO

# Validate PKGBUILD
print_status "Validating PKGBUILD..."
namcap PKGBUILD || print_warning "PKGBUILD validation completed with warnings"

# Build package
print_status "Building package..."
if makepkg --syncdeps --noconfirm; then
    print_success "Package built successfully!"
else
    print_error "Package build failed!"
    exit 1
fi

# Find the built package
PACKAGE_FILE=$(ls -t *.pkg.tar.zst 2>/dev/null | head -1)
if [ -z "$PACKAGE_FILE" ]; then
    print_error "No package file found after build!"
    exit 1
fi

print_success "Package file: $PACKAGE_FILE"

# Validate package
print_status "Validating package..."
if namcap "$PACKAGE_FILE"; then
    print_success "Package validation completed!"
else
    print_warning "Package validation completed with warnings"
fi

# Show package info
print_status "Package information:"
pacman -Qip "$PACKAGE_FILE"

# Show package contents
print_status "Package contents:"
tar -tf "$PACKAGE_FILE" | head -20
echo "... (showing first 20 files)"

print_success "MAMEUIX v0.1.4 package build completed successfully!"
print_status "Package file: $PACKAGE_FILE"
print_status "Size: $(du -h "$PACKAGE_FILE" | cut -f1)"

echo ""
print_status "Next steps:"
echo "1. Test the package: sudo pacman -U $PACKAGE_FILE"
echo "2. Upload to AUR: git push origin main"
echo "3. For CachyOS: Submit to CachyOS package repository" 