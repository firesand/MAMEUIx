#!/bin/bash

set -e

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

# Function to detect distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    else
        echo "unknown"
    fi
}

# Function to check dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("rust")
    fi
    
    # Check for pkg-config
    if ! command -v pkg-config &> /dev/null; then
        missing_deps+=("pkg-config")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        print_status "Please install the missing dependencies and try again."
        exit 1
    fi
}

# Function to build Debian package
build_deb() {
    print_status "Building Debian package..."
    
    if ! command -v debuild &> /dev/null; then
        print_error "debuild not found. Please install devscripts:"
        echo "  sudo apt install devscripts"
        return 1
    fi
    
    if ! command -v dh &> /dev/null; then
        print_error "debhelper not found. Please install debhelper:"
        echo "  sudo apt install debhelper"
        return 1
    fi
    
    # Clean previous builds
    rm -rf target/
    rm -f ../mame-frontend_*.deb ../mame-frontend_*.dsc ../mame-frontend_*.tar.gz \
          ../mame-frontend_*.buildinfo ../mame-frontend_*.changes 2>/dev/null || true
    
    # Copy man page
    cp debian/mame-frontend.1 .
    
    # Build the package
    debuild -b -us -uc
    
    print_success "Debian package built successfully!"
    ls -la ../mame-frontend_*.deb 2>/dev/null || print_warning "No .deb files found"
}

# Function to build RPM package
build_rpm() {
    print_status "Building RPM package..."
    
    if ! command -v rpmbuild &> /dev/null; then
        print_error "rpmbuild not found. Please install rpm-build:"
        echo "  sudo dnf install rpm-build (Fedora/RHEL)"
        echo "  sudo yum install rpm-build (CentOS/RHEL)"
        return 1
    fi
    
    # Get version
    VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
    
    # Create source tarball
    tar --exclude='.git' --exclude='target' --exclude='*.deb' --exclude='*.rpm' \
        --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
        --exclude='*.changes' --exclude='*.dsc' -czf mame-frontend-$VERSION.tar.gz .
    
    # Create RPM build directory structure
    RPM_BUILD_DIR="$HOME/rpmbuild"
    mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
    
    # Copy files
    cp mame-frontend-$VERSION.tar.gz "$RPM_BUILD_DIR/SOURCES/"
    cp mame-frontend.spec "$RPM_BUILD_DIR/SPECS/"
    cp debian/mame-frontend.1 .
    
    # Build the RPM
    rpmbuild -ba "$RPM_BUILD_DIR/SPECS/mame-frontend.spec"
    
    # Copy built packages
    cp "$RPM_BUILD_DIR/RPMS/x86_64/mame-frontend-$VERSION"*.rpm . 2>/dev/null || true
    cp "$RPM_BUILD_DIR/SRPMS/mame-frontend-$VERSION"*.src.rpm . 2>/dev/null || true
    
    print_success "RPM package built successfully!"
    ls -la mame-frontend-$VERSION*.rpm 2>/dev/null || print_warning "No .rpm files found"
}

# Function to build Arch package
build_arch() {
    print_status "Building Arch Linux package..."
    
    if ! command -v makepkg &> /dev/null; then
        print_error "makepkg not found. This script should be run on an Arch Linux system."
        return 1
    fi
    
    # Get version
    VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
    
    # Create source tarball
    tar --exclude='.git' --exclude='target' --exclude='*.deb' --exclude='*.rpm' \
        --exclude='*.tar.gz' --exclude='*.tar.xz' --exclude='*.buildinfo' \
        --exclude='*.changes' --exclude='*.dsc' -czf mame-frontend-$VERSION.tar.gz .
    
    # Copy man page
    cp debian/mame-frontend.1 .
    
    # Build the package
    makepkg -f
    
    print_success "Arch Linux package built successfully!"
    ls -la mame-frontend-$VERSION*.pkg.tar.zst 2>/dev/null || print_warning "No .pkg.tar.zst files found"
}

# Main script
main() {
    print_status "MAME Frontend Package Builder"
    print_status "============================="
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the project root."
        exit 1
    fi
    
    # Check dependencies
    check_dependencies
    
    # Parse command line arguments
    BUILD_DEB=false
    BUILD_RPM=false
    BUILD_ARCH=false
    
    if [ $# -eq 0 ]; then
        # No arguments, detect and build for current distro
        DISTRO=$(detect_distro)
        print_status "No package type specified. Detected distribution: $DISTRO"
        
        case $DISTRO in
            "ubuntu"|"debian"|"linuxmint"|"pop")
                BUILD_DEB=true
                ;;
            "fedora"|"rhel"|"centos"|"rocky"|"alma")
                BUILD_RPM=true
                ;;
            "arch"|"manjaro"|"endeavouros")
                BUILD_ARCH=true
                ;;
            *)
                print_warning "Unknown distribution: $DISTRO"
                print_status "Building all package types..."
                BUILD_DEB=true
                BUILD_RPM=true
                BUILD_ARCH=true
                ;;
        esac
    else
        # Build specified package types
        for arg in "$@"; do
            case $arg in
                "deb"|"debian")
                    BUILD_DEB=true
                    ;;
                "rpm"|"redhat"|"fedora")
                    BUILD_RPM=true
                    ;;
                "arch"|"archlinux")
                    BUILD_ARCH=true
                    ;;
                "all")
                    BUILD_DEB=true
                    BUILD_RPM=true
                    BUILD_ARCH=true
                    ;;
                "--help"|"-h"|"help")
                    print_status "MAME Frontend Package Builder"
                    print_status "Usage: $0 [package_type]"
                    print_status ""
                    print_status "Package types:"
                    print_status "  deb, debian    - Build Debian package (.deb)"
                    print_status "  rpm, redhat    - Build RPM package (.rpm)"
                    print_status "  arch, archlinux - Build Arch package (.pkg.tar.zst)"
                    print_status "  all            - Build all package types"
                    print_status ""
                    print_status "Examples:"
                    print_status "  $0              - Auto-detect distribution and build"
                    print_status "  $0 deb          - Build Debian package only"
                    print_status "  $0 rpm          - Build RPM package only"
                    print_status "  $0 arch         - Build Arch package only"
                    print_status "  $0 all          - Build all package types"
                    print_status ""
                    print_status "Dependencies:"
                    print_status "  Debian: sudo apt install devscripts debhelper"
                    print_status "  RPM:    sudo dnf install rpm-build"
                    print_status "  Arch:   makepkg (included with pacman)"
                    exit 0
                    ;;
                *)
                    print_warning "Unknown package type: $arg"
                    print_status "Available types: deb, rpm, arch, all"
                    print_status "Use '$0 --help' for more information"
                    exit 1
                    ;;
            esac
        done
    fi
    
    # Build packages
    if [ "$BUILD_DEB" = true ]; then
        build_deb
    fi
    
    if [ "$BUILD_RPM" = true ]; then
        build_rpm
    fi
    
    if [ "$BUILD_ARCH" = true ]; then
        build_arch
    fi
    
    print_success "Package building completed!"
    print_status "Built packages:"
    ls -la *.deb *.rpm *.pkg.tar.zst 2>/dev/null || print_warning "No packages found in current directory"
}

# Run main function with all arguments
main "$@" 