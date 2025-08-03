# Advanced MAME Settings - Browse Button Functionality Fix

## Overview

This document describes the improvements made to the Advanced MAME Settings dialog in MAMEUIx, specifically focusing on fixing the browse button functionality for path and directory inputs.

## Problem Description

The original implementation of the Advanced MAME Settings dialog had browse buttons that only displayed `println!` messages instead of actually opening file dialogs. This affected:

1. **ROM Paths** - Browse buttons for ROM directory selection
2. **Artwork Path** - Browse button for artwork directory selection  
3. **Snapshot/snap Path** - Browse button for screenshot directory selection
4. **Output Directories** - Browse buttons for various output directory selections
5. **History & Information Files** - Browse buttons for DAT and INI file selections

## Solution Implemented

### 1. Fixed Browse Button in Main Advanced MAME Settings

**File**: `src/ui/components/advanced_mame_settings.rs`

**Function**: `render_path_setting()`

```rust
if browse_btn.clicked() {
    // Open file dialog for directory selection
    let mut dialog = rfd::FileDialog::new()
        .set_title(&format!("Select {} Directory", name));
    
    // Set starting directory if current value exists
    if !value.is_empty() {
        if let Ok(path) = std::path::Path::new(value).canonicalize() {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
    }
    
    if let Some(folder) = dialog.pick_folder() {
        *value = folder.display().to_string();
    }
}
```

### 2. Fixed Browse Button for Multi-Path Settings

**Function**: `render_multi_path_setting()`

```rust
if browse_btn.clicked() {
    // Open file dialog for directory selection
    let mut dialog = rfd::FileDialog::new()
        .set_title(&format!("Select {} Directory", name));
    
    // Set starting directory if current value exists
    if !path.is_empty() {
        if let Ok(path_obj) = std::path::Path::new(path).canonicalize() {
            if let Some(parent) = path_obj.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
    }
    
    if let Some(folder) = dialog.pick_folder() {
        *path = folder.display().to_string();
    }
}
```

### 3. Fixed Browse Button in Viewport Version

**File**: `src/ui/components/advanced_mame_settings_viewport.rs`

**Function**: `render_path_setting()`

```rust
if browse_btn.clicked() {
    // Open file dialog for directory selection
    let mut dialog = rfd::FileDialog::new()
        .set_title(&format!("Select {} Directory", label));
    
    // Set starting directory if current value exists
    if !value.is_empty() {
        if let Ok(path) = std::path::Path::new(value).canonicalize() {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
    }
    
    if let Some(folder) = dialog.pick_folder() {
        *value = folder.display().to_string();
    }
}
```

### 4. Fixed Browse Button for ROM Paths in Viewport

**Function**: `render_search_paths_tab()`

```rust
if browse_btn.clicked() {
    // Open file dialog for directory selection
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select ROM Directory");
    
    // Set starting directory if current value exists
    if !path.is_empty() {
        if let Ok(path_obj) = std::path::Path::new(path).canonicalize() {
            if let Some(parent) = path_obj.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
    }
    
    if let Some(folder) = dialog.pick_folder() {
        *path = folder.display().to_string();
    }
}
```

### 5. Fixed Browse Button for Single Path Settings in Viewport

**Function**: `render_single_path_setting()`

```rust
if browse_btn.clicked() {
    // Open file dialog for directory selection
    let mut dialog = rfd::FileDialog::new()
        .set_title(&format!("Select {} Directory", label));
    
    // Set starting directory if current value exists
    if !value.is_empty() {
        if let Ok(path) = std::path::Path::new(value).canonicalize() {
            if let Some(parent) = path.parent() {
                dialog = dialog.set_directory(parent);
            }
        }
    }
    
    if let Some(folder) = dialog.pick_folder() {
        *value = folder.display().to_string();
    }
}
```

## NEW: File Selection for History & Information Files

### Problem
The History & Information Files tab was using folder selection instead of file selection, which was incorrect since these are specific files with specific extensions.

### Solution: File Selection Implementation

#### 1. New File Selection Function

**Function**: `render_file_setting()`

```rust
fn render_file_setting(ui: &mut egui::Ui, name: &str, description: &str, value: &mut String, extensions: &[&str]) {
    // ... UI setup ...
    
    if browse_btn.clicked() {
        // Open file dialog for file selection
        let mut dialog = rfd::FileDialog::new()
            .set_title(&format!("Select {} File", name));
        
        // Add file extensions filter
        for ext in extensions {
            dialog = dialog.add_filter(&format!("{} files", ext.to_uppercase()), &[ext]);
        }
        
        // Set starting directory if current value exists
        if !value.is_empty() {
            if let Ok(path) = std::path::Path::new(value).canonicalize() {
                if let Some(parent) = path.parent() {
                    dialog = dialog.set_directory(parent);
                }
            }
        }
        
        if let Some(file) = dialog.pick_file() {
            *value = file.display().to_string();
        }
    }
}
```

#### 2. Updated History & Information Files Tab

**Function**: `render_history_info_tab()`

```rust
// Catver.ini file (INI format)
Self::render_file_setting(ui, "Catver Path",
    "Path to catver.ini file for game categorization",
    &mut self.settings.paths.catver_path, &["ini"]);

// History.xml file (XML format)
Self::render_file_setting(ui, "History Path",
    "Path to history.xml file for game history information",
    &mut self.settings.paths.history_path, &["xml"]);

// MAMEinfo.dat file (DAT format)
Self::render_file_setting(ui, "MAME Info Path",
    "Path to mameinfo.dat file for detailed game information",
    &mut self.settings.paths.mameinfo_path, &["dat"]);

// Command.dat file (DAT format)
Self::render_file_setting(ui, "Command Path",
    "Path to command.dat file for game command information",
    &mut self.settings.paths.command_path, &["dat"]);

// NPlayers.ini file (INI format)
Self::render_file_setting(ui, "NPlayers Path",
    "Path to nplayers.ini file for number of players information",
    &mut self.settings.paths.nplayers_path, &["ini"]);

// Languages.ini file (INI format)
Self::render_file_setting(ui, "Languages Path",
    "Path to languages.ini file for language information",
    &mut self.settings.paths.languages_path, &["ini"]);
```

## Features Added

### 1. Smart Directory Memory
- The file dialog remembers the last used directory for each path type
- If a path already exists, the dialog opens in the parent directory of that path
- This improves user experience by reducing navigation time

### 2. Proper Error Handling
- Uses `canonicalize()` to handle relative paths and symlinks
- Gracefully handles cases where paths don't exist
- Falls back to default directory if path resolution fails

### 3. Consistent UI Behavior
- All browse buttons now work consistently across the application
- Same behavior in both main dialog and viewport versions
- Proper integration with the existing UI framework

### 4. File Extension Filtering
- **INI files** - Filtered for `.ini` extension
- **XML files** - Filtered for `.xml` extension  
- **DAT files** - Filtered for `.dat` extension
- **Multiple extensions** - Support for multiple file types per field

### 5. File vs Directory Selection
- **Directory selection** - For paths that need folders (ROMs, artwork, etc.)
- **File selection** - For specific files (DAT, INI, XML files)
- **Smart filtering** - Only shows relevant file types in dialog

## Path Types Supported

### Search Paths Tab
- **ROM Paths** - Multiple directories for ROM files and CHDs
- **Sample Path** - Sound sample files directory
- **Artwork Path** - Game artwork files (bezels, overlays)
- **Snapshot/snap Path** - Game screenshots directory
- **Flyer Path** - Game flyer images directory
- **Marquees Path** - Marquee artwork directory
- **Titles Path** - Title screen images directory
- **Cabinets Path** - Cabinet artwork directory
- **Control Panels Path** - Control panel artwork directory
- **PCBs Path** - PCB artwork directory

### Output Directories Tab
- **Snapshot Directory** - Directory for saving screenshots
- **Config Directory** - Directory for configuration files
- **NVRAM Directory** - Directory for NVRAM contents
- **Input Directory** - Directory for input recordings
- **State Directory** - Directory for save states
- **Memory Card Directory** - Directory for memory card files
- **Hard Disk Diff Directory** - Directory for hard disk diff files

### History & Information Files Tab
- **Catver Path** - `catver.ini` file for game categorization (INI format)
- **History Path** - `history.xml` file for game history (XML format)
- **MAME Info Path** - `mameinfo.dat` file for detailed game information (DAT format)
- **Command Path** - `command.dat` file for game command information (DAT format)
- **NPlayers Path** - `nplayers.ini` file for number of players (INI format)
- **Languages Path** - `languages.ini` file for language information (INI format)

## Technical Implementation

### Dependencies Used
- **rfd** - Native file dialog library for Rust
- **std::path** - Standard library for path manipulation
- **egui** - UI framework for the dialog interface

### Key Functions
1. **`rfd::FileDialog::new()`** - Creates a new file dialog
2. **`.set_title()`** - Sets the dialog title
3. **`.set_directory()`** - Sets the starting directory
4. **`.pick_folder()`** - Opens folder selection dialog
5. **`.pick_file()`** - Opens file selection dialog
6. **`.add_filter()`** - Adds file extension filters
7. **`Path::canonicalize()`** - Resolves relative paths and symlinks

### Error Handling Strategy
```rust
if !value.is_empty() {
    if let Ok(path) = std::path::Path::new(value).canonicalize() {
        if let Some(parent) = path.parent() {
            dialog = dialog.set_directory(parent);
        }
    }
}
```

This approach:
- Checks if the current value is not empty
- Attempts to canonicalize the path (handles relative paths and symlinks)
- Gets the parent directory if the path exists
- Sets the dialog's starting directory to the parent
- Gracefully handles any errors by falling back to default behavior

### File Extension Filtering
```rust
// Add file extensions filter
for ext in extensions {
    dialog = dialog.add_filter(&format!("{} files", ext.to_uppercase()), &[ext]);
}
```

This approach:
- Accepts an array of file extensions
- Creates a filter for each extension type
- Shows only relevant file types in the dialog
- Improves user experience by reducing clutter

## Testing

### Demo Application
The improvements were tested using the demo application:
```bash
cargo run --example advanced_mame_settings_demo
```

### Main Application
The improvements are integrated into the main MAMEUIx application and can be accessed via:
**Options → Advanced MAME Settings...**

## Benefits

1. **Improved User Experience** - Users can now actually browse and select directories/files
2. **Consistent Behavior** - All browse buttons work the same way
3. **Smart Memory** - Remembers last used directories for efficiency
4. **Error Resilience** - Handles edge cases gracefully
5. **Native Integration** - Uses native file dialogs for better platform integration
6. **File Type Filtering** - Only shows relevant file types for each field
7. **Correct Selection Type** - Uses folder selection for directories and file selection for files

## Future Enhancements

1. **Multiple File Selection** - Support for selecting multiple files at once
2. **Path Validation** - Real-time validation of entered paths
3. **Recent Paths** - Maintain a list of recently used paths
4. **Path Templates** - Predefined path templates for common MAME setups
5. **File Preview** - Preview contents of selected files
6. **Auto-download** - Automatic download of missing DAT/INI files

## Conclusion

The browse button functionality in the Advanced MAME Settings dialog has been completely fixed and now provides a fully functional directory and file selection experience. Users can now easily configure all MAME paths, directories, and files through the intuitive file dialog interface with proper filtering for each file type.