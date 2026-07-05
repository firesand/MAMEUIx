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

BUILD_TMP_DIR=$(mktemp -d)
trap 'rm -rf "$BUILD_TMP_DIR"' EXIT

# Create source tarball
echo "Creating source tarball..."
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

# Run makepkg outside the project root so it cannot collide with the Rust src/ directory.
ARCH_BUILD_DIR="$BUILD_TMP_DIR/arch"
mkdir -p "$ARCH_BUILD_DIR"
cp PKGBUILD .SRCINFO "$SOURCE_ARCHIVE" "$ARCH_BUILD_DIR/"

# Build the package
echo "Building Arch package..."
(cd "$ARCH_BUILD_DIR" && makepkg -f)
cp "$ARCH_BUILD_DIR"/mameuix-"$VERSION"*.pkg.tar.zst .

echo "Arch Linux package built successfully!"
echo "Package files:"
ls -la mameuix-$VERSION*.pkg.tar.zst
