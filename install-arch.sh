#!/bin/bash

# MAMEUIx Installation Script for Arch Linux
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

print_status "Starting MAMEUIx installation for Arch Linux..."

# Update package list
print_status "Updating package list..."
sudo pacman -Sy

# Install system dependencies
print_status "Installing system dependencies..."
sudo pacman -S --needed --noconfirm \
    base-devel \
    curl \
    pkg-config \
    openssl \
    gtk3 \
    webkit2gtk \
    libappindicator-gtk3 \
    librsvg \
    gstreamer \
    gst-plugins-base \
    gst-plugins-good \
    gst-plugins-bad \
    gst-plugins-ugly \
    gst-libav \
    cmake \
    git

# Rust installation
if command -v rustc &> /dev/null; then
    echo "[INFO] Rust is already installed. Skipping update."
    # rustup update  # Skipped for system Rust
else
    echo "[INFO] Installing Rust..."
    sudo pacman -S --needed --noconfirm rust
fi

# MAME installation
if command -v mame &> /dev/null; then
    MAME_VERSION=$(mame --version | head -n1 | grep -oE '[0-9]+(\.[0-9]+)*' | head -n1)
    REQUIRED_VERSION=0.200
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$MAME_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        echo "[WARNING] MAME version ($MAME_VERSION) is older than required ($REQUIRED_VERSION). Some features may not work."
    else
        echo "[INFO] MAME is already installed (version $MAME_VERSION)."
    fi
else
    echo "[INFO] Installing MAME..."
    sudo pacman -S --needed --noconfirm mame
fi

# Verify Rust installation
print_status "Verifying Rust installation..."
rustc --version
cargo --version

# Clone the repository if not already present
if [ ! -d "MAMEUIx" ]; then
    print_status "Cloning MAMEUIx repository..."
    git clone https://github.com/firesand/MAMEUIx.git
    cd MAMEUIx
else
    print_status "Repository already exists. Updating..."
    cd MAMEUIx
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
