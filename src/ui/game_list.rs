// src/ui/game_list.rs
// Optimized untuk handle 48,000+ games dengan virtual scrolling yang benar
// Kunci: hanya render yang terlihat, gunakan index untuk O(1) lookups

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::models::{Game, FilterSettings, FilterCategory, GameIndex, RomStatus, SortColumn, RomSetType, GameStats, VisibleColumns, ColumnWidths};
use crate::hardware_filter::HardwareFilter;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// GameList dengan TRUE virtual scrolling
/// Tidak seperti versi lama yang masih process semua games,
/// versi ini HANYA process games yang terlihat di viewport
pub struct GameList {
    // Sorting state
    sort_column: SortColumn,
    sort_ascending: bool,

    // Virtual scrolling state
    pub visible_start: usize,
    pub visible_end: usize,
    row_height: f32,
    last_viewport: Option<egui::Rect>,

    // Cache untuk performance
    filtered_indices_cache: Vec<usize>,     // Games yang pass filter
    pub expanded_rows_cache: Vec<RowData>,      // Actual rows to display (includes clones)
    cache_valid: bool,
    last_filter_hash: u64,

    // Frame skipping untuk smooth performance
    last_render_time: Instant,
    skip_frame_count: u32,

    // Search state
    last_search_text: String,
    
    // Scroll control
    pub scroll_to_row: Option<usize>,


}


// Data untuk single row di table
#[derive(Debug, Clone)]
pub struct RowData {
    pub game_idx: usize,      // Index di games array
    pub is_clone: bool,       // Apakah ini clone row
    pub indent_level: u8,     // Level indentasi (0 = parent, 1 = clone)
    pub parent_idx: Option<usize>, // Index parent jika ini clone
}

impl GameList {
    pub fn new() -> Self {
        Self {
            sort_column: SortColumn::Name,
            sort_ascending: true,
            visible_start: 0,
            visible_end: 50,
            row_height: 24.0, // Pixels per row
            last_viewport: None,
            filtered_indices_cache: Vec::new(),
            expanded_rows_cache: Vec::new(),
            cache_valid: false,
            last_filter_hash: 0,
            last_render_time: Instant::now(),
            skip_frame_count: 0,
            last_search_text: String::new(),
            scroll_to_row: None,
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    /// Main show function - entry point untuk rendering
    /// Returns (double_clicked, favorite_toggled_game)
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        games: &[Game],
        filters: &FilterSettings,
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &mut HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
        game_index: Option<&GameIndex>,
        category: FilterCategory,
        column_widths: &mut crate::models::ColumnWidths,
        visible_columns: &crate::models::VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, crate::models::GameStats>,
        hardware_filter: Option<&HardwareFilter>,
        has_catver: bool,
        pre_filtered_indices: Option<&[usize]>,
    ) -> (bool, Option<String>) {
        // Remove aggressive frame skipping - it's causing glitches
        // Let egui handle frame pacing instead

        // Check apakah filter berubah
        let current_filter_hash = self.calculate_filter_hash(filters, favorites, expanded_parents, category);
        
        if current_filter_hash != self.last_filter_hash || filters.search_text != self.last_search_text {
            self.cache_valid = false;
            self.last_filter_hash = current_filter_hash;
            self.last_search_text = filters.search_text.clone();
        }

        // Update cache jika perlu
        if !self.cache_valid {
            self.update_cache(games, filters, favorites, expanded_parents, game_index, category, hardware_filter, pre_filtered_indices);
        }

        let total_rows = self.expanded_rows_cache.len();
        
        // Calculate available height first
        let available_height = ui.available_height();

        if total_rows == 0 {
            // Use all available height for empty state to match main list behavior
            
            // Allocate space for the empty state using all available height
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), available_height),
                egui::Sense::hover()
            );
            
            // Draw centered content in the allocated space
            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.vertical_centered(|ui| {
                    // Add vertical centering by adding space
                    let spacing = (available_height - 200.0) / 2.0; // Approximate content height
                    ui.add_space(spacing.max(20.0));
                    
                    ui.heading("No games found");
                    ui.add_space(20.0);
                    ui.label("Try adjusting your filters or search criteria");
                    
                    // Add more helpful information based on current filter
                    ui.add_space(40.0);
                    ui.separator();
                    ui.add_space(20.0);
                    
                    match category {
                        FilterCategory::Favorites => {
                            ui.label("No favorite games yet.");
                            ui.label("Click the ☆ star next to any game to add it to favorites.");
                        }
                        FilterCategory::Available => {
                            ui.label("No available games found.");
                            ui.label("Check your ROM directories in Options → Directories.");
                        }
                        FilterCategory::Missing => {
                            ui.label("No missing games found.");
                            ui.label("This means all scanned games have ROMs available.");
                        }
                        _ => {
                            ui.label("No games match the current filter criteria.");
                        }
                    }
                });
            });
            
            return (false, None);
        }

        // Show stats untuk large collections
        self.show_stats(ui, games.len());

        // Add column width management context menu
        ui.horizontal(|ui| {
            ui.label("Column Widths:");
            if ui.button("Reset to Defaults").clicked() {
                column_widths.reset_to_defaults();
            }
            ui.menu_button("Customize", |ui| {
                self.show_column_width_menu(ui, column_widths);
            });
        });

        // Allocate the full available height for the table container
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), available_height),
            egui::Sense::hover()
        );

        // Create a child UI with the allocated rect to ensure full height usage
        let (double_clicked, favorite_toggled) = ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(rect),
            |ui| {
                // Main content area dengan PROPER virtual scrolling
                self.render_virtual_table(
                    ui,
                    games,
                    selected,
                    expanded_parents,
                    favorites,
                    icons,
                    show_icons,
                    icon_size,
                    game_index,
                    available_height,
                    column_widths,
                    visible_columns,
                    default_icon,
                    game_stats,
                    has_catver,
                )
            }
        ).inner;
        
        (double_clicked, favorite_toggled)
    }

    /// Render table dengan TRUE virtual scrolling
    /// Returns (double_clicked, favorite_toggled_game)
    // In src/ui/game_list.rs

    fn render_virtual_table(
        &mut self,
        ui: &mut egui::Ui,
        games: &[Game],
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
        game_index: Option<&GameIndex>,
        _available_height: f32,
        column_widths: &mut ColumnWidths,
        visible_columns: &VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, GameStats>,
        has_catver: bool,
    ) -> (bool, Option<String>) {
        let mut double_clicked = false;
        let mut favorite_toggled: Option<String> = None;

        let total_rows = self.expanded_rows_cache.len();
        let icon_width = if show_icons { 32.0 } else { 0.0 };
        let status_width = 25.0; // Status column is always shown

        // Capture header styling colors before creating the table
        let header_bg_color = ui.visuals().extreme_bg_color;
        let header_text_color = ui.visuals().strong_text_color();

        // Build table with ALL columns resizable and persistent widths
        let mut table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(column_widths.expand)
                .resizable(true)
                .clip(true))
            .column(Column::initial(column_widths.favorite)
                .resizable(true)
                .clip(true));

        if show_icons {
            table = table.column(Column::initial(column_widths.icon)
                .resizable(true)
                .clip(true));
        }
        
        // Status column is always shown
        table = table.column(Column::initial(column_widths.status)
            .resizable(true)
            .clip(true));

        // Game name column
        table = table.column(Column::initial(column_widths.game)
            .resizable(true)
            .clip(true));

        if visible_columns.play_count {
            table = table.column(Column::initial(column_widths.play_count)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.manufacturer {
            table = table.column(Column::initial(column_widths.manufacturer)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.year {
            table = table.column(Column::initial(column_widths.year)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.driver {
            table = table.column(Column::initial(column_widths.driver)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.driver_status {
            table = table.column(Column::initial(column_widths.driver_status)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.category {
            table = table.column(Column::initial(column_widths.category)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.rom {
            table = table.column(Column::initial(column_widths.rom)
                .resizable(true)
                .clip(true));
        }
        if visible_columns.chd {
            table = table.column(Column::initial(column_widths.chd)
                .resizable(true)
                .clip(true));
        }

        // Capture the table response to get column width changes
        let response = table
            .header(24.0, |mut header| {
                // Helper closure to render bold header text with background
                let render_header = |ui: &mut egui::Ui, text: &str| {
                    // Add background fill
                    let rect = ui.available_rect_before_wrap();
                    ui.painter().rect_filled(rect, 0.0, header_bg_color);
                    
                    // Render bold text
                    ui.label(egui::RichText::new(text).strong().color(header_text_color));
                };
                
                // Header columns with bold text and solid background
                header.col(|ui| { render_header(ui, ""); });
                header.col(|ui| { render_header(ui, "★"); });

                if show_icons {
                    header.col(|ui| { render_header(ui, "Icon"); });
                }
                // Status column is always shown
                header.col(|ui| { render_header(ui, "St"); });

                header.col(|ui| {
                    render_header(ui, "Game");
                });

                if visible_columns.play_count {
                    header.col(|ui| { render_header(ui, "Plays"); });
                }
                if visible_columns.manufacturer {
                    header.col(|ui| { render_header(ui, "Manufacturer"); });
                }
                if visible_columns.year {
                    header.col(|ui| { render_header(ui, "Year"); });
                }
                if visible_columns.driver {
                    header.col(|ui| { render_header(ui, "Driver"); });
                }
                if visible_columns.driver_status {
                    header.col(|ui| { render_header(ui, "Driver Status"); });
                }
                if visible_columns.category {
                    header.col(|ui| {
                        if has_catver {
                            render_header(ui, "Category");
                        } else {
                            render_header(ui, "Category (No catver.ini)");
                        }
                    });
                }
                if visible_columns.rom {
                    header.col(|ui| { render_header(ui, "ROM"); });
                }
                if visible_columns.chd {
                    header.col(|ui| { render_header(ui, "CHD"); });
                }
            })
            .body(|body| {
                // Body ini akan bisa digulung.
                if let Some(_target_row) = self.scroll_to_row.take() {
                    // Note: scroll_to_row is not a method on TableBody, we'll handle scrolling differently
                    // body.scroll_to_row(target_row, Some(egui::Align::Center));
                }

                body.rows(self.row_height, total_rows, |mut row| {
                    let row_idx = row.index();
                    if let Some(row_data) = self.expanded_rows_cache.get(row_idx).cloned() {
                        if let Some(game) = games.get(row_data.game_idx) {
                            let (row_double_clicked, row_favorite_toggled) = self.render_single_row(
                                &mut row,
                                game,
                                &row_data,
                                selected,
                                expanded_parents,
                                favorites,
                                icons,
                                show_icons,
                                icon_size,
                                game_index,
                                visible_columns,
                                default_icon,
                                game_stats,
                            );

                            if row_double_clicked {
                                double_clicked = true;
                            }
                            if let Some(game_name) = row_favorite_toggled {
                                favorite_toggled = Some(game_name);
                            }
                        }
                    }
                });
            });

        // TODO: Implement column width persistence when egui_extras supports it
        // For now, columns are resizable but widths are not automatically saved
        // Column widths are still saved in config and restored on startup

        (double_clicked, favorite_toggled)
    }

    /// Render single row - dipanggil HANYA untuk visible rows
    /// Returns (double_clicked, favorite_toggled_game)
    fn render_single_row(
        &mut self,
        row: &mut egui_extras::TableRow,
        game: &Game,
        row_data: &RowData,
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
        game_index: Option<&GameIndex>,
        visible_columns: &VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, GameStats>,
    ) -> (bool, Option<String>) {
        let is_selected = selected.map_or(false, |s| s == row_data.game_idx);
        let is_favorite = favorites.contains(&game.name);
        let mut double_clicked = false;
        let mut favorite_toggled = None;

        // Expand/collapse button
        row.col(|ui| {
            if !row_data.is_clone {
                if let Some(index) = game_index {
                    if index.has_clones(&game.name) {
                        let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false);
                        if ui.button(if is_expanded { "▼" } else { "▶" }).clicked() {
                            expanded_parents.insert(game.name.clone(), !is_expanded);
                            self.invalidate_cache();
                        }
                    }
                }
            }
        });

        // Favorite toggle
        row.col(|ui| {
            let star = if is_favorite { "★" } else { "☆" };
            if ui.selectable_label(false, star).clicked() {
                // Toggle favorite - return the game name to parent
                favorite_toggled = Some(game.name.clone());
            }
        });

        // Icon
        if show_icons {
            row.col(|ui| {
                // Display icon if available, otherwise show default icon
                if let Some(texture) = icons.get(&game.name) {
                    ui.add(egui::Image::new(texture).fit_to_exact_size(egui::Vec2::splat(icon_size as f32)));
                } else if let Some(default) = default_icon {
                    // Show default icon as placeholder
                    ui.add(egui::Image::new(default).fit_to_exact_size(egui::Vec2::splat(icon_size as f32)));
                } else {
                    // Show empty space if no default icon
                    ui.add_space(icon_size as f32);
                }
            });
        }

        // Status icon - always shown to match header
        row.col(|ui| {
            ui.label(game.status.to_icon());
        });

        // Game name dengan indentasi untuk clones
        row.col(|ui| {
            let name = if row_data.is_clone {
                format!("  └─ {}", game.description)
            } else {
                game.description.clone()
            };

            let response = ui.selectable_label(is_selected, name);
            if response.clicked() {
                *selected = Some(row_data.game_idx);
            }

            // Store double-click state for parent to handle
            if response.double_clicked() {
                // Return the double-clicked game index through the selected parameter
                // The main window will detect this and launch the game
                *selected = Some(row_data.game_idx);
                double_clicked = true;
            }
        });

        // Play Count (optional)
        if visible_columns.play_count {
            row.col(|ui| {
                // Get play count from game stats
                let play_count = game_stats.get(&game.name)
                    .map(|stats| stats.play_count)
                    .unwrap_or(0);
                ui.label(play_count.to_string());
            });
        }

        // Manufacturer (optional)
        if visible_columns.manufacturer {
            row.col(|ui| {
                ui.label(&game.manufacturer);
            });
        }

        // Year (optional)
        if visible_columns.year {
            row.col(|ui| {
                ui.label(&game.year);
            });
        }

        // Driver (optional)
        if visible_columns.driver {
            row.col(|ui| {
                ui.label(&game.driver);
            });
        }

        // Driver Status (optional)
        if visible_columns.driver_status {
            row.col(|ui| {
                let (icon, text) = game.get_driver_status_display();
                let color = game.get_driver_status_color();
                let display = format!("{} {}", icon, text);
                ui.colored_label(color, display);
            });
        }

        // Category (optional)
        if visible_columns.category {
            row.col(|ui| {
                ui.label(&game.category);
            });
        }

        // ROM (optional)
        if visible_columns.rom {
            row.col(|ui| {
                ui.label(&game.name);
            });
        }

        // Status text column removed - status is shown as icon in the status column

        // CHD information (only if column is enabled)
        if visible_columns.chd {
            row.col(|ui| {
                if game.requires_chd {
                    if let Some(chd_name) = &game.chd_name {
                        ui.label(chd_name);
                    } else {
                        ui.label("Required");
                    }
                } else {
                    ui.label("None");
                }
            });
        }
        
        (double_clicked, favorite_toggled)
    }

    /// Update cache dengan filtered dan expanded games
    fn update_cache(
        &mut self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        expanded_parents: &HashMap<String, bool>,
        game_index: Option<&GameIndex>,
        category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>,
        pre_filtered_indices: Option<&[usize]>,
    ) {
        let start = Instant::now();
        


        // Step 1: Get filtered game indices
        let mut filtered_indices = if let Some(pre_filtered) = pre_filtered_indices {
            pre_filtered.to_vec()
        } else if let Some(index) = game_index {
            self.filter_with_index(games, filters, favorites, index, category, hardware_filter)
        } else {
            self.filter_manual(games, filters, favorites, category, hardware_filter)
        };

        // Step 1.5: Apply ROM set type specific filtering to prevent duplicates
        filtered_indices = self.apply_rom_set_filtering(games, filtered_indices, filters, game_index);
        
        self.filtered_indices_cache = filtered_indices;

        // Step 2: Apply sorting to the filtered indices
        self.apply_sorting(games);

        // Step 3: Build expanded rows dengan clones
        self.expanded_rows_cache.clear();
        self.expanded_rows_cache.reserve(self.filtered_indices_cache.len() * 2); // Reserve space

        for &idx in &self.filtered_indices_cache {
            if let Some(game) = games.get(idx) {
                // Add parent row
                self.expanded_rows_cache.push(RowData {
                    game_idx: idx,
                    is_clone: game.is_clone,
                    indent_level: 0,
                    parent_idx: None,
                });

                // Add clone rows jika parent expanded atau auto expand enabled
                let should_expand = expanded_parents.get(&game.name).copied().unwrap_or(false) || 
                                   filters.auto_expand_clones;
                if !game.is_clone && should_expand {
                    if let Some(index) = game_index {
                        // O(1) clone lookup thanks to GameIndex!
                        for &clone_idx in index.get_clones(&game.name) {
                            self.expanded_rows_cache.push(RowData {
                                game_idx: clone_idx,
                                is_clone: true,
                                indent_level: 1,
                                parent_idx: Some(idx),
                            });
                        }
                    }
                }
            }
        }

        self.cache_valid = true;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            println!("Warning: Cache update took {}ms for {} games",
                     elapsed.as_millis(), self.expanded_rows_cache.len());
        }
    }

    /// Apply sorting to the filtered indices
    fn apply_sorting(&mut self, games: &[Game]) {
        let sort_column = self.sort_column;
        let sort_ascending = self.sort_ascending;
        
        self.filtered_indices_cache.sort_by(|&a, &b| {
            let game_a = &games[a];
            let game_b = &games[b];
            
            let ordering = match sort_column {
                SortColumn::Name => game_a.description.cmp(&game_b.description),
                SortColumn::Manufacturer => game_a.manufacturer.cmp(&game_b.manufacturer),
                SortColumn::Year => game_a.year.cmp(&game_b.year),
                SortColumn::Status => game_a.status.description().cmp(&game_b.status.description()),
                SortColumn::Category => game_a.category.cmp(&game_b.category),
            };
            
            if sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }

    /// Fast filtering menggunakan GameIndex
    fn filter_with_index(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        index: &GameIndex,
        category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>,
    ) -> Vec<usize> {
        // Check search cache first
        if !filters.search_text.is_empty() {
            if let Some(cached) = index.get_cached_search(&filters.search_text) {
                return cached.to_vec();
            }
        }

        // Start dengan category filter
        let mut result: Vec<usize> = match category {
            FilterCategory::All => {
                // All games - gunakan sorted indices jika ada
                if let Some(sorted) = index.get_sorted("name", true) {
                    sorted.to_vec()
                } else {
                    (0..games.len()).collect()
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

        // Apply additional filters
        if filters.show_favorites_only {
            result.retain(|&idx| favorites.contains(&games[idx].name));
        }

        if filters.hide_non_games {
            result.retain(|&idx| {
                let game = &games[idx];
                !game.is_device && !game.is_bios
            });
        }

        // Clone filtering removed

        // INI filter removed - non-game filtering is now handled by hide_non_games flag

        // Apply text search last
        if !filters.search_text.is_empty() {
            let search_lower = filters.search_text.to_lowercase();

            // Use parallel search untuk large datasets
            if result.len() > 1000 {
                use rayon::prelude::*;
                result = result.into_par_iter()
                .filter(|&idx| {
                    if let Some(game) = games.get(idx) {
                        match filters.search_mode {
                            crate::models::filters::SearchMode::GameTitle => {
                                game.description.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Manufacturer => {
                                game.manufacturer.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::RomFileName => {
                                game.name.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Year => {
                                game.year.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Status => {
                                game.status.description().to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Cpu => {
                                // Use hardware filter if available
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_cpu(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                            crate::models::filters::SearchMode::Device => {
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_device(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                            crate::models::filters::SearchMode::Sound => {
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_sound(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                        }
                    } else {
                        false
                    }
                })
                .collect();
            } else {
                result.retain(|&idx| {
                    if let Some(game) = games.get(idx) {
                        match filters.search_mode {
                            crate::models::filters::SearchMode::GameTitle => {
                                game.description.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Manufacturer => {
                                game.manufacturer.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::RomFileName => {
                                game.name.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Year => {
                                game.year.to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Status => {
                                game.status.description().to_lowercase().contains(&search_lower)
                            }
                            crate::models::filters::SearchMode::Cpu => {
                                // Use hardware filter if available
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_cpu(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                            crate::models::filters::SearchMode::Device => {
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_device(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                            crate::models::filters::SearchMode::Sound => {
                                if let Some(hw_filter) = hardware_filter {
                                    hw_filter.game_uses_sound(&game.name, &search_lower)
                                } else {
                                    false
                                }
                            }
                        }
                    } else {
                        false
                    }
                });
            }
        }

        result
    }

    /// Apply ROM set type specific filtering to prevent duplicates
    fn apply_rom_set_filtering(
        &self,
        games: &[Game],
        mut filtered_indices: Vec<usize>,
        filters: &FilterSettings,
        game_index: Option<&GameIndex>,
    ) -> Vec<usize> {
        // Special handling for "All Games" filter with auto expand clones
        // When auto expand is enabled, we want to show parent games and their clones
        // but avoid showing standalone clones (clones without parents in the list)
        
        if filters.auto_expand_clones {
            // For auto expand mode, we need to:
            // 1. Keep all parent games
            // 2. Keep clones that have their parent in the filtered list
            // 3. Remove standalone clones (clones whose parent is not in the list)
            
            let parent_names: HashSet<String> = filtered_indices.iter()
                .filter_map(|&idx| {
                    if let Some(game) = games.get(idx) {
                        if !game.is_clone {
                            Some(game.name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            
            // Keep parent games and clones that have their parent in the list
            filtered_indices.retain(|&idx| {
                if let Some(game) = games.get(idx) {
                    if !game.is_clone {
                        // Always keep parent games
                        true
                    } else {
                        // For clones, check if their parent is in the list
                        if let Some(index) = game_index {
                            // Find the parent of this clone
                            if let Some(parent_name) = self.get_parent_name(games, game, index) {
                                parent_names.contains(&parent_name)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                } else {
                    false
                }
            });
        } else {
            // Standard ROM set type filtering
            match filters.rom_set_type {
                RomSetType::NonMerged => {
                    // For non-merged sets, show only parent games to avoid duplicates
                    // unless user explicitly wants to see clones
                    if !filters.show_clones_in_split {
                        filtered_indices.retain(|&idx| {
                            if let Some(game) = games.get(idx) {
                                !game.is_clone
                            } else {
                                false
                            }
                        });
                    }
                },
                RomSetType::Split => {
                    // For split sets, show parent games and optionally clones
                    if !filters.show_clones_in_split {
                        filtered_indices.retain(|&idx| {
                            if let Some(game) = games.get(idx) {
                                !game.is_clone
                            } else {
                                false
                            }
                        });
                    }
                },
                RomSetType::Merged => {
                    // For merged sets, show only parent games (clones are merged into parent)
                    filtered_indices.retain(|&idx| {
                        if let Some(game) = games.get(idx) {
                            !game.is_clone
                        } else {
                            false
                        }
                    });
                },
                RomSetType::Unknown => {
                    // If type is unknown, try to detect based on clone ratio
                    let total_games = filtered_indices.len();
                    let clone_count = filtered_indices.iter()
                        .filter(|&&idx| {
                            if let Some(game) = games.get(idx) {
                                game.is_clone
                            } else {
                                false
                            }
                        })
                        .count();
                    
                    let clone_ratio = if total_games > 0 {
                        clone_count as f64 / total_games as f64
                    } else {
                        0.0
                    };

                    // If more than 30% are clones, likely non-merged or split set
                    if clone_ratio > 0.3 && !filters.show_clones_in_split {
                        filtered_indices.retain(|&idx| {
                            if let Some(game) = games.get(idx) {
                                !game.is_clone
                            } else {
                                false
                            }
                        });
                    }
                }
            }
        }

        // Remove duplicates based on game name (but preserve parent/clone relationships)
        let mut seen_names = std::collections::HashSet::new();
        filtered_indices.retain(|&idx| {
            if let Some(game) = games.get(idx) {
                seen_names.insert(game.name.clone())
            } else {
                false
            }
        });

        filtered_indices
    }

    /// Helper function to get parent name for a clone game
    fn get_parent_name(&self, _games: &[Game], clone_game: &Game, _index: &GameIndex) -> Option<String> {
        // Use the parent field directly from the Game struct
        clone_game.parent.clone()
    }

    /// Manual filtering fallback (tanpa GameIndex)
    fn filter_manual(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>,
    ) -> Vec<usize> {
        let search_lower = filters.search_text.to_lowercase();

        games.iter()
        .enumerate()
        .filter(|(_, game)| {
            // Category filter
            match category {
                FilterCategory::All => {},
                FilterCategory::Available => {
                    if !matches!(game.status, RomStatus::Available) {
                        return false;
                    }
                }
                FilterCategory::Missing => {
                    if !matches!(game.status, RomStatus::Missing) {
                        return false;
                    }
                }
                FilterCategory::Favorites => {
                    if !favorites.contains(&game.name) {
                        return false;
                    }
                }
                FilterCategory::Parents => {
                    if game.is_clone {
                        return false;
                    }
                }
                FilterCategory::Clones => {
                    if !game.is_clone {
                        return false;
                    }
                }
                FilterCategory::ChdGames => {
                    if !game.requires_chd {
                        return false;
                    }
                }
                FilterCategory::NonMerged => {
                    if game.is_clone {
                        return false;
                    }
                }
                _ => {}
            }

            // Additional filters
            if filters.show_favorites_only && !favorites.contains(&game.name) {
                return false;
            }

            if filters.hide_non_games && (game.is_device || game.is_bios) {
                return false;
            }

            // Clone filtering removed

            // Search filter
            if !search_lower.is_empty() {
                let matches = match filters.search_mode {
                    crate::models::filters::SearchMode::GameTitle => {
                        game.description.to_lowercase().contains(&search_lower)
                    }
                    crate::models::filters::SearchMode::Manufacturer => {
                        game.manufacturer.to_lowercase().contains(&search_lower)
                    }
                    crate::models::filters::SearchMode::RomFileName => {
                        game.name.to_lowercase().contains(&search_lower)
                    }
                    crate::models::filters::SearchMode::Year => {
                        game.year.to_lowercase().contains(&search_lower)
                    }
                    crate::models::filters::SearchMode::Status => {
                        game.status.description().to_lowercase().contains(&search_lower)
                    }
                    crate::models::filters::SearchMode::Cpu => {
                        // Use hardware filter if available
                        if let Some(hw_filter) = hardware_filter {
                            hw_filter.game_uses_cpu(&game.name, &search_lower)
                        } else {
                            false
                        }
                    }
                    crate::models::filters::SearchMode::Device => {
                        if let Some(hw_filter) = hardware_filter {
                            hw_filter.game_uses_device(&game.name, &search_lower)
                        } else {
                            false
                        }
                    }
                    crate::models::filters::SearchMode::Sound => {
                        if let Some(hw_filter) = hardware_filter {
                            hw_filter.game_uses_sound(&game.name, &search_lower)
                        } else {
                            false
                        }
                    }
                };
                if !matches {
                    return false;
                }
            }

            // INI filter removed - non-game filtering is now handled by hide_non_games flag

            true
        })
        .map(|(idx, _)| idx)
        .collect()
    }

    /// Show statistics bar
    fn show_stats(&self, ui: &mut egui::Ui, total_games: usize) {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Showing {} of {} games",
                self.filtered_indices_cache.len(),
                             total_games
            ));

            if self.expanded_rows_cache.len() > self.filtered_indices_cache.len() {
                ui.label(format!(
                    "({} rows with expanded clones)",
                                 self.expanded_rows_cache.len()
                ));
            }

            // Performance indicator
            if total_games > 10000 {
                ui.separator();
                ui.colored_label(
                    egui::Color32::from_rgb(100, 150, 255),
                                 "⚡ Virtual scrolling active"
                );
            }
        });
        ui.separator();
    }

    /// Toggle sort dan invalidate cache
    fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
        self.invalidate_cache();
    }

    /// Calculate hash untuk cache invalidation
    fn calculate_filter_hash(
        &self,
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        expanded: &HashMap<String, bool>,
        category: FilterCategory,
    ) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash semua filter state
        category.hash(&mut hasher);
        filters.show_favorites_only.hash(&mut hasher);
        filters.hide_non_games.hash(&mut hasher);
        // Clone filtering removed from hash

        // Hash catver category filter - CRITICAL for cache invalidation
        if let Some(ref catver_category) = filters.catver_category {
            catver_category.hash(&mut hasher);
        } else {
            // Hash None state to distinguish from Some("")
            "NONE".hash(&mut hasher);
        }

        // Hash ukuran collections (cheaper than hashing contents)
        favorites.len().hash(&mut hasher);
        expanded.len().hash(&mut hasher);

        // Hash sort state
        self.sort_column.hash(&mut hasher);
        self.sort_ascending.hash(&mut hasher);

        // Hash INI filter state - CRITICAL for cache invalidation
        // INI filter removed from hash

        hasher.finish()
    }

    /// Show column width management context menu
    fn show_column_width_menu(&self, ui: &mut egui::Ui, column_widths: &mut crate::models::ColumnWidths) {
        ui.label("Adjust Column Widths:");
        ui.separator();
        
        let columns = [
            ("Game", &mut column_widths.game, 100.0, 500.0),
            ("Manufacturer", &mut column_widths.manufacturer, 80.0, 400.0),
            ("Year", &mut column_widths.year, 40.0, 100.0),
            ("Driver", &mut column_widths.driver, 60.0, 200.0),
            ("Driver Status", &mut column_widths.driver_status, 80.0, 200.0),
            ("Category", &mut column_widths.category, 80.0, 300.0),
            ("ROM", &mut column_widths.rom, 80.0, 300.0),
            ("Play Count", &mut column_widths.play_count, 40.0, 100.0),
            ("Status", &mut column_widths.status, 60.0, 200.0),
        ];
        
        for (name, width, min, max) in columns {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", name));
                ui.add(egui::Slider::new(width, min..=max).text("px"));
            });
        }
        
        ui.separator();
        if ui.button("Reset All to Default").clicked() {
            column_widths.reset_to_defaults();
        }
    }

    /// Get filtered games based on ROM set type and user preferences
    fn get_filtered_games(&self, games: &[Game], filter_settings: &FilterSettings) -> Vec<usize> {
        let mut filtered_indices = Vec::new();
        
        for (idx, game) in games.iter().enumerate() {
            if self.should_show_game(game, filter_settings) {
                filtered_indices.push(idx);
            }
        }
        
        filtered_indices
    }

    /// Determine if a game should be shown based on ROM set type and settings
    fn should_show_game(&self, game: &Game, filter_settings: &FilterSettings) -> bool {
        match filter_settings.rom_set_type {
            RomSetType::NonMerged => {
                // Non-Merged: Show all games by default, or only parents if requested
                if filter_settings.show_clones_in_split {
                    true // Show all games
                } else {
                    !game.is_clone // Show only parents
                }
            },
            RomSetType::Split => {
                // Split: Show parents + clones based on user preference
                if filter_settings.show_clones_in_split {
                    true // Show all games
                } else {
                    !game.is_clone // Show only parents
                }
            },
            RomSetType::Merged => {
                // Merged: Show only parents (clones are included in parent archives)
                if filter_settings.show_clones_in_merged {
                    true // User wants to see clones (unusual for merged sets)
                } else {
                    !game.is_clone // Show only parents
                }
            },
            RomSetType::Unknown => {
                // Unknown: Default to showing all games
                true
            }
        }
    }
}
