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

# Arch prerequisite: use a fully upgraded system and its existing package database.
print_warning "Before installing, update your Arch system with: sudo pacman -Syu"
print_status "Using the existing package database; no system upgrade is performed here."
if PENDING_UPGRADES=$(pacman -Qu); then
    if [[ -n "$PENDING_UPGRADES" ]]; then
        print_error "Pending system upgrades detected. Run sudo pacman -Syu, then retry."
        exit 1
    fi
else
    PACMAN_STATUS=$?
    # pacman returns 1 when the query has no matching packages.
    if [[ $PACMAN_STATUS -ne 1 ]]; then
        print_error "Could not check pending system upgrades."
        exit "$PACMAN_STATUS"
    fi
fi

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

# Rust installation: preserve existing distro or rustup toolchains.
if command -v rustc >/dev/null 2>&1 || command -v cargo >/dev/null 2>&1; then
    print_status "Using the existing Rust toolchain."
elif command -v rustup >/dev/null 2>&1; then
    print_error "rustup is installed, but Rust/Cargo are unavailable. Configure your rustup toolchain and PATH, then retry."
    exit 1
else
    print_status "Installing Rust and Cargo from the Arch repositories..."
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
MIN_RUST_VERSION="1.88.0"
for RUST_TOOL in rustc cargo; do
    if ! command -v "$RUST_TOOL" >/dev/null 2>&1; then
        print_error "$RUST_TOOL is missing. Install both Rust and Cargo using the same toolchain manager."
        exit 1
    fi
    RUST_TOOL_OUTPUT=$("$RUST_TOOL" --version)
    RUST_TOOL_VERSION=${RUST_TOOL_OUTPUT#* }
    RUST_TOOL_VERSION=${RUST_TOOL_VERSION%% *}
    RUST_TOOL_VERSION=${RUST_TOOL_VERSION%%-*}
    if [[ ! "$RUST_TOOL_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] ||
        [[ "$(printf '%s\n%s\n' "$MIN_RUST_VERSION" "$RUST_TOOL_VERSION" | sort -V | head -n1)" != "$MIN_RUST_VERSION" ]]; then
        print_error "$RUST_TOOL $MIN_RUST_VERSION or newer is required (found: $RUST_TOOL_OUTPUT). Update it with your existing toolchain manager."
        exit 1
    fi
    print_status "$RUST_TOOL_OUTPUT"
done

# Select the source checkout without changing an existing checkout's Git state.
is_mameuix_checkout() {
    [[ -f "$1/Cargo.toml" ]] && grep -Eq '^name[[:space:]]*=[[:space:]]*"mameuix"[[:space:]]*$' "$1/Cargo.toml"
}
if is_mameuix_checkout .; then
    print_status "Using the current MAMEUIx checkout."
elif is_mameuix_checkout MAMEUIx; then
    print_status "Using the existing MAMEUIx checkout."
    cd MAMEUIx
elif [[ -e MAMEUIx ]]; then
    print_error "MAMEUIx already exists but is not a MAMEUIx source checkout. Choose another working directory."
    exit 1
else
    print_status "Cloning MAMEUIx repository..."
    git clone https://github.com/firesand/MAMEUIx.git MAMEUIx
    cd MAMEUIx
fi

# Build the project
print_status "Building MAMEUIx (this may take several minutes)..."
cargo build --release --locked

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
