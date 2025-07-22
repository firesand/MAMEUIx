# MAMEUIX Arch/CachyOS Package

## Overview

This directory contains the Arch Linux and CachyOS package files for MAMEUIX v0.1.3, a modern GUI frontend for the MAME arcade emulator with thread pool performance and enhanced UI.

## Package Information

- **Package Name**: `mameuix`
- **Version**: `0.1.3`
- **Architecture**: `x86_64`
- **License**: MIT
- **Maintainer**: edo hikmahtiar <edohikmahtiar@me.com>

## Features

### 🚀 Performance Improvements
- **Thread pool icon loading** for 48,000+ games
- **8x faster performance** on multi-core systems
- **Real-time performance monitoring**
- **Non-blocking UI operations**

### 🎨 UI Enhancements
- **Enhanced window resizing** (600x550 default, up to 1200x900)
- **Improved game history layout** (55% history vs 45% artwork)
- **Window persistence** across sessions
- **Better text display and formatting**

## Dependencies

### Required Dependencies
- `mame>=0.200` - MAME arcade emulator
- `glibc` - GNU C Library
- `gcc-libs` - GCC runtime libraries

### Build Dependencies
- `rust>=1.70` - Rust programming language
- `pkgconf` - Package configuration utility
- `zstd` - Zstandard compression library
- `git` - Distributed version control system

### Optional Dependencies
- `mame-roms` - Game ROMs for MAME

## Installation

### From AUR (Arch User Repository)

```bash
# Using yay
yay -S mameuix

# Using paru
paru -S mameuix

# Using manual AUR helper
git clone https://aur.archlinux.org/mameuix.git
cd mameuix
makepkg -si
```

### From Source

```bash
# Clone the repository
git clone https://github.com/firesand/MAMEUIx.git
cd MAMEUIx

# Build and install
makepkg -si
```

### Manual Build

```bash
# Build package
./build-arch-package.sh

# Install package
sudo pacman -U mameuix-0.1.3-1-x86_64.pkg.tar.zst
```

## Package Contents

The package installs the following components:

### Binary
- `/usr/bin/mameuix` - Main application binary

### Desktop Integration
- `/usr/share/applications/mameuix.desktop` - Desktop entry
- `/usr/share/icons/hicolor/*/apps/mameuix.png` - Application icons
- `/usr/share/icons/hicolor/scalable/apps/mameuix.svg` - Scalable icon

### Documentation
- `/usr/share/doc/mameuix/README.md` - Main documentation
- `/usr/share/doc/mameuix/CHANGELOG.md` - Version history
- `/usr/share/doc/mameuix/LICENSE` - License information
- `/usr/share/doc/mameuix/ICON_LOADING_PERFORMANCE.md` - Performance guide

### Manual Pages
- `/usr/share/man/man1/mameuix.1` - Manual page (if available)

## Building the Package

### Prerequisites

Ensure you have the required build tools:

```bash
sudo pacman -S base-devel namcap
```

### Build Process

1. **Clean build environment**:
   ```bash
   rm -rf pkg/ src/ *.pkg.tar.zst
   ```

2. **Update package metadata**:
   ```bash
   makepkg --printsrcinfo > .SRCINFO
   ```

3. **Build package**:
   ```bash
   makepkg --syncdeps --noconfirm
   ```

4. **Validate package**:
   ```bash
   namcap mameuix-0.1.3-1-x86_64.pkg.tar.zst
   ```

### Automated Build

Use the provided build script:

```bash
./build-arch-package.sh
```

## Package Validation

The package is validated using `namcap` to ensure:

- **Dependency accuracy** - All dependencies are correctly specified
- **Package structure** - Files are installed in appropriate locations
- **Security** - No obvious security issues
- **Best practices** - Follows Arch packaging guidelines

## Performance Optimizations

### Build Optimizations
- **LTO (Link Time Optimization)** enabled
- **Native CPU optimizations** (`-C target-cpu=native`)
- **Release mode** with maximum optimizations
- **Stripped binaries** for smaller package size

### Runtime Optimizations
- **Thread pool** for parallel icon loading
- **Memory-efficient caching** with automatic cleanup
- **Non-blocking UI operations**
- **Adaptive loading rates** based on system performance

## Troubleshooting

### Common Issues

1. **Build fails with zstd error**:
   ```bash
   export ZSTD_LIB_DIR=/usr/lib
   export ZSTD_STATIC=0
   ```

2. **Missing Rust dependencies**:
   ```bash
   rustup update
   cargo clean
   ```

3. **Package validation warnings**:
   - Check `namcap` output for specific issues
   - Update dependencies if needed

### Performance Issues

1. **Slow icon loading**:
   - Ensure you have a multi-core CPU
   - Check available memory
   - Verify MAME installation

2. **UI responsiveness**:
   - Update graphics drivers
   - Check system resources
   - Verify egui compatibility

## Contributing

To contribute to the package:

1. **Fork the repository**
2. **Make your changes**
3. **Test the package build**
4. **Submit a pull request**

### Package Guidelines

- Follow [Arch packaging standards](https://wiki.archlinux.org/title/PKGBUILD)
- Use `namcap` for validation
- Test on clean systems
- Document all changes

## License

This package is licensed under the MIT License. See the LICENSE file for details.

## Support

For support and issues:

- **GitHub Issues**: https://github.com/firesand/MAMEUIx/issues
- **AUR Package**: https://aur.archlinux.org/packages/mameuix
- **Email**: edohikmahtiar@me.com

## Changelog

### v0.1.3
- **Major performance improvements** with thread pool icon loading
- **Enhanced UI** with better window resizing and layout
- **Performance monitoring** system
- **Window persistence** across sessions
- **Updated dependencies** and build optimizations

### v0.1.2
- Initial Arch package release
- Basic MAME frontend functionality
- Standard desktop integration

---

**Note**: This package is maintained for both Arch Linux and CachyOS distributions. The same package should work on both systems. 