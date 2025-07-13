#!/bin/bash

# Icon Generation Script for MAMEUIx
# This script generates different icon sizes from the SVG source

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

print_status "Generating MAMEUIx icons..."

# Check if we have the source SVG
if [ ! -f "assets/mameuix-icon.svg" ]; then
    print_error "Source SVG icon not found: assets/mameuix-icon.svg"
    print_status "Please create assets/mameuix-icon.svg first"
    exit 1
fi

# Create icon directories
mkdir -p assets/icons/16x16
mkdir -p assets/icons/32x32
mkdir -p assets/icons/48x48
mkdir -p assets/icons/64x64
mkdir -p assets/icons/128x128
mkdir -p assets/icons/256x256
mkdir -p assets/icons/scalable

# Check if ImageMagick is available
if command -v convert &> /dev/null; then
    print_status "Using ImageMagick to generate icons..."
    
    # Generate different sizes
    convert assets/mameuix-icon.svg -resize 16x16 assets/icons/16x16/mameuix.png
    convert assets/mameuix-icon.svg -resize 32x32 assets/icons/32x32/mameuix.png
    convert assets/mameuix-icon.svg -resize 48x48 assets/icons/48x48/mameuix.png
    convert assets/mameuix-icon.svg -resize 64x64 assets/icons/64x64/mameuix.png
    convert assets/mameuix-icon.svg -resize 128x128 assets/icons/128x128/mameuix.png
    convert assets/mameuix-icon.svg -resize 256x256 assets/icons/256x256/mameuix.png
    
    # Copy SVG to scalable directory
    cp assets/mameuix-icon.svg assets/icons/scalable/mameuix.svg
    
    print_success "Icons generated successfully!"
    
elif command -v rsvg-convert &> /dev/null; then
    print_status "Using rsvg-convert to generate icons..."
    
    # Generate different sizes
    rsvg-convert -w 16 -h 16 assets/mameuix-icon.svg -o assets/icons/16x16/mameuix.png
    rsvg-convert -w 32 -h 32 assets/mameuix-icon.svg -o assets/icons/32x32/mameuix.png
    rsvg-convert -w 48 -h 48 assets/mameuix-icon.svg -o assets/icons/48x48/mameuix.png
    rsvg-convert -w 64 -h 64 assets/mameuix-icon.svg -o assets/icons/64x64/mameuix.png
    rsvg-convert -w 128 -h 128 assets/mameuix-icon.svg -o assets/icons/128x128/mameuix.png
    rsvg-convert -w 256 -h 256 assets/mameuix-icon.svg -o assets/icons/256x256/mameuix.png
    
    # Copy SVG to scalable directory
    cp assets/mameuix-icon.svg assets/icons/scalable/mameuix.svg
    
    print_success "Icons generated successfully!"
    
else
    print_warning "Neither ImageMagick nor rsvg-convert found"
    print_status "Copying existing icons..."
    
    # Just copy the existing files
    if [ -f "assets/mameuix-icon.png" ]; then
        cp assets/mameuix-icon.png assets/icons/256x256/mameuix.png
        print_success "Copied existing PNG icon"
    fi
    
    cp assets/mameuix-icon.svg assets/icons/scalable/mameuix.svg
    print_success "Copied SVG icon"
    
    print_warning "To generate all icon sizes, install ImageMagick or librsvg2-bin:"
    echo "  - Debian/Ubuntu: sudo apt install imagemagick"
    echo "  - Fedora/RHEL: sudo dnf install ImageMagick"
    echo "  - Arch: sudo pacman -S imagemagick"
fi

print_status "Icon generation complete!"
print_status "Icons are available in: assets/icons/" 