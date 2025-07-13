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

## What the Installation Scripts Do

The installation scripts will automatically:

1. **Update your system packages**
2. **Install system dependencies** including:
   - Build tools (gcc, make, cmake)
   - Development libraries (GTK3, WebKit, OpenSSL)
   - Multimedia libraries (GStreamer)
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
sudo apt install build-essential curl pkg-config libssl-dev \
    libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev \
    librsvg2-dev gstreamer1.0-dev gstreamer1.0-plugins-base-dev \
    gstreamer1.0-plugins-good gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly gstreamer1.0-libav cmake git mame
```

#### Fedora/RHEL/CentOS:
```bash
sudo dnf update
sudo dnf install gcc gcc-c++ curl pkg-config openssl-devel \
    gtk3-devel webkit2gtk3-devel libappindicator-gtk3-devel \
    librsvg2-devel gstreamer1-devel gstreamer1-plugins-base-devel \
    gstreamer1-plugins-good gstreamer1-plugins-bad-free \
    gstreamer1-plugins-ugly-free gstreamer1-libav cmake git mame
```

#### Arch Linux:
```bash
sudo pacman -Sy
sudo pacman -S base-devel curl pkg-config openssl gtk3 webkit2gtk \
    libappindicator-gtk3 librsvg gstreamer gst-plugins-base \
    gst-plugins-good gst-plugins-bad gst-plugins-ugly gst-libav \
    cmake git mame
```

### 2. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Clone and Build

```bash
git clone https://github.com/yourusername/mameuix.git
cd mameuix
cargo build --release
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

#### Build failures
- Check that all dependencies are installed
- Try cleaning and rebuilding: `cargo clean && cargo build --release`
- Check the Rust toolchain: `rustup show`

### Getting Help

If you encounter issues:

1. **Check the logs**: Run with debug output: `RUST_LOG=debug ./target/release/mameuix`
2. **Verify dependencies**: Ensure all system packages are installed
3. **Check Rust version**: `rustc --version` (should be 1.70+)
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