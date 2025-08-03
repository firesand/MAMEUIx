# MAME Executable Paths Feature

## Overview

The MAME Executable Paths feature allows users to configure multiple MAME versions for testing and compatibility. This feature is accessible through **Options → MAME Executable Paths** in the main menu.

## Features

### 1. Multiple MAME Version Support
- Configure multiple MAME executables on your system
- Switch between different MAME versions easily
- Test compatibility with different MAME releases

### 2. Auto-Detection System
- **First Launch Detection**: Automatically scans for MAME executables when dialog is first opened
- **Common Locations**: Scans `/usr/bin/mame`, `/usr/local/bin/mame`, `/opt/mame/mame`, etc.
- **Version Detection**: Automatically detects MAME version using `--version` flag
- **Notification System**: Shows success messages when executables are found

### 3. Manual File Browser
- **Add MAME Executable**: Browse and select MAME executable files
- **File Filtering**: Filters for executable files (`.exe`, `.bin`, etc.)
- **Version Detection**: Automatically detects version from selected file

### 4. Visual Interface
Based on the mockup design from `/Plans/mame-executable-paths.html`, the dialog provides:

- **Executable List** - Shows all configured MAME executables
- **Status Indicators** - Visual status for each executable (Active, Available, Not Found)
- **Properties Display** - Shows build date, ROM count, CHD support, debug build status
- **Action Buttons** - Set Active, Remove, Add new executable
- **Notification System** - Shows success messages for auto-detection and manual additions

## Implementation Details

### File Structure
```
src/ui/components/
├── mame_executable_paths.rs    # Main dialog implementation
├── dialog_manager.rs           # Dialog management with auto-detect
└── mod.rs                     # Module exports
```

### Key Components

#### 1. Auto-Detection Functionality
```rust
fn auto_detect_mame() -> Vec<PathBuf> {
    let common_paths = vec![
        "/usr/bin/mame",
        "/usr/local/bin/mame",
        "/usr/bin/mame64",
        "/usr/local/bin/mame64",
        "/opt/mame/mame",
        "/opt/mame/mame64",
    ];
    // Scan and return found executables
}
```

#### 2. Version Detection
```rust
fn detect_mame_version(path: &PathBuf) -> String {
    // Try to run MAME with --version flag
    // Fallback to filename-based detection
}
```

#### 3. File Browser Integration
```rust
if let Some(path) = rfd::FileDialog::new()
    .set_title("Select MAME Executable")
    .add_filter("Executable files", &["exe", "bin", ""])
    .pick_file() {
    // Add executable to configuration
}
```

## Auto-Detection Process

### 1. First Launch Detection
When the MAME Executable Paths dialog is opened for the first time:

1. **Check if executables exist**: If no MAME executables are configured
2. **Scan common locations**: Automatically scan `/usr/bin/mame`, `/usr/local/bin/mame`, etc.
3. **Validate executables**: Check if files exist and are executable
4. **Detect versions**: Run `mame --version` to get version information
5. **Add to configuration**: Automatically add found executables to the list
6. **Show notification**: Display success message with number of executables found

### 2. Manual Auto-Detect
Users can manually trigger auto-detection:

1. **Click "Auto-Detect"**: Button in the dialog
2. **Scan locations**: Same process as first launch
3. **Add to list**: Found executables are added to existing list
4. **Show notification**: Success message with count of new executables

### 3. Common Scan Locations
The system scans these locations for MAME executables:

- `/usr/bin/mame` - Standard Linux installation
- `/usr/local/bin/mame` - User-installed MAME
- `/usr/bin/mame64` - 64-bit version
- `/usr/local/bin/mame64` - User-installed 64-bit version
- `/opt/mame/mame` - Alternative installation
- `/opt/mame/mame64` - Alternative 64-bit installation

## Manual File Addition

### 1. File Browser
Users can manually add MAME executables:

1. **Click "➕ Add MAME Executable"**: Opens file browser
2. **Select executable**: Browse to MAME executable file
3. **Version detection**: System automatically detects version
4. **Add to list**: Executable is added to configuration
5. **Show notification**: Success message with file path

### 2. File Filtering
The file browser includes filters for:
- **Executable files**: `.exe`, `.bin`, and files without extension
- **All files**: For systems with different naming conventions

## Version Detection Methods

### 1. Primary Method: `--version` Flag
```bash
mame --version
# Output: MAME 0.264 (mame0264-0.264)
```

### 2. Fallback Method: Filename Analysis
If `--version` fails, the system analyzes the filename:
- `mame` → "MAME (detected from mame)"
- `mame64` → "MAME (detected from mame64)"
- `mame0264` → "MAME (detected from mame0264)"

### 3. Default Method: Unknown Version
If both methods fail:
- Returns "MAME (unknown version)"

## Notification System

### 1. Auto-Detection Notifications
- **Success**: "Auto-detected X MAME executable(s)"
- **Console output**: "MAME executable detected in /usr/bin/mame"

### 2. Manual Addition Notifications
- **Success**: "Added MAME executable: /path/to/mame"

### 3. Visual Feedback
- **Green text**: Success messages in dialog
- **Console logging**: Detailed information for debugging

## Configuration Storage

The MAME executable paths are stored in the application configuration:

```rust
pub struct AppConfig {
    pub mame_executables: Vec<MameExecutable>,
    pub selected_mame_index: usize,
    // ... other fields
}

pub struct MameExecutable {
    pub name: String,        // User-friendly name
    pub path: String,        // Full path to executable
    pub version: String,     // MAME version number
    pub total_games: usize,  // Total supported games
    pub working_games: usize, // Working games count
}
```

## Usage Examples

### 1. First Launch Experience
```
1. User opens MAMEUIx for the first time
2. User clicks "Options → MAME Executable Paths"
3. System automatically scans for MAME
4. Found: "MAME executable detected in /usr/bin/mame"
5. Dialog shows: "Auto-detected 1 MAME executable(s)"
6. MAME 0.264 appears in the executable list
```

### 2. Manual Addition
```
1. User clicks "➕ Add MAME Executable"
2. File browser opens
3. User selects "/home/user/custom/mame"
4. System detects version: "MAME 0.263"
5. Dialog shows: "Added MAME executable: /home/user/custom/mame"
6. New executable appears in list
```

### 3. Auto-Detect Button
```
1. User clicks "Auto-Detect"
2. System scans common locations
3. Found: "/usr/bin/mame" and "/usr/local/bin/mame64"
4. Dialog shows: "Auto-detected 2 MAME executable(s)"
5. Both executables added to list
```

## Error Handling

### 1. No Executables Found
- **First launch**: Shows empty list with "Add MAME Executable" button
- **Manual auto-detect**: No notification, list remains unchanged

### 2. Permission Issues
- **Non-executable files**: Skipped during scan
- **Permission denied**: Logged to console, file skipped

### 3. Version Detection Failures
- **Command not found**: Falls back to filename analysis
- **Invalid output**: Uses "unknown version" fallback

## Future Enhancements

### 1. Enhanced Detection
- **Recursive scanning**: Search subdirectories
- **Custom paths**: User-defined search locations
- **Network detection**: Find MAME on network shares

### 2. Advanced Features
- **Version comparison**: Compare MAME versions
- **Compatibility checking**: Check ROM compatibility
- **Performance testing**: Test MAME performance

### 3. UI Improvements
- **Progress indicators**: Show scan progress
- **Detailed results**: Show scan details
- **Error reporting**: Better error messages

## Technical Notes

### 1. Cross-Platform Support
- **Linux**: Uses `/usr/bin`, `/usr/local/bin`, `/opt`
- **Windows**: Would use `Program Files`, `C:\mame`
- **macOS**: Would use `/Applications`, `/usr/local/bin`

### 2. Performance Considerations
- **Lazy scanning**: Only scan when dialog opens
- **Caching**: Cache scan results
- **Background scanning**: Non-blocking UI

### 3. Security Considerations
- **Executable validation**: Check file permissions
- **Path validation**: Ensure paths are safe
- **Version verification**: Validate MAME output

## Conclusion

The MAME Executable Paths feature provides a comprehensive solution for managing multiple MAME installations. The auto-detection system makes it easy for new users to get started, while the manual file browser gives advanced users full control over their MAME setup. The notification system provides clear feedback about the detection and addition process. 