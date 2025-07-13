// src/models/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use crate::graphics::GraphicsConfig;
use crate::models::GameStats; // Important: Import GameStats from models
use crate::models::Preferences;

// MameExecutable represents a MAME emulator executable
// Unlike simple paths, this contains metadata about each MAME version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MameExecutable {
    pub name: String,        // User-friendly name for this MAME version
    pub path: String,        // Full path to the executable
    pub version: String,     // MAME version number
    pub total_games: usize,  // Total number of games this version supports
    pub working_games: usize, // Number of games that work properly
}

// VideoSettings controls how MAME displays games
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub video_backend: String,    // OpenGL, DirectX, etc.
    pub window_mode: bool,        // Windowed vs fullscreen
    pub maximize: bool,           // Start maximized
    pub wait_vsync: bool,         // Wait for vertical sync
    pub sync_refresh: bool,       // Sync to monitor refresh
    pub prescale: u8,             // Pre-scaling factor
    pub keep_aspect: bool,        // Maintain original aspect ratio
    pub filter: bool,             // Enable bilinear filtering
    pub num_screens: u8,          // Number of screens to use
    pub custom_args: String,      // Additional command-line arguments
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            video_backend: "auto".to_string(),
            window_mode: true,
            maximize: false,
            wait_vsync: false,
            sync_refresh: false,
            prescale: 0,
            keep_aspect: true,
            filter: true,
            num_screens: 1,
            custom_args: String::new(),
        }
    }
}

// Theme controls the visual appearance of the frontend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    DarkBlue,     // Professional dark theme with blue accents
    DarkGrey,     // Neutral dark theme
    ArcadePurple, // Retro arcade-inspired theme
}

impl Theme {
    /// Apply this theme's colors to the UI context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        match self {
            Theme::DarkBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 25, 40);
                visuals.window_fill = egui::Color32::from_rgb(25, 30, 45);
            }
            Theme::DarkGrey => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 35, 40);
            }
            Theme::ArcadePurple => {
                visuals.panel_fill = egui::Color32::from_rgb(25, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 25, 45);
                visuals.selection.bg_fill = egui::Color32::from_rgb(100, 50, 150);
            }
        }

        ctx.set_visuals(visuals);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::DarkBlue
    }
}

// Column width settings for the game list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnWidths {
    pub game: f32,
    pub manufacturer: f32,
    pub year: f32,
    pub driver: f32,
    pub category: f32,
    pub rom: f32,
    pub play_count: f32,
    pub status: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            game: 300.0,        // Default width for game column
            manufacturer: 200.0, // Default width for manufacturer column
            year: 60.0,         // Default width for year column
            driver: 80.0,       // Default width for driver column
            category: 100.0,    // Default width for category column
            rom: 100.0,         // Default width for ROM column
            play_count: 60.0,   // Default width for play count column
            status: 80.0,       // Default width for status column
        }
    }
}

impl ColumnWidths {
    /// Reset all column widths to default values
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }
    
    /// Get width for a specific column type
    pub fn get_width(&self, column_type: &str) -> f32 {
        match column_type {
            "game" => self.game,
            "manufacturer" => self.manufacturer,
            "year" => self.year,
            "driver" => self.driver,
            "category" => self.category,
            "rom" => self.rom,
            "play_count" => self.play_count,
            "status" => self.status,
            _ => 100.0, // Default fallback
        }
    }
    
    /// Set width for a specific column type
    pub fn set_width(&mut self, column_type: &str, width: f32) {
        match column_type {
            "game" => self.game = width,
            "manufacturer" => self.manufacturer = width,
            "year" => self.year = width,
            "driver" => self.driver = width,
            "category" => self.category = width,
            "rom" => self.rom = width,
            "play_count" => self.play_count = width,
            "status" => self.status = width,
            _ => {}, // Ignore unknown column types
        }
    }
}

// AppConfig is the main configuration struct that stores all settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    // MAME executable management
    pub mame_executables: Vec<MameExecutable>,  // List of available MAME versions
    pub selected_mame_index: usize,              // Currently selected MAME

    // ROM directory configuration
    // Note: We have both rom_dirs and rom_paths for compatibility
    // Eventually these should be consolidated
    pub rom_dirs: Vec<PathBuf>,      // Primary ROM directories
    pub rom_paths: Vec<PathBuf>,     // Additional ROM paths (for UI compatibility)
    pub sample_paths: Vec<PathBuf>,  // Sound sample directories
    pub extra_rom_dirs: Vec<PathBuf>, // Extra ROM search paths
    pub extra_asset_dirs: Vec<PathBuf>, // Artwork, icons, etc.
    
    // MAME Support Files paths
    pub artwork_path: Option<PathBuf>,   // Artwork directory
    pub snap_path: Option<PathBuf>,      // Screenshots directory
    pub cabinet_path: Option<PathBuf>,   // Cabinet artwork directory
    pub title_path: Option<PathBuf>,     // Title screens directory
    pub flyer_path: Option<PathBuf>,     // Promotional flyers directory
    pub marquee_path: Option<PathBuf>,   // Marquee artwork directory
    pub cheats_path: Option<PathBuf>,    // Cheat files directory
    pub icons_path: Option<PathBuf>,     // Icons directory
    
    // History and DAT files
    pub history_path: Option<PathBuf>,    // History XML file path
    pub mameinfo_dat_path: Option<PathBuf>,  // mameinfo.dat file path
    pub hiscore_dat_path: Option<PathBuf>,   // hiscore.dat file path
    pub gameinit_dat_path: Option<PathBuf>,  // gameinit.dat file path
    pub command_dat_path: Option<PathBuf>,   // command.dat file path

    // Filter and display settings
    pub filter_settings: super::FilterSettings,  // Game filtering options
    pub sort_column: super::SortColumn,         // Current sort column
    pub sort_direction: super::SortDirection,   // Ascending or descending

    // Game-specific settings
    pub game_preferred_mame: HashMap<String, usize>, // Preferred MAME for each game
    pub favorite_games: HashSet<String>,            // User's favorite games
    pub game_stats: HashMap<String, GameStats>,     // Play statistics per game

    // UI preferences
    pub show_filters: bool,          // Show filter panel
    pub selected_rom: Option<String>, // Currently selected ROM
    pub theme: Theme,                // Visual theme
    pub show_rom_icons: bool,        // Display game icons
    pub icon_size: u32,              // Icon size in pixels
    pub max_cached_icons: usize,     // Maximum icons to keep in memory
    pub column_widths: ColumnWidths, // Column width settings

    // MAME audit settings
    pub use_mame_audit: bool,                    // Use MAME's built-in audit
    pub mame_audit_times: HashMap<String, String>, // Last audit time per directory
    pub assume_merged_sets: bool,                // Assume ROMs are merged sets

    // Graphics and video
    pub graphics_config: GraphicsConfig, // Graphics backend configuration
    pub video_settings: VideoSettings,   // MAME video settings
    pub preferences: Preferences,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // Initialize with empty MAME list
            mame_executables: vec![],
            selected_mame_index: 0,

            // Initialize all path lists as empty
            rom_dirs: vec![],
            rom_paths: vec![],      // Important: This was missing in original
            sample_paths: vec![],   // Important: This was missing in original
            extra_rom_dirs: vec![],
            extra_asset_dirs: vec![],
            
            // MAME Support Files paths
            artwork_path: None,
            snap_path: None,
            cabinet_path: None,
            title_path: None,
            flyer_path: None,
            marquee_path: None,
            cheats_path: None,
            icons_path: None,
            
            // History and DAT files
            history_path: None,
            mameinfo_dat_path: None,
            hiscore_dat_path: None,
            gameinit_dat_path: None,
            command_dat_path: None,

            // Use default filter settings
            filter_settings: super::FilterSettings::default(),
            sort_column: super::SortColumn::default(),
            sort_direction: super::SortDirection::default(),

            // Empty game-specific maps
            game_preferred_mame: HashMap::new(),
            favorite_games: HashSet::new(),
            game_stats: HashMap::new(),

            // Default UI settings
            show_filters: false,
            selected_rom: None,
            theme: Theme::default(),
            show_rom_icons: true,
            icon_size: 32,
            max_cached_icons: 500,
            column_widths: ColumnWidths::default(),

            // Audit disabled by default
            use_mame_audit: false,
            mame_audit_times: HashMap::new(),
            assume_merged_sets: false,

            // Default graphics settings
            graphics_config: GraphicsConfig::default(),
            video_settings: VideoSettings::default(),
            preferences: Preferences::default(),  // Add this line
        }
    }
}
