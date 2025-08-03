# MAMEUIX Interface Redesign Concept

## Overview
This document outlines the proposed redesign of the MAMEUIX interface, featuring floating Settings and Filters windows with improved navigation and real-time functionality.

## New Layout Design

### Main Window Layout (Updated User Concept)
```
┌─────────────────────────────────────────────────────────────────┐
│ File  Settings  Filters  Tools  Help                        [X] │
│ 🔄 Refresh  🔍 Search: [________________________]              │
├─────────────────────────┬───────────────────────────────────────┤
│                         │                                       │
│  [ Game List (60%) ]    │   +───────────────────────────────+   │
│  +─────────────────────+ │   │        Artwork Panel          │   │
│  | ▼ ★ [I] Pac-Man   5 1980│ │   │      (Top 50%, 40% Width)   │   │
│  | ▶ ☆ [I] Galaga    12 1981│ │   │   +─────────────────────+  │   │
│  |   ☆ [I] Ms. Pac-Man 3 1981│ │   │   │  [Screenshot Image] │  │   │
│  |   ★ [I] Ponpoko     1 1982│ │   │   │                      │   │
│  | ▼ ☆ [I] Donkey Kong 8 1981│ │   │   │  Game Information   │   │
│  |   ...                     │ │   │   │                      │   │
│  |                           │ │   │   +─────────────────────+  │   │
│  | (Scrollable game list)    │ │   +───────────────────────────────+   │
│  +─────────────────────+ │                                       │
│                         │   +───────────────────────────────+   │
│                         │   │        History Panel          │   │
│                         │   │     (Bottom 50%, 40% Width)   │   │
│                         │   │   +─────────────────────+  │   │
│                         │   │   │  [History & Info Text]│  │   │
│                         │   │   │                      │  │   │
│                         │   │   │  Year: 1980          │  │   │
│                         │   │   │  Manufacturer: Namco │  │   │
│                         │   │   │  Driver: pacman      │  │   │
│                         │   │   │  Category: Maze       │  │   │
│                         │   │   │  ...                 │  │   │
│                         │   │   +─────────────────────+  │   │
│                         │   +───────────────────────────────+   │
└─────────────────────────┴───────────────────────────────────────┘
```

### Floating Filters Window (Real-time Updates)
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔍 Filters                                    [─] [□] [✕]     │
├─────────────────────────────────────────────────────────────────┤
│                                                               │
│ 🔍 Search: [_______pacman________]                            │
│ Search by: [Game Title ▼]                                     │
│                                                               │
│ Show:                                                         │
│ ( ) All Games   (●) Available                                 │
│ ( ) Parents     ( ) Working                                   │
│ ( ) Missing     ( ) Favorites                                 │
│                                                               │
│ 🔧 Hardware                                                   │
│ CPU: [Z80 ▼]  Sound: [All ▼]                                 │
│ Device: [All ▼]  Category: [All ▼]                           │
│                                                               │
│ Options:                                                      │
│ ☑ Hide non-games (devices/BIOS)                              │
│ ☑ Show only verified ROMs                                    │
│                                                               │
│ [Reset Filters] [Save Preset...] [Load Preset ▼]              │
│                                                               │
│ Results: 1,247 games found (filtered from 48,247 total)      │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### Floating Settings Window (Vertical Navigation)
```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙️ Settings                                    [─] [□] [✕]     │
├─────────────────────────┬───────────────────────────────────────┤
│                         │                                       │
│  📁 Directories         │  +─────────────────────────────────+ │
│                         │  │ MAME Executables                │ │
│  ⚙️ Preferences         │  │ [Path 1] [Browse]               │ │
│                         │  │ [Path 2] [Browse]               │ │
│  🎮 Default Game        │  │ [Add] [Remove]                  │ │
│     Properties          │  │                                 │ │
│                         │  │ ROM Directories                 │ │
│                         │  │ [Path A] [Browse]               │ │
│                         │  │ [Path B] [Browse]               │ │
│                         │  │ [Add] [Remove]                  │ │
│                         │  │                                 │ │
│                         │  │ Artwork Path: [________] [..]   │ │
│                         │  │ CHD Path: [________] [..]       │ │
│                         │  │ History/DAT: [________] [..]    │ │
│                         │  │                                 │ │
│                         │  │ [Apply] [OK] [Cancel]           │ │
│                         │  +─────────────────────────────────+ │
│                         │                                       │
└─────────────────────────┴───────────────────────────────────────┘
```

### Settings Navigation Examples

#### Directories Panel
```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙️ Settings                                    [─] [□] [✕]     │
├─────────────────────────┬───────────────────────────────────────┤
│                         │                                       │
│  📁 Directories         │  +─────────────────────────────────+ │
│                         │  │ 📁 MAME Executables             │ │
│  ⚙️ Preferences         │  │ [Path 1] [Browse]               │ │
│                         │  │ [Path 2] [Browse]               │ │
│  🎮 Default Game        │  │ [Add] [Remove]                  │ │
│     Properties          │  │                                 │ │
│                         │  │ 📁 ROM Directories              │ │
│                         │  │ [Path A] [Browse]               │ │
│                         │  │ [Path B] [Browse]               │ │
│                         │  │ [Add] [Remove]                  │ │
│                         │  │                                 │ │
│                         │  │ 📁 Artwork Path                 │ │
│                         │  │ [________] [Browse]             │ │
│                         │  │                                 │ │
│                         │  │ 📁 CHD Path                     │ │
│                         │  │ [________] [Browse]             │ │
│                         │  │                                 │ │
│                         │  │ [Apply] [OK] [Cancel]           │ │
│                         │  +─────────────────────────────────+ │
│                         │                                       │
└─────────────────────────┴───────────────────────────────────────┘
```

#### Preferences Panel
```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙️ Settings                                    [─] [□] [✕]     │
├─────────────────────────┬───────────────────────────────────────┤
│                         │                                       │
│  📁 Directories         │  +─────────────────────────────────+ │
│                         │  │ ⚙️ General Settings             │ │
│  ⚙️ Preferences         │  │ ☑ Show Icons                    │ │
│                         │  │ ☑ Show Favorites                │ │
│  🎮 Default Game        │  │ ☑ Auto-refresh                  │ │
│     Properties          │  │ ☑ Remember Window Size          │ │
│                         │  │ ☑ Enable Categories             │ │
│                         │  │ ☑ Column Width Persistence      │ │
│                         │  │                                 │ │
│                         │  │ 🎨 Theme Selection              │ │
│                         │  │ 🎨 Dark Blue  🎨 Neon Green     │ │
│                         │  │ 🎨 Arcade Purple 🎨 Light Classic│ │
│                         │  │ 🎨 Retro Orange 🎨 Cyber Blue   │ │
│                         │  │                                 │ │
│                         │  │ [Apply] [OK] [Cancel]           │ │
│                         │  +─────────────────────────────────+ │
│                         │                                       │
└─────────────────────────┴───────────────────────────────────────┘
```

#### Default Game Properties Panel
```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙️ Settings                                    [─] [□] [✕]     │
├─────────────────────────┬───────────────────────────────────────┤
│                         │                                       │
│  📁 Directories         │  +─────────────────────────────────+ │
│                         │  │ 🎮 Video Settings               │ │
│  ⚙️ Preferences         │  │ BGFX Backend: [Auto ▼]          │ │
│                         │  │ Shader: [None ▼]                │ │
│  🎮 Default Game        │  │ Integer Scale: [1x ▼]           │ │
│     Properties          │  │                                 │ │
│                         │  │ 🎮 Performance                  │ │
│                         │  │ Auto-frameskip: ☑              │ │
│                         │  │ Frameskip: [0 ▼]               │ │
│                         │  │ Emulation Speed: [1.0x ▼]       │ │
│                         │  │                                 │ │
│                         │  │ 🎮 Advanced                     │ │
│                         │  │ [Shader Manager...]             │ │
│                         │  │ [Performance Monitor...]        │ │
│                         │  │ [Hardware Filtering...]         │ │
│                         │  │                                 │ │
│                         │  │ [Apply] [OK] [Cancel]           │ │
│                         │  +─────────────────────────────────+ │
│                         │                                       │
└─────────────────────────┴───────────────────────────────────────┘
```

## Key Design Features

### 1. Main Window Layout
- **Top Toolbar**: Main tabs + Quick action buttons (Refresh, Search)
- **Simple Search**: Real-time search by game name only
- **60/40 Split**: Game list (60%) + Info panels (40%)
- **Dual Info Panels**: Artwork (50% top) + History (50% bottom)

### 2. Floating Filters Window
- **Real-time Updates**: Changes immediately reflect in main window
- **Advanced Search**: Multiple search modes (Game Title, Manufacturer, Fuzzy, Full-Text, Regex)
- **Hardware Filtering**: CPU, Sound, Device, Category filters
- **Filter Presets**: Save and load common filter combinations
- **Live Results**: Shows filtered count in real-time

### 3. Floating Settings Window
- **Vertical Navigation**: Left sidebar with 3 main sections
- **Dynamic Content**: Right panel changes based on selection
- **Organized Sections**:
  - **📁 Directories**: MAME, ROM, CHD, Artwork paths
  - **⚙️ Preferences**: General settings, themes, UI options
  - **🎮 Default Game Properties**: Video, performance, advanced settings

## Layout Specifications

### Main Window Proportions
- **Game List Panel**: 60% width (left side)
- **Right Panel**: 40% width (right side)
  - **Artwork Panel**: 50% height (top half)
  - **History Panel**: 50% height (bottom half)

### Top Toolbar Elements
- **Main Tabs**: File, Settings, Filters, Tools, Help
- **Quick Actions**: Refresh button, Simple search (game name only)
- **Search Behavior**: Real-time filtering as user types

### Game List Features
- **Expandable Rows**: ▼/▶ for games with multiple versions
- **Favorite Stars**: ★/☆ for favorite games
- **Game Icons**: [I] for game artwork thumbnails
- **Status Indicators**: ✅❌⚠️❓ for ROM verification status
- **Play Count**: [number] showing how many times played
- **Resizable Columns**: All columns can be resized

### Right Panel Content
- **Artwork Panel (Top 50%)**:
  - Game screenshot/image
  - Basic game information (name, year, manufacturer)
  - ROM status indicator
  - Quick action buttons

- **History Panel (Bottom 50%)**:
  - Detailed game history and description
  - Technical information (driver, category, etc.)
  - Controls and gameplay information
  - Scrollable text area

## Floating Window Features

### Filters Window Behavior
- **Real-time Updates**: All changes immediately reflect in main game list
- **Movable & Resizable**: Can be positioned anywhere on screen
- **Search Modes**: Game Title, Manufacturer, Fuzzy, Full-Text, Regex
- **Filter Presets**: Save and load common filter combinations
- **Hardware Filtering**: Advanced filtering by CPU, sound, device types
- **Live Statistics**: Shows filtered results count

### Settings Window Behavior
- **Vertical Navigation**: Left sidebar with 3 main sections
- **Dynamic Content**: Right panel changes based on left selection
- **Organized Layout**: Clear separation of different setting types
- **Movable & Resizable**: Can be positioned anywhere on screen
- **Persistent State**: Remembers position, size, and last selected section

### Window States
```
┌─ Minimized State ──────────────────────────────────────────────┐
│ [🔍] Filters (minimized to taskbar)                            │
│ [⚙️] Settings (minimized to taskbar)                           │
└─────────────────────────────────────────────────────────────────┘

┌─ Always on Top State ──────────────────────────────────────────┐
│ 🔍 Filters ★                                    [─] [□] [✕]   │
│ ⚙️ Settings ★                                    [─] [□] [✕]   │
│ (Windows stay above all other applications)                    │
└─────────────────────────────────────────────────────────────────┘

┌─ Outside Main Window ──────────────────────────────────────────┐
│                    Main MAMEUIX Window                        │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ File  Settings  Filters  Tools  Help                        │ │
│ │                                                             │ │
│ │                    Game List                                │ │
│ │                                                             │ │
│ └─────────────────────────────────────────────────────────────┘ │
│                                                               │
│ 🔍 Filters Window          ⚙️ Settings Window                │
│ ┌─────────────────────┐   ┌─────────────────────────────┐     │
│ │ Search & Filters    │   │ Directories | Preferences   │     │
│ │                     │   │                             │     │
│ └─────────────────────┘   └─────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────┘
```

## Tab Navigation Structure

### Main Tabs
```
┌─────────────────────────────────────────────────────────────────┐
│ [File] [Settings] [Filters] [Tools] [Help]                      │
└─────────────────────────────────────────────────────────────────┘
```

**File Tab:**
- New Project
- Open Project
- Save Project
- Export Game List
- Import Settings
- Recent Files
- Exit

**Settings Tab:**
- Opens floating Settings window with vertical navigation
- Quick access to common settings
- Settings window state indicator

**Filters Tab:**
- Opens floating Filters window with real-time updates
- Advanced search and filtering capabilities
- Filter presets and hardware filtering

**Tools Tab:**
- ROM Verification
- Shader Manager
- Performance Monitor
- Hardware Filtering
- Plugin Detection
- Category Manager

**Help Tab:**
- About MAMEUIX
- Documentation
- Keyboard Shortcuts
- Check for Updates
- Report Bug
- Donate

## Enhanced User Experience

### Real-time Filtering
1. **Instant Updates**: Changes in Filters window immediately update main game list
2. **Live Search**: Type in search box and see results instantly
3. **Visual Feedback**: Filter status shown in main window
4. **Performance**: Efficient filtering for large game collections

### Settings Navigation
1. **Vertical Layout**: Clear left sidebar navigation
2. **Dynamic Content**: Right panel changes based on selection
3. **Organized Sections**: Logical grouping of related settings
4. **Quick Access**: Easy switching between different setting types

### Keyboard Shortcuts
- **Ctrl+,**: Open Settings window
- **Ctrl+Shift+,**: Open Settings in new window
- **Ctrl+F**: Open Filters window
- **Alt+S**: Focus Settings tab
- **Alt+F**: Focus Filters tab
- **Alt+T**: Focus Tools tab
- **F1**: Help
- **Ctrl+Q**: Exit application

### Window Management
- **Snap to Edges**: Windows can snap to screen edges
- **Snap to Other Windows**: Settings/Filters can snap to main window
- **Grid Layout**: Optional grid snapping for precise positioning
- **Window Groups**: Group related windows together
- **Workspace Memory**: Remembers window layouts between sessions

## Implementation Benefits

### 1. Flexibility
- **Multi-monitor support**: Windows can be on different screens
- **Custom workflows**: Users can arrange windows as they prefer
- **Context switching**: Keep settings/filters visible while working
- **Screen real estate**: Better use of available space

### 2. Productivity
- **Quick access**: Settings and filters always available
- **Parallel work**: Can adjust settings while browsing games
- **Real-time filtering**: Instant results without waiting
- **Reference mode**: Keep settings open for reference

### 3. User Control
- **Personal preference**: Each user can arrange windows their way
- **Task-specific layouts**: Different arrangements for different tasks
- **Accessibility**: Larger windows for better visibility
- **Workflow optimization**: Custom layouts for specific workflows

### 4. Modern UI Patterns
- **Familiar behavior**: Users expect floating windows
- **Professional appearance**: More polished and modern interface
- **Consistent with OS**: Follows operating system window management
- **Touch-friendly**: Better support for touch devices

## Technical Considerations

### 1. Window Management
- **Cross-platform**: Works on Windows, macOS, and Linux
- **Window state persistence**: Saves position, size, and state
- **Z-order management**: Proper window stacking behavior
- **Focus management**: Proper focus handling between windows

### 2. Performance
- **Efficient rendering**: Only render visible content
- **Lazy loading**: Load content on demand
- **Memory management**: Proper cleanup of window resources
- **Event handling**: Efficient event propagation between windows

### 3. Real-time Updates
- **Data synchronization**: Changes propagate immediately
- **UI responsiveness**: Smooth updates without blocking
- **Filter optimization**: Efficient filtering algorithms
- **State management**: Consistent state across all windows

### 4. Accessibility
- **Keyboard navigation**: Full keyboard support for all controls
- **Screen reader support**: Proper ARIA labels and descriptions
- **High contrast**: Support for high contrast themes
- **Font scaling**: Proper scaling of text and controls

This design provides maximum flexibility while maintaining a clean and organized interface structure that users will find intuitive and powerful. 