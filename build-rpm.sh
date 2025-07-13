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
tar --exclude='.git' --exclude='target' --exclude='*.deb' --exclude='*.rpm' \
    --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
    --exclude='*.changes' --exclude='*.dsc' -czf mame-frontend-$VERSION.tar.gz .

# Create RPM build directory structure
RPM_BUILD_DIR="$HOME/rpmbuild"
mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Copy files to RPM build directories
cp mame-frontend-$VERSION.tar.gz "$RPM_BUILD_DIR/SOURCES/"
cp mame-frontend.spec "$RPM_BUILD_DIR/SPECS/"

# Copy man page
cp debian/mame-frontend.1 .

# Build the RPM
echo "Building RPM package..."
rpmbuild -ba "$RPM_BUILD_DIR/SPECS/mame-frontend.spec"

# Copy built packages to current directory
echo "Copying built packages..."
cp "$RPM_BUILD_DIR/RPMS/x86_64/mame-frontend-$VERSION"*.rpm .
cp "$RPM_BUILD_DIR/SRPMS/mame-frontend-$VERSION"*.src.rpm .

echo "RPM package built successfully!"
echo "Package files:"
ls -la mame-frontend-$VERSION*.rpm 