// src/models/mod.rs
// File ini mendefinisikan semua model data yang digunakan aplikasi
// Termasuk struktur baru untuk optimasi performance

pub mod game;
pub mod config;
pub mod filters;
pub mod game_properties;

// Re-export everything from submodules
pub use game::*;
pub use config::*;
pub use filters::*;
pub use game_properties::*;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Loading stage and message types for background operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadingStage {
    Idle,
    LoadingMame,
    ScanningRoms,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
pub enum LoadingMessage {
    MameLoadStarted,
    MameLoadProgress(String),
    MameLoadComplete(Vec<crate::models::Game>, Vec<String>),
    MameLoadFailed(String),
    RomScanStarted,
    RomScanProgress(usize, usize),
    RomScanComplete(Vec<crate::models::Game>),
    RomScanFailed(String),
}

// GameIndex for managing game data efficiently
#[derive(Debug, Clone)]
pub struct GameIndex {
    pub games: Vec<crate::models::Game>,
    pub favorites: HashSet<String>,
    pub available_games: Vec<usize>,
    pub missing_games: Vec<usize>,
    pub favorite_games: Vec<usize>,
    pub parent_games: Vec<usize>,
    pub clone_games: Vec<usize>,
    pub working_games: Vec<usize>,
    pub chd_games: Vec<usize>,
    pub search_cache: HashMap<String, Vec<usize>>,
    pub max_cache_size: usize,
}

impl GameIndex {
    pub fn build(games: Vec<crate::models::Game>, favorites: HashSet<String>) -> Self {
        let mut index = Self {
            games,
            favorites,
            available_games: Vec::new(),
            missing_games: Vec::new(),
            favorite_games: Vec::new(),
            parent_games: Vec::new(),
            clone_games: Vec::new(),
            working_games: Vec::new(),
            chd_games: Vec::new(),
            search_cache: HashMap::new(),
            max_cache_size: 100,
        };
        index.build_indices();
        index
    }

    fn build_indices(&mut self) {
        for (i, game) in self.games.iter().enumerate() {
            // Available games
            if matches!(game.status, RomStatus::Available) {
                self.available_games.push(i);
            }
            
            // Missing games
            if matches!(game.status, RomStatus::Missing) {
                self.missing_games.push(i);
            }
            
            // Favorite games
            if self.favorites.contains(&game.name) {
                self.favorite_games.push(i);
            }
            
            // Parent games (non-clones)
            if !game.is_clone {
                self.parent_games.push(i);
            } else {
                self.clone_games.push(i);
            }
            
            // Working games
            if game.driver_status == "good" {
                self.working_games.push(i);
            }
            
            // CHD games
            if game.requires_chd {
                self.chd_games.push(i);
            }
        }
    }

    pub fn has_clones(&self, game_name: &str) -> bool {
        self.games.iter().any(|g| g.parent.as_ref().map(|p| p.as_str()) == Some(game_name))
    }

    pub fn get_clones(&self, game_name: &str) -> Vec<usize> {
        self.games.iter()
            .enumerate()
            .filter_map(|(i, g)| {
                if g.parent.as_ref().map(|p| p.as_str()) == Some(game_name) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_cached_search(&self, search_text: &str) -> Option<Vec<usize>> {
        self.search_cache.get(search_text).cloned()
    }

    pub fn get_sorted(&self, _column: &str, _ascending: bool) -> Option<Vec<usize>> {
        // Simple implementation - return all games sorted by name
        let mut indices: Vec<usize> = (0..self.games.len()).collect();
        indices.sort_by(|&a, &b| self.games[a].name.cmp(&self.games[b].name));
        Some(indices)
    }

    pub fn cache_search(&mut self, search_text: String, results: Vec<usize>) {
        if self.search_cache.len() >= self.max_cache_size {
            // Remove oldest entry
            let oldest_key = self.search_cache.keys().next().cloned();
            if let Some(key) = oldest_key {
                self.search_cache.remove(&key);
            }
        }
        self.search_cache.insert(search_text, results);
    }

    pub fn clear_cache(&mut self) {
        self.search_cache.clear();
    }

    pub fn update_favorites(&mut self, _games: &[Game], favorites: &HashSet<String>) {
        self.favorites = favorites.clone();
        self.favorite_games.clear();
        for (i, game) in self.games.iter().enumerate() {
            if self.favorites.contains(&game.name) {
                self.favorite_games.push(i);
            }
        }
    }
}

// IconInfo for managing game icons
#[derive(Debug, Clone)]
pub struct IconInfo {
    pub path: String,
    pub loaded: bool,
    pub last_accessed: std::time::Instant,
}

// CategoryLoader trait for loading game categories
pub trait CategoryLoader: Send + Sync {
    fn categories(&self) -> &HashMap<String, String>;
}

// FilterCategory enum - kategori filter untuk game list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FilterCategory {
    All,
    Available,
    Missing,
    Clones,
    NonClones,
    Favorites,
    Working,
    NotWorking,
    Parents,
    ChdGames, // New filter for CHD games
    NonMerged, // Special filter for non-merged sets (parents only)
}

impl Default for FilterCategory {
    fn default() -> Self {
        FilterCategory::All
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RomSetType {
    NonMerged,  // Each game is independent with all required files
    Split,      // Parent contains normal data, clones contain only changes
    Merged,     // All clones are merged into parent archive
    Unknown,    // Type not detected yet
}

impl Default for RomSetType {
    fn default() -> Self {
        RomSetType::Unknown
    }
}

// GameStats melacak statistik bermain untuk setiap game
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStats {
    pub play_count: u32,
    pub last_played: Option<String>,
    pub total_play_time: u32, // dalam detik
}

// VisibleColumns mengontrol kolom mana yang ditampilkan di game list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleColumns {
    pub game_name: bool,
    pub play_count: bool,
    pub manufacturer: bool,
    pub year: bool,
    pub driver: bool,
    pub category: bool,
    pub rom: bool,
    pub chd: bool, // CHD column visibility
    pub driver_status: bool, // Driver status column visibility
}

impl Default for VisibleColumns {
    fn default() -> Self {
        Self {
            game_name: true,
            play_count: true,
            manufacturer: true,
            year: true,
            driver: false,
            category: true,
            rom: false,
            chd: false, // CHD column hidden by default
            driver_status: true, // Driver status column visible by default
        }
    }
}

// Preferences menyimpan semua opsi yang bisa dikonfigurasi user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    // Window settings
    pub window_width: f32,
    pub window_height: f32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub show_fps: bool,

    // General preferences
    pub language: String,
    pub auto_save: bool,
    pub confirm_exit: bool,

    // MAME-specific preferences
    pub search_new_games: bool,
    pub version_mismatch_warning: bool,
    pub use_mame_defaults: bool,
    pub joystick_selection: bool,

    // Display preferences
    pub visible_columns: VisibleColumns,
    pub clone_color: [f32; 3], // RGB color untuk clone games

    // BARU: Performance settings
    pub performance: PerformanceSettings,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            window_width: 1280.0,
            window_height: 720.0,
            fullscreen: false,
            vsync: true,
            show_fps: false,
            language: "English".to_string(),
            auto_save: true,
            confirm_exit: true,
            search_new_games: true,
            version_mismatch_warning: true,
            use_mame_defaults: false,
            joystick_selection: false,
            visible_columns: VisibleColumns::default(),
            clone_color: [0.7, 0.7, 1.0],
            performance: PerformanceSettings::default(),
        }
    }
}

// BARU: PerformanceSettings - konfigurasi untuk optimasi performance
// Struktur ini memungkinkan user fine-tune performance vs quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    // Virtual scrolling - hanya render items yang terlihat
    pub enable_virtual_scrolling: bool,
    pub virtual_scroll_buffer: usize, // Berapa banyak item extra di luar viewport
    pub max_visible_items: usize,     // Maksimum items yang di-render sekaligus

    // Icon loading - lazy load untuk memory efficiency
    pub enable_lazy_icons: bool,
    pub icon_cache_size: usize,       // Maksimum icons di memory

    // Search optimization
    pub search_debounce_ms: u64,      // Delay sebelum search dijalankan
    pub enable_search_cache: bool,     // Cache hasil search

    // Render optimization
    pub enable_fps_limit: bool,
    pub target_fps: u32,
    pub enable_low_quality_mode: bool, // Mode untuk PC lemah
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            enable_virtual_scrolling: true,
            virtual_scroll_buffer: 20,    // 20 items di atas/bawah viewport
            max_visible_items: 100,       // Maksimal render 100 items
            enable_lazy_icons: true,
            icon_cache_size: 500,         // Cache 500 icons
            search_debounce_ms: 300,      // 300ms delay untuk search
            enable_search_cache: true,
            enable_fps_limit: true,
            target_fps: 60,
            enable_low_quality_mode: false,
        }
    }
}

// centralized sort enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortColumn {
    Name,
    Manufacturer,
    Year,
    Status,
    Category,
}

impl Default for SortColumn {
    fn default() -> Self {
        Self::Name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        Self::Ascending
    }
}









