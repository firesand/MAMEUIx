#!/bin/bash

# Test script for MAMEUIx installation scripts
# This script validates the installation scripts without running them

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "MAMEUIx Installation Scripts Test"
echo "======================================="
echo ""

# Test 1: Check if all scripts exist
print_status "Checking if all installation scripts exist..."

SCRIPTS=("install.sh" "install-debian.sh" "install-rpm.sh" "install-arch.sh")
MISSING_SCRIPTS=()

for script in "${SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        print_success "Found $script"
    else
        print_error "Missing $script"
        MISSING_SCRIPTS+=("$script")
    fi
done

if [ ${#MISSING_SCRIPTS[@]} -gt 0 ]; then
    print_error "Missing scripts: ${MISSING_SCRIPTS[*]}"
    exit 1
fi

# Test 2: Check if scripts are executable
print_status "Checking if scripts are executable..."

for script in "${SCRIPTS[@]}"; do
    if [ -x "$script" ]; then
        print_success "$script is executable"
    else
        print_error "$script is not executable"
        chmod +x "$script"
        print_status "Made $script executable"
    fi
done

# Test 3: Check script syntax
print_status "Checking script syntax..."

for script in "${SCRIPTS[@]}"; do
    if bash -n "$script" 2>/dev/null; then
        print_success "$script syntax is valid"
    else
        print_error "$script has syntax errors"
    fi
done

# Test 4: Test universal installer detection
print_status "Testing universal installer distribution detection..."

# Test detection logic
if [ -f /etc/os-release ]; then
    . /etc/os-release
    DISTRO=$ID
    print_success "Detected distribution: $DISTRO"
else
    print_warning "Could not detect distribution from /etc/os-release"
fi

# Test 5: Check if INSTALL.md exists
print_status "Checking if INSTALL.md exists..."

if [ -f "INSTALL.md" ]; then
    print_success "Found INSTALL.md"
else
    print_error "Missing INSTALL.md"
fi

# Test 6: Check if README.md mentions installation scripts
print_status "Checking if README.md mentions installation scripts..."

if grep -q "install.sh" README.md; then
    print_success "README.md mentions installation scripts"
else
    print_warning "README.md doesn't mention installation scripts"
fi

# Test 7: Check if assets and icons exist
print_status "Checking if assets and icons exist..."

if [ -f "assets/mameuix-icon.svg" ]; then
    print_success "✓ SVG icon found"
else
    print_warning "⚠ SVG icon not found: assets/mameuix-icon.svg"
fi

if [ -d "assets/icons" ]; then
    print_success "Found generated icons directory"
else
    print_warning "Generated icons directory not found (run ./generate-icons.sh)"
fi

# Test 8: Check if desktop file exists
print_status "Checking if desktop file exists..."

if [ -f "mameuix.desktop" ]; then
    print_success "✓ Desktop file found"
else
    print_warning "⚠ Desktop file not found: mameuix.desktop"
fi

# Test 9: Check if icon generation script exists
print_status "Checking if icon generation script exists..."

if [ -f "generate-icons.sh" ]; then
    print_success "Found icon generation script"
else
    print_error "Icon generation script not found"
fi

echo ""
print_success "All tests completed!"
echo ""
print_status "To run the actual installation:"
echo "  ./install.sh"
echo ""
print_status "For more information, see INSTALL.md" 