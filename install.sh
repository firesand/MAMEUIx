#!/bin/bash

# Universal MAME Frontend Installation Script
# This script detects your Linux distribution and runs the appropriate installation script

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

# Function to detect Linux distribution
detect_distribution() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        VERSION=$VERSION_ID
    elif [ -f /etc/redhat-release ]; then
        DISTRO="rhel"
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
    elif [ -f /etc/arch-release ]; then
        DISTRO="arch"
    else
        print_error "Could not detect Linux distribution"
        exit 1
    fi
    
    print_status "Detected distribution: $DISTRO $VERSION"
}

# Function to determine which script to run
get_install_script() {
    case $DISTRO in
        "ubuntu"|"debian"|"linuxmint"|"pop"|"elementary"|"kali"|"parrot")
            echo "install-debian.sh"
            ;;
        "fedora"|"rhel"|"centos"|"rocky"|"almalinux"|"oracle"|"amazon")
            echo "install-rpm.sh"
            ;;
        "arch"|"manjaro"|"endeavouros"|"garuda"|"artix")
            echo "install-arch.sh"
            ;;
        *)
            print_error "Unsupported distribution: $DISTRO"
            print_status "Supported distributions:"
            echo "  - Debian-based: Ubuntu, Debian, Linux Mint, Pop!_OS, Elementary OS"
            echo "  - RPM-based: Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux"
            echo "  - Arch-based: Arch Linux, Manjaro, EndeavourOS, Garuda Linux"
            exit 1
            ;;
    esac
}

# Main installation function
main() {
    print_status "MAME Frontend Universal Installer"
    echo "========================================"
    echo ""
    
    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        print_error "This script should not be run as root"
        exit 1
    fi
    
    # Check if we're on Linux
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        print_error "This script is designed for Linux systems only"
        exit 1
    fi
    
    # Detect distribution
    detect_distribution
    
    # Get the appropriate installation script
    INSTALL_SCRIPT=$(get_install_script)
    
    # Check if the script exists
    if [ ! -f "$INSTALL_SCRIPT" ]; then
        print_error "Installation script not found: $INSTALL_SCRIPT"
        print_status "Please ensure all installation scripts are in the same directory"
        exit 1
    fi
    
    # Make the script executable
    chmod +x "$INSTALL_SCRIPT"
    
    print_status "Running $INSTALL_SCRIPT for $DISTRO..."
    echo ""
    
    # Run the appropriate installation script
    ./"$INSTALL_SCRIPT"
}

# Run the main function
main "$@" 