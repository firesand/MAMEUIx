// src/ui/main_window.rs
// File utama yang mengkoordinasikan seluruh aplikasi
// FIXED VERSION dengan optimasi untuk handle 48,000+ games

use eframe::egui;
use crate::models::*;
use crate::mame::GameScanner;
use crate::rom_utils::RomLoader;
use super::{game_list::GameList, sidebar::Sidebar, dialogs::*, artwork_panel::ArtworkPanel, history_panel::HistoryPanel};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::time::{Instant, Duration};
use std::thread;
use rayon::prelude::*; // Add for parallel processing

pub struct MameApp {
    // Core data
    pub config: AppConfig,
    pub games: Vec<Game>,
    pub game_metadata: HashMap<String, Game>,

    // UI state
    pub selected_filter: FilterCategory,
    pub selected_game: Option<usize>,
    pub show_directories_dialog: bool,
    pub show_preferences_dialog: bool,
    pub show_rom_info_dialog: bool,

    pub show_about_dialog: bool,
    pub show_hidden_categories_dialog: bool,

    // UI components
    pub game_list: GameList,
    pub sidebar: Sidebar,
    pub artwork_panel: ArtworkPanel,
    pub history_panel: HistoryPanel,

    // Data organization
    pub all_manufacturers: Vec<String>,
    pub running_games: HashMap<String, (std::process::Child, Instant)>,
    pub expanded_parents: HashMap<String, bool>,

    // Icon management
    pub rom_icons: HashMap<String, egui::TextureHandle>,
    pub default_icon_texture: Option<egui::TextureHandle>,
    pub icon_load_queue: VecDeque<String>,
    pub icon_info: HashMap<String, IconInfo>,
    pub last_icon_cleanup: Instant,

    // Loading state
    pub roms_loading: bool,
    pub roms_tx: Option<mpsc::Receiver<Vec<Game>>>,
    pub loading_rx: Option<mpsc::Receiver<LoadingMessage>>,
    pub loading_stage: LoadingStage,
    pub loading_progress: (usize, usize),
    pub loading_start_time: Option<Instant>,
    pub need_reload_after_dialog: bool,

    // Performance optimization fields
    pub game_index: Option<GameIndex>,           // Fast lookup index
    pub filtered_games_cache: Vec<usize>,        // Cache hasil filter
    pub filter_cache_dirty: bool,                // Apakah cache perlu update
    pub search_debounce_timer: Option<Instant>,  // Timer untuk search debouncing
    pub pending_search: Option<String>,          // Search text yang pending
    pub performance_monitor: PerformanceMonitor, // Monitor FPS dan lag
    pub last_filter_update: Instant,             // Kapan filter terakhir diupdate
    
    // Category management
    pub category_manager: Option<filters::CategoryManager>, // Manages catver.ini categories
    pub category_loader: Option<crate::mame::CategoryLoader>, // Stores the category loader
    
    // MAME finder dialog state
    pub show_mame_finder_dialog: bool,
    pub found_mame_executables: Vec<crate::ui::dialogs::FoundMame>,
    pub show_manual_mame_dialog: bool,
    
    // ROM verification window state
    pub rom_verify_dialog: crate::ui::dialogs::RomVerifyDialog,
    
    // Game Properties dialog state
    pub show_game_properties_dialog: bool,
    pub game_properties_dialog: Option<GamePropertiesDialog>,
    
    // Column width persistence
    pub last_column_save: Instant,
}

impl MameApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut config = crate::config::load_config().unwrap_or_default();
        
        // Check if this is first launch (no MAME executables configured)
        let mut show_mame_finder = false;
        let mut found_mames = Vec::new();
        
        if config.mame_executables.is_empty() {
            println!("First launch detected - searching for MAME executables...");
            found_mames = crate::ui::dialogs::MameFinderDialog::find_mame_executables();
            
            if !found_mames.is_empty() {
                println!("Found {} MAME executable(s)", found_mames.len());
                for mame in &found_mames {
                    println!("  - {} ({})", mame.path, mame.version);
                }
                show_mame_finder = true;
            } else {
                println!("No MAME executables found in standard locations");
                show_mame_finder = true; // Will show manual selection dialog
            }
        }

        let mut app = Self {
            games: vec![],
            game_metadata: HashMap::new(),
            selected_filter: FilterCategory::All,
            selected_game: None,
            config,
            show_directories_dialog: false,
            show_preferences_dialog: false,
            show_rom_info_dialog: false,
    
            show_about_dialog: false,
            show_hidden_categories_dialog: false,
            game_list: GameList::new(),
            sidebar: Sidebar::new(),
            artwork_panel: ArtworkPanel::new(),
            history_panel: HistoryPanel::new(),
            all_manufacturers: Vec::new(),
            running_games: HashMap::new(),
            rom_icons: HashMap::new(),
            default_icon_texture: None,
                icon_load_queue: VecDeque::new(),
                icon_info: HashMap::new(),
                last_icon_cleanup: Instant::now(),
                roms_loading: false,
                roms_tx: None,
                expanded_parents: HashMap::new(),
                loading_rx: None,
                loading_stage: LoadingStage::Idle,
                loading_progress: (0, 0),
                loading_start_time: None,
                need_reload_after_dialog: false,
                // Performance fields
                game_index: None,
                filtered_games_cache: Vec::new(),
                filter_cache_dirty: true,
                search_debounce_timer: None,
                pending_search: None,
                performance_monitor: PerformanceMonitor::new(),
                last_filter_update: Instant::now(),
                
                // Category management
                category_manager: None,
                category_loader: None,
                
                // MAME finder dialog state
                show_mame_finder_dialog: show_mame_finder,
                found_mame_executables: found_mames,
                show_manual_mame_dialog: false,
                
                // ROM verification window state
                rom_verify_dialog: crate::ui::dialogs::RomVerifyDialog::default(),
                
                // Game Properties dialog state
                show_game_properties_dialog: false,
                game_properties_dialog: None,
                
                // Column width persistence
                last_column_save: Instant::now(),
        };

        // Load categories if path is configured (do this before MAME finder logic)
        if let Some(catver_path) = &app.config.catver_ini_path {
            if catver_path.exists() {
                if let Ok(loader) = crate::mame::CategoryLoader::new(catver_path) {
                    app.category_loader = Some(loader.clone());
                    
                    let mut manager = filters::CategoryManager::new();
                    manager.load_from_catver_map(&loader.categories);
                    app.category_manager = Some(manager);
                }
            }
        }

        // Only start initial load if we already have MAME configured
        if !app.config.mame_executables.is_empty() &&
           app.config.selected_mame_index < app.config.mame_executables.len() {
            println!("Initial load: Found {} MAME executables", app.config.mame_executables.len());
            app.start_initial_load();
        }

            app
    }

    /// Build game index untuk fast lookup - CRITICAL untuk performance!
    pub fn build_game_index(&mut self) {
        println!("Building optimized game index for {} games...", self.games.len());
        let start = Instant::now();

        self.game_index = Some(GameIndex::build(&self.games, &self.config.favorite_games));

        let elapsed = start.elapsed();
        println!("Game index built in {:.2}s", elapsed.as_secs_f32());

        // Force filter update dengan index baru
        self.filter_cache_dirty = true;
        self.game_list.invalidate_cache();
    }

    /// OPTIMIZED: Update filtered games cache dengan GameIndex
    pub fn update_filtered_games_cache(&mut self) {
        if !self.filter_cache_dirty {
            return;
        }

        let start = Instant::now();

        // CRITICAL: Use pre-computed index lists when available
        if let Some(index) = &self.game_index {
            // Fast path dengan O(1) access ke pre-filtered lists
            self.filtered_games_cache = match self.selected_filter {
                FilterCategory::All => {
                    // Use sorted indices if available for better performance
                    if let Some(sorted) = index.get_sorted("name", true) {
                        sorted.to_vec()
                    } else {
                        (0..self.games.len()).collect()
                    }
                }
                FilterCategory::Available => index.available_games.clone(),
                FilterCategory::Missing => index.missing_games.clone(),
                FilterCategory::Favorites => index.favorite_games.clone(),
                FilterCategory::Parents => index.parent_games.clone(),
                FilterCategory::Clones => index.clone_games.clone(),
                FilterCategory::Working => index.working_games.clone(),
                FilterCategory::NotWorking => index.missing_games.clone(),
                FilterCategory::NonClones => index.parent_games.clone(),
                FilterCategory::ChdGames => index.chd_games.clone(),
                FilterCategory::NonMerged => index.parent_games.clone(), // Same as Parents for now
            };

            // Apply catver.ini category filter if set
            if let Some(ref category_name) = self.config.filter_settings.catver_category {
                if let Some(ref category_loader) = self.category_loader {
                    let category_name_lower = category_name.to_lowercase();
                    self.filtered_games_cache.retain(|&idx| {
                        if let Some(game) = self.games.get(idx) {
                            // Try exact match first
                            if let Some(game_category) = category_loader.get_category(&game.name) {
                                // Case-insensitive category comparison
                                game_category.to_lowercase() == category_name_lower
                            } else {
                                // Try case-insensitive match on game name
                                let game_name_lower = game.name.to_lowercase();
                                category_loader.categories.iter().any(|(catver_name, catver_category)| {
                                    catver_name.to_lowercase() == game_name_lower &&
                                    catver_category.to_lowercase() == category_name_lower
                                })
                            }
                        } else {
                            false
                        }
                    });
                }
            }

            // Apply hidden categories filter if enabled
            if self.config.filter_settings.apply_hidden_categories && !self.config.hidden_categories.is_empty() {
                if let Some(ref category_loader) = self.category_loader {
                    self.filtered_games_cache.retain(|&idx| {
                        if let Some(game) = self.games.get(idx) {
                            // Check if the game's category is in the hidden list
                            if let Some(game_category) = category_loader.get_category(&game.name) {
                                !self.config.hidden_categories.contains(game_category)
                            } else {
                                true // If we can't determine the category, show the game
                            }
                        } else {
                            false
                        }
                    });
                }
            }

            // Apply search filter only if there's text
            if !self.config.filter_settings.search_text.is_empty() {
                // Check cache first for instant results!
                let search_key = self.config.filter_settings.search_text.clone();
 
                 if let Some(cached) = index.get_cached_search(&search_key) {
                     // Cache hit! No need to search
                     self.filtered_games_cache = cached.to_vec();
                 } else {
                     // Cache miss - search and cache the result
                     self.apply_search_filter_optimized();
 
                     // Store in cache for next time
                     if let Some(index) = &mut self.game_index {
                         index.cache_search(
                             &search_key,
                             self.filtered_games_cache.clone()
                         );
                     }
                 }
            }
        } else {
            // Fallback without index (should rarely happen)
            self.filtered_games_cache = self.filter_games_manual();
            if !self.config.filter_settings.search_text.is_empty() {
                self.apply_search_filter_optimized();
            }
        }

        self.filter_cache_dirty = false;
        self.last_filter_update = Instant::now();

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 50 {
            println!("Warning: Filter update took {}ms for {} results",
                     elapsed.as_millis(), self.filtered_games_cache.len());
        }
    }

    /// Manual filtering fallback (rarely used)
    fn filter_games_manual(&self) -> Vec<usize> {
        self.games.iter()
        .enumerate()
        .filter(|(_, game)| {
            // First apply the main filter category
            let main_filter_passed = match self.selected_filter {
                FilterCategory::All => true,
                FilterCategory::Available => matches!(game.status, RomStatus::Available),
                FilterCategory::Missing => matches!(game.status, RomStatus::Missing),
                FilterCategory::Favorites => self.config.favorite_games.contains(&game.name),
                FilterCategory::Parents => !game.is_clone,
                FilterCategory::Clones => game.is_clone,
                FilterCategory::Working => matches!(game.status, RomStatus::Available),
                FilterCategory::NotWorking => !matches!(game.status, RomStatus::Available),
                FilterCategory::NonClones => !game.is_clone,
                FilterCategory::ChdGames => game.requires_chd,
                FilterCategory::NonMerged => !game.is_clone, // Same as Parents for now
            };
            
            if !main_filter_passed {
                return false;
            }
            
            // Then apply catver.ini category filter if set
            if let Some(ref category_name) = self.config.filter_settings.catver_category {
                if let Some(ref category_loader) = self.category_loader {
                    let category_name_lower = category_name.to_lowercase();
                    if let Some(game_category) = category_loader.get_category(&game.name) {
                        // Case-insensitive category comparison
                        return game_category.to_lowercase() == category_name_lower;
                    } else {
                        return false; // Game not found in catver.ini
                    }
                } else {
                    return false; // No category loader available
                }
            }
            
            true // No category filter applied
        })
        .map(|(idx, _)| idx)
        .collect()
    }

    /// OPTIMIZED: Apply search filter with parallel processing
    fn apply_search_filter_optimized(&mut self) {
        let search_lower = self.config.filter_settings.search_text.to_lowercase();
        let search_mode = &self.config.filter_settings.search_mode;

        // Use parallel processing for large datasets (huge speedup!)
        if self.filtered_games_cache.len() > 1000 {
            self.filtered_games_cache = self.filtered_games_cache
            .par_iter() // Parallel iterator from rayon
            .filter(|&&idx| {
                if let Some(game) = self.games.get(idx) {
                    match search_mode {
                        crate::models::SearchMode::GameTitle => {
                            game.description.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Manufacturer => {
                            game.manufacturer.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::RomFileName => {
                            game.name.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Year => {
                            game.year.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Status => {
                            game.status.description().to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Cpu => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                        crate::models::SearchMode::Device => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                        crate::models::SearchMode::Sound => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                    }
                } else {
                    false
                }
            })
            .copied()
            .collect();
        } else {
            // Serial processing for small datasets
            self.filtered_games_cache.retain(|&idx| {
                if let Some(game) = self.games.get(idx) {
                    match search_mode {
                        crate::models::SearchMode::GameTitle => {
                            game.description.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Manufacturer => {
                            game.manufacturer.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::RomFileName => {
                            game.name.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Year => {
                            game.year.to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Status => {
                            game.status.description().to_lowercase().contains(&search_lower)
                        }
                        crate::models::SearchMode::Cpu => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                        crate::models::SearchMode::Device => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                        crate::models::SearchMode::Sound => {
                            // For now, return false as we don't have hardware filter here
                            false
                        }
                    }
                } else {
                    false
                }
            });
        }
    }

    /// Handle search input dengan proper debouncing
    pub fn handle_search_input(&mut self, new_text: String) {
        self.pending_search = Some(new_text);
        self.search_debounce_timer = Some(Instant::now());
    }

    /// Process pending search after debounce delay
    pub fn process_pending_search(&mut self) {
        if let Some(pending) = &self.pending_search {
            if let Some(timer) = self.search_debounce_timer {
                let delay = Duration::from_millis(
                    self.config.preferences.performance.search_debounce_ms
                );

                if timer.elapsed() >= delay {
                    // Apply search
                    self.config.filter_settings.search_text = pending.clone();
                    self.filter_cache_dirty = true;
                    self.pending_search = None;
                    self.search_debounce_timer = None;
                    self.game_list.invalidate_cache();
                }
            }
        }
    }

    /// IMPROVED: Smart resource cleanup
    fn cleanup_resources(&mut self) {
        // Icon cache cleanup with LRU algorithm
        let icon_limit = self.config.preferences.performance.icon_cache_size;

        if self.rom_icons.len() > icon_limit {
            // Sort by last access time
            let mut icon_ages: Vec<(String, Instant)> = self.icon_info.iter()
            .map(|(name, info)| (name.clone(), info.last_accessed))
            .collect();

            icon_ages.sort_by_key(|(_, time)| *time);

            // Remove oldest 25%
            let remove_count = self.rom_icons.len() / 4;
            for (name, _) in icon_ages.iter().take(remove_count) {
                self.rom_icons.remove(name);
                self.icon_info.remove(name);
            }

            println!("Cleaned up {} old icons from cache", remove_count);
        }

        // Clear oversized search cache
        if let Some(index) = &mut self.game_index {
            if index.search_cache.len() > index.max_cache_size {
                index.clear_cache();
                println!("Cleared search cache");
            }
        }

        self.last_icon_cleanup = Instant::now();
    }

    /// Clean up old icons dari cache
    fn cleanup_icon_cache(&mut self) {
        self.cleanup_resources(); // Use the improved version
    }

    // ... Keep all the existing methods below unchanged ...

    fn start_initial_load(&mut self) {
        if self.loading_stage != LoadingStage::Idle {
            return;
        }
        self.load_mame_data_threaded();
    }

    pub fn load_mame_data_threaded(&mut self) {
        if self.loading_stage != LoadingStage::Idle && self.loading_stage != LoadingStage::Complete {
            println!("Load already in progress, skipping...");
            return;
        }

        let mame = match self.config.mame_executables.get(self.config.selected_mame_index) {
            Some(m) if !m.path.is_empty() => m.clone(),
            _ => {
                eprintln!("No valid MAME executable configured");
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.loading_rx = Some(rx);
        self.loading_stage = LoadingStage::LoadingMame;
        self.loading_start_time = Some(Instant::now());

        // Get catver.ini path for category support
        let catver_path = self.config.catver_ini_path.clone();
        
        thread::spawn(move || {
            println!("Starting MAME data load in background thread...");
            let _ = tx.send(LoadingMessage::MameLoadStarted);

            // Initialize CategoryLoader if catver.ini path is configured
            let category_loader = if let Some(ref catver_path) = catver_path {
                match crate::mame::CategoryLoader::new(catver_path) {
                    Ok(loader) => Some(loader),
                    Err(e) => {
                        eprintln!("Failed to load categories from {:?}: {}", catver_path, e);
                        None
                    }
                }
            } else {
                None
            };
            
            // Create scanner with category loader if available
            let mut scanner = GameScanner::new(&mame.path);
            if let Some(ref loader) = category_loader {
                scanner = scanner.with_category_loader(loader.clone());
            }

            let _ = tx.send(LoadingMessage::MameLoadProgress(
                "Running mame -listxml... This may take 10-30 seconds".to_string()
            ));

            match scanner.scan_games() {
                Ok(games) => {
                    println!("MAME scan complete: {} games found", games.len());

                    let mut manufacturers: Vec<String> = games.iter()
                    .map(|g| g.manufacturer.clone())
                    .filter(|m| !m.is_empty())
                    .collect();
                    manufacturers.sort();
                    manufacturers.dedup();

                    // Box the category loader for sending through the channel
                    let boxed_loader: Option<Box<dyn std::any::Any + Send>> = category_loader.map(|loader| {
                        Box::new(loader) as Box<dyn std::any::Any + Send>
                    });
                    let _ = tx.send(LoadingMessage::MameLoadComplete(games, manufacturers, boxed_loader));
                }
                Err(e) => {
                    eprintln!("MAME scan failed: {}", e);
                    let _ = tx.send(LoadingMessage::MameLoadFailed(e.to_string()));
                }
            }
        });
    }

    pub fn reload_roms_threaded(&mut self) {
        if self.game_metadata.is_empty() {
            println!("No game metadata - need to load MAME data first");
            self.loading_stage = LoadingStage::Error;
            return;
        }

        if self.loading_stage == LoadingStage::ScanningRoms {
            println!("ROM scan already in progress");
            return;
        }

        let valid_dirs: Vec<_> = self.config.rom_paths.iter()
        .filter(|dir| dir.exists() && dir.is_dir())
        .cloned()
        .collect();

        if valid_dirs.is_empty() {
            eprintln!("No valid ROM directories configured");
            self.loading_stage = LoadingStage::Error;
            return;
        }

        println!("Starting ROM scan with {} directories", valid_dirs.len());

        let metadata = self.game_metadata.clone();
        let (tx, rx) = mpsc::channel();
        self.loading_rx = Some(rx);
        self.loading_stage = LoadingStage::ScanningRoms;
        self.loading_progress = (0, 0);

        thread::spawn(move || {
            let _ = tx.send(LoadingMessage::RomScanStarted);

            let loader = RomLoader::new(valid_dirs);
            let games = loader.load_roms(metadata);

            println!("ROM scan complete in thread: {} games", games.len());
            let _ = tx.send(LoadingMessage::RomScanComplete(games));
        });
    }

    pub fn process_loading_messages(&mut self) {
        if let Some(rx) = self.loading_rx.take() {
            let mut messages = Vec::new();
            let mut need_rom_scan = false;
            let mut need_index_build = false;

            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }

            let mut should_keep_receiver = true;

            for msg in messages {
                match msg {
                    LoadingMessage::MameLoadStarted => {
                        println!("UI: MAME load started");
                        self.loading_stage = LoadingStage::LoadingMame;
                    }

                    LoadingMessage::MameLoadProgress(msg) => {
                        println!("UI: MAME load progress: {}", msg);
                    }

                    LoadingMessage::MameLoadComplete(games, manufacturers, category_loader) => {
                        println!("UI: MAME load complete with {} games", games.len());

                        self.game_metadata = games.iter()
                        .map(|g| (g.name.clone(), g.clone()))
                        .collect();

                        self.all_manufacturers = manufacturers;
                        
                        // Store category loader and initialize category manager
                        if let Some(boxed_loader) = category_loader {
                            // Try to downcast the boxed loader back to CategoryLoader
                            if let Ok(loader) = boxed_loader.downcast::<crate::mame::CategoryLoader>() {
                                let loader = *loader;

                                let mut manager = filters::CategoryManager::new();
                                manager.load_from_catver_map(&loader.categories);
                                self.category_manager = Some(manager);
                                
                                self.category_loader = Some(loader);
                            }
                        }

                        if !self.config.rom_paths.is_empty() {
                            need_rom_scan = true;
                            should_keep_receiver = false;
                        } else {
                            self.loading_stage = LoadingStage::Complete;
                            should_keep_receiver = false;
                        }
                    }

                    LoadingMessage::MameLoadFailed(error) => {
                        eprintln!("UI: MAME load failed: {}", error);
                        self.loading_stage = LoadingStage::Error;
                        should_keep_receiver = false;
                    }

                    LoadingMessage::RomScanStarted => {
                        println!("UI: ROM scan started");
                        self.loading_stage = LoadingStage::ScanningRoms;
                        self.loading_progress = (0, 0);
                    }

                    LoadingMessage::RomScanProgress(current, total) => {
                        self.loading_progress = (current, total);
                    }

                    LoadingMessage::RomScanComplete(games) => {
                        // Apply categories to games if category loader is available
                        let mut games_with_categories = games;
                        if let Some(ref category_loader) = self.category_loader {
                            for game in &mut games_with_categories {
                                // Use get_category_with_parent to handle clones properly
                                if let Some(category) = category_loader.get_category_with_parent(&game.name, game.parent.as_deref()) {
                                    game.category = category.to_string();
                                } else if game.category.is_empty() {
                                    // Ensure games without categories in catver.ini have "Misc."
                                    game.category = "Misc.".to_string();
                                }
                            }
                        }
                        
                        self.games = games_with_categories;
                        self.loading_stage = LoadingStage::Complete;
                        self.loading_start_time = None;
                        should_keep_receiver = false;
                        need_index_build = true; // CRITICAL: Build index after loading!
                        println!("UI: Loading stage set to: {:?}", self.loading_stage);
                        
                        // Check plugin support after loading is complete
                        self.show_plugin_info();
                    }

                    LoadingMessage::RomScanFailed(error) => {
                        eprintln!("UI: ROM scan failed: {}", error);
                        self.loading_stage = LoadingStage::Error;
                        should_keep_receiver = false;
                    }
                }
            }

            if should_keep_receiver {
                self.loading_rx = Some(rx);
            } else {
                self.loading_rx = None;
            }

            // Post-processing
            if need_rom_scan {
                self.reload_roms_threaded();
            }

            if need_index_build {
                self.build_game_index(); // Build optimized index!
                self.filter_cache_dirty = true;
            }
        }
    }

    pub fn on_refresh_clicked(&mut self) {
        match self.loading_stage {
            LoadingStage::Idle | LoadingStage::Complete | LoadingStage::Error => {
                if self.game_metadata.is_empty() {
                    self.load_mame_data_threaded();
                } else {
                    self.reload_roms_threaded();
                }
            }
            _ => {
                println!("Load already in progress, please wait...");
            }
        }
    }

    pub fn on_directories_changed(&mut self) {
        println!("\nDirectories configuration changed, reloading data...");

        self.save_config();
        self.games.clear();
        self.game_index = None;
        self.filter_cache_dirty = true;

        if !self.config.mame_executables.is_empty() &&
            self.config.selected_mame_index < self.config.mame_executables.len() {
                self.load_mame_data_threaded();
            }
    }

    /// Reload categories from catver.ini file
    pub fn reload_categories(&mut self) {
        // Clear existing category data
        self.category_loader = None;
        self.category_manager = None;
        
        // Load categories if path is configured
        if let Some(catver_path) = &self.config.catver_ini_path {
            if catver_path.exists() {
                match crate::mame::CategoryLoader::new(catver_path) {
                    Ok(loader) => {
                        self.category_loader = Some(loader.clone());
                        
                        // Update category manager
                        let mut manager = filters::CategoryManager::new();
                        manager.load_from_catver_map(&loader.categories);
                        self.category_manager = Some(manager);
                        
                        // Apply categories to existing games if any
                        if !self.games.is_empty() {
                            for game in &mut self.games {
                                // Use get_category_with_parent to handle clones properly
                                if let Some(category) = loader.get_category_with_parent(&game.name, game.parent.as_deref()) {
                                    game.category = category.to_string();
                                } else if game.category.is_empty() {
                                    // Ensure games without categories in catver.ini have "Misc."
                                    game.category = "Misc.".to_string();
                                }
                            }
                            
                            // CRITICAL: Force complete UI refresh after applying categories
                            // Clear all caches to ensure UI shows updated categories
                            self.filter_cache_dirty = true;
                            self.game_list.invalidate_cache();
                            self.game_list.expanded_rows_cache.clear(); // Clear row cache
                            self.game_list.visible_start = 0; // Reset scroll position
                            self.game_list.visible_end = 0;
                            
                            // Rebuild index with new categories
                            self.build_game_index();
                            
                            // Force immediate filter update
                            self.update_filtered_games_cache();
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load categories from {:?}: {}", catver_path, e);
                    }
                }
            }
        }
    }

    pub fn save_config(&self) {
        println!("DEBUG: save_config called");
        if let Some(dialog) = &self.game_properties_dialog {
            println!("DEBUG: Generating MAME args from dialog:");
            let args = dialog.generate_mame_args();
            println!("DEBUG: Generated MAME args: {:?}", args);
        }
        if let Err(e) = crate::config::save_config(&self.config) {
            eprintln!("Failed to save config: {}", e);
        }
    }

    pub fn toggle_favorite(&mut self, rom_name: &str) {
        if self.config.favorite_games.contains(rom_name) {
            self.config.favorite_games.remove(rom_name);
        } else {
            self.config.favorite_games.insert(rom_name.to_string());
        }

        // Update index favorites list
        if let Some(index) = &mut self.game_index {
            index.update_favorites(&self.games, &self.config.favorite_games);
        }

        self.filter_cache_dirty = true;
        self.save_config();
    }

    pub fn update_game_stats(&mut self, rom_name: &str, play_time: u32) {
        let stats = self.config.game_stats.entry(rom_name.to_string())
        .or_insert_with(GameStats::default);

        stats.play_count += 1;
        stats.last_played = Some(chrono::Local::now().to_rfc3339());
        stats.total_play_time += play_time;

        self.save_config();
    }

    pub fn check_running_games(&mut self) {
        let mut finished_games = Vec::new();
        let mut still_running = HashMap::new();

        let running_games = std::mem::take(&mut self.running_games);

        for (rom_name, (mut child, start_time)) in running_games {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let play_time = start_time.elapsed().as_secs() as u32;
                    finished_games.push((rom_name, play_time));
                }
                Ok(None) => {
                    still_running.insert(rom_name, (child, start_time));
                }
                Err(_) => {}
            }
        }

        self.running_games = still_running;

        for (rom_name, play_time) in finished_games {
            self.update_game_stats(&rom_name, play_time);
        }
    }

    pub fn init_default_icon(&mut self, ctx: &egui::Context) {
        let size = self.config.icon_size as usize;
        let pixels = vec![80u8; size * size * 4];

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [size, size],
            &pixels,
        );

        self.default_icon_texture = Some(ctx.load_texture(
            "default_icon",
            color_image,
            egui::TextureOptions::default(),
        ));
    }

    pub fn queue_icon_load(&mut self, rom_name: String) {
        if self.config.preferences.performance.enable_lazy_icons {
            if !self.rom_icons.contains_key(&rom_name)
                && !self.icon_load_queue.contains(&rom_name)
                && !self.icon_info.contains_key(&rom_name) {
                    self.icon_load_queue.push_back(rom_name);
                }
        }
    }

    /// Load icon from file system
    fn load_icon_from_file(&self, ctx: &egui::Context, rom_name: &str) -> Option<egui::TextureHandle> {
        // Check if icons path is configured
        let icons_path = self.config.icons_path.as_ref()?;
        
        // Try to load .ico file
        let ico_path = icons_path.join(format!("{}.ico", rom_name));
        
        if ico_path.exists() {
            // Read the ico file
            if let Ok(ico_data) = std::fs::read(&ico_path) {
                // Try to load as ICO format
                if let Ok(image) = image::load_from_memory_with_format(&ico_data, image::ImageFormat::Ico) {
                    // Convert to RGBA8
                    let rgba_image = image.to_rgba8();
                    
                    // Resize to configured icon size if needed
                    let icon_size = self.config.icon_size as u32;
                    let resized = if rgba_image.width() != icon_size || rgba_image.height() != icon_size {
                        image::imageops::resize(
                            &rgba_image,
                            icon_size,
                            icon_size,
                            image::imageops::FilterType::Lanczos3
                        )
                    } else {
                        rgba_image
                    };
                    
                    let size = [resized.width() as usize, resized.height() as usize];
                    let pixels = resized.into_raw();
                    
                    // Create egui ColorImage
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                    
                    // Create texture
                    return Some(ctx.load_texture(
                        format!("icon_{}", rom_name),
                        color_image,
                        egui::TextureOptions::default(),
                    ));
                }
            }
        }
        
        None
    }

    /// OPTIMIZED: Adaptive icon loading based on performance
    pub fn process_icon_queue(&mut self, ctx: &egui::Context) {
        if !self.config.show_rom_icons || self.icon_load_queue.is_empty() {
            return;
        }

        // Adaptive loading based on current FPS - increased limits for faster loading
        let fps = self.performance_monitor.get_average_fps();
        let max_per_frame = if fps < 25.0 {
            2  // Increased from 1
        } else if fps < 40.0 {
            5  // Increased from 2
        } else if fps < 50.0 {
            8  // New tier
        } else {
            12 // Increased from 3 - load many more when FPS is good
        };

        // Pre-allocate vector for batch loading
        let mut icons_to_load = Vec::with_capacity(max_per_frame);
        
        // Collect icons to load
        for _ in 0..max_per_frame {
            if let Some(rom_name) = self.icon_load_queue.pop_front() {
                // Skip if already loaded
                if !self.rom_icons.contains_key(&rom_name) {
                    icons_to_load.push(rom_name);
                }
            } else {
                break;
            }
        }

        // Load icons in batch
        for rom_name in icons_to_load {
            // Try to load icon from file
            let icon_texture = self.load_icon_from_file(ctx, &rom_name)
                .or_else(|| self.default_icon_texture.clone());

            if let Some(texture) = icon_texture {
                self.rom_icons.insert(rom_name.clone(), texture);
                self.icon_info.insert(rom_name, IconInfo {
                    loaded: true,
                    last_accessed: Instant::now(),
                });
            }
        }
    }

    pub fn get_rom_icon(&mut self, rom_name: &str) -> Option<egui::TextureHandle> {
        if let Some(info) = self.icon_info.get_mut(rom_name) {
            info.last_accessed = Instant::now();
        }

        self.rom_icons.get(rom_name).cloned()
        .or_else(|| self.default_icon_texture.clone())
    }

    /// Jump to the first game that starts with the given character
    pub fn jump_to_game_starting_with(&mut self, character: char) {
        let search_char = character.to_lowercase().to_string();
        
        // First, ensure the filtered games cache is up to date
        if self.filter_cache_dirty {
            self.update_filtered_games_cache();
        }
        
        // Search through the expanded rows cache (which includes the current filter and sort)
        if let Some(row_index) = self.game_list.expanded_rows_cache.iter().position(|row| {
            if let Some(game) = self.games.get(row.game_idx) {
                // Jump based on game description (what's shown in the Game column)
                game.description.to_lowercase().starts_with(&search_char)
            } else {
                false
            }
        }) {
            // Found a game - get the actual game index
            if let Some(row_data) = self.game_list.expanded_rows_cache.get(row_index) {
                // Update selection
                self.selected_game = Some(row_data.game_idx);
                
                // Calculate the scroll position to center the selected game
                let visible_rows = 30; // Approximate number of visible rows
                let target_start = row_index.saturating_sub(visible_rows / 2);
                
                // Update the game list's visible range
                self.game_list.visible_start = target_start;
                self.game_list.visible_end = (target_start + visible_rows).min(self.game_list.expanded_rows_cache.len());
                
                // Force the game list to scroll to this position
                // We'll need to add a flag to tell the game list to scroll
                self.game_list.scroll_to_row = Some(row_index);
                
                println!("Jumping to game at row {} starting with '{}'", row_index, character);
            }
        } else {
            println!("No game found starting with '{}'", character);
        }
    }

    fn show_about_dialog(&mut self, ctx: &egui::Context) {
        let mut should_close = false;
        egui::Window::new("About MAMEUIx")
            .open(&mut self.show_about_dialog)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("🎮 MAMEUIx");
                    ui.label("A modern, fast, and user-friendly frontend for MAME");
                    ui.label("Written in Rust using the egui framework");
                    ui.add_space(10.0);
                    
                    ui.label("Version: 0.1.2 (Development)");
                    ui.label("Built with Rust 1.88+ and egui 0.32");
                    ui.add_space(10.0);
                    
                    ui.label("🚀 Recent Improvements:");
                    ui.label("• Modernized egui API (0.32)");
                    ui.label("• Enhanced performance optimizations");
                    ui.label("• Updated dependencies");
                    ui.label("• Improved code quality");
                    ui.label("• Fixed category loading and persistence");
                    ui.add_space(10.0);
                    
                    ui.label("✨ Key Features:");
                    ui.label("• Fast game loading (48,000+ games)");
                    ui.label("• CHD game support");
                    ui.label("• Virtual scrolling for performance");
                    ui.label("• 10 beautiful themes");
                    ui.label("• Persistent column widths");
                    ui.label("• Advanced filtering & search");
                    ui.label("• Cross-platform compatibility");
                    ui.label("• Category support via catver.ini");
                    ui.add_space(10.0);
                    
                    ui.label("🛠️ Technical Details:");
                    ui.label("• Optimized release builds (LTO enabled)");
                    ui.label("• Background processing");
                    ui.label("• Memory-efficient design");
                    ui.label("• Comprehensive packaging support");
                    ui.add_space(10.0);
                    
                    ui.label("🙏 Acknowledgments:");
                    ui.label("• MAME Team - For the excellent arcade emulator");
                    ui.label("• egui - For the modern GUI framework");
                    ui.label("• Rust Community - For the amazing ecosystem");
                    ui.add_space(10.0);
                    
                    ui.label("⚠️ Important Note:");
                    ui.label("This frontend requires MAME to be installed separately.");
                    ui.label("It does not include ROM files or MAME itself.");
                    ui.add_space(10.0);
                    
                    if ui.button("Close").clicked() {
                        should_close = true;
                    }
                });
            });
        
        if should_close {
            self.show_about_dialog = false;
        }
    }
}

/// CRITICAL: Optimized App trait implementation
impl eframe::App for MameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update performance monitor
        self.performance_monitor.frame_start();

        // Batch all state updates before rendering
        let mut needs_repaint = false;

        // Process background messages
        if self.loading_rx.is_some() {
            self.process_loading_messages();
            needs_repaint = true;
        }

        // Process pending search with debouncing
        if let Some(_pending) = &self.pending_search {
            if let Some(timer) = self.search_debounce_timer {
                let delay = Duration::from_millis(
                    self.config.preferences.performance.search_debounce_ms
                );

                if timer.elapsed() >= delay {
                    self.process_pending_search();
                    needs_repaint = true;
                }
            }
        }

        // Handle keyboard input for quick navigation
        // Store if we should process keyboard navigation
        let mut should_process_keyboard_nav = false;
        let mut typed_char = None;
        
        // Check if any dialog is open
        let dialog_open = self.show_directories_dialog ||
                         self.show_preferences_dialog ||
                         self.show_rom_info_dialog ||
                 
                         self.show_mame_finder_dialog ||
                         self.show_manual_mame_dialog ||
                         self.show_game_properties_dialog;
        
        ctx.input(|i| {
            // Don't process keyboard navigation if:
            // 1. No games loaded
            // 2. Any dialog is open
            // 3. Search text is being edited (indicated by pending search)
            let search_active = self.pending_search.is_some();
            
            if !self.games.is_empty() && !dialog_open && !search_active {
                // Look for text input events
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        // Get the first character typed
                        if let Some(first_char) = text.chars().next() {
                            if first_char.is_alphanumeric() {
                                should_process_keyboard_nav = true;
                                typed_char = Some(first_char);
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        // Process keyboard navigation outside of input closure
        if should_process_keyboard_nav {
            if let Some(character) = typed_char {
                self.jump_to_game_starting_with(character);
                needs_repaint = true;
            }
        }

        // Update filter cache with rate limiting (but immediate for category changes)
        let category_just_changed = {
            let current_category = self.config.filter_settings.catver_category.clone();
            let prev_category = self.sidebar.get_previous_category();
            current_category != prev_category
        };

        if self.filter_cache_dirty {
            // Immediate update for category changes, rate limited for other changes
            if category_just_changed || self.last_filter_update.elapsed() > Duration::from_millis(10) {
                self.update_filtered_games_cache();
                needs_repaint = true;
            }
        }

            // Cleanup resources periodically (not every frame!)
            if self.last_icon_cleanup.elapsed() > Duration::from_secs(120) {
                self.cleanup_resources();
            }

            // Save column widths periodically (every 5 seconds)
            if self.last_column_save.elapsed() > Duration::from_secs(5) {
                self.save_config();
                self.last_column_save = Instant::now();
            }

            // Smart repaint scheduling
            if needs_repaint || (self.loading_stage == LoadingStage::LoadingMame ||
                self.loading_stage == LoadingStage::ScanningRoms) {
                ctx.request_repaint_after(Duration::from_millis(100));
                }

                // Apply theme
                self.config.theme.apply(ctx);

            // Initialize resources once
            if self.default_icon_texture.is_none() && self.config.show_rom_icons {
                self.init_default_icon(ctx);
            }

            // Update running games periodically
            if self.performance_monitor.frame_count % 30 == 0 {
                self.check_running_games();
            }

            // Process icons with adaptive rate
            if self.config.preferences.performance.enable_lazy_icons {
                self.process_icon_queue(ctx);
            }

                // Show toolbar
                self.show_toolbar(ctx);

            // Sidebar
            egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                let old_filter = self.selected_filter;
                let old_search = self.config.filter_settings.search_text.clone();
                let old_category = self.config.filter_settings.catver_category.clone();
                let old_hidden_categories_len = self.config.hidden_categories.len();
                let old_apply_hidden_categories = self.config.filter_settings.apply_hidden_categories;
                self.sidebar.show(
                    ui, 
                    &mut self.selected_filter, 
                    &mut self.config.filter_settings, 
                    self.category_manager.as_ref(),
                    &mut self.config.hidden_categories,
                    &mut self.show_hidden_categories_dialog,
                );
                
                // Check if filter, search, category, or hidden categories changed
                let hidden_categories_changed = self.config.hidden_categories.len() != old_hidden_categories_len ||
                    self.config.filter_settings.apply_hidden_categories != old_apply_hidden_categories;
                
                if self.selected_filter != old_filter || 
                   self.config.filter_settings.search_text != old_search ||
                   self.config.filter_settings.catver_category != old_category ||
                   hidden_categories_changed {
                    self.filter_cache_dirty = true;
                    self.game_list.invalidate_cache();
                    
                    // IMMEDIATE UPDATE for category changes - CRITICAL FIX
                    if self.config.filter_settings.catver_category != old_category || hidden_categories_changed {
                        self.update_filtered_games_cache();
                    }
                }

                // Show performance info in debug mode
                if self.config.preferences.show_fps {
                    ui.separator();
                    ui.label("Performance:");
                    self.performance_monitor.show_debug_info(ui);

                }
            });

            // Artwork panel
            egui::SidePanel::right("artwork")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    // Split the panel vertically
                    let available_height = ui.available_height();
                    let artwork_height = available_height * 0.5; // 50% for artwork
                    let history_height = available_height * 0.5; // 50% for history
                    
                    // Top half - Artwork panel
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), artwork_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            self.artwork_panel.show(
                                ui,
                                &self.selected_game,
                                &self.games,
                                &self.config.extra_asset_dirs,
                                &self.config
                            );
                        }
                    );
                    
                    ui.separator();
                    
                    // Bottom half - History panel
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), history_height),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            // Update history panel with selected game
                            if let Some(idx) = self.selected_game {
                                if let Some(game) = self.games.get(idx) {
                                    self.history_panel.set_selected_game(Some(game.name.clone()), Some(game.name.clone()), &self.config);
                                }
                            } else {
                                self.history_panel.set_selected_game(None, None, &self.config);
                            }
                            
                            self.history_panel.show(ui, &self.config);
                        }
                    );
            });

            // Main content
            egui::CentralPanel::default().show(ctx, |ui| {
                match self.loading_stage {
                    LoadingStage::LoadingMame => {
                        ui.centered_and_justified(|ui| {
                            ui.heading("Loading MAME Database");
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(20.0);
                            ui.label("Scanning MAME for available games...");
                            ui.label("This may take 10-30 seconds for 40,000+ games");

                            if let Some(start_time) = self.loading_start_time {
                                let elapsed = start_time.elapsed().as_secs();
                                ui.add_space(10.0);
                                ui.label(format!("Time elapsed: {} seconds", elapsed));
                            }
                        });
                    }

                    LoadingStage::ScanningRoms => {
                        ui.centered_and_justified(|ui| {
                            ui.heading("Scanning ROM Files");
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(20.0);

                            let (current, total) = self.loading_progress;
                            if total > 0 {
                                let progress = current as f32 / total as f32;
                                ui.add(egui::ProgressBar::new(progress)
                                .text(format!("{}/{} files", current, total))
                                .desired_width(300.0));
                            } else {
                                ui.label("Checking ROM directories for available games...");
                            }

                            ui.add_space(10.0);
                            ui.label(format!("Scanning {} ROM directories", self.config.rom_paths.len()));
                        });
                    }

                    LoadingStage::Error => {
                        ui.centered_and_justified(|ui| {
                            ui.heading("⚠ Loading Error");
                            ui.add_space(20.0);
                            ui.label("Failed to load game data. Please check:");
                            ui.label("• MAME executable is correctly configured");
                            ui.label("• MAME executable has proper permissions");
                            ui.label("• ROM directories are accessible");
                            ui.add_space(20.0);
                            if ui.button("Open Directories Settings").clicked() {
                                self.show_directories_dialog = true;
                            }
                        });
                    }

                    _ => {
                        if self.games.is_empty() && !self.config.rom_paths.is_empty() &&
                            self.loading_stage == LoadingStage::Complete {
                                ui.centered_and_justified(|ui| {
                                    ui.heading("No games found");
                                    ui.add_space(10.0);
                                    ui.label("ROM directories were scanned but no matching games were found.");
                                    ui.label("Please check:");
                                    ui.label("• ROM files are in .zip format");
                                    ui.label("• ROM files have correct names (e.g., pacman.zip)");
                                    ui.label("• ROM directories contain actual game files");
                                    ui.add_space(20.0);
                                    if ui.button("Configure Directories").clicked() {
                                        self.show_directories_dialog = true;
                                    }
                                });
                            } else if self.games.is_empty() && self.loading_stage == LoadingStage::Idle {
                                ui.centered_and_justified(|ui| {
                                    ui.heading("Welcome to MAMEUIx");
                                    ui.add_space(10.0);
                                    ui.label("To get started:");
                                    ui.label("1. Configure your MAME executable");
                                    ui.label("2. Add ROM directories");
                                    ui.label("3. Click OK to scan for games");
                                    ui.add_space(20.0);
                                    if ui.button("Configure Directories").clicked() {
                                        self.show_directories_dialog = true;
                                    }
                                });
                            } else {
                                // CRITICAL: Update filter cache BEFORE showing game list
                                if self.filter_cache_dirty {
                                    self.update_filtered_games_cache();
                                }
                                
                                // Queue icons for visible games before showing the list
                                if self.config.show_rom_icons {
                                    // Get visible range from game list
                                    let visible_start = self.game_list.visible_start;
                                    let visible_end = self.game_list.visible_end;
                                    
                                    // Extended range for pre-loading (load 10 games before and after visible range)
                                    let preload_start = visible_start.saturating_sub(10);
                                    let preload_end = (visible_end + 10).min(self.game_list.expanded_rows_cache.len());
                                    
                                    // Collect game names to queue with priority
                                    let mut high_priority_games = Vec::new();
                                    let mut low_priority_games = Vec::new();
                                    
                                    // Process all games in extended range
                                    if let Some(rows) = self.game_list.expanded_rows_cache.get(preload_start..preload_end) {
                                        for (idx, row_data) in rows.iter().enumerate() {
                                            if let Some(game) = self.games.get(row_data.game_idx) {
                                                let absolute_idx = preload_start + idx;
                                                
                                                // High priority for visible games
                                                if absolute_idx >= visible_start && absolute_idx < visible_end {
                                                    high_priority_games.push(game.name.clone());
                                                } else {
                                                    // Low priority for pre-load games
                                                    low_priority_games.push(game.name.clone());
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Queue high priority games first (visible games)
                                    for game_name in high_priority_games {
                                        if !self.rom_icons.contains_key(&game_name) && !self.icon_load_queue.contains(&game_name) {
                                            self.icon_load_queue.push_front(game_name);
                                        }
                                    }
                                    
                                    // Then queue low priority games (pre-load)
                                    for game_name in low_priority_games {
                                        if !self.rom_icons.contains_key(&game_name) && !self.icon_load_queue.contains(&game_name) {
                                            self.icon_load_queue.push_back(game_name);
                                        }
                                    }
                                }
                                
                                // Show game list with optimizations
                                let (double_clicked, favorite_toggled) = self.game_list.show(
                                    ui,
                                    &self.games,
                                    &self.config.filter_settings,
                                    &mut self.selected_game,
                                    &mut self.expanded_parents,
                                    &self.config.favorite_games,
                                    &mut self.rom_icons,
                                    self.config.show_rom_icons,
                                    self.config.icon_size,
                                    self.game_index.as_ref(),
                                    self.selected_filter,
                                    &mut self.config.column_widths,
                                    &self.config.preferences.visible_columns,
                                    self.default_icon_texture.as_ref(),
                                    &self.config.game_stats,
                                    None, // hardware_filter - not available here
                                    self.config.catver_ini_path.is_some(),
                                    Some(&self.filtered_games_cache), // Pass pre-filtered indices
                                );
                                
                                // Handle favorite toggle
                                if let Some(game_name) = favorite_toggled {
                                    self.toggle_favorite(&game_name);
                                }
                                
                                // Handle double-click to launch game
                                if double_clicked {
                                    if let Some(idx) = self.selected_game {
                                        if let Some(game) = self.games.get(idx) {
                                            if let Ok(child) = crate::mame::launch_game(&game.name, &self.config) {
                                                self.running_games.insert(game.name.clone(), (child, Instant::now()));
                                            }
                                        }
                                    }
                                }
                            }
                    }
                }
            });

            // Status bar
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    match self.loading_stage {
                        LoadingStage::LoadingMame => {
                            ui.label("Loading MAME database...");
                            ui.spinner();
                        }
                        LoadingStage::ScanningRoms => {
                            ui.label("Scanning ROM files...");
                            ui.spinner();
                        }
                        LoadingStage::Complete => {
                            ui.label(format!("{} games loaded", self.games.len()));
                        }
                        _ => {
                            ui.label(format!("{} games", self.games.len()));
                            if !self.games.is_empty() {
                                let available = self.games.iter()
                                .filter(|g| matches!(g.status, RomStatus::Available))
                                .count();
                                ui.label(format!("({} available)", available));
                            }

                            // Show filtered count if filter active
                            if !self.filtered_games_cache.is_empty() &&
                                self.filtered_games_cache.len() < self.games.len() {
                                    ui.separator();
                                    ui.label(format!("Showing {} filtered",
                                                     self.filtered_games_cache.len()));
                                }
                        }
                    }

                    // Performance info
                    if self.config.preferences.show_fps {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let fps = self.performance_monitor.get_average_fps();
                            let color = if fps < 20.0 {
                                egui::Color32::RED
                            } else if fps < 30.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GREEN
                            };
                            ui.colored_label(color, format!("FPS: {:.1}", fps));
                        });
                    }
                });
            });

            // Handle dialogs
            if self.show_directories_dialog {
                let changed = DirectoriesDialog::show(ctx, &mut self.config, &mut self.show_directories_dialog);

                // Check if dialog was closed (not just hidden)
                if !self.show_directories_dialog {
                    // Always save config when dialog is closed to ensure persistence
                    self.save_config();
                    
                    // Check if catver.ini was just configured
                    if self.category_loader.is_none() && self.config.catver_ini_path.is_some() {
                        // This is first-time catver.ini configuration - load categories immediately
                        self.reload_categories();
                        
                        // Force UI refresh if games are already loaded
                        if !self.games.is_empty() {
                            // Force immediate UI update
                            ctx.request_repaint();
                        }
                    } else if changed {
                        // For other changes, reload everything
                        self.need_reload_after_dialog = true;
                    }
                }

                if !self.show_directories_dialog && self.need_reload_after_dialog {
                    self.on_directories_changed();
                    self.need_reload_after_dialog = false;
                }
            }

            if self.show_preferences_dialog {
                PreferencesDialog::show(ctx, &mut self.config.preferences, &mut self.config.theme, &mut self.show_preferences_dialog, self.config.catver_ini_path.is_some());
            }

            if self.show_rom_info_dialog {
                if let Some(idx) = self.selected_game {
                    if let Some(game) = self.games.get(idx) {
                        RomInfoDialog::show(ctx, game, &mut self.show_rom_info_dialog);
                    }
                }
            }

            

            if self.show_about_dialog {
                self.show_about_dialog(ctx);
            }

            if self.show_hidden_categories_dialog {
                HiddenCategoriesDialog::show(
                    ctx, 
                    &mut self.config.hidden_categories, 
                    self.category_manager.as_ref(),
                    &mut self.show_hidden_categories_dialog
                );
            }
            
            // Handle MAME finder dialog
            if self.show_mame_finder_dialog {
                if !self.found_mame_executables.is_empty() {
                    if MameFinderDialog::show_selection_dialog(
                        ctx,
                        &self.found_mame_executables,
                        &mut self.config,
                        &mut self.show_mame_finder_dialog,
                    ) {
                        self.save_config();
                        self.start_initial_load();
                    } else if !self.show_mame_finder_dialog {
                        // User chose to browse manually
                        self.show_manual_mame_dialog = true;
                    }
                } else {
                    // No MAME found, show manual selection
                    self.show_mame_finder_dialog = false;
                    self.show_manual_mame_dialog = true;
                }
            }
            
            if self.show_manual_mame_dialog {
                if MameFinderDialog::show_manual_selection_dialog(
                    ctx,
                    &mut self.config,
                    &mut self.show_manual_mame_dialog,
                ) {
                    self.save_config();
                    self.start_initial_load();
                }
            }
            
            // Show ROM verification window
            if self.rom_verify_dialog.is_open() {
                self.rom_verify_dialog.show_window(ctx, &self.config, &self.games);
            }
            
            // Show Game Properties dialog
            if self.show_game_properties_dialog {
                if let Some(dialog) = &mut self.game_properties_dialog {
                    if dialog.show(ctx, &mut self.show_game_properties_dialog, &mut self.config) {
                        // Properties were applied
                        self.save_config();
                    }
                }
            }
    }
}

impl MameApp {
    // Add this helper function to show plugin support info
    fn show_plugin_info(&self) {
        if let Some(mame) = self.config.mame_executables.get(self.config.selected_mame_index) {
            match crate::mame::verify_plugin_support(&mame.path) {
                Ok(support) => {
                    println!("\n=== MAME Plugin Support ===");
                    println!("Plugin system available: {}", support.has_plugin_support);
                    println!("Hiscore plugin: {}", support.hiscore_available);
                    println!("Cheat plugin: {}", support.cheat_available);
                    println!("Autofire plugin: {}", support.autofire_available);
                    println!("Available plugins: {:?}", support.available_plugins);
                    
                    // You could show this in a dialog instead
                }
                Err(e) => {
                    eprintln!("Failed to check plugin support: {}", e);
                }
            }
        }
    }
    
    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("MAME Manager").clicked() {
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    // Add ROM verification option
                    if ui.button("🔍 Verify ROMs...").clicked() {
                        self.rom_verify_dialog.open();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Game", |ui| {
                    if ui.button("🎮 Play").clicked() {
                        if let Some(idx) = self.selected_game {
                            if let Some(game) = self.games.get(idx) {
                                if let Ok(child) = crate::mame::launch_game(&game.name, &self.config) {
                                    self.running_games.insert(game.name.clone(), (child, Instant::now()));
                                }
                            }
                        }
                        ui.close();
                    }
                    
                    ui.separator();
                    

                    
                    if ui.button("ℹ ROM Info...").clicked() {
                        self.show_rom_info_dialog = true;
                        ui.close();
                    }
                });

                ui.menu_button("Options", |ui| {
                    if ui.button("Directories").clicked() {
                        self.show_directories_dialog = true;
                        ui.close();
                    }
                    if ui.button("Preferences").clicked() {
                        self.show_preferences_dialog = true;
                        ui.close();
                    }

                    
                    ui.separator();
                    
                    if ui.button("⚙️ Default Game Properties...").clicked() {
                        self.game_properties_dialog = Some(GamePropertiesDialog::new_with_config(None, &self.config));
                        self.show_game_properties_dialog = true;
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if ui.button("🔍 Find MAME Executables").clicked() {
                        self.found_mame_executables = MameFinderDialog::find_mame_executables();
                        if !self.found_mame_executables.is_empty() {
                            self.show_mame_finder_dialog = true;
                        } else {
                            self.show_manual_mame_dialog = true;
                        }
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    // Quick theme selector
                    ui.menu_button("Theme", |ui| {
                        let themes = [
                            crate::models::Theme::DarkBlue,
                            crate::models::Theme::DarkGrey,
                            crate::models::Theme::ArcadePurple,
                            crate::models::Theme::LightClassic,
                            crate::models::Theme::NeonGreen,
                            crate::models::Theme::SunsetOrange,
                            crate::models::Theme::OceanBlue,
                            crate::models::Theme::MidnightBlack,
                            crate::models::Theme::ForestGreen,
                            crate::models::Theme::RetroAmber,
                        ];
                        
                        for theme in themes {
                            if ui.radio(self.config.theme == theme, theme.display_name()).clicked() {
                                self.config.theme = theme;
                                ui.close();
                            }
                        }
                    });
                });

                // Add Tools menu
                ui.menu_button("Tools", |ui| {
                    if ui.button("🔍 ROM Verification").clicked() {
                        self.rom_verify_dialog.open();
                        ui.close_menu();
                    }
                    
                    if ui.button("🎯 Verify Selected ROM").clicked() {
                        if let Some(idx) = self.selected_game {
                            if let Some(game) = self.games.get(idx) {
                                self.rom_verify_dialog.open();
                                // You could pass the selected game to the dialog here
                                ui.close_menu();
                            }
                        }
                    }
                    
                    ui.separator();
                    
                    if ui.button("📊 Plugin Support Info").clicked() {
                        self.show_plugin_info();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.show_about_dialog = true;
                        ui.close();
                    }
                });
            });

            ui.horizontal(|ui| {
                let loading = self.loading_stage != LoadingStage::Idle &&
                self.loading_stage != LoadingStage::Complete &&
                self.loading_stage != LoadingStage::Error;

                ui.add_enabled_ui(!loading, |ui| {
                    if ui.button("🎮 Play Game").clicked() {
                        if let Some(idx) = self.selected_game {
                            if let Some(game) = self.games.get(idx) {
                                if let Ok(child) = crate::mame::launch_game(&game.name, &self.config) {
                                    self.running_games.insert(game.name.clone(), (child, Instant::now()));
                                }
                            }
                        }
                    }

                    if ui.button("ℹ Properties").clicked() {
                        self.show_rom_info_dialog = true;
                    }

                    if loading {
                        ui.add_enabled(false, egui::Button::new("🔄 Loading..."));
                    } else {
                        if ui.button("🔄 Refresh").clicked() {
                            self.on_refresh_clicked();
                        }
                    }
                });

            });
        });
    }
}

// Extension trait for PerformanceMonitor
impl PerformanceMonitor {
    pub fn show_debug_info(&mut self, ui: &mut egui::Ui) {
        let fps = self.get_average_fps();
        let color = if fps < 20.0 {
            egui::Color32::RED
        } else if fps < 30.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };

        ui.colored_label(color, format!("FPS: {:.1}", fps));

        if self.is_lagging() {
            ui.colored_label(egui::Color32::RED, "⚠ Lag detected");
        }

        let lag_spikes = self.get_lag_spike_count();
        if lag_spikes > 0 {
            ui.label(format!("Lag spikes: {}", lag_spikes));
        }
    }
}
