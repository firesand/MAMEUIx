// src/ui/main_window.rs
// File utama yang mengkoordinasikan seluruh aplikasi
// FIXED VERSION dengan optimasi untuk handle 48,000+ games

use eframe::egui;
use crate::models::*;
use crate::mame::GameScanner;
use crate::utils::rom_utils::RomLoader;
use crate::ui::panels::{GameList, GameListView, Sidebar, ArtworkPanel, HistoryPanel, IconManager, PerformanceManager, GameIndexManager};
use crate::ui::{DialogManager, DialogType, DialogAction};
use crate::ui::components::game_properties::GamePropertiesDialog;
use crate::ui::components::mame_finder::MameFinderDialog;
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
    pub selected_filter: FilterCategory, // DEPRECATED - kept for compatibility during transition
    pub selected_game: Option<usize>,

    // UI components
    pub game_list: GameList,
    pub game_list_view: GameListView,
    pub sidebar: Sidebar,
    pub artwork_panel: ArtworkPanel,
    pub history_panel: HistoryPanel,

    // Data organization
    pub all_manufacturers: Vec<String>,
    pub running_games: HashMap<String, (std::process::Child, Instant)>,
    pub expanded_parents: HashMap<String, bool>,

    // Icon management
    pub icon_manager: IconManager,

    // Loading state
    pub loading_rx: Option<mpsc::Receiver<LoadingMessage>>,
    pub loading_stage: LoadingStage,
    pub loading_progress: (usize, usize),
    pub loading_start_time: Option<Instant>,
    pub need_reload_after_dialog: bool,
    pub roms_loading: bool,
    pub roms_tx: Option<mpsc::Sender<LoadingMessage>>,

    // Performance optimization fields
    pub game_index_manager: GameIndexManager,    // Game indexing, filtering, and search management
    pub performance_manager: PerformanceManager, // Monitor FPS dan lag
    
    // Category management - REMOVED
    
    // Dialog management
    pub dialog_manager: DialogManager,
    
    // Column width persistence
    pub last_column_save: Instant,
    
    // Theme management
    pub theme_applied: bool,
}

impl MameApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut config = crate::config::load_config().unwrap_or_default();
        
        // Check if this is first launch (no MAME executables configured)
        let mut show_mame_finder = false;
        let mut found_mames = Vec::new();
        
        if config.mame_executables.is_empty() {
            println!("First launch detected - searching for MAME executables...");
            found_mames = MameFinderDialog::find_mame_executables();
            
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

        // Migrate old filter settings if needed
        config.filter_settings.migrate_from_legacy();
        
        let mut app = Self {
            games: vec![],
            game_metadata: HashMap::new(),
            selected_filter: FilterCategory::All, // Deprecated
            selected_game: None,
            config: config.clone(),
            game_list: GameList::new(),
            game_list_view: GameListView::new(),
            sidebar: Sidebar::new(),
            artwork_panel: ArtworkPanel::new(),
            history_panel: HistoryPanel::new(),
            all_manufacturers: Vec::new(),
            running_games: HashMap::new(),
            icon_manager: IconManager::new(&config),
            roms_loading: false,
            roms_tx: None,
            expanded_parents: HashMap::new(),
            loading_rx: None,
            loading_stage: LoadingStage::Idle,
            loading_progress: (0, 0),
            loading_start_time: None,
            need_reload_after_dialog: false,
            // Performance fields
            game_index_manager: GameIndexManager::new()
                .with_settings(
                    config.preferences.performance.search_debounce_ms,
                    100, // max_cache_size
                ),
            performance_manager: PerformanceManager::new(),
            
            
            // Dialog management
            dialog_manager: DialogManager::new(),
            
            // Column width persistence
            last_column_save: Instant::now(),
            
            // Theme management
            theme_applied: false,
        };

        // Category loading removed - this functionality is no longer needed

        // Initialize MAME finder dialog if needed
        if show_mame_finder {
            app.dialog_manager.set_found_mame_executables(found_mames.clone());
            if !found_mames.is_empty() {
                app.dialog_manager.open_dialog(DialogType::MameFinder);
            } else {
                app.dialog_manager.open_dialog(DialogType::ManualMame);
            }
        }

        // Only start initial load if we already have MAME configured
        if !app.config.mame_executables.is_empty() &&
           app.config.selected_mame_index < app.config.mame_executables.len() {
            println!("Initial load: Found {} MAME executables", app.config.mame_executables.len());
            app.start_initial_load();
        }

        // Apply initial theme
        // Note: This will be applied when the first frame is rendered
        println!("Initial theme: {}", app.config.theme.display_name());

        app
    }

    /// Build game index untuk fast lookup - CRITICAL untuk performance!
    pub fn build_game_index(&mut self) {
        self.game_index_manager.build_game_index(&self.games, &self.config.favorite_games);
        self.game_list.invalidate_cache();
    }

    /// OPTIMIZED: Update filtered games cache dengan GameIndex
    pub fn update_filtered_games_cache(&mut self) {
        self.game_index_manager.update_filtered_games_cache(
            &self.games,
            self.selected_filter,
            &self.config.filter_settings,
            &self.config.hidden_categories,
        );
    }





    /// Process pending search after debounce delay
    pub fn process_pending_search(&mut self) {
        if let Some(search_text) = self.game_index_manager.process_pending_search() {
            // Apply search
            self.config.filter_settings.search_text = search_text;
            self.game_index_manager.mark_cache_dirty();
            self.game_list.invalidate_cache();
        }
    }

    /// IMPROVED: Smart resource cleanup
    fn cleanup_resources(&mut self) {
        // Clean up old icons using IconManager
        self.icon_manager.cleanup_old_icons();

        // Clear oversized search cache using GameIndexManager
        let (cache_size, max_size) = self.game_index_manager.get_cache_stats();
        if cache_size > max_size {
            GameIndexManager::clear_regex_cache();
        }
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

                    let _ = tx.send(LoadingMessage::MameLoadComplete(games, manufacturers));
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

                    LoadingMessage::MameLoadComplete(games, manufacturers) => {
                        println!("UI: MAME load complete with {} games", games.len());

                        self.game_metadata = games.iter()
                        .map(|g| (g.name.clone(), g.clone()))
                        .collect();

                        self.all_manufacturers = manufacturers;
                        

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
                        // Categories are now handled during MAME scanning
                        self.games = games;
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
                self.game_index_manager.mark_cache_dirty();
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
        self.game_index_manager.reset();

        if !self.config.mame_executables.is_empty() &&
            self.config.selected_mame_index < self.config.mame_executables.len() {
                self.load_mame_data_threaded();
            }
    }

    /// Reload categories from catver.ini file
    pub fn reload_categories(&mut self) {
        // Category reloading is no longer needed - categories are loaded during MAME scan
        // Just trigger a refresh if needed
        if !self.games.is_empty() {
            self.game_index_manager.mark_cache_dirty();
            self.game_list.invalidate_cache();
            self.update_filtered_games_cache();
        }
    }

    pub fn save_config(&self) {
        println!("DEBUG: save_config called");
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
        self.game_index_manager.update_favorites(&self.games, &self.config.favorite_games);
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
        self.icon_manager.init_default_icon(ctx, self.config.icon_size);
    }

    pub fn queue_icon_load(&mut self, rom_name: String) {
        self.icon_manager.queue_icon_load(rom_name, self.config.preferences.performance.enable_lazy_icons);
    }

    /// OPTIMIZED: Adaptive icon loading based on performance
    pub fn process_icon_queue(&mut self, ctx: &egui::Context) {
        let fps = self.performance_manager.get_average_fps();
        self.icon_manager.process_icon_queue(ctx, &self.config, fps);
    }

    /// Update game verification statuses from verification manager
    pub fn update_game_verification_statuses(&mut self) {
        let verification_manager = self.dialog_manager.verification_manager();
        
        for game in &mut self.games {
            verification_manager.update_game_status(game);
        }
    }

    /// Jump to the first game that starts with the given character
    pub fn jump_to_game_starting_with(&mut self, character: char) {
        let search_char = character.to_lowercase().to_string();
        
        // First, ensure the filtered games cache is up to date
        if self.game_index_manager.is_cache_dirty() {
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


}

/// CRITICAL: Optimized App trait implementation
impl eframe::App for MameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update performance monitor
        self.performance_manager.frame_start();

        // Batch all state updates before rendering
        let mut needs_repaint = false;

        // Process background messages
        if self.loading_rx.is_some() {
            self.process_loading_messages();
            needs_repaint = true;
        }

        // Process pending search with debouncing
        if self.game_index_manager.has_pending_search() {
            if self.game_index_manager.should_process_pending_search(
                self.config.preferences.performance.search_debounce_ms
            ) {
                self.process_pending_search();
                needs_repaint = true;
            }
        }

        // Handle keyboard input for quick navigation
        // Store if we should process keyboard navigation
        let mut should_process_keyboard_nav = false;
        let mut typed_char = None;
        
        // Check if any dialog is open
        let dialog_open = self.dialog_manager.is_any_dialog_open();
        
        ctx.input(|i| {
            // Don't process keyboard navigation if:
            // 1. No games loaded
            // 2. Any dialog is open
            // 3. Search text is being edited (indicated by pending search)
            let search_active = self.game_index_manager.has_pending_search();
            
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

        // Update filter cache with rate limiting
        if self.game_index_manager.is_cache_dirty() {
            if self.game_index_manager.last_filter_update.elapsed() > Duration::from_millis(10) {
                self.update_filtered_games_cache();
                needs_repaint = true;
            }
        }

        // Cleanup resources periodically (not every frame!)
        if self.icon_manager.last_icon_cleanup.elapsed() > Duration::from_secs(120) {
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

        // Apply theme if not already applied
        if !self.theme_applied {
            self.config.theme.apply(ctx);
            self.theme_applied = true;
        }

        // Initialize resources once
        if self.icon_manager.default_icon_texture.is_none() && self.config.show_rom_icons {
            self.init_default_icon(ctx);
        }

        // Update running games periodically
        if self.performance_manager.frame_count % 30 == 0 {
            self.check_running_games();
        }

        // Process icons with adaptive rate
        if self.config.preferences.performance.enable_lazy_icons {
            self.process_icon_queue(ctx);
        }

        // Update game verification statuses
        self.update_game_verification_statuses();

        // IMPROVED LAYOUT: Modern, spacious design
        // Top toolbar with better spacing
        egui::TopBottomPanel::top("toolbar")
            .exact_height(60.0) // Increased height for better spacing
            .show(ctx, |ui| {
                ui.add_space(8.0); // Add top padding
                self.show_toolbar(ui);
                ui.add_space(8.0); // Add bottom padding
            });

        // Left sidebar with improved styling - ENHANCED: More flexible resizing
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(280.0) // Default width for better readability
            .min_width(100.0) // ENHANCED: Much smaller minimum for maximum flexibility
            .max_width(500.0) // ENHANCED: Larger maximum for wide sidebars
            .show(ctx, |ui| {
                ui.add_space(12.0); // Add top padding
                
                let old_search = self.config.filter_settings.search_text.clone();
                let old_hidden_categories_len = self.config.hidden_categories.len();
                let old_apply_hidden_categories = self.config.filter_settings.apply_hidden_categories;
                
                // Store old filter states to detect changes
                let old_availability = self.config.filter_settings.availability_filters.clone();
                let old_status = self.config.filter_settings.status_filters.clone();
                let old_others = self.config.filter_settings.other_filters.clone();
                
                self.sidebar.show(
                    ui,
                    &mut self.selected_filter, // Deprecated parameter
                    &mut self.config.filter_settings,
                    None, // category_manager removed
                    &mut self.config.hidden_categories,
                    &mut self.dialog_manager,
                );
                
                // Check if any filters changed
                let filters_changed =
                    self.config.filter_settings.availability_filters.show_available != old_availability.show_available ||
                    self.config.filter_settings.availability_filters.show_unavailable != old_availability.show_unavailable ||
                    self.config.filter_settings.status_filters.show_working != old_status.show_working ||
                    self.config.filter_settings.status_filters.show_not_working != old_status.show_not_working ||
                    self.config.filter_settings.other_filters.show_favorites != old_others.show_favorites ||
                    self.config.filter_settings.other_filters.show_parents_only != old_others.show_parents_only ||
                    self.config.filter_settings.other_filters.show_chd_games != old_others.show_chd_games;
                
                let hidden_categories_changed = self.config.hidden_categories.len() != old_hidden_categories_len ||
                    self.config.filter_settings.apply_hidden_categories != old_apply_hidden_categories;
                
                if filters_changed ||
                   self.config.filter_settings.search_text != old_search ||
                   hidden_categories_changed {
                    self.game_index_manager.mark_cache_dirty();
                    self.game_list.invalidate_cache();
                    
                    // IMMEDIATE UPDATE for filter changes
                    if hidden_categories_changed || filters_changed {
                        self.update_filtered_games_cache();
                    }
                }

                // Show performance info in debug mode with better styling
                if self.config.preferences.show_fps {
                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Performance").heading().color(egui::Color32::from_rgb(64, 156, 255)));
                    self.performance_manager.show_debug_info(ui);
                }
                
                ui.add_space(12.0); // Add bottom padding
            });

        // Right panel with improved layout - ENHANCED: Much more flexible resizing
        egui::SidePanel::right("artwork")
            .resizable(true)
            .default_width(350.0) // Default width
            .min_width(100.0) // ENHANCED: Much smaller minimum for maximum flexibility
            .max_width(1000.0) // ENHANCED: Much larger maximum for wide panels
            .show(ctx, |ui| {
                ui.add_space(12.0); // Add top padding
                
                // Split the panel vertically with better proportions
                // ENHANCED: Give history panel more space for MAME info content
                let available_height = ui.available_height();
                let artwork_height = available_height * 0.45; // 45% for artwork (reduced from 60%)
                let history_height = available_height * 0.55; // 55% for history (increased from 40%)
                
                // Top section - Artwork panel
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), artwork_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.label(egui::RichText::new("Game Artwork").heading().color(egui::Color32::from_rgb(64, 156, 255)));
                        ui.add_space(8.0);
                        self.artwork_panel.show(
                            ui,
                            &self.selected_game,
                            &self.games,
                            &self.config.extra_asset_dirs,
                            &self.config
                        );
                    }
                );
                
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
                
                // Bottom section - History panel
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), history_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.label(egui::RichText::new("Game History").heading().color(egui::Color32::from_rgb(64, 156, 255)));
                        ui.add_space(8.0);
                        
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
                
                ui.add_space(12.0); // Add bottom padding
            });

        // Main content area with improved styling
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0); // Add top padding
            
            match self.loading_stage {
                LoadingStage::LoadingMame => {
                    ui.centered_and_justified(|ui| {
                        ui.add_space(40.0);
                        ui.heading(egui::RichText::new("Loading MAME Database").heading().size(24.0));
                        ui.add_space(20.0);
                        ui.spinner();
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("Scanning MAME for available games...").size(16.0));
                        ui.label(egui::RichText::new("This may take 10-30 seconds for 40,000+ games").weak());

                        if let Some(start_time) = self.loading_start_time {
                            let elapsed = start_time.elapsed().as_secs();
                            ui.add_space(10.0);
                            ui.label(format!("Time elapsed: {} seconds", elapsed));
                        }
                    });
                }

                LoadingStage::ScanningRoms => {
                    ui.centered_and_justified(|ui| {
                        ui.add_space(40.0);
                        ui.heading(egui::RichText::new("Scanning ROM Files").heading().size(24.0));
                        ui.add_space(20.0);
                        ui.spinner();
                        ui.add_space(20.0);

                        let (current, total) = self.loading_progress;
                        if total > 0 {
                            let progress = current as f32 / total as f32;
                            ui.add(egui::ProgressBar::new(progress)
                                .text(format!("{}/{} files", current, total))
                                .desired_width(400.0)); // Increased width
                        } else {
                            ui.label(egui::RichText::new("Checking ROM directories for available games...").size(16.0));
                        }

                        ui.add_space(10.0);
                        ui.label(format!("Scanning {} ROM directories", self.config.rom_paths.len()));
                    });
                }

                LoadingStage::Error => {
                    ui.centered_and_justified(|ui| {
                        ui.add_space(40.0);
                        ui.heading(egui::RichText::new("⚠ Loading Error").heading().size(24.0).color(egui::Color32::RED));
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("Failed to load game data. Please check:").size(16.0));
                        ui.label("• MAME executable is correctly configured");
                        ui.label("• MAME executable has proper permissions");
                        ui.label("• ROM directories are accessible");
                        ui.add_space(20.0);
                        if ui.button(egui::RichText::new("Open Directories Settings").size(16.0)).clicked() {
                            self.dialog_manager.open_dialog(DialogType::Directories);
                        }
                    });
                }

                _ => {
                    if self.games.is_empty() && !self.config.rom_paths.is_empty() &&
                        self.loading_stage == LoadingStage::Complete {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(40.0);
                            ui.heading(egui::RichText::new("No games found").heading().size(24.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("ROM directories were scanned but no matching games were found.").size(16.0));
                            ui.label("Please check:");
                            ui.label("• ROM files are in .zip format");
                            ui.label("• ROM files have correct names (e.g., pacman.zip)");
                            ui.label("• ROM directories contain actual game files");
                            ui.add_space(20.0);
                            if ui.button(egui::RichText::new("Configure Directories").size(16.0)).clicked() {
                                self.dialog_manager.open_dialog(DialogType::Directories);
                            }
                        });
                    } else if self.games.is_empty() && self.loading_stage == LoadingStage::Idle {
                        ui.centered_and_justified(|ui| {
                            ui.add_space(40.0);
                            ui.heading(egui::RichText::new("Welcome to MAMEUIx").heading().size(28.0));
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new("To get started:").size(18.0));
                            ui.label("1. Configure your MAME executable");
                            ui.label("2. Add ROM directories");
                            ui.label("3. Click OK to scan for games");
                            ui.add_space(20.0);
                            if ui.button(egui::RichText::new("Configure Directories").size(16.0)).clicked() {
                                self.dialog_manager.open_dialog(DialogType::Directories);
                            }
                        });
                    } else {
                        // CRITICAL: Update filter cache BEFORE showing game list
                        if self.game_index_manager.is_cache_dirty() {
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
                                self.queue_icon_load(game_name);
                            }
                            
                            // Then queue low priority games (pre-load)
                            for game_name in low_priority_games {
                                self.queue_icon_load(game_name);
                            }
                        }
                        
                        // Show game list with improved styling - switch between table and list view
                        let theme_colors = crate::models::GameListColors::for_theme(self.config.theme.clone());
                        let (double_clicked, favorite_toggled, properties_requested) = match self.config.view_mode {
                            crate::models::config::ViewMode::Table => {
                                self.game_list.show(
                                    ui,
                                    &self.games,
                                    &self.config.filter_settings,
                                    &mut self.selected_game,
                                    &mut self.expanded_parents,
                                    &self.config.favorite_games,
                                    &mut self.icon_manager.rom_icons,
                                    self.config.show_rom_icons,
                                    self.config.icon_size,
                                    self.game_index_manager.game_index.as_ref(),
                                    self.selected_filter,
                                    &mut self.config.column_widths,
                                    &self.config.preferences.visible_columns,
                                    self.icon_manager.default_icon_texture.as_ref(),
                                    &self.config.game_stats,
                                    None, // hardware_filter - not available here
                                    self.config.catver_ini_path.is_some(),
                                    Some(self.game_index_manager.get_filtered_games()), // Pass pre-filtered indices
                                    Some(&theme_colors), // Pass theme colors
                                )
                            },
                            crate::models::config::ViewMode::List => {
                                self.game_list_view.show(
                                    ui,
                                    &self.games,
                                    &self.config.filter_settings,
                                    &mut self.selected_game,
                                    &mut self.expanded_parents,
                                    &self.config.favorite_games,
                                    &mut self.icon_manager.rom_icons,
                                    self.config.show_rom_icons,
                                    self.config.icon_size,
                                    self.game_index_manager.game_index.as_ref(),
                                    self.selected_filter,
                                    &mut self.config.column_widths,
                                    &self.config.preferences.visible_columns,
                                    self.icon_manager.default_icon_texture.as_ref(),
                                    &self.config.game_stats,
                                    None, // hardware_filter placeholder
                                    self.config.catver_ini_path.is_some(),
                                    Some(self.game_index_manager.get_filtered_games()), // Pass pre-filtered indices
                                    Some(&theme_colors), // Pass theme colors
                                )
                            }
                        };
                        
                        // Handle favorite toggle
                        if let Some(game_name) = favorite_toggled {
                            self.toggle_favorite(&game_name);
                        }
                        
                        // Handle properties request
                        if properties_requested {
                            if let Some(selected_idx) = self.selected_game {
                                if let Some(game) = self.games.get(selected_idx) {
                                    // Open the game properties dialog
                                    let dialog = crate::ui::components::game_properties::GamePropertiesDialog::new_with_config(
                                        Some(game), 
                                        &self.config
                                    );
                                    self.dialog_manager.set_game_properties_dialog(Some(dialog));
                                    self.dialog_manager.open_dialog(crate::ui::DialogType::GameProperties);
                                }
                            }
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
            
            ui.add_space(12.0); // Add bottom padding
        });

        // Improved status bar with better styling
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(40.0) // Increased height
            .show(ctx, |ui| {
                ui.add_space(8.0); // Add top padding
                ui.horizontal(|ui| {
                    match self.loading_stage {
                        LoadingStage::LoadingMame => {
                            ui.label(egui::RichText::new("Loading MAME database...").size(14.0));
                            ui.spinner();
                        }
                        LoadingStage::ScanningRoms => {
                            ui.label(egui::RichText::new("Scanning ROM files...").size(14.0));
                            ui.spinner();
                        }
                        LoadingStage::Complete => {
                            ui.label(egui::RichText::new(format!("{} games loaded", self.games.len())).size(14.0));
                        }
                        _ => {
                            ui.label(egui::RichText::new(format!("{} games", self.games.len())).size(14.0));
                            if !self.games.is_empty() {
                                let available = self.games.iter()
                                .filter(|g| matches!(g.status, RomStatus::Available))
                                .count();
                                ui.label(egui::RichText::new(format!("({} available)", available)).weak());
                            }

                            // Show filtered count if filter active
                            let filtered_games = self.game_index_manager.get_filtered_games();
                            if !filtered_games.is_empty() &&
                                filtered_games.len() < self.games.len() {
                                    ui.separator();
                                    ui.label(egui::RichText::new(format!("Showing {} filtered", filtered_games.len())).weak());
                                }
                        }
                    }

                    // Performance info with better styling
                    if self.config.preferences.show_fps {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let fps = self.performance_manager.get_average_fps();
                            let color = if fps < 20.0 {
                                egui::Color32::RED
                            } else if fps < 30.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::GREEN
                            };
                            ui.colored_label(color, egui::RichText::new(format!("FPS: {:.1}", fps)).size(14.0));
                        });
                    }
                });
                ui.add_space(8.0); // Add bottom padding
            });

        // Handle dialogs using DialogManager
        let dialog_actions = self.dialog_manager.render_dialogs(
            ctx,
            &mut self.config,
            &self.games,
            self.selected_game,
            None, // category_manager removed
            &mut self.need_reload_after_dialog,
        );
        
        // Process dialog actions
        for action in dialog_actions {
            match action {
                DialogAction::SaveConfig => self.save_config(),
                DialogAction::StartInitialLoad => self.start_initial_load(),
                DialogAction::ReloadCategories => self.reload_categories(),
                DialogAction::OnDirectoriesChanged => self.on_directories_changed(),
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
    
    fn show_toolbar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("MAME Manager").clicked() {
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    // Add ROM verification option
                    if ui.button("🔍 Verify ROMs...").clicked() {
                        self.dialog_manager.rom_verify_dialog().open();
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
                        self.dialog_manager.open_dialog(DialogType::RomInfo);
                        ui.close();
                    }
                });

                ui.menu_button("Options", |ui| {
                    if ui.button("Directories").clicked() {
                        self.dialog_manager.open_dialog(DialogType::Directories);
                        ui.close();
                    }
                    if ui.button("Preferences").clicked() {
                        self.dialog_manager.open_dialog(DialogType::Preferences);
                        ui.close();
                    }

                    
                    ui.separator();
                    
                    if ui.button("⚙️ Default Game Properties...").clicked() {
                        self.dialog_manager.set_game_properties_dialog(Some(GamePropertiesDialog::new_with_config(None, &self.config)));
                        self.dialog_manager.open_dialog(DialogType::GameProperties);
                        ui.close();
                    }
                    
                    ui.separator();
                    
                    if ui.button("🔍 Find MAME Executables").clicked() {
                        let found_mames = MameFinderDialog::find_mame_executables();
                        self.dialog_manager.set_found_mame_executables(found_mames.clone());
                        if !found_mames.is_empty() {
                            self.dialog_manager.open_dialog(DialogType::MameFinder);
                        } else {
                            self.dialog_manager.open_dialog(DialogType::ManualMame);
                        }
                        ui.close();
                    }
                    
                    ui.separator();
                });

                // Add Tools menu
                ui.menu_button("Tools", |ui| {
                    if ui.button("🔍 ROM Verification").clicked() {
                        self.dialog_manager.rom_verify_dialog().open();
                        ui.close_menu();
                    }
                    
                    if ui.button("🎯 Verify Selected ROM").clicked() {
                        if let Some(idx) = self.selected_game {
                            if let Some(game) = self.games.get(idx) {
                                self.dialog_manager.rom_verify_dialog().open();
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
                        self.dialog_manager.open_dialog(DialogType::About);
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
                        self.dialog_manager.open_dialog(DialogType::RomInfo);
                    }

                    if loading {
                        ui.add_enabled(false, egui::Button::new("🔄 Loading..."));
                    } else {
                        if ui.button("🔄 Refresh").clicked() {
                            self.on_refresh_clicked();
                        }
                    }
                    
                    ui.separator();
                    
                    // View mode toggle
                    ui.horizontal(|ui| {
                        ui.label("View:");
                        if ui.selectable_label(
                            self.config.view_mode == crate::models::config::ViewMode::Table,
                            "📊 Table"
                        ).clicked() {
                            self.config.view_mode = crate::models::config::ViewMode::Table;
                            self.save_config();
                        }
                        if ui.selectable_label(
                            self.config.view_mode == crate::models::config::ViewMode::List,
                            "📋 List"
                        ).clicked() {
                            self.config.view_mode = crate::models::config::ViewMode::List;
                            self.save_config();
                        }
                    });
                });

        });
    }
}


