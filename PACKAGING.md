# MAMEUIx Packaging Guide

This document provides detailed information about building and distributing MAMEUIx packages for different Linux distributions.

## Overview

MAMEUIx supports three main Linux package formats:
- **Debian (.deb)**: For Ubuntu, Debian, Linux Mint, Pop!_OS, etc.
- **RPM (.rpm)**: For Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux, etc.
- **Arch (.pkg.tar.zst)**: For Arch Linux, Manjaro, EndeavourOS, etc.

## Prerequisites

### Common Dependencies
All package builds require:
- **Rust**: Latest stable version
- **Cargo**: Rust package manager
- **pkg-config**: Package configuration utility

### Distribution-Specific Dependencies

#### Debian/Ubuntu
```bash
sudo apt install devscripts debhelper
```

#### Red Hat/Fedora
```bash
sudo dnf install rpm-build
# or
sudo yum install rpm-build
```

#### Arch Linux
```bash
# makepkg is included with pacman
```

## Package Structure

### Files Included in Packages
- **Binary**: `/usr/bin/mameuix`
- **Desktop File**: `/usr/share/applications/mameuix.desktop`
- **Icons**: Multiple sizes in `/usr/share/icons/hicolor/`
- **Man Page**: `/usr/share/man/man1/mameuix.1`
- **Documentation**: `/usr/share/doc/mameuix/`

### Dependencies
- **Runtime**: `mame >= 0.200`, `gtk3`, `webkit2gtk`
- **Build**: `rust`, `cargo`, `pkg-config`, `openssl-devel`, `gtk3-devel`, `webkit2gtk3-devel`

## Building Packages

### Quick Start

Use the universal package builder:
```bash
# Auto-detect distribution and build appropriate package
./build-packages.sh

# Build specific package types
./build-packages.sh deb    # Debian package
./build-packages.sh rpm    # RPM package
./build-packages.sh arch   # Arch package
./build-packages.sh all    # All package types
```

### Individual Build Scripts

#### Debian Package
```bash
./build-deb.sh
```
**Output**: `../mameuix_0.1.1_amd64.deb`

#### RPM Package
```bash
./build-rpm.sh
```
**Output**: `mameuix-0.1.1-1.x86_64.rpm`

#### Arch Package
```bash
./build-arch.sh
```
**Output**: `mameuix-0.1.1-1-x86_64.pkg.tar.zst`

## Package Configuration Files

### Debian Package Files

#### `debian/control`
Package metadata, dependencies, and descriptions.

#### `debian/rules`
Build instructions and installation rules.

#### `debian/changelog`
Version history and release notes.

#### `debian/compat`
Debhelper compatibility level.

### RPM Package Files

#### `mameuix.spec`
Complete RPM specification including build, install, and file lists.

### Arch Package Files

#### `PKGBUILD`
Package build instructions and metadata.

## Installation

### Debian/Ubuntu
```bash
sudo dpkg -i mameuix_*.deb
sudo apt-get install -f  # Install missing dependencies
```

### Red Hat/Fedora
```bash
sudo dnf install mameuix-*.rpm
# or
sudo yum install mameuix-*.rpm
```

### Arch Linux
```bash
sudo pacman -U mameuix-*.pkg.tar.zst
```

## Package Management

### Updating Packages
```bash
# Remove old version
sudo apt remove mameuix          # Debian/Ubuntu
sudo dnf remove mameuix          # Red Hat/Fedora
sudo pacman -R mameuix           # Arch Linux

# Install new version
sudo dpkg -i mameuix_*.deb       # Debian/Ubuntu
sudo dnf install mameuix-*.rpm   # Red Hat/Fedora
sudo pacman -U mameuix-*.pkg.tar.zst  # Arch Linux
```

### Uninstalling
```bash
sudo apt remove mameuix          # Debian/Ubuntu
sudo dnf remove mameuix          # Red Hat/Fedora
sudo pacman -R mameuix           # Arch Linux
```

## Distribution-Specific Notes

### Debian/Ubuntu
- Uses `debuild` for package building
- Generates source packages (.dsc, .tar.gz)
- Supports multiple architectures
- Includes debug packages

### Red Hat/Fedora
- Uses `rpmbuild` for package building
- Generates both binary and source RPMs
- Supports multiple architectures
- Includes debug information

### Arch Linux
- Uses `makepkg` for package building
- Generates compressed packages (.pkg.tar.zst)
- Supports multiple architectures
- Includes source tarball

## Troubleshooting

### Common Build Issues

#### "debuild not found"
```bash
sudo apt install devscripts
```

#### "rpmbuild not found"
```bash
sudo dnf install rpm-build
```

#### "makepkg not found"
This script should be run on an Arch Linux system or in an Arch container.

#### "Missing dependencies"
```bash
# Install build dependencies
sudo apt install build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.0-dev
sudo dnf install gcc pkgconfig openssl-devel gtk3-devel webkit2gtk3-devel
sudo pacman -S base-devel pkgconf openssl gtk3 webkit2gtk
```

### Package Installation Issues

#### Dependency Resolution
```bash
# Debian/Ubuntu
sudo apt-get install -f

# Red Hat/Fedora
sudo dnf install mame

# Arch Linux
sudo pacman -S mame
```

#### Permission Issues
```bash
# Ensure proper ownership
sudo chown root:root mameuix_*.deb
sudo chmod 644 mameuix_*.deb
```

## Version Management

### Updating Version Numbers
1. Update `Cargo.toml` version field
2. Update package files:
   - `debian/changelog`
   - `mameuix.spec`
   - `PKGBUILD`
3. Update `debian/mameuix.1` man page if needed

### Release Process
1. Update version numbers
2. Build packages: `./build-packages.sh all`
3. Test packages on target distributions
4. Create GitHub release with package files
5. Update documentation

## Contributing to Packaging

### Adding New Distributions
1. Create distribution-specific build script
2. Add detection logic to `build-packages.sh`
3. Update documentation
4. Test on target distribution

### Improving Package Quality
- Add proper dependencies
- Include comprehensive documentation
- Test installation and uninstallation
- Verify desktop integration
- Check icon installation

## Resources

- [Debian Packaging Guide](https://www.debian.org/doc/manuals/maint-guide/)
- [RPM Packaging Guide](https://rpm-packaging-guide.github.io/)
- [Arch Linux Packaging Standards](https://wiki.archlinux.org/title/PKGBUILD)
- [Rust Packaging Best Practices](https://doc.rust-lang.org/cargo/reference/publishing.html) 