# Advanced MAME Settings - Paths & Directories Fix

## Overview
This document describes the fixes implemented to make the Advanced MAME Settings Viewport's Paths & Directories section fully functional, matching the capabilities of the DirectoriesDialog.

## Changes Implemented

### 1. **AppConfig Integration**
- Modified `AdvancedMameSettingsViewport` to accept and use `AppConfig`
- Added `config: AppConfig` and `changes_made: bool` fields to track configuration and changes
- Updated constructor to initialize settings from the provided config

### 2. **Browse Functionality**
- Replaced placeholder `println!` statements with actual file/folder dialogs using `rfd::FileDialog`
- Implemented proper browse functionality for:
  - ROM paths (multiple directories)
  - Single path settings (artwork, snap, etc.)
  - File selections (history.xml, mameinfo.dat, etc.)

### 3. **Smart Directory Memory**
- Integrated with the existing `last_directories` HashMap in AppConfig
- Categories used:
  - `"rom_directories"` - for ROM paths
  - `"artwork_extra"` - for artwork-related paths
  - `"dat_files"` - for DAT and XML files
  - `"support_files"` - for other support files
- Automatically remembers the last used directory for each category

### 4. **Path Validation**
- Added visual validation with color coding:
  - Normal color for valid paths or empty fields
  - Red color for invalid/non-existent paths
- Validation checks:
  - Directory existence for folder paths
  - File existence for specific files (history.xml, mameinfo.dat, etc.)

### 5. **Apply Changes Implementation**
- Implemented `apply_settings_to_config()` method to save changes back to AppConfig
- Properly converts between String (in MameSettings) and PathBuf (in AppConfig)
- Handles both Some/None options for optional paths
- Resets `changes_made` flag after applying

### 6. **Change Detection**
- Added `changes_made` flag that is set to true when:
  - Text fields are edited
  - Browse dialogs select new paths
  - Paths are added or removed
- Used to show warning when closing without saving (future enhancement)

## Usage

### Creating the Dialog
```rust
// Pass the current AppConfig when creating the dialog
let mut settings_viewport = AdvancedMameSettingsViewport::new(config.clone());
```

### Getting Updated Config
```rust
// After user clicks Apply
let updated_config = settings_viewport.get_config();
// Save the config to file
```

### Checking for Changes
```rust
if settings_viewport.has_changes() {
    // Warn user about unsaved changes
}
```

## Features Now Working

1. ✅ **Browse ROM Paths** - Full file dialog with smart directory memory
2. ✅ **Browse Snapshot/Snap Folder** - Folder selection with validation
3. ✅ **Browse History.xml** - File selection with .xml filter
4. ✅ **Browse mameinfo.dat** - File selection with .dat filter
5. ✅ **All Other Paths** - Appropriate file/folder selection
6. ✅ **Path Validation** - Red text for invalid paths
7. ✅ **Smart Directory Memory** - Remembers last used folders
8. ✅ **Apply Changes** - Saves back to AppConfig

## Remaining Work

### Auto Reload Detection
To complete the auto-reload functionality, the main window needs to:
1. Check if paths that affect game loading have changed (ROM paths, catver.ini)
2. Trigger a game list reload when these critical paths change

This can be implemented by comparing the config before and after the dialog:
```rust
let old_rom_paths = config.rom_paths.clone();
let old_catver = config.catver_ini_path.clone();

// Show dialog and apply changes...

if config.rom_paths != old_rom_paths || config.catver_ini_path != old_catver {
    // Trigger game reload
}
```

## Technical Notes

### Rounding vs CornerRadius
The code was updated to use `Rounding` instead of the deprecated `CornerRadius`:
```rust
// Old
CornerRadius::same(12)
// New
Rounding::same(12.0)
```

### File Type Detection
The browse functionality intelligently detects whether to show file or folder picker based on the field name:
- Fields containing ".xml", ".dat", or "Catver" use file picker
- All other fields use folder picker

### Path Conversion
Proper conversion between String (UI) and PathBuf (config):
```rust
// String to PathBuf
PathBuf::from(&string_path)
// PathBuf to String
path_buf.display().to_string()