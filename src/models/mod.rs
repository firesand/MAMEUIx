// src/models/mod.rs
// File ini mendefinisikan semua model data yang digunakan aplikasi
// Termasuk struktur baru untuk optimasi performance

pub mod game;
pub mod config;
pub mod filters;

// Re-export everything from submodules
pub use game::*;
pub use config::*;
pub use filters::*;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

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
}

impl Default for FilterCategory {
    fn default() -> Self {
        FilterCategory::All
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
}

impl Default for VisibleColumns {
    fn default() -> Self {
        Self {
            game_name: true,
            play_count: true,
            manufacturer: true,
            year: true,
            driver: false,
            category: false,
            rom: false,
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

// IconInfo melacak status loading untuk game icons
#[derive(Debug, Clone)]
pub struct IconInfo {
    pub loaded: bool,
    pub last_accessed: std::time::Instant,
}

impl Default for IconInfo {
    fn default() -> Self {
        Self {
            loaded: false,
            last_accessed: std::time::Instant::now(),
        }
    }
}

// LoadingMessage untuk komunikasi antar thread
#[derive(Debug, Clone)]
pub enum LoadingMessage {
    MameLoadStarted,
    MameLoadProgress(String),
    MameLoadComplete(Vec<Game>, Vec<String>),
    MameLoadFailed(String),
    RomScanStarted,
    RomScanProgress(usize, usize),
    RomScanComplete(Vec<Game>),
    RomScanFailed(String),
}

// LoadingStage melacak tahap loading saat ini
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadingStage {
    Idle,
    LoadingMame,
    ScanningRoms,
    Complete,
    Error,
}

// CRITICAL OPTIMIZATION: Enhanced GameIndex dengan O(1) clone lookups
// Struktur ini adalah kunci untuk menangani 48,000+ games dengan cepat
#[derive(Debug, Clone)]
pub struct GameIndex {
    // Existing index fields
    pub by_name: HashMap<String, usize>,
    pub by_manufacturer: HashMap<String, Vec<usize>>,
    pub by_year: HashMap<String, Vec<usize>>,

    // Pre-filtered lists untuk common filters - O(1) access!
    pub available_games: Vec<usize>,
    pub missing_games: Vec<usize>,
    pub favorite_games: Vec<usize>,
    pub parent_games: Vec<usize>,
    pub clone_games: Vec<usize>,
    pub working_games: Vec<usize>,

    // CRITICAL NEW FIELDS untuk menghilangkan O(n) clone scanning
    pub parent_to_clones: HashMap<String, Vec<usize>>, // Parent name -> clone indices
    pub clone_count: HashMap<String, usize>,           // Parent name -> jumlah clones

    // Search optimization
    pub search_cache: HashMap<String, Vec<usize>>,     // Cache search results
    pub max_cache_size: usize,                         // Limit cache memory usage

    // Sorting optimization
    pub sorted_indices: HashMap<SortKey, Vec<usize>>,  // Pre-sorted untuk common sorts
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct SortKey {
    pub column: String,
    pub ascending: bool,
}

impl GameIndex {
    /// Build index dari game list untuk ultra-fast lookup
    /// Ini mengubah operasi O(n) menjadi O(1) untuk sebagian besar queries
    pub fn build(games: &[Game], favorites: &HashSet<String>) -> Self {
        println!("Building optimized game index for {} games...", games.len());
        let start = Instant::now();

        let mut by_name = HashMap::with_capacity(games.len());
        let mut by_manufacturer: HashMap<String, Vec<usize>> = HashMap::new();
        let mut by_year: HashMap<String, Vec<usize>> = HashMap::new();
        let mut available_games = Vec::with_capacity(games.len() / 2);
        let mut missing_games = Vec::with_capacity(games.len() / 2);
        let mut favorite_games = Vec::with_capacity(favorites.len());
        let mut parent_games = Vec::with_capacity(games.len() / 3);
        let mut clone_games = Vec::with_capacity(games.len() * 2 / 3);
        let mut working_games = Vec::with_capacity(games.len() / 2);

        // CRITICAL: Clone relationship maps untuk O(1) lookup
        let mut parent_to_clones: HashMap<String, Vec<usize>> = HashMap::new();
        let mut clone_count: HashMap<String, usize> = HashMap::new();

        // Single pass untuk build semua indices
        for (idx, game) in games.iter().enumerate() {
            // Name index untuk instant lookup by name
            by_name.insert(game.name.clone(), idx);

            // Manufacturer index untuk filter by manufacturer
            by_manufacturer
            .entry(game.manufacturer.clone())
            .or_insert_with(Vec::new)
            .push(idx);

            // Year index untuk filter by year
            by_year
            .entry(game.year.clone())
            .or_insert_with(Vec::new)
            .push(idx);

            // Status-based indices untuk instant filtering
            match game.status {
                RomStatus::Available => {
                    available_games.push(idx);
                    working_games.push(idx);
                }
                RomStatus::Missing => missing_games.push(idx),
                _ => {}
            }

            // Favorite index
            if favorites.contains(&game.name) {
                favorite_games.push(idx);
            }

            // CRITICAL: Build parent-clone relationships
            if let Some(parent_name) = &game.parent {
                // Ini adalah clone
                clone_games.push(idx);
                parent_to_clones
                .entry(parent_name.clone())
                .or_insert_with(Vec::new)
                .push(idx);
                *clone_count.entry(parent_name.clone()).or_insert(0) += 1;
            } else if !game.is_clone {
                // Ini adalah parent
                parent_games.push(idx);
                // Initialize empty clone list bahkan jika tidak punya clones
                parent_to_clones.entry(game.name.clone()).or_insert_with(Vec::new);
                clone_count.entry(game.name.clone()).or_insert(0);
            }
        }

        // Pre-compute sorted indices untuk common sort orders
        let mut sorted_indices = HashMap::new();

        // Sort by name ascending - paling sering digunakan
        let mut name_asc: Vec<usize> = (0..games.len()).collect();
        name_asc.sort_by(|&a, &b| {
            games[a].description.to_lowercase().cmp(&games[b].description.to_lowercase())
        });
        sorted_indices.insert(
            SortKey { column: "name".to_string(), ascending: true },
                              name_asc
        );

        // Sort by manufacturer
        let mut manuf_asc: Vec<usize> = (0..games.len()).collect();
        manuf_asc.sort_by(|&a, &b| {
            games[a].manufacturer.cmp(&games[b].manufacturer)
            .then(games[a].description.cmp(&games[b].description))
        });
        sorted_indices.insert(
            SortKey { column: "manufacturer".to_string(), ascending: true },
                              manuf_asc
        );

        let elapsed = start.elapsed();
        println!("Game index built in {:.2}s", elapsed.as_secs_f32());
        println!("  - {} parent games", parent_games.len());
        println!("  - {} clone games", clone_games.len());
        println!("  - {} available games", available_games.len());
        println!("  - {} parents with clones",
                 parent_to_clones.values().filter(|v| !v.is_empty()).count());

        Self {
            by_name,
            by_manufacturer,
            by_year,
            available_games,
            missing_games,
            favorite_games,
            parent_games,
            clone_games,
            working_games,
            parent_to_clones,
            clone_count,
            search_cache: HashMap::new(),
            max_cache_size: 100, // Cache up to 100 search results
            sorted_indices,
        }
    }

    /// Get clones untuk parent game - O(1) operation!
    /// Ini menggantikan O(n) scanning dengan instant HashMap lookup
    #[inline]
    pub fn get_clones(&self, parent_name: &str) -> &[usize] {
        self.parent_to_clones
        .get(parent_name)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    }

    /// Check apakah game punya clones - O(1) operation!
    #[inline]
    pub fn has_clones(&self, parent_name: &str) -> bool {
        self.clone_count.get(parent_name).copied().unwrap_or(0) > 0
    }

    /// Get games berdasarkan filter category dengan O(1) access
    pub fn get_by_category(&self, category: FilterCategory) -> &[usize] {
        match category {
            FilterCategory::All => &[], // Special case - return empty, caller should use all indices
            FilterCategory::Available => &self.available_games,
            FilterCategory::Missing => &self.missing_games,
            FilterCategory::Favorites => &self.favorite_games,
            FilterCategory::Parents => &self.parent_games,
            FilterCategory::Clones => &self.clone_games,
            FilterCategory::Working => &self.working_games,
            FilterCategory::NotWorking => &self.missing_games,
            FilterCategory::NonClones => &self.parent_games,
        }
    }

    /// Get pre-sorted indices jika tersedia
    pub fn get_sorted(&self, column: &str, ascending: bool) -> Option<&[usize]> {
        let key = SortKey {
            column: column.to_string(),
            ascending,
        };
        self.sorted_indices.get(&key).map(|v| v.as_slice())
    }

    /// Cache search result untuk reuse
    pub fn cache_search(&mut self, query: &str, results: Vec<usize>) {
        // Limit cache size untuk prevent memory bloat
        if self.search_cache.len() >= self.max_cache_size {
            // Remove oldest entry (simple FIFO)
            if let Some(first_key) = self.search_cache.keys().next().cloned() {
                self.search_cache.remove(&first_key);
            }
        }
        self.search_cache.insert(query.to_string(), results);
    }

    /// Get cached search result
    pub fn get_cached_search(&self, query: &str) -> Option<&[usize]> {
        self.search_cache.get(query).map(|v| v.as_slice())
    }

    /// Clear search cache
    pub fn clear_cache(&mut self) {
        self.search_cache.clear();
    }

    /// Update favorites list saat favorites berubah
    pub fn update_favorites(&mut self, games: &[Game], favorites: &HashSet<String>) {
        self.favorite_games.clear();
        for (idx, game) in games.iter().enumerate() {
            if favorites.contains(&game.name) {
                self.favorite_games.push(idx);
            }
        }
    }
}

// PerformanceMonitor untuk tracking FPS dan lag
pub struct PerformanceMonitor {
    frame_times: VecDeque<Duration>,
    last_frame: Instant,
    slow_frame_threshold: Duration,
    pub frame_count: u64,
    total_time: Duration,
    // Tambahan untuk better monitoring
    last_fps_calculation: Instant,
    cached_fps: f32,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120), // Track 2 seconds at 60fps
            last_frame: Instant::now(),
            slow_frame_threshold: Duration::from_millis(33), // Target 30fps minimum
            frame_count: 0,
            total_time: Duration::ZERO,
            last_fps_calculation: Instant::now(),
            cached_fps: 60.0,
        }
    }

    /// Call at frame start
    pub fn frame_start(&mut self) {
        let now = Instant::now();
        let frame_time = now - self.last_frame;
        self.last_frame = now;

        self.frame_times.push_back(frame_time);
        if self.frame_times.len() > 120 {
            self.frame_times.pop_front();
        }

        self.frame_count += 1;
        self.total_time += frame_time;
    }

    /// Get average FPS - cached untuk performance
    pub fn get_average_fps(&mut self) -> f32 {
        // Only recalculate FPS every second
        if self.last_fps_calculation.elapsed() >= Duration::from_secs(1) {
            if !self.frame_times.is_empty() {
                let avg_frame_time = self.frame_times.iter()
                    .sum::<Duration>() / self.frame_times.len() as u32;
                
                if avg_frame_time.as_secs_f32() > 0.0 {
                    self.cached_fps = 1.0 / avg_frame_time.as_secs_f32();
                } else {
                    self.cached_fps = 60.0; // Default to 60 FPS if calculation fails
                }
                self.last_fps_calculation = Instant::now();
            }
        }
        
        self.cached_fps
    }

    /// Check if experiencing lag
    pub fn is_lagging(&self) -> bool {
        if let Some(last_frame) = self.frame_times.back() {
            *last_frame > self.slow_frame_threshold
        } else {
            false
        }
    }

    /// Get lag spike count
    pub fn get_lag_spike_count(&self) -> usize {
        self.frame_times.iter()
        .filter(|&&time| time > self.slow_frame_threshold)
        .count()
    }

    /// Reset monitor
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        self.last_frame = Instant::now();
    }
}
