#!/bin/bash

# MAMEUIx Installation Script for Debian/Ubuntu
# This script installs Rust, system dependencies, and builds the MAME frontend

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

print_status "Starting MAMEUIx installation for Debian/Ubuntu..."

# Update package list
print_status "Updating package list..."
sudo apt update

# Install system dependencies
print_status "Installing system dependencies..."
sudo apt install -y \
    build-essential \
    curl \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-tools \
    gstreamer1.0-x \
    gstreamer1.0-alsa \
    gstreamer1.0-gl \
    gstreamer1.0-gtk3 \
    gstreamer1.0-qt5 \
    gstreamer1.0-pulseaudio \
    cmake \
    git

# Install MAME
print_status "Installing MAME..."
sudo apt install -y mame

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
if [ ! -d "mameuix" ]; then
    print_status "Cloning MAMEUIx repository..."
    git clone https://github.com/firesand/MAMEUIx.git
    cd mameuix
else
    print_status "Repository already exists. Updating..."
    cd mameuix
    git pull origin main
fi

# Build the project
print_status "Building MAMEUIx (this may take several minutes)..."
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
if [ -f "assets/mameuix-icon.svg" ]; then
    cp assets/mameuix-icon.svg ~/.local/share/icons/hicolor/scalable/apps/mameuix.svg
    print_success "SVG icon installed"
fi

# Copy generated icons if available
if [ -d "assets/icons" ]; then
    # Copy all generated icon sizes
    for size in 16x16 32x32 48x48 64x64 128x128 256x256; do
        if [ -f "assets/icons/$size/mameuix.png" ]; then
            cp "assets/icons/$size/mameuix.png" ~/.local/share/icons/hicolor/$size/apps/
            print_success "$size icon installed"
        fi
    done
else
    # Fallback to single PNG icon
    if [ -f "assets/mameuix-icon.png" ]; then
        cp assets/mameuix-icon.png ~/.local/share/icons/hicolor/256x256/apps/mameuix.png
        print_success "PNG icon installed"
    fi
fi

# Create desktop entry
print_status "Creating desktop entry..."
mkdir -p ~/.local/share/applications

cat > ~/.local/share/applications/mameuix.desktop << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=MAMEUIx
Comment=Modern frontend for MAME arcade emulator
Exec=$PWD/target/release/mameuix
Icon=mameuix
Terminal=false
Categories=Game;Emulator;
Keywords=mame;arcade;emulator;game;
StartupNotify=true
EOF

# Make the binary executable
chmod +x target/release/mameuix

# Create a symlink in ~/.local/bin for easy access
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/mameuix" ~/.local/bin/mameuix

print_success "Installation completed successfully!"
print_status "You can now run the application with:"
echo "  mameuix"
echo "  or"
echo "  $PWD/target/release/mameuix"
echo ""
print_status "The application should also appear in your application menu."
echo ""
print_warning "Don't forget to configure your ROM directories in the application settings!" 