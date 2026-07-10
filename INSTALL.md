# MAMEUIx Installation Guide

This guide provides installation instructions for the MAMEUIx on various Linux distributions.

## Quick Installation

### Universal Installer (Recommended)

For most users, simply run the universal installer which will automatically detect your distribution:

```bash
chmod +x install.sh
./install.sh
```

### Optional: Generate Icons

For better icon support across different desktop environments, you can generate multiple icon sizes:

```bash
chmod +x generate-icons.sh
./generate-icons.sh
```

This will create icons in various sizes (16x16 to 256x256) for optimal display quality.

### Distribution-Specific Installers

If you prefer to use a specific installer for your distribution:

#### Debian/Ubuntu-based Distributions
```bash
chmod +x install-debian.sh
./install-debian.sh
```

**Supported distributions:**
- Ubuntu
- Debian
- Linux Mint
- Pop!_OS
- Elementary OS
- Kali Linux
- Parrot OS

#### RPM-based Distributions
```bash
chmod +x install-rpm.sh
./install-rpm.sh
```

**Supported distributions:**
- Fedora
- Red Hat Enterprise Linux (RHEL)
- CentOS
- Rocky Linux
- AlmaLinux
- Oracle Linux
- Amazon Linux

#### Arch-based Distributions
```bash
chmod +x install-arch.sh
./install-arch.sh
```

**Supported distributions:**
- Arch Linux
- Manjaro
- EndeavourOS
- Garuda Linux
- Artix Linux

#### AppImage (portable, any x86_64/aarch64 Linux)
```bash
chmod +x build-appimage.sh
./build-appimage.sh
./MAMEUIx-0.1.5-$(uname -m).AppImage
```

Or download a pre-built AppImage from [GitHub Releases](https://github.com/firesand/MAMEUIx/releases).

**Notes:**
- The AppImage bundles MAMEUIx and its GUI libraries only — **MAME must be installed separately**
- Set the MAME executable path in **Options → Directories & Paths** if it is not on `PATH`
- If FUSE is unavailable: `APPIMAGE_EXTRACT_AND_RUN=1 ./MAMEUIx-*.AppImage`
- For Gentoo users: `app-emulation/mameuix` is also available via the [EDORP overlay](https://github.com/firesand/edorp-overlay) (`/home/edo/EDORP` locally)
- **glibc:** Release AppImages are built on **Ubuntu 22.04** (glibc 2.35). If you see `GLIBC_2.43 not found`, download the AppImage from [GitHub Releases](https://github.com/firesand/MAMEUIx/releases) again — do not use builds made on bleeding-edge hosts without the portable pipeline.

## What the Installation Scripts Do

The installation scripts will automatically:

1. **Update your system packages**
2. **Install system dependencies** including:
   - Build tools (gcc, make, cmake)
   - X11, XCB, xkbcommon, and Wayland development libraries
   - Rust 1.85+ and Cargo
   - Git for cloning the repository

3. **Install MAME** from your distribution's package manager

4. **Install Rust** (if not already installed) using rustup

5. **Clone the repository** from GitHub

6. **Build the application** in release mode

7. **Create desktop integration**:
   - Application icons (SVG and PNG)
   - Desktop entry file
   - Application menu shortcut
   - Command-line symlink

## Prerequisites

### System Requirements
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 2GB free space for build
- **CPU**: Any modern x86_64 processor
- **Graphics**: Any graphics card with OpenGL support

### User Permissions
- **sudo access** for installing packages
- **Internet connection** for downloading dependencies
- **Git** for cloning the repository

## Manual Installation

If you prefer to install manually or the scripts don't work for your setup:

### 1. Install System Dependencies

#### Debian/Ubuntu:
```bash
sudo apt update
sudo apt install build-essential curl pkg-config cmake git mame \
    rustc cargo devscripts debhelper \
    libx11-dev libx11-xcb-dev libxcb1-dev libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev libxrandr-dev \
    libxinerama-dev libxcursor-dev libxi-dev libxkbcommon-dev \
    libwayland-dev
```

#### Fedora/RHEL/CentOS:
```bash
sudo dnf update
sudo dnf install gcc gcc-c++ curl pkgconfig cmake git mame \
    rust cargo rpm-build libxcb-devel libxkbcommon-devel \
    libX11-devel libXcursor-devel libXi-devel libXinerama-devel \
    libXrandr-devel wayland-devel
```

#### Arch Linux:
```bash
sudo pacman -Sy
sudo pacman -S base-devel curl rust pkgconf zstd cmake ninja mame \
    libx11 libxcb libxrandr libxinerama libxcursor libxi \
    libxkbcommon wayland
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Clone and Build

```bash
git clone https://github.com/firesand/MAMEUIx.git
cd MAMEUIx
cargo build --release --locked
```

### 4. Create Desktop Integration

```bash
# Install application icons
mkdir -p ~/.local/share/icons/hicolor/scalable/apps
mkdir -p ~/.local/share/icons/hicolor/256x256/apps

# Copy icons
cp assets/mameuix-icon.svg ~/.local/share/icons/hicolor/scalable/apps/mameuix.svg
cp assets/mameuix-icon.png ~/.local/share/icons/hicolor/256x256/apps/mameuix.png

# Create desktop entry
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

# Create symlink
mkdir -p ~/.local/bin
ln -sf "$PWD/target/release/mameuix" ~/.local/bin/mameuix
chmod +x target/release/mameuix
```

## Troubleshooting

### Common Issues

#### "Permission denied" errors
- Make sure the installation scripts are executable: `chmod +x install*.sh`
- Don't run as root - the scripts will use sudo when needed

#### "Package not found" errors
- Update your package lists first: `sudo apt update` or `sudo dnf update`
- Some distributions may have different package names

#### "Rust not found" after installation
- Restart your terminal or run: `source ~/.cargo/env`
- Add `export PATH="$HOME/.cargo/bin:$PATH"` to your shell profile

#### "MAME not found" error
- Ensure MAME is installed: `mame -version`
- Set the correct MAME path in the application settings

#### `GLIBC_2.43 not found` (or similar) when running AppImage

The AppImage was built on a system with a **newer glibc** than your machine. Official release AppImages are built on Ubuntu 22.04 (glibc 2.35).

- Re-download from [GitHub Releases](https://github.com/firesand/MAMEUIx/releases) after the portable build is published
- Maintainers: build with `./build-appimage-docker.sh` or `gh workflow run appimage.yml`

#### Build failures
- Check that all dependencies are installed
- Try cleaning and rebuilding: `cargo clean && cargo build --release`
- Check the Rust toolchain: `rustup show`

### Getting Help

If you encounter issues:

1. **Check the logs**: Run with debug output: `RUST_LOG=debug ./target/release/mameuix`
2. **Verify dependencies**: Ensure all system packages are installed
3. **Check Rust version**: `rustc --version` (should be 1.85+)
4. **Report issues**: Create an issue on GitHub with your distribution and error details

## Post-Installation

After successful installation:

1. **Configure ROM directories** in the application settings
2. **Set MAME executable path** if not auto-detected
3. **Add CHD directories** for CHD games
4. **Customize themes** and preferences
5. **Import your ROM collection**

## Uninstallation

### Automated Uninstallation

Use the provided uninstall script:

```bash
chmod +x uninstall.sh
./uninstall.sh
```

The script will:
- Remove desktop entry and symlinks
- Optionally remove the installation directory
- Optionally remove configuration files
- Provide guidance on removing system dependencies

### Manual Uninstallation

To remove the MAMEUIx manually:

```bash
# Remove the binary
rm ~/.local/bin/mameuix

# Remove desktop entry
rm ~/.local/share/applications/mameuix.desktop

# Remove application icons
rm ~/.local/share/icons/hicolor/scalable/apps/mameuix.svg
rm ~/.local/share/icons/hicolor/256x256/apps/mameuix.png

# Remove the build directory (optional)
rm -rf ~/mameuix

# Note: System dependencies and MAME are not removed
# Use your package manager if you want to remove them
```

## Support

For additional support:
- Check the main [README.md](README.md) for usage instructions
- Report issues on the GitHub repository
- Check the troubleshooting section above 
