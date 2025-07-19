// src/models/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use crate::graphics::GraphicsConfig;
use super::{GameStats, Preferences, FilterSettings, SortColumn, SortDirection};

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


// Theme controls the visual appearance of the frontend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    DarkBlue,        // Professional dark theme with blue accents
    DarkGrey,        // Neutral dark theme
    ArcadePurple,    // Retro arcade-inspired theme
    LightClassic,    // Classic light theme
    NeonGreen,       // Cyberpunk green theme
    SunsetOrange,    // Warm orange theme
    OceanBlue,       // Deep ocean blue theme
    MidnightBlack,   // Pure black theme
    ForestGreen,     // Nature-inspired green theme
    RetroAmber,      // Vintage amber theme
}

impl Theme {
    /// Apply this theme's colors to the UI context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            Theme::LightClassic => egui::Visuals::light(),
            _ => egui::Visuals::dark(),
        };

        match self {
            Theme::DarkBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 25, 40);
                visuals.window_fill = egui::Color32::from_rgb(25, 30, 45);
                visuals.selection.bg_fill = egui::Color32::from_rgb(60, 80, 120);
                visuals.hyperlink_color = egui::Color32::from_rgb(100, 150, 255);
            }
            Theme::DarkGrey => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 35, 40);
                visuals.selection.bg_fill = egui::Color32::from_rgb(60, 60, 70);
                visuals.hyperlink_color = egui::Color32::from_rgb(120, 120, 140);
            }
            Theme::ArcadePurple => {
                visuals.panel_fill = egui::Color32::from_rgb(25, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 25, 45);
                visuals.selection.bg_fill = egui::Color32::from_rgb(100, 50, 150);
                visuals.hyperlink_color = egui::Color32::from_rgb(180, 100, 255);
            }
            Theme::LightClassic => {
                visuals.panel_fill = egui::Color32::from_rgb(240, 240, 245);
                visuals.window_fill = egui::Color32::from_rgb(245, 245, 250);
                visuals.selection.bg_fill = egui::Color32::from_rgb(200, 220, 255);
                visuals.hyperlink_color = egui::Color32::from_rgb(0, 100, 200);
            }
            Theme::NeonGreen => {
                visuals.panel_fill = egui::Color32::from_rgb(15, 25, 15);
                visuals.window_fill = egui::Color32::from_rgb(20, 30, 20);
                visuals.selection.bg_fill = egui::Color32::from_rgb(0, 100, 0);
                visuals.hyperlink_color = egui::Color32::from_rgb(0, 255, 100);
            }
            Theme::SunsetOrange => {
                visuals.panel_fill = egui::Color32::from_rgb(35, 20, 15);
                visuals.window_fill = egui::Color32::from_rgb(45, 25, 20);
                visuals.selection.bg_fill = egui::Color32::from_rgb(150, 80, 40);
                visuals.hyperlink_color = egui::Color32::from_rgb(255, 150, 80);
            }
            Theme::OceanBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(10, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(15, 25, 40);
                visuals.selection.bg_fill = egui::Color32::from_rgb(30, 60, 100);
                visuals.hyperlink_color = egui::Color32::from_rgb(80, 160, 255);
            }
            Theme::MidnightBlack => {
                visuals.panel_fill = egui::Color32::from_rgb(10, 10, 10);
                visuals.window_fill = egui::Color32::from_rgb(15, 15, 15);
                visuals.selection.bg_fill = egui::Color32::from_rgb(40, 40, 40);
                visuals.hyperlink_color = egui::Color32::from_rgb(100, 100, 100);
            }
            Theme::ForestGreen => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 35, 20);
                visuals.window_fill = egui::Color32::from_rgb(25, 40, 25);
                visuals.selection.bg_fill = egui::Color32::from_rgb(60, 100, 60);
                visuals.hyperlink_color = egui::Color32::from_rgb(120, 200, 120);
            }
            Theme::RetroAmber => {
                visuals.panel_fill = egui::Color32::from_rgb(25, 20, 10);
                visuals.window_fill = egui::Color32::from_rgb(35, 25, 15);
                visuals.selection.bg_fill = egui::Color32::from_rgb(100, 80, 40);
                visuals.hyperlink_color = egui::Color32::from_rgb(255, 200, 100);
            }
        }

        ctx.set_visuals(visuals);
    }

    /// Get a human-readable name for the theme
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::DarkBlue => "Dark Blue",
            Theme::DarkGrey => "Dark Grey",
            Theme::ArcadePurple => "Arcade Purple",
            Theme::LightClassic => "Light Classic",
            Theme::NeonGreen => "Neon Green",
            Theme::SunsetOrange => "Sunset Orange",
            Theme::OceanBlue => "Ocean Blue",
            Theme::MidnightBlack => "Midnight Black",
            Theme::ForestGreen => "Forest Green",
            Theme::RetroAmber => "Retro Amber",
        }
    }

    /// Get a description for the theme
    pub fn description(&self) -> &'static str {
        match self {
            Theme::DarkBlue => "Professional dark theme with blue accents",
            Theme::DarkGrey => "Neutral dark theme for easy reading",
            Theme::ArcadePurple => "Retro arcade-inspired purple theme",
            Theme::LightClassic => "Classic light theme for traditional look",
            Theme::NeonGreen => "Cyberpunk green theme with neon accents",
            Theme::SunsetOrange => "Warm orange theme inspired by sunsets",
            Theme::OceanBlue => "Deep ocean blue theme for calm experience",
            Theme::MidnightBlack => "Pure black theme for OLED displays",
            Theme::ForestGreen => "Nature-inspired green theme",
            Theme::RetroAmber => "Vintage amber theme like old terminals",
        }
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
    pub expand: f32,      // Expand/collapse arrow column
    pub favorite: f32,    // Favorite star column
    pub icon: f32,        // Game icon column
    pub status: f32,      // Status column
    pub game: f32,
    pub play_count: f32,
    pub manufacturer: f32,
    pub year: f32,
    pub driver: f32,
    pub driver_status: f32,
    pub category: f32,
    pub rom: f32,
    pub chd: f32,
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            expand: 25.0,       // Default width for expand column
            favorite: 25.0,     // Default width for favorite column
            icon: 40.0,         // Default width for icon column
            status: 30.0,       // Default width for status column
            game: 300.0,        // Default width for game column
            play_count: 60.0,   // Default width for play count column
            manufacturer: 200.0, // Default width for manufacturer column
            year: 60.0,         // Default width for year column
            driver: 80.0,       // Default width for driver column
            driver_status: 120.0, // Default width for driver status column
            category: 100.0,    // Default width for category column
            rom: 80.0,          // Default width for ROM column
            chd: 60.0,          // Default width for CHD column
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
            "expand" => self.expand,
            "favorite" => self.favorite,
            "icon" => self.icon,
            "status" => self.status,
            "game" => self.game,
            "play_count" => self.play_count,
            "manufacturer" => self.manufacturer,
            "year" => self.year,
            "driver" => self.driver,
            "driver_status" => self.driver_status,
            "category" => self.category,
            "rom" => self.rom,
            "chd" => self.chd,
            _ => 100.0, // Default fallback
        }
    }
    
    /// Set width for a specific column type
    pub fn set_width(&mut self, column_type: &str, width: f32) {
        match column_type {
            "expand" => self.expand = width,
            "favorite" => self.favorite = width,
            "icon" => self.icon = width,
            "status" => self.status = width,
            "game" => self.game = width,
            "play_count" => self.play_count = width,
            "manufacturer" => self.manufacturer = width,
            "year" => self.year = width,
            "driver" => self.driver = width,
            "driver_status" => self.driver_status = width,
            "category" => self.category = width,
            "rom" => self.rom = width,
            "chd" => self.chd = width,
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
    
    // Additional MAME Search Paths
    pub ctrlr_path: Option<PathBuf>,     // Controller definitions directory
    pub crosshair_path: Option<PathBuf>, // Crosshair files directory
    pub font_path: Option<PathBuf>,      // Font files directory
    pub plugins_path: Option<PathBuf>,   // Plugin files directory
    pub language_path: Option<PathBuf>,  // UI translation files directory
    pub sw_path: Option<PathBuf>,        // Loose software directory
    pub hash_path: Option<PathBuf>,      // Software definition files directory
    pub ini_path: Option<PathBuf>,       // INI files directory
    pub home_path: Option<PathBuf>,      // Base folder for plugin data (read/write)
    
    // History and DAT files
    pub history_path: Option<PathBuf>,    // History XML file path
    pub mameinfo_dat_path: Option<PathBuf>,  // mameinfo.dat file path
    pub hiscore_dat_path: Option<PathBuf>,   // hiscore.dat file path
    pub gameinit_dat_path: Option<PathBuf>,  // gameinit.dat file path
    pub command_dat_path: Option<PathBuf>,   // command.dat file path
    pub catver_ini_path: Option<PathBuf>,    // catver.ini file path for category support

    // MAME Internal Folders Configuration
    pub cfg_path: Option<PathBuf>,        // Configuration files directory
    pub nvram_path: Option<PathBuf>,      // Non-volatile RAM directory
    pub input_path: Option<PathBuf>,      // Input configuration directory
    pub state_path: Option<PathBuf>,      // Save state directory
    pub diff_path: Option<PathBuf>,       // Hard disk diff directory
    pub comment_path: Option<PathBuf>,    // Comment files directory

    // Filter and display settings
    pub filter_settings: FilterSettings,  // Game filtering options
    pub sort_column: SortColumn,         // Current sort column
    pub sort_direction: SortDirection,   // Ascending or descending

    // Game-specific settings
    pub game_preferred_mame: HashMap<String, usize>, // Preferred MAME for each game
    pub favorite_games: HashSet<String>,            // User's favorite games
    pub game_stats: HashMap<String, GameStats>,     // Play statistics per game
    pub game_properties: HashMap<String, super::game_properties::GameProperties>, // Per-game properties
    pub default_game_properties: super::game_properties::GameProperties, // Default properties for all games
    
    // Hidden categories - categories that should not be shown in the game list
    pub hidden_categories: HashSet<String>,

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
    pub bgfx_path: Option<PathBuf>,      // BGFX shader path

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
            
            // Additional MAME Search Paths
            ctrlr_path: None,
            crosshair_path: None,
            font_path: None,
            plugins_path: None,
            language_path: None,
            sw_path: None,
            hash_path: None,
            ini_path: None,
            home_path: None,
            
            // History and DAT files
            history_path: None,
            mameinfo_dat_path: None,
            hiscore_dat_path: None,
            gameinit_dat_path: None,
            command_dat_path: None,
            catver_ini_path: None,

            // MAME Internal Folders Configuration
            cfg_path: None,
            nvram_path: None,
            input_path: None,
            state_path: None,
            diff_path: None,
            comment_path: None,

            // Use default filter settings
            filter_settings: FilterSettings::default(),
            sort_column: SortColumn::default(),
            sort_direction: SortDirection::default(),

            // Empty game-specific maps
            game_preferred_mame: HashMap::new(),
            favorite_games: HashSet::new(),
            game_stats: HashMap::new(),
            game_properties: HashMap::new(),
            default_game_properties: super::game_properties::GameProperties::default(),
            
            // Initialize empty hidden categories
            hidden_categories: HashSet::new(),

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
            bgfx_path: None,
    
            preferences: Preferences::default(),  // Add this line
        }
    }
}
