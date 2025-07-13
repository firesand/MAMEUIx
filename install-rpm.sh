#!/bin/bash

# MAME Frontend Installation Script for RPM-based distributions
# This script installs Rust, system dependencies, and builds the MAME frontend
# Compatible with Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux, etc.

set -e  # Exit on any error

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

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   print_error "This script should not be run as root"
   exit 1
fi

print_status "Starting MAME Frontend installation for RPM-based distributions..."

# Detect distribution
if command -v dnf &> /dev/null; then
    PKG_MANAGER="dnf"
elif command -v yum &> /dev/null; then
    PKG_MANAGER="yum"
else
    print_error "No supported package manager found (dnf or yum)"
    exit 1
fi

print_status "Using package manager: $PKG_MANAGER"

# Update package list
print_status "Updating package list..."
sudo $PKG_MANAGER update -y

# Install system dependencies
print_status "Installing system dependencies..."
sudo $PKG_MANAGER install -y \
    gcc \
    gcc-c++ \
    curl \
    pkg-config \
    openssl-devel \
    gtk3-devel \
    webkit2gtk3-devel \
    libappindicator-gtk3-devel \
    librsvg2-devel \
    gstreamer1-devel \
    gstreamer1-plugins-base-devel \
    gstreamer1-plugins-bad-free-devel \
    gstreamer1-plugins-good \
    gstreamer1-plugins-bad-free \
    gstreamer1-plugins-ugly-free \
    gstreamer1-libav \
    gstreamer1-plugins-base \
    gstreamer1-plugins-extra \
    cmake \
    git \
    make

# Install MAME
print_status "Installing MAME..."
sudo $PKG_MANAGER install -y mame

# Check if Rust is already installed
if command -v rustc &> /dev/null; then
    print_status "Rust is already installed. Updating..."
    rustup update
else
    print_status "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Verify Rust installation
print_status "Verifying Rust installation..."
rustc --version
cargo --version

# Clone the repository if not already present
if [ ! -d "mame-frontend" ]; then
    print_status "Cloning MAME Frontend repository..."
    git clone https://github.com/yourusername/mame-frontend.git
    cd mame-frontend
else
    print_status "Repository already exists. Updating..."
    cd mame-frontend
    git pull origin main
fi

# Build the project
print_status "Building MAME Frontend (this may take several minutes)..."
cargo build --release

# Install application icons
print_status "Installing application icons..."

# Create icon directories
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
mkdir -p ~/.local/share/icons/hicolor/16x16/apps
mkdir -p ~/.local/share/icons/hicolor/32x32/apps
mkdir -p ~/.local/share/icons/hicolor/48x48/apps
mkdir -p ~/.local/share/icons/hicolor/64x64/apps
mkdir -p ~/.local/share/icons/hicolor/128x128/apps
mkdir -p ~/.local/share/icons/hicolor/256x256/apps

# Copy SVG icon (scalable)
if [ -f "assets/mame-frontend-icon.svg" ]; then
    cp assets/mame-frontend-icon.svg ~/.local/share/icons/hicolor/scalable/apps/mame-frontend.svg
    print_success "SVG icon installed"
fi

# Copy generated icons if available
if [ -d "assets/icons" ]; then
    # Copy all generated icon sizes
    for size in 16x16 32x32 48x48 64x64 128x128 256x256; do
        if [ -f "assets/icons/$size/mame-frontend.png" ]; then
            cp "assets/icons/$size/mame-frontend.png" ~/.local/share/icons/hicolor/$size/apps/
            print_success "$size icon installed"
        fi
    done
else
    # Fallback to single PNG icon
    if [ -f "assets/mame-frontend-icon.png" ]; then
        cp assets/mame-frontend-icon.png ~/.local/share/icons/hicolor/256x256/apps/mame-frontend.png
        print_success "PNG icon installed"
    fi
fi

# Create desktop entry
print_status "Creating desktop entry..."
mkdir -p ~/.local/share/applications

cat > ~/.local/share/applications/mame-frontend.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=MAME Frontend
Comment=Modern frontend for MAME arcade emulator
Exec=$PWD/target/release/mame-frontend
Icon=mame-frontend
Terminal=false
Categories=Game;Emulator;
Keywords=mame;arcade;emulator;game;
StartupNotify=true
EOF

# Make the binary executable
chmod +x target/release/mame-frontend

# Create a symlink in ~/.local/bin for easy access
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/mame-frontend" ~/.local/bin/mame-frontend

print_success "Installation completed successfully!"
print_status "You can now run the application with:"
echo "  mame-frontend"
echo "  or"
echo "  $PWD/target/release/mame-frontend"
echo ""
print_status "The application should also appear in your application menu."
echo ""
print_warning "Don't forget to configure your ROM directories in the application settings!" 