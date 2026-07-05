#!/bin/bash

set -e

echo "Building RPM package..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check if rpmbuild is available
if ! command -v rpmbuild &> /dev/null; then
    echo "Error: rpmbuild not found. Please install rpm-build:"
    echo "  sudo dnf install rpm-build (Fedora/RHEL)"
    echo "  sudo yum install rpm-build (CentOS/RHEL)"
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
echo "Building version: $VERSION"

# Create source tarball
echo "Creating source tarball..."
SOURCE_ROOT="mameuix-$VERSION"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
mkdir -p "$TMP_DIR/$SOURCE_ROOT"
tar --exclude='.git' --exclude='.cargo' --exclude='target' --exclude='pkg' \
    --exclude='cfg' --exclude='nvram' --exclude='diff' \
    --exclude='pS_CatVer_277' --exclude='MAMEUI-inifiles' \
    --exclude='memory.md' --exclude='*.log' --exclude='*.tmp' --exclude='*.temp' \
    --exclude='*.deb' --exclude='*.rpm' --exclude='*.pkg.tar.zst' \
    --exclude='*.pkg.tar.zst.sig' \
    --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
    --exclude='*.changes' --exclude='*.dsc' \
    -cf - . | tar -C "$TMP_DIR/$SOURCE_ROOT" -xf -
tar -C "$TMP_DIR" -czf "mameuix-$VERSION.tar.gz" "$SOURCE_ROOT"

# Create RPM build directory structure
RPM_BUILD_DIR="$HOME/rpmbuild"
mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy files to RPM build directories
cp mameuix-$VERSION.tar.gz "$RPM_BUILD_DIR/SOURCES/"
cp mameuix.spec "$RPM_BUILD_DIR/SPECS/"

# Build the RPM
echo "Building RPM package..."
rpmbuild -ba "$RPM_BUILD_DIR/SPECS/mameuix.spec"

# Copy built packages to current directory
echo "Copying built packages..."
cp "$RPM_BUILD_DIR/RPMS/x86_64/mameuix-$VERSION"*.rpm .
cp "$RPM_BUILD_DIR/SRPMS/mameuix-$VERSION"*.src.rpm .

echo "RPM package built successfully!"
echo "Package files:"
ls -la mameuix-$VERSION*.rpm 
