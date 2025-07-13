#!/bin/bash

set -e

echo "Building Arch Linux package..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if makepkg is available
if ! command -v makepkg &> /dev/null; then
    echo "Error: makepkg not found. Please install pacman:"
    echo "  This script should be run on an Arch Linux system or in an Arch container."
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
echo "Building version: $VERSION"

# Create source tarball
echo "Creating source tarball..."
tar --exclude='.git' --exclude='target' --exclude='*.deb' --exclude='*.rpm' \
    --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
    --exclude='*.changes' --exclude='*.dsc' -czf mameuix-$VERSION.tar.gz .

# Copy man page
cp debian/mameuix.1 .

# Build the package
echo "Building Arch package..."
makepkg -f

echo "Arch Linux package built successfully!"
echo "Package files:"
ls -la mameuix-$VERSION*.pkg.tar.zst 