# MAME Folder Configuration Feature

## Overview

MAMEUIx now includes a comprehensive MAME folder configuration feature that allows users to customize where MAME stores its various internal files. This feature gives users full control over MAME's file organization and storage locations.

## Features

### Supported MAME Folders

The following MAME internal folders can be configured (all supported by MAME v0.278):

1. **Configuration Files (cfg)** - MAME configuration files directory
   - Command-line option: `-cfg_directory`
   - Default location: `~/.mame/cfg/` (Linux)

2. **NVRAM** - Non-volatile RAM directory
   - Command-line option: `-nvram_directory`
   - Default location: `~/.mame/nvram/` (Linux)

3. **Input Configuration (input)** - Input configuration files directory
   - Command-line option: `-input_directory`
   - Default location: `~/.mame/input/` (Linux)

4. **Save States (state)** - Save state files directory
   - Command-line option: `-state_directory`
   - Default location: `~/.mame/state/` (Linux)

5. **Hard Disk Diffs (diff)** - Hard disk diff files directory
   - Command-line option: `-diff_directory`
   - Default location: `~/.mame/diff/` (Linux)

6. **Comment Files (comment)** - Comment files directory
   - Command-line option: `-comment_directory`
   - Default location: `~/.mame/comment/` (Linux)



## How to Use

### Accessing the Configuration

1. Open MAMEUIx
2. Go to **Options → Directories**
3. Click on the **"MAME Internal Folders"** tab
4. Configure the paths for each MAME folder type you want to customize

### Configuration Interface

The MAME Internal Folders tab provides:

- **Clear descriptions** for each folder type
- **Path validation** with visual indicators (green for valid paths, red for invalid)
- **Browse buttons** for easy folder selection
- **Disabled state** for unsupported options (marked in red with "(Disabled)" label)
- **Real-time feedback** on path validity

### Example Configuration

```
Configuration Files (cfg): /home/user/mame/configs
NVRAM: /home/user/mame/nvram
Input Configuration (input): /home/user/mame/inputs
Save States (state): /home/user/mame/states
Hard Disk Diffs (diff): /home/user/mame/diffs
Comment Files (comment): /home/user/mame/comments
```

**Note:** All 6 folder types are fully supported and will be passed as command-line arguments to MAME when launching games.

## Benefits

### 1. Centralized Management
- All MAME folder configurations are managed through MAMEUIx
- No need to manually edit MAME configuration files
- Consistent interface across all MAME folder types

### 2. Persistent Settings
- Configurations are saved and restored between sessions
- Settings persist across MAMEUIx restarts
- Automatic backup of configuration data

### 3. Flexible Configuration
- Configure some folders while leaving others at MAME defaults
- Mix custom and default locations as needed
- Easy to revert to default settings

### 4. User-Friendly Interface
- Clear descriptions and validation for each folder type
- Visual feedback on path validity
- Easy folder selection with browse dialogs

### 5. Cross-Platform Support
- Works on all platforms where MAMEUIx runs
- Handles platform-specific path separators
- Compatible with different MAME versions

## Technical Implementation

### Command-Line Integration

When launching games, MAMEUIx automatically passes the configured folder paths as command-line arguments to MAME:

```bash
mame -cfg_directory /path/to/configs \
     -nvram_directory /path/to/nvram \
     -input_directory /path/to/inputs \
     -state_directory /path/to/states \
     -diff_directory /path/to/diffs \
     -comment_directory /path/to/comments \
     gamename
```

### Configuration Storage

Folder configurations are stored in MAMEUIx's configuration file alongside other settings:

```toml
[app_config]
cfg_path = "/home/user/mame/configs"
nvram_path = "/home/user/mame/nvram"
input_path = "/home/user/mame/inputs"
state_path = "/home/user/mame/states"
diff_path = "/home/user/mame/diffs"
comment_path = "/home/user/mame/comments"
```

### Validation

- Paths are validated for existence and directory status
- Invalid paths are highlighted in red
- Empty paths are treated as "use MAME default"
- Relative paths are supported and resolved correctly

## Troubleshooting

### Common Issues

1. **"Unknown option" errors**
   - Some MAME versions may not support all directory options
   - Check MAME version compatibility
   - Use `mame -showusage` to see available options

2. **Permission errors**
   - Ensure MAME has write access to configured directories
   - Create directories if they don't exist
   - Check file system permissions

3. **Path not found**
   - Verify the configured path exists
   - Use absolute paths for better reliability
   - Check for typos in path names

### Best Practices

1. **Use absolute paths** for better reliability
2. **Create directories** before configuring them
3. **Test configurations** with a simple game first
4. **Backup configurations** before making major changes
5. **Use descriptive folder names** for better organization

## Future Enhancements

Potential improvements for future versions:

1. **Automatic directory creation** when configuring paths
2. **Template configurations** for common setups
3. **Import/export** of folder configurations
4. **Advanced validation** with MAME compatibility checking
5. **Folder size monitoring** and cleanup suggestions

## Conclusion

The MAME folder configuration feature provides users with complete control over MAME's file organization while maintaining ease of use through an intuitive interface. This feature enhances MAMEUIx's role as a comprehensive MAME management tool. 