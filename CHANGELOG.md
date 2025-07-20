# MAMEUIx v0.1.2 Release Notes

## 🎉 Release Date: July 20, 2025

### 📦 Package Information
- **Version**: 0.1.2
- **Architecture**: x86_64
- **Package Size**: 5.5MB
- **Dependencies**: mame>=0.200
- **Build Dependencies**: rust, pkgconf, zstd

### 🔄 Changes from v0.1.1

#### Dependency Updates
- Updated `serde_json` from 1.0.140 to 1.0.141
- Updated `tempfile` from 3.10 to 3.20
- All other dependencies remain at their latest stable versions

#### Build Improvements
- Fixed man page installation in PKGBUILD
- Improved source tarball generation using git archive
- Enhanced build process reliability
- **Corrected dependencies**: Removed unnecessary GTK3, WebKit2GTK, and OpenSSL dependencies
- **Minimal dependencies**: Now only requires mame>=0.200 at runtime

#### Code Quality
- Maintained compatibility with egui 0.32
- Preserved all existing features and functionality
- No breaking changes from v0.1.1

### 🚀 Features (Carried Over from Previous Versions)

#### Core Features
- **Fast Game Loading**: Efficiently loads and displays 48,000+ MAME games
- **Smart ROM Detection**: Automatically detects available ROMs and CHD files
- **Advanced Filtering**: Filter games by availability, manufacturer, year, and more
- **CHD Game Support**: Full support for CHD (Compressed Hunks of Data) games
- **Virtual Scrolling**: Smooth performance with large game lists
- **Persistent Settings**: Remembers your preferences and column widths

#### User Interface
- **Modern Design**: Clean, intuitive interface built with egui
- **10 Beautiful Themes**: Choose from Dark Blue, Neon Green, Arcade Purple, Light Classic, and 6 more themes
- **Responsive Layout**: Adapts to different screen sizes
- **Fully Resizable Columns**: All table columns can be resized to any width with persistent settings
- **Artwork Display**: Shows game artwork and screenshots
- **Search Functionality**: Quick search through game names and descriptions
- **Favorites System**: Mark and filter your favorite games

#### Advanced Features
- **Background Scanning**: Non-blocking ROM and MAME data loading
- **Performance Monitoring**: Built-in performance tracking
- **Debug Tools**: Comprehensive logging and debugging options
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **Hardware Filtering**: Filter games by CPU, device, and sound chip types
- **BGFX/GLSL Support**: Advanced graphics backend integration with shader support
- **INI File Processing**: Support for MAME INI files and hardware categorization
- **Plugin Detection**: Automatic detection of MAME plugins (hiscore, cheat, autofire)

### 📋 Installation

#### Arch Linux (AUR)
```bash
# Using yay
yay -S mameuix

# Using paru
paru -S mameuix

# Manual installation
git clone https://aur.archlinux.org/mameuix.git
cd mameuix
makepkg -si
```

#### Manual Installation
```bash
# Download and install the package
sudo pacman -U mameuix-0.1.2-1-x86_64.pkg.tar.zst
```

### 🔧 System Requirements
- **Rust**: 1.88.0 or later (recommended)
- **MAME**: Any recent version (0.200+ recommended)
- **Memory**: 4GB RAM minimum, 8GB recommended for large ROM collections
- **Storage**: 100MB for application, additional space for ROMs and artwork
- **Graphics**: OpenGL 3.3+ for BGFX support, DirectX 11+ for Windows

### 🐛 Known Issues
- Some compiler warnings (non-critical, functionality unaffected)
- Deprecated egui API usage (will be addressed in future versions)

### 🔮 Future Plans
- Address compiler warnings and deprecated API usage
- Performance optimizations for large ROM collections
- Enhanced theme customization options
- Improved BGFX/GLSL integration

### 📞 Support
- **GitHub**: https://github.com/firesand/MAMEUIx
- **Issues**: https://github.com/firesand/MAMEUIx/issues
- **Documentation**: See README.md for detailed usage instructions

---

**Enjoy your arcade gaming experience with MAMEUIx! 🕹️** 