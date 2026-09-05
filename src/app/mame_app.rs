// src/app/mame_app.rs
// File utama yang mengkoordinasikan seluruh aplikasi
// FIXED VERSION dengan optimasi untuk handle 48,000+ games

use crate::mame::GameScanner;
use crate::models::*;
use crate::ui::components::mame_finder::MameFinderDialog;
use crate::ui::components::steam_ui::SteamUi;
use crate::ui::dock::{DockTab, MameTabViewer, create_default_layout, dock_style};
use crate::ui::notifications::NotificationManager;
use crate::ui::panels::{
    ArtworkPanel, GameIndexManager, GameList, GameListView, HistoryPanel, IconManager,
    PerformanceManager, Sidebar, SoftwareListPanel,
};
use crate::ui::redesign::{RedesignShell, tokens::RedesignTokens};
use crate::ui::{DialogAction, DialogManager, DialogType};
use crate::utils::hardware_filter::HardwareFilter;
use crate::utils::rom_utils::RomLoader;
use eframe::egui;
use egui_dock::DockState;
use std::collections::HashMap;
use std::mem;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Only an in-flight scan should prevent a new load; errors are retryable.
fn can_start_mame_load(stage: LoadingStage) -> bool {
    matches!(
        stage,
        LoadingStage::Idle | LoadingStage::Complete | LoadingStage::Error
    )
}

#[derive(Debug, Default)]
struct PerPassRenderGuard {
    last_rendered_pass: Option<u64>,
}

impl PerPassRenderGuard {
    fn claim(&mut self, pass: u64) -> bool {
        if self.last_rendered_pass == Some(pass) {
            return false;
        }
        self.last_rendered_pass = Some(pass);
        true
    }
}

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
    pub software_list_panel: SoftwareListPanel,

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
    pub game_index_manager: GameIndexManager, // Game indexing, filtering, and search management
    pub performance_manager: PerformanceManager, // Monitor FPS dan lag

    // Category management - REMOVED

    // Dialog management
    pub dialog_manager: DialogManager,

    // Protect globally identified chrome from duplicate rendering in one pass.
    toolbar_render_guard: PerPassRenderGuard,

    pub dock_tree: DockState<DockTab>,
    pub hardware_filter: Option<HardwareFilter>,
    pub notifications: NotificationManager,
    pub redesign_shell: RedesignShell,
}

impl MameApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Register named Public Sans weight families before the first UI pass.
        // Redesign styles can then safely reference them without affecting the
        // legacy shell's default proportional font family.
        RedesignTokens::install_fonts(&cc.egui_ctx);

        let force_redesign = std::env::args().any(|a| a == "--redesign");
        let mut config = crate::config::load_config().unwrap_or_default();
        if force_redesign {
            config.preferences.ui_shell = UiShellMode::RedesignPreview;
            println!("Redesign preview shell enabled (--redesign)");
        }

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
            software_list_panel: SoftwareListPanel::new(),
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
            game_index_manager: GameIndexManager::new().with_settings(
                config.preferences.performance.search_debounce_ms,
                100, // max_cache_size
            ),
            performance_manager: PerformanceManager::new(),

            // Dialog management
            dialog_manager: DialogManager::new(),

            // Theme management
            toolbar_render_guard: PerPassRenderGuard::default(),
            dock_tree: create_default_layout(),
            hardware_filter: HardwareFilter::load_from_config(&config),
            notifications: NotificationManager::new(),
            redesign_shell: RedesignShell::default(),
        };

        // Category loading removed - this functionality is no longer needed

        // Initialize MAME finder dialog if needed
        if show_mame_finder {
            app.dialog_manager
                .set_found_mame_executables(found_mames.clone());
            if !found_mames.is_empty() {
                app.dialog_manager.open_dialog(DialogType::MameFinder);
            } else {
                app.dialog_manager.open_dialog(DialogType::ManualMame);
            }
        }

        // Only start initial load if we already have MAME configured
        if !app.config.mame_executables.is_empty()
            && app.config.selected_mame_index < app.config.mame_executables.len()
        {
            println!(
                "Initial load: Found {} MAME executables",
                app.config.mame_executables.len()
            );
            app.start_initial_load();
        }

        // Apply initial theme
        // Note: This will be applied when the first frame is rendered
        println!("Initial theme: {}", app.config.theme.display_name());

        app
    }

    /// Build game index untuk fast lookup - CRITICAL untuk performance!
    pub fn build_game_index(&mut self) {
        self.game_index_manager
            .build_game_index(&self.games, &self.config.favorite_games);
        self.game_list.invalidate_cache();
        self.game_list_view.invalidate_cache();
        self.redesign_shell.state.mark_table_dirty();
        self.redesign_shell.state.mark_sidebar_stats_dirty();
    }

    /// OPTIMIZED: Update filtered games cache dengan GameIndex
    pub fn update_filtered_games_cache(&mut self) {
        self.game_index_manager.update_filtered_games_cache(
            &self.games,
            self.selected_filter,
            &self.config.filter_settings,
            &self.config.hidden_categories,
            self.hardware_filter.as_ref(),
        );
    }

    pub fn reload_hardware_filter(&mut self) {
        self.hardware_filter = HardwareFilter::load_from_config(&self.config);
        self.game_index_manager.mark_cache_dirty();
        self.game_list.invalidate_cache();

        if self.config.preferences.enable_toast_notifications {
            if let Some(hw) = &self.hardware_filter {
                self.notifications.success(
                    "Hardware INI loaded",
                    format!(
                        "{} CPUs, {} devices, {} sound chips",
                        hw.cpu_count(),
                        hw.device_count(),
                        hw.sound_count()
                    ),
                );
            } else {
                self.notifications.warning(
                    "Hardware INI",
                    "cpu.ini / device.ini / sound.ini not found in INI directory",
                );
            }
        }
    }

    pub fn launch_game_at_index(&mut self, idx: usize) {
        let Some(game) = self.games.get(idx) else {
            return;
        };
        let game_name = game.name.clone();
        let game_description = game.description.clone();

        match crate::mame::launch_game(&game_name, &self.config) {
            Ok(child) => {
                self.running_games
                    .insert(game_name, (child, Instant::now()));
                if self.config.preferences.enable_toast_notifications {
                    self.notifications.info("Launching", game_description);
                }
            }
            Err(error) => {
                if self.config.preferences.enable_toast_notifications {
                    self.notifications.error("Launch failed", error.to_string());
                }
            }
        }
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
        if !can_start_mame_load(self.loading_stage) {
            println!("Load already in progress, skipping...");
            return;
        }

        let mame = match self
            .config
            .mame_executables
            .get(self.config.selected_mame_index)
        {
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
                "Running mame -listxml... This may take 10-30 seconds".to_string(),
            ));

            match scanner.scan_games() {
                Ok(games) => {
                    println!("MAME scan complete: {} games found", games.len());

                    let mut manufacturers: Vec<String> = games
                        .iter()
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

        let valid_dirs: Vec<_> = self
            .config
            .rom_paths
            .iter()
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
            let progress_tx = tx.clone();
            let games = loader.load_roms_with_progress(metadata, move |current, total| {
                let _ = progress_tx.send(LoadingMessage::RomScanProgress(current, total));
            });

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

                        self.game_metadata =
                            games.iter().map(|g| (g.name.clone(), g.clone())).collect();

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

                        if self.config.preferences.enable_toast_notifications {
                            self.notifications.success(
                                "Games loaded",
                                format!("{} games ready", self.games.len()),
                            );
                        }
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
        self.reload_hardware_filter();
        self.software_list_panel.invalidate();
        self.games.clear();
        self.game_index_manager.reset();

        if !self.config.mame_executables.is_empty()
            && self.config.selected_mame_index < self.config.mame_executables.len()
        {
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

    pub fn save_config(&mut self) {
        self.config.preferences.sync_legacy_layout_flag();
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
        self.game_index_manager
            .update_favorites(&self.games, &self.config.favorite_games);
        self.save_config();
    }

    pub fn update_game_stats(&mut self, rom_name: &str, play_time: u32) {
        let stats = self
            .config
            .game_stats
            .entry(rom_name.to_string())
            .or_default();

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
        self.icon_manager
            .init_default_icon(ctx, self.config.icon_size);
    }

    pub fn queue_icon_load(&mut self, rom_name: String) {
        self.icon_manager.queue_icon_load(
            rom_name,
            self.config.preferences.performance.enable_lazy_icons,
        );
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
                self.game_list.visible_end =
                    (target_start + visible_rows).min(self.game_list.expanded_rows_cache.len());

                // Force the game list to scroll to this position
                // We'll need to add a flag to tell the game list to scroll
                self.game_list.scroll_to_row = Some(row_index);

                println!(
                    "Jumping to game at row {} starting with '{}'",
                    row_index, character
                );
            }
        } else {
            println!("No game found starting with '{}'", character);
        }
    }

    /// Move selection up/down; page=true jumps ~20 rows (Page Up/Down).
    pub fn navigate_game_selection(&mut self, direction: i32, page: bool) {
        if self.games.is_empty() {
            return;
        }

        if self.game_index_manager.is_cache_dirty() {
            self.update_filtered_games_cache();
        }

        let filtered = self.game_index_manager.get_filtered_games();
        if filtered.is_empty() {
            return;
        }

        let step = if page { 20 } else { 1 };
        let delta = direction * step;

        let current_pos = self
            .selected_game
            .and_then(|idx| filtered.iter().position(|&i| i == idx))
            .unwrap_or(0);

        let new_pos = (current_pos as i32 + delta).clamp(0, filtered.len() as i32 - 1) as usize;
        let new_game_idx = filtered[new_pos];
        self.selected_game = Some(new_game_idx);

        let is_table = matches!(
            self.config.view_mode,
            crate::models::config::ViewMode::Table
        );
        if is_table {
            let row = self
                .game_list
                .row_for_game_idx(new_game_idx)
                .unwrap_or(new_pos);
            self.game_list.scroll_to_row = Some(row);
        } else {
            let row = self
                .game_list_view
                .row_for_game_idx(new_game_idx)
                .unwrap_or(new_pos);
            self.game_list_view.scroll_to_row = Some(row);
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
        if self.game_index_manager.has_pending_search()
            && self.game_index_manager.should_process_pending_search(
                self.config.preferences.performance.search_debounce_ms,
            )
        {
            self.process_pending_search();
            needs_repaint = true;
        }

        // Handle keyboard input for game list navigation
        let mut should_process_keyboard_nav = false;
        let mut typed_char = None;
        let mut list_nav_direction: Option<i32> = None;
        let mut list_nav_page = false;

        // Check if any dialog is open
        let dialog_open = self.dialog_manager.is_any_dialog_open();
        let keyboard_blocked = ctx.wants_keyboard_input();

        ctx.input(|i| {
            let search_active = self.game_index_manager.has_pending_search();

            if !self.games.is_empty() && !dialog_open && !keyboard_blocked && !search_active {
                for event in &i.events {
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        if modifiers.alt || modifiers.ctrl || modifiers.command {
                            continue;
                        }
                        match key {
                            egui::Key::PageDown => {
                                list_nav_direction = Some(1);
                                list_nav_page = true;
                                break;
                            }
                            egui::Key::PageUp => {
                                list_nav_direction = Some(-1);
                                list_nav_page = true;
                                break;
                            }
                            egui::Key::ArrowDown => {
                                list_nav_direction = Some(1);
                                break;
                            }
                            egui::Key::ArrowUp => {
                                list_nav_direction = Some(-1);
                                break;
                            }
                            _ => {}
                        }
                    }

                    if let egui::Event::Text(text) = event
                        && list_nav_direction.is_none()
                        && let Some(first_char) = text.chars().next()
                        && first_char.is_alphanumeric()
                    {
                        should_process_keyboard_nav = true;
                        typed_char = Some(first_char);
                        break;
                    }
                }
            }
        });

        if let Some(direction) = list_nav_direction {
            self.navigate_game_selection(direction, list_nav_page);
            needs_repaint = true;
        } else if should_process_keyboard_nav && let Some(character) = typed_char {
            self.jump_to_game_starting_with(character);
            needs_repaint = true;
        }

        // Update filter cache with rate limiting
        if self.game_index_manager.is_cache_dirty()
            && self.game_index_manager.last_filter_update.elapsed() > Duration::from_millis(10)
        {
            self.update_filtered_games_cache();
            needs_repaint = true;
        }

        // Cleanup resources periodically (not every frame!)
        if self.icon_manager.last_icon_cleanup.elapsed() > Duration::from_secs(120) {
            self.cleanup_resources();
        }

        // Smart repaint scheduling
        if needs_repaint
            || (self.loading_stage == LoadingStage::LoadingMame
                || self.loading_stage == LoadingStage::ScanningRoms)
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        if self.icon_manager.default_icon_texture.is_none() && self.config.show_rom_icons {
            self.init_default_icon(ctx);
        }

        if self.performance_manager.frame_count.is_multiple_of(30) {
            self.check_running_games();
        }

        if self.config.preferences.performance.enable_lazy_icons {
            self.process_icon_queue(ctx);
        }

        self.update_game_verification_statuses();

        self.show_main_layout(ctx);

        // Dialogs + toasts run for both legacy and redesign shells
        let dialog_actions = self.dialog_manager.render_dialogs(
            ctx,
            &mut self.config,
            &self.games,
            self.selected_game,
            None,
            &mut self.need_reload_after_dialog,
        );

        for action in dialog_actions {
            match action {
                DialogAction::SaveConfig => self.save_config(),
                DialogAction::StartInitialLoad => self.start_initial_load(),
                DialogAction::ReloadCategories => self.reload_categories(),
                DialogAction::OnDirectoriesChanged => self.on_directories_changed(),
            }
        }

        if self.config.preferences.enable_toast_notifications {
            self.notifications.show(ctx);
        }
    }
}

impl MameApp {
    // Add this helper function to show plugin support info
    fn show_plugin_info(&self) {
        if let Some(mame) = self
            .config
            .mame_executables
            .get(self.config.selected_mame_index)
        {
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

    fn show_main_layout(&mut self, ctx: &egui::Context) {
        if self.config.preferences.ui_shell == UiShellMode::RedesignPreview {
            let mut shell = mem::take(&mut self.redesign_shell);
            shell.show(ctx, self);
            self.redesign_shell = shell;
        } else {
            // Apply the selected full style before every legacy frame. Dialogs can
            // change the theme or shell live, so a one-time flag can become stale.
            self.redesign_shell.state.style_applied = false;
            self.config.theme.apply(ctx);
            // Both legacy layouts share one toolbar; each renders only its content.
            self.show_toolbar(ctx);

            // Reserve the footer before the content panels consume the remaining area.
            egui::TopBottomPanel::bottom("status_bar")
                .frame(
                    egui::Frame::new()
                        .fill(ctx.style().visuals.panel_fill)
                        .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke)
                        .inner_margin(egui::Margin::symmetric(16, 8)),
                )
                .exact_height(40.0)
                .show(ctx, |ui| {
                    self.show_legacy_status_bar(ui);
                });

            match self.config.preferences.ui_shell {
                UiShellMode::LegacyClassic => self.show_classic_content(ctx),
                UiShellMode::LegacyDock | UiShellMode::RedesignPreview => {
                    self.show_dock_layout(ctx);
                }
            }
        }
    }

    fn show_classic_content(&mut self, ctx: &egui::Context) {
        let outer_frame = egui::Frame::new()
            .fill(ctx.style().visuals.panel_fill)
            .inner_margin(egui::Margin::same(12));
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(300.0)
            .min_width(100.0)
            .max_width(500.0)
            .frame(outer_frame)
            .show(ctx, |ui| self.render_sidebar_panel(ui));

        egui::SidePanel::right("artwork")
            .resizable(true)
            .default_width(350.0)
            .min_width(100.0)
            .max_width(1000.0)
            .frame(outer_frame)
            .show(ctx, |ui| {
                let artwork_height = (ui.available_height() - 12.0).max(0.0) * 0.45;
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), artwork_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        SteamUi::content_frame(ui.style()).show(ui, |ui| {
                            self.render_artwork_panel(ui);
                        });
                    },
                );
                ui.add_space(12.0);
                SteamUi::content_frame(ui.style()).show(ui, |ui| {
                    self.render_history_panel(ui);
                });
            });

        egui::CentralPanel::default()
            .frame(outer_frame)
            .show(ctx, |ui| {
                SteamUi::content_frame(ui.style()).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    self.render_game_list_panel(ui, ctx);
                });
            });
    }

    pub fn show_dock_layout(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut tree = mem::replace(&mut self.dock_tree, create_default_layout());
            let mut viewer = MameTabViewer { ctx, app: self };
            egui_dock::DockArea::new(&mut tree)
                .style(dock_style(ui))
                .show_inside(ui, &mut viewer);
            self.dock_tree = tree;
        });
    }

    pub fn render_sidebar_panel(&mut self, ui: &mut egui::Ui) {
        let old_search = self.config.filter_settings.search_text.clone();
        let old_search_mode = self.config.filter_settings.search_mode.clone();
        let old_hide_romless = self.config.filter_settings.hide_romless_systems;
        let old_hidden_categories_len = self.config.hidden_categories.len();
        let old_apply_hidden_categories = self.config.filter_settings.apply_hidden_categories;
        let old_availability = self.config.filter_settings.availability_filters.clone();
        let old_status = self.config.filter_settings.status_filters.clone();
        let old_others = self.config.filter_settings.other_filters.clone();
        let old_cpu = self.config.filter_settings.cpu_filter.clone();
        let old_device = self.config.filter_settings.device_filter.clone();
        let old_sound = self.config.filter_settings.sound_filter.clone();
        let old_manufacturer = self.config.filter_settings.manufacturer.clone();
        let old_selected_manufacturers = self.config.filter_settings.selected_manufacturers.clone();

        egui::ScrollArea::vertical()
            .id_salt("main_sidebar_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.sidebar.show(
                    ui,
                    &mut self.selected_filter,
                    &mut self.config.filter_settings,
                    None,
                    &mut self.config.hidden_categories,
                    &mut self.dialog_manager,
                    self.hardware_filter.as_ref(),
                    &self.all_manufacturers,
                )
            });

        let rom_requirement_changed =
            self.config.filter_settings.hide_romless_systems != old_hide_romless;
        let filters_changed = rom_requirement_changed
            || self.config.filter_settings.search_mode != old_search_mode
            || self
                .config
                .filter_settings
                .availability_filters
                .show_available
                != old_availability.show_available
            || self
                .config
                .filter_settings
                .availability_filters
                .show_unavailable
                != old_availability.show_unavailable
            || self.config.filter_settings.status_filters.show_working != old_status.show_working
            || self.config.filter_settings.status_filters.show_not_working
                != old_status.show_not_working
            || self.config.filter_settings.other_filters.show_favorites
                != old_others.show_favorites
            || self.config.filter_settings.other_filters.show_parents_only
                != old_others.show_parents_only
            || self.config.filter_settings.other_filters.show_chd_games
                != old_others.show_chd_games
            || self.config.filter_settings.cpu_filter != old_cpu
            || self.config.filter_settings.device_filter != old_device
            || self.config.filter_settings.sound_filter != old_sound
            || self.config.filter_settings.manufacturer != old_manufacturer
            || self.config.filter_settings.selected_manufacturers != old_selected_manufacturers;

        let hidden_categories_changed = self.config.hidden_categories.len()
            != old_hidden_categories_len
            || self.config.filter_settings.apply_hidden_categories != old_apply_hidden_categories;

        if filters_changed
            || self.config.filter_settings.search_text != old_search
            || hidden_categories_changed
        {
            self.game_index_manager.mark_cache_dirty();
            self.game_list.invalidate_cache();
            self.game_list_view.invalidate_cache();
            if hidden_categories_changed || filters_changed {
                self.update_filtered_games_cache();
            }
        }

        if rom_requirement_changed {
            self.redesign_shell.state.mark_table_dirty();
            self.redesign_shell.state.mark_sidebar_stats_dirty();
            self.save_config();
            ui.ctx().request_repaint();
        }

        if self.config.preferences.show_fps {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Performance")
                    .size(14.5)
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            self.performance_manager.show_debug_info(ui);
        }
        ui.add_space(12.0);
    }

    pub fn render_artwork_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Game Artwork")
                .size(14.5)
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
        ui.add_space(8.0);
        self.artwork_panel.show(
            ui,
            &self.selected_game,
            &self.games,
            &self.config.extra_asset_dirs,
            &self.config,
        );
    }

    pub fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.label(
            egui::RichText::new("Game History")
                .size(14.5)
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
        ui.add_space(8.0);
        if let Some(idx) = self.selected_game {
            if let Some(game) = self.games.get(idx) {
                self.history_panel.set_selected_game(
                    Some(game.name.clone()),
                    Some(game.name.clone()),
                    &self.config,
                );
            }
        } else {
            self.history_panel
                .set_selected_game(None, None, &self.config);
        }
        self.history_panel.show(ui, &self.config);
    }

    pub fn render_software_list_panel(&mut self, ui: &mut egui::Ui) {
        self.software_list_panel.show(ui, &self.config);
    }

    pub fn render_game_list_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let _ = ctx;

        match self.loading_stage {
            LoadingStage::LoadingMame => {
                ui.centered_and_justified(|ui| {
                    ui.add_space(40.0);
                    ui.heading(
                        egui::RichText::new("Loading MAME Database")
                            .heading()
                            .size(24.0),
                    );
                    ui.add_space(20.0);
                    ui.spinner();
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Scanning MAME for available games...").size(16.0),
                    );
                    ui.label(
                        egui::RichText::new("This may take 10-30 seconds for 40,000+ games").weak(),
                    );

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
                    ui.heading(
                        egui::RichText::new("Scanning ROM Files")
                            .heading()
                            .size(24.0),
                    );
                    ui.add_space(20.0);
                    ui.spinner();
                    ui.add_space(20.0);

                    let (current, total) = self.loading_progress;
                    if total > 0 {
                        let progress = current as f32 / total as f32;
                        ui.add(
                            egui::ProgressBar::new(progress)
                                .text(format!("{}/{} files", current, total))
                                .desired_width(400.0),
                        ); // Increased width
                    } else {
                        ui.label(
                            egui::RichText::new("Checking ROM directories for available games...")
                                .size(16.0),
                        );
                    }

                    ui.add_space(10.0);
                    ui.label(format!(
                        "Scanning {} ROM directories",
                        self.config.rom_paths.len()
                    ));
                });
            }

            LoadingStage::Error => {
                ui.centered_and_justified(|ui| {
                    ui.add_space(40.0);
                    ui.heading(
                        egui::RichText::new("⚠ Loading Error")
                            .heading()
                            .size(24.0)
                            .color(ui.visuals().error_fg_color),
                    );
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Failed to load game data. Please check:").size(16.0),
                    );
                    ui.label("• MAME executable is correctly configured");
                    ui.label("• MAME executable has proper permissions");
                    ui.label("• ROM directories are accessible");
                    ui.add_space(20.0);
                    if ui
                        .button(egui::RichText::new("Open Directories Settings").size(16.0))
                        .clicked()
                    {
                        self.dialog_manager.open_dialog(DialogType::Directories);
                    }
                });
            }

            _ => {
                if self.games.is_empty()
                    && !self.config.rom_paths.is_empty()
                    && self.loading_stage == LoadingStage::Complete
                {
                    ui.centered_and_justified(|ui| {
                        ui.add_space(40.0);
                        ui.heading(egui::RichText::new("No games found").heading().size(24.0));
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(
                                "ROM directories were scanned but no matching games were found.",
                            )
                            .size(16.0),
                        );
                        ui.label("Please check:");
                        ui.label("• ROM files are in .zip format");
                        ui.label("• ROM files have correct names (e.g., pacman.zip)");
                        ui.label("• ROM directories contain actual game files");
                        ui.add_space(20.0);
                        if ui
                            .button(egui::RichText::new("Configure Directories").size(16.0))
                            .clicked()
                        {
                            self.dialog_manager.open_dialog(DialogType::Directories);
                        }
                    });
                } else if self.games.is_empty() && self.loading_stage == LoadingStage::Idle {
                    ui.centered_and_justified(|ui| {
                        ui.add_space(40.0);
                        ui.heading(
                            egui::RichText::new("Welcome to MAMEUIx")
                                .heading()
                                .size(28.0),
                        );
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new("To get started:").size(18.0));
                        ui.label("1. Configure your MAME executable");
                        ui.label("2. Add ROM directories");
                        ui.label("3. Click OK to scan for games");
                        ui.add_space(20.0);
                        if ui
                            .button(egui::RichText::new("Configure Directories").size(16.0))
                            .clicked()
                        {
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
                        let preload_end =
                            (visible_end + 10).min(self.game_list.expanded_rows_cache.len());

                        // Collect game names to queue with priority
                        let mut high_priority_games = Vec::new();
                        let mut low_priority_games = Vec::new();

                        // Process all games in extended range
                        if let Some(rows) = self
                            .game_list
                            .expanded_rows_cache
                            .get(preload_start..preload_end)
                        {
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
                    let theme_colors =
                        crate::models::GameListColors::for_theme(self.config.theme.clone());
                    let (play_requested, favorite_toggled, properties_requested) =
                        match self.config.view_mode {
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
                                    self.hardware_filter.as_ref(),
                                    self.config.catver_ini_path.is_some(),
                                    Some(self.game_index_manager.get_filtered_games()), // Pass pre-filtered indices
                                    Some(&theme_colors), // Pass theme colors
                                )
                            }
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
                                    self.hardware_filter.as_ref(),
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
                    if let Some(target_idx) = properties_requested
                        && let Some(game) = self.games.get(target_idx)
                    {
                        let dialog = crate::ui::components::game_properties::GamePropertiesDialog::new_with_config(
                                Some(game),
                                &self.config
                            );
                        self.dialog_manager.set_game_properties_dialog(Some(dialog));
                        self.dialog_manager
                            .open_dialog(crate::ui::DialogType::GameProperties);
                    }

                    // Handle double-click to launch game
                    if let Some(idx) = play_requested {
                        self.launch_game_at_index(idx);
                    }
                }
            }
        }

        ui.add_space(12.0); // Add bottom padding
    }

    fn show_legacy_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
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
                    ui.label(
                        egui::RichText::new(format!("{} games loaded", self.games.len()))
                            .size(14.0),
                    );
                }
                _ => {
                    ui.label(egui::RichText::new(format!("{} games", self.games.len())).size(14.0));
                    if !self.games.is_empty() {
                        let available = self
                            .games
                            .iter()
                            .filter(|g| matches!(g.status, RomStatus::Available))
                            .count();
                        ui.label(egui::RichText::new(format!("({available} available)")).weak());
                    }

                    let filtered_games = self.game_index_manager.get_filtered_games();
                    if !filtered_games.is_empty() && filtered_games.len() < self.games.len() {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "Showing {} filtered",
                                filtered_games.len()
                            ))
                            .weak(),
                        );
                    }
                }
            }

            if !self.config.mame_executables.is_empty()
                && self.config.selected_mame_index < self.config.mame_executables.len()
            {
                let mame_version =
                    &self.config.mame_executables[self.config.selected_mame_index].version;
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("🎮 MAME {mame_version}")).size(12.0),
                        )
                        .fill(ui.visuals().faint_bg_color)
                        .min_size(egui::Vec2::new(80.0, 24.0)),
                    );
                });
            }

            if self.config.preferences.show_fps {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fps = self.performance_manager.get_average_fps();
                    let color = if fps < 20.0 {
                        SteamUi::DANGER
                    } else if fps < 30.0 {
                        SteamUi::WARNING
                    } else {
                        SteamUi::SUCCESS
                    };
                    ui.colored_label(
                        color,
                        egui::RichText::new(format!("FPS: {fps:.1}")).size(14.0),
                    );
                });
            }
        });
        ui.add_space(8.0);
    }

    fn show_toolbar(&mut self, ctx: &egui::Context) {
        if !self.toolbar_render_guard.claim(ctx.cumulative_pass_nr()) {
            return;
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(ctx.style().visuals.panel_fill)
                    .stroke(ctx.style().visuals.widgets.noninteractive.bg_stroke)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        ui.separator();

                        // Add ROM verification option
                        if ui.button("🔍 Verify ROMs...").clicked() {
                            self.dialog_manager.rom_verify_dialog().open();
                            ui.close();
                        }

                        ui.separator();

                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });

                    ui.menu_button("Game", |ui| {
                        if ui.button("🎮 Play").clicked() {
                            if let Some(idx) = self.selected_game {
                                self.launch_game_at_index(idx);
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
                        // New modern UI for Directories & Paths
                        if ui.button("📁 Directories & Paths").clicked() {
                            self.dialog_manager.open_dialog(DialogType::Directories);
                            ui.close();
                        }

                        if ui.button("Preferences").clicked() {
                            self.dialog_manager.open_dialog(DialogType::Preferences);
                            ui.close();
                        }

                        ui.menu_button("Advanced MAME Settings", |ui| {
                            if ui.button("⚙️ Advanced MAME Settings").clicked() {
                                self.dialog_manager.set_advanced_mame_settings_dialog(Some(
                                    crate::ui::components::AdvancedMameSettingsDialog::new(
                                        &self.config,
                                    ),
                                ));
                                self.dialog_manager
                                    .open_dialog(DialogType::AdvancedMameSettings);
                                ui.close();
                            }
                        });

                        ui.separator();

                        if ui.button("🔍 Find MAME Executables").clicked() {
                            let found_mames = MameFinderDialog::find_mame_executables();
                            self.dialog_manager
                                .set_found_mame_executables(found_mames.clone());
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
                            ui.close();
                        }

                        if ui.button("🎯 Verify Selected ROM").clicked()
                            && self.selected_game.is_some()
                        {
                            self.dialog_manager.rom_verify_dialog().open();
                            ui.close();
                        }

                        ui.separator();

                        if ui.button("📊 Plugin Support Info").clicked() {
                            self.show_plugin_info();
                            ui.close();
                        }
                    });

                    ui.menu_button("Help", |ui| {
                        if ui.button("About").clicked() {
                            self.dialog_manager.open_dialog(DialogType::About);
                            ui.close();
                        }
                    });
                });

                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    let loading = matches!(
                        self.loading_stage,
                        LoadingStage::LoadingMame | LoadingStage::ScanningRoms
                    );
                    ui.add_enabled_ui(!loading, |ui| {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("▶ Play").strong())
                                    .fill(ui.visuals().selection.bg_fill)
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        ui.visuals().hyperlink_color,
                                    ))
                                    .min_size(egui::vec2(100.0, 36.0)),
                            )
                            .clicked()
                            && let Some(idx) = self.selected_game
                        {
                            self.launch_game_at_index(idx);
                        }
                        if ui
                            .add_sized([120.0, 36.0], egui::Button::new("ℹ Properties"))
                            .clicked()
                        {
                            self.dialog_manager.open_dialog(DialogType::RomInfo);
                        }
                        ui.separator();
                        ui.label(
                            egui::RichText::new("View:").color(ui.visuals().weak_text_color()),
                        );
                        for (mode, label) in [
                            (crate::models::config::ViewMode::Table, "⊞ Table"),
                            (crate::models::config::ViewMode::List, "☰ List"),
                        ] {
                            if ui
                                .add_sized(
                                    [80.0, 36.0],
                                    egui::Button::selectable(self.config.view_mode == mode, label),
                                )
                                .clicked()
                            {
                                self.config.view_mode = mode;
                                self.save_config();
                            }
                        }
                    });
                    ui.separator();
                    if loading {
                        ui.add_enabled(false, egui::Button::new("🔄 Loading..."));
                    } else if ui
                        .add_sized([100.0, 36.0], egui::Button::new("🔄 Refresh"))
                        .clicked()
                    {
                        self.on_refresh_clicked();
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn toolbar_render_guard_allows_only_one_render_per_egui_pass() {
        let mut guard = PerPassRenderGuard::default();

        assert!(guard.claim(41));
        assert!(!guard.claim(41));
        assert!(guard.claim(42));
        assert!(!guard.claim(42));
    }

    use super::*;

    fn app_for_loading_test(config: AppConfig) -> MameApp {
        MameApp {
            icon_manager: IconManager::new(&config),
            config,
            games: Vec::new(),
            game_metadata: HashMap::new(),
            selected_filter: FilterCategory::All,
            selected_game: None,
            game_list: GameList::new(),
            game_list_view: GameListView::new(),
            sidebar: Sidebar::new(),
            artwork_panel: ArtworkPanel::new(),
            history_panel: HistoryPanel::new(),
            software_list_panel: SoftwareListPanel::new(),
            all_manufacturers: Vec::new(),
            running_games: HashMap::new(),
            expanded_parents: HashMap::new(),
            loading_rx: None,
            loading_stage: LoadingStage::Error,
            loading_progress: (0, 0),
            loading_start_time: None,
            need_reload_after_dialog: false,
            roms_loading: false,
            roms_tx: None,
            game_index_manager: GameIndexManager::new(),
            performance_manager: PerformanceManager::new(),
            dialog_manager: DialogManager::new(),
            toolbar_render_guard: PerPassRenderGuard::default(),
            dock_tree: create_default_layout(),
            hardware_filter: None,
            notifications: NotificationManager::new(),
            redesign_shell: RedesignShell::default(),
        }
    }

    #[test]
    fn main_style_tracks_theme_and_shell_changes_after_preferences() {
        use crate::ui::components::preferences::PreferencesDialog;

        let mut app = app_for_loading_test(AppConfig::default());
        let context = egui::Context::default();
        crate::ui::redesign::fonts::install(&context);
        let mut time = 0.0;
        for (mode, theme) in [
            (UiShellMode::LegacyClassic, Theme::DarkBlue),
            (UiShellMode::LegacyClassic, Theme::ModernSpacious),
            (UiShellMode::LegacyDock, Theme::LightClassic),
            (UiShellMode::RedesignPreview, Theme::ModernSpacious),
            (UiShellMode::LegacyClassic, Theme::ModernSpacious),
            (UiShellMode::RedesignPreview, Theme::ModernSpacious),
            (UiShellMode::LegacyDock, Theme::ModernSpacious),
        ] {
            app.config.preferences.ui_shell = mode;
            app.config.theme = theme.clone();
            let expected = egui::Context::default();
            if mode == UiShellMode::RedesignPreview {
                RedesignTokens::apply(&expected);
            } else if theme == Theme::ModernSpacious {
                // Compare the main window with the reference dialog style,
                // rather than merely checking Theme::apply against itself.
                SteamUi::apply(&expected);
            } else {
                theme.apply(&expected);
            }
            time += 0.05;
            let _ = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 1000.0),
                    )),
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    app.show_main_layout(ctx);
                    let main_style = ctx.style();
                    assert_eq!(main_style.visuals, expected.style().visuals);
                    assert_eq!(main_style.text_styles, expected.style().text_styles);
                    assert_eq!(
                        main_style.spacing.button_padding,
                        expected.style().spacing.button_padding
                    );
                    assert_eq!(
                        main_style.spacing.item_spacing,
                        expected.style().spacing.item_spacing
                    );
                    let mut open = true;
                    PreferencesDialog::show(
                        ctx,
                        &mut app.config.preferences,
                        &mut app.config.theme,
                        &mut open,
                        false,
                    );
                    assert_eq!(ctx.style().as_ref(), main_style.as_ref());
                },
            );
        }
    }

    #[test]
    fn shell_switching_renders_each_legacy_toolbar_control_once() {
        fn count_label(shape: &egui::Shape, label: &str) -> usize {
            match shape {
                egui::Shape::Text(text) => usize::from(text.galley.job.text == label),
                egui::Shape::Vec(shapes) => {
                    shapes.iter().map(|shape| count_label(shape, label)).sum()
                }
                _ => 0,
            }
        }

        for stage in [LoadingStage::Error, LoadingStage::Complete] {
            let mut app = app_for_loading_test(AppConfig::default());
            app.loading_stage = stage;
            let context = egui::Context::default();
            crate::ui::redesign::fonts::install(&context);
            let mut time = 0.0;

            for mode in [
                UiShellMode::LegacyClassic,
                UiShellMode::LegacyDock,
                UiShellMode::LegacyClassic,
                UiShellMode::RedesignPreview,
                UiShellMode::LegacyClassic,
            ] {
                app.config.preferences.ui_shell = mode;
                // Exercise the same shell dispatch as App::update. The initial
                // frame measures toolbar groups before egui paints their contents.
                for frame in 0..3 {
                    time += 0.05;
                    let output = context.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(1280.0, 900.0),
                            )),
                            time: Some(time),
                            ..Default::default()
                        },
                        |ctx| app.show_main_layout(ctx),
                    );
                    if frame == 0 {
                        continue;
                    }
                    for label in [
                        "File",
                        "Game",
                        "Options",
                        "Tools",
                        "Help",
                        "▶ Play",
                        "ℹ Properties",
                        "⊞ Table",
                        "☰ List",
                        "🔄 Refresh",
                    ] {
                        let count: usize = output
                            .shapes
                            .iter()
                            .map(|shape| count_label(&shape.shape, label))
                            .sum();
                        assert_eq!(
                            count,
                            usize::from(mode != UiShellMode::RedesignPreview),
                            "unexpected count for {label:?} in {mode:?}, {stage:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn refresh_restarts_a_failed_initial_mame_load() {
        let directory = tempfile::tempdir().unwrap();
        let config = AppConfig {
            mame_executables: vec![MameExecutable {
                name: "Retry fixture".into(),
                path: directory.path().join("missing-mame").display().to_string(),
                version: String::new(),
                total_games: 0,
                working_games: 0,
            }],
            ..AppConfig::default()
        };
        let mut app = app_for_loading_test(config);

        app.on_refresh_clicked();

        assert_eq!(app.loading_stage, LoadingStage::LoadingMame);
        assert!(app.loading_rx.is_some());
        assert!(app.loading_start_time.is_some());
    }

    #[test]
    fn retry_guard_preserves_an_in_flight_scan_receiver() {
        for stage in [LoadingStage::LoadingMame, LoadingStage::ScanningRoms] {
            let mut app = app_for_loading_test(AppConfig::default());
            let (sender, receiver) = mpsc::channel();
            app.loading_stage = stage;
            app.loading_rx = Some(receiver);

            app.load_mame_data_threaded();

            assert_eq!(app.loading_stage, stage);
            sender.send(LoadingMessage::MameLoadStarted).unwrap();
            assert!(matches!(
                app.loading_rx.as_ref().unwrap().try_recv(),
                Ok(LoadingMessage::MameLoadStarted)
            ));
        }
    }

    #[test]
    fn changing_sidebar_search_mode_recomputes_an_unchanged_query() {
        fn game(name: &str, title: &str, manufacturer: &str) -> Game {
            Game {
                name: name.into(),
                description: title.into(),
                manufacturer: manufacturer.into(),
                year: "1990".into(),
                driver: "fixture.cpp".into(),
                driver_status: "good".into(),
                status: RomStatus::Available,
                parent: None,
                category: String::new(),
                play_count: 0,
                is_clone: false,
                is_device: false,
                is_bios: false,
                controls: String::new(),
                requires_chd: false,
                requires_roms: true,
                chd_name: None,
                verification_status: None,
            }
        }
        fn label_position(output: &egui::FullOutput, label: &str) -> egui::Pos2 {
            fn find(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
                match shape {
                    egui::Shape::Text(text) if text.galley.job.text == label => {
                        Some(text.pos + text.galley.size() / 2.0)
                    }
                    egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| find(shape, label)),
                    _ => None,
                }
            }
            // Popup labels are painted after the sidebar's identical section
            // label; interact with the topmost matching text.
            output
                .shapes
                .iter()
                .rev()
                .find_map(|shape| find(&shape.shape, label))
                .unwrap()
        }
        fn click(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        }

        let mut app = app_for_loading_test(AppConfig::default());
        app.games = vec![
            game("sega", "Capcom Fighter", "Sega"),
            game("capcom", "Fighter", "Capcom"),
        ];
        app.config.filter_settings.search_text = "Capcom".into();
        app.build_game_index();
        app.update_filtered_games_cache();
        assert_eq!(app.game_index_manager.get_filtered_games(), &[0]);

        let context = egui::Context::default();
        let mut time = 0.0;
        let mut frame = |events| {
            time += 0.05;
            context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(360.0, 900.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.render_sidebar_panel(ui));
                },
            )
        };
        frame(Vec::new());
        let output = frame(Vec::new());
        let mode = label_position(&output, "🎯 Game Title");
        frame(click(mode, true));
        frame(click(mode, false));
        let output = frame(Vec::new());
        let manufacturer = label_position(&output, "🏭 Manufacturer");
        frame(click(manufacturer, true));
        frame(click(manufacturer, false));

        assert_eq!(app.config.filter_settings.search_text, "Capcom");
        assert_eq!(
            app.config.filter_settings.search_mode,
            SearchMode::Manufacturer
        );
        assert_eq!(app.game_index_manager.get_filtered_games(), &[1]);
    }

    #[test]
    fn romless_filter_ui_persists_and_switches_shells_in_an_isolated_process() {
        let directory = tempfile::tempdir().unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "app::mame_app::tests::romless_filter_ui_child",
                "--ignored",
                "--nocapture",
            ])
            .env("MAMEUIX_ROMLESS_UI_CHILD", "1")
            .env("XDG_CONFIG_HOME", directory.path().join("config"))
            .env("HOME", directory.path().join("home"))
            .env("APPDATA", directory.path().join("appdata"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "child UI regression failed:\n{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }

    #[test]
    #[ignore = "invoked by isolated UI regression with a temporary config home"]
    fn romless_filter_ui_child() {
        use crate::ui::panels::sidebar::ROMLESS_FILTER_LABEL;
        use crate::ui::redesign::{RedesignState, test_library as library};
        assert_eq!(
            std::env::var("MAMEUIX_ROMLESS_UI_CHILD").as_deref(),
            Ok("1")
        );

        fn text_position(output: &egui::FullOutput, text: &str) -> Option<egui::Pos2> {
            fn find(shape: &egui::Shape, text: &str) -> Option<egui::Pos2> {
                match shape {
                    egui::Shape::Text(item) if item.galley.job.text.contains(text) => {
                        Some(item.pos + item.galley.size() / 2.0)
                    }
                    egui::Shape::Vec(shapes) => {
                        shapes.iter().rev().find_map(|shape| find(shape, text))
                    }
                    _ => None,
                }
            }
            output
                .shapes
                .iter()
                .rev()
                .find_map(|shape| find(&shape.shape, text))
        }
        fn frame(
            context: &egui::Context,
            app: &mut MameApp,
            state: &mut RedesignState,
            redesign: bool,
            time: &mut f64,
            events: Vec<egui::Event>,
        ) -> egui::FullOutput {
            *time += 0.05;
            context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1200.0, 900.0),
                    )),
                    time: Some(*time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    if redesign {
                        library::show(ctx, app, state);
                    } else {
                        egui::SidePanel::left("test_legacy_sidebar")
                            .exact_width(300.0)
                            .show(ctx, |ui| app.render_sidebar_panel(ui));
                        egui::CentralPanel::default()
                            .show(ctx, |ui| app.render_game_list_panel(ui, ctx));
                    }
                },
            )
        }
        fn click(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        }
        fn assert_visible(
            app: &MameApp,
            state: &RedesignState,
            output: &egui::FullOutput,
            redesign: bool,
        ) {
            let hide = app.config.filter_settings.hide_romless_systems;
            let expected: &[usize] = if hide { &[0] } else { &[0, 1] };
            assert_eq!(app.game_index_manager.get_filtered_games(), expected);
            assert!(text_position(output, "Audit Needs ROMs").is_some());
            assert_eq!(text_position(output, "Audit ROM Free").is_some(), !hide);
            if redesign {
                assert_eq!(state.sidebar_stats.all, expected.len());
                assert_eq!(state.sidebar_stats.available, expected.len());
                assert_eq!(state.sidebar_stats.favorites, expected.len());
                assert_eq!(state.table_rows.len(), expected.len());
            }
        }

        let media_game = Game {
            name: "media".into(),
            description: "Audit Needs ROMs".into(),
            manufacturer: "Audit".into(),
            year: "1990".into(),
            driver: "fixture.cpp".into(),
            driver_status: "good".into(),
            status: RomStatus::Available,
            parent: None,
            category: String::new(),
            play_count: 0,
            is_clone: false,
            is_device: false,
            is_bios: false,
            controls: String::new(),
            requires_chd: false,
            requires_roms: true,
            chd_name: None,
            verification_status: None,
        };
        let mut romfree_game = media_game.clone();
        romfree_game.name = "romfree".into();
        romfree_game.description = "Audit ROM Free".into();
        romfree_game.requires_roms = false;
        let games = vec![media_game, romfree_game];
        let config = AppConfig {
            show_rom_icons: false,
            favorite_games: std::collections::HashSet::from(["media".into(), "romfree".into()]),
            ..AppConfig::default()
        };
        let mut app = app_for_loading_test(config);
        app.games = games.clone();
        app.loading_stage = LoadingStage::Complete;
        app.build_game_index();
        let context = egui::Context::default();
        RedesignTokens::install_fonts(&context);
        let mut state = RedesignState::default();
        let mut time = 0.0;

        for redesign in [false, true] {
            if redesign {
                RedesignTokens::apply(&context);
            }
            frame(
                &context,
                &mut app,
                &mut state,
                redesign,
                &mut time,
                Vec::new(),
            );
            let mut output = frame(
                &context,
                &mut app,
                &mut state,
                redesign,
                &mut time,
                Vec::new(),
            );
            assert_visible(&app, &state, &output, redesign);
            // Exercise both directions in each real UI. The final off state in
            // legacy must be read directly by the previously unused redesign.
            let expected_values: &[bool] = if redesign {
                &[true, false, true]
            } else {
                &[false, true, false]
            };
            for &expected_hide in expected_values {
                let checkbox =
                    text_position(&output, ROMLESS_FILTER_LABEL).expect("ROM requirement checkbox");
                frame(
                    &context,
                    &mut app,
                    &mut state,
                    redesign,
                    &mut time,
                    click(checkbox, true),
                );
                frame(
                    &context,
                    &mut app,
                    &mut state,
                    redesign,
                    &mut time,
                    click(checkbox, false),
                );
                output = frame(
                    &context,
                    &mut app,
                    &mut state,
                    redesign,
                    &mut time,
                    Vec::new(),
                );
                assert_eq!(
                    app.config.filter_settings.hide_romless_systems,
                    expected_hide
                );
                assert_visible(&app, &state, &output, redesign);
                assert_eq!(
                    crate::config::load_config()
                        .unwrap()
                        .filter_settings
                        .hide_romless_systems,
                    expected_hide
                );
            }
        }
        // Returning to a cached legacy table must honor the redesign toggle.
        let output = frame(&context, &mut app, &mut state, false, &mut time, Vec::new());
        assert_visible(&app, &state, &output, false);

        // A fresh app state loaded from disk retains the same option.
        let mut restarted = app_for_loading_test(crate::config::load_config().unwrap());
        restarted.games = games;
        restarted.loading_stage = LoadingStage::Complete;
        restarted.build_game_index();
        let output = frame(
            &context,
            &mut restarted,
            &mut state,
            false,
            &mut time,
            Vec::new(),
        );
        assert_visible(&restarted, &state, &output, false);
    }
}
