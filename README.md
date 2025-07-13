# MAME Frontend

A modern, fast, and user-friendly frontend for MAME (Multiple Arcade Machine Emulator) written in Rust using the egui framework.

## Features

### 🎮 Core Features
- **Fast Game Loading**: Efficiently loads and displays 48,000+ MAME games
- **Smart ROM Detection**: Automatically detects available ROMs and CHD files
- **Advanced Filtering**: Filter games by availability, manufacturer, year, and more
- **CHD Game Support**: Full support for CHD (Compressed Hunks of Data) games
- **Virtual Scrolling**: Smooth performance with large game lists
- **Persistent Settings**: Remembers your preferences and column widths

### 🎨 User Interface
- **Modern Design**: Clean, intuitive interface built with egui
- **10 Beautiful Themes**: Choose from Dark Blue, Neon Green, Arcade Purple, Light Classic, and 6 more themes
- **Responsive Layout**: Adapts to different screen sizes
- **Customizable Columns**: Resize and reorder game list columns with persistent widths
- **Artwork Display**: Shows game artwork and screenshots
- **Search Functionality**: Quick search through game names and descriptions
- **Favorites System**: Mark and filter your favorite games
- **Theme Customization**: Easy theme switching via menu or preferences

### 🔧 Advanced Features
- **Background Scanning**: Non-blocking ROM and MAME data loading
- **Performance Monitoring**: Built-in performance tracking
- **Debug Tools**: Comprehensive logging and debugging options
- **Cross-Platform**: Runs on Windows, macOS, and Linux

## Installation

### Quick Installation (Linux)

For Linux users, we provide automated installation scripts:

```bash
# Universal installer (detects your distribution automatically)
chmod +x install.sh
./install.sh

# Or use distribution-specific installers:
./install-debian.sh    # For Ubuntu, Debian, Linux Mint, etc.
./install-rpm.sh       # For Fedora, RHEL, CentOS, etc.
./install-arch.sh      # For Arch Linux, Manjaro, etc.
```

See [INSTALL.md](INSTALL.md) for detailed installation instructions.

### Manual Installation

#### Prerequisites
- **Rust**: Install Rust from [rustup.rs](https://rustup.rs/)
- **MAME**: Install MAME on your system
  - **Linux**: `sudo pacman -S mame` (Arch) or `sudo apt install mame` (Ubuntu)
  - **Windows**: Download from [mamedev.org](https://www.mamedev.org/)
  - **macOS**: `brew install mame`

#### Building from Source

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd mame-frontend
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the application**:
   ```bash
   cargo run --release
   ```

## Configuration

### ROM Directories
1. Open the application
2. Go to **Settings → Directories**
3. Add your ROM directories (containing `.zip` files)
4. Add your CHD directories (containing `.chd` files)

### MAME Executable
1. Go to **Settings → Directories**
2. Set the path to your MAME executable
3. The application will automatically detect MAME version and game list

## Usage

### Basic Navigation
- **Game List**: Browse and select games from the main list
- **Search**: Use the search bar to find specific games
- **Filters**: Use the sidebar to filter games by various criteria
- **Double-click**: Launch a game (if ROMs are available)

### Advanced Features
- **Theme Selection**: Choose from 10 beautiful themes via Options → Theme menu
- **Column Customization**: Right-click column headers to customize with persistent widths
- **Favorites**: Click the star icon to mark favorite games
- **Artwork**: View game artwork in the right panel
- **Game Info**: See detailed game information and ROM status
- **Preferences**: Comprehensive settings dialog for UI customization

### Keyboard Shortcuts
- **Ctrl+F**: Focus search bar
- **Ctrl+O**: Open directories dialog
- **Ctrl+P**: Open preferences
- **F5**: Refresh game list
- **Escape**: Clear search
- **Options → Theme**: Quick theme switching

## File Structure

```
src/
├── main.rs              # Application entry point
├── config/              # Configuration management
├── graphics/            # Graphics and rendering
├── mame/                # MAME integration
│   ├── launcher.rs      # Game launching
│   ├── scanner.rs       # ROM scanning
│   └── mod.rs
├── models/              # Data models
│   ├── game.rs          # Game data structure
│   ├── config.rs        # Configuration models
│   ├── filters.rs       # Filtering logic
│   └── mod.rs
├── rom_utils/           # ROM utilities
├── ui/                  # User interface
│   ├── main_window.rs   # Main application window
│   ├── game_list.rs     # Game list component
│   ├── sidebar.rs       # Sidebar with filters
│   ├── artwork_panel.rs # Artwork display
│   ├── dialogs/         # Dialog windows
│   └── mod.rs
```

## Performance

The application is optimized for performance:
- **Virtual Scrolling**: Only renders visible game rows
- **Background Processing**: Non-blocking UI during scans
- **Efficient Indexing**: Fast search and filtering
- **Memory Management**: Optimized for large game collections

## Troubleshooting

### Common Issues

**"MAME executable not found"**
- Ensure MAME is installed and in your PATH
- Set the correct path in Settings → Directories

**"No ROMs found"**
- Check that ROM directories are correctly configured
- Verify ROM files are in the expected format (.zip)

**"CHD games not detected"**
- Ensure CHD directories are added to settings
- Check that CHD files are properly named

**"Slow performance"**
- Use release builds: `cargo run --release`
- Reduce the number of ROM directories
- Close other applications to free memory

### Debug Mode
Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- **MAME Team**: For the excellent arcade emulator
- **egui**: For the modern GUI framework
- **Rust Community**: For the amazing ecosystem

## Version History

### v0.1.1
- **10 Beautiful Themes**: Added comprehensive theme system with 10 different visual themes
- **Theme Customization**: Easy theme switching via menu and preferences dialog
- **Persistent Column Widths**: Column widths are now saved and restored
- **Enhanced UI**: Improved preferences dialog with theme selection
- **Better Performance**: Optimized rendering and reduced UI lag
- **Bug Fixes**: Fixed borrow checker issues and compilation warnings

### v0.1.0
- Initial release
- Basic MAME integration
- Game list with filtering
- ROM detection
- CHD support
- Modern UI with egui

---

**Note**: This frontend requires MAME to be installed separately. It does not include ROM files or MAME itself.
