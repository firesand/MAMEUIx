#!/bin/bash

set -e

echo "Building Debian package..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if debuild is available
if ! command -v debuild &> /dev/null; then
    echo "Error: debuild not found. Please install devscripts:"
    echo "  sudo apt install devscripts"
    exit 1
fi

# Check if debhelper is available
if ! command -v dh &> /dev/null; then
    echo "Error: debhelper not found. Please install debhelper:"
    echo "  sudo apt install debhelper"
    exit 1
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf target/
rm -f ../mame-frontend_*.deb
rm -f ../mame-frontend_*.dsc
rm -f ../mame-frontend_*.tar.gz
rm -f ../mame-frontend_*.buildinfo
rm -f ../mame-frontend_*.changes

# Copy man page to debian directory
cp debian/mame-frontend.1 .

# Build the package
echo "Building package..."
debuild -b -us -uc

echo "Debian package built successfully!"
echo "Package files:"
ls -la ../mame-frontend_* 