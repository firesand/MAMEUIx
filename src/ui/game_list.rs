// src/ui/game_list.rs
// Optimized untuk handle 48,000+ games dengan virtual scrolling yang benar
// Kunci: hanya render yang terlihat, gunakan index untuk O(1) lookups

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::models::{Game, FilterSettings, FilterCategory, GameIndex, RomStatus};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortColumn {
    Name,
    Manufacturer,
    Year,
    Status,
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
            self.update_cache(games, filters, favorites, expanded_parents, game_index, category);
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
                )
            }
        ).inner;
        
        (double_clicked, favorite_toggled)
    }

    /// Render table dengan TRUE virtual scrolling
    /// Returns (double_clicked, favorite_toggled_game)
    fn render_virtual_table(
        &mut self,
        ui: &mut egui::Ui,
        games: &[Game],
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &mut HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
        game_index: Option<&GameIndex>,
        available_height: f32,
        column_widths: &mut crate::models::ColumnWidths,
        visible_columns: &crate::models::VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, crate::models::GameStats>,
    ) -> (bool, Option<String>) {
        let total_rows = self.expanded_rows_cache.len();

        // Calculate column widths
        let icon_width = if show_icons { icon_size as f32 + 4.0 } else { 0.0 };
        
        let mut double_clicked = false;
        let mut favorite_toggled: Option<String> = None;

        // Use ScrollArea untuk virtual scrolling with horizontal scrolling enabled
        // Force the scroll area to always use full available height
        let mut scroll_area = egui::ScrollArea::both()
            .id_salt("game_list_main")
            .min_scrolled_height(available_height)  // Never shorter than available height
            .max_height(available_height)  // Also set max height to prevent expansion
            .auto_shrink([false, false]);  // Don't shrink the scroll area
        
        // Handle scroll to specific row if requested
        if let Some(target_row) = self.scroll_to_row.take() {
            let scroll_offset = target_row as f32 * self.row_height;
            scroll_area = scroll_area.vertical_scroll_offset(scroll_offset);
        }
        
        scroll_area.show_rows(
            ui,
            self.row_height,
            total_rows,
            |ui, row_range| {
                // Update visible range
                self.visible_start = row_range.start;
                self.visible_end = row_range.end;

                // Build table ONLY untuk visible rows
                let mut table_builder = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(25.0))     // Expand/collapse
                .column(Column::exact(25.0))     // Favorite
                .column(Column::exact(icon_width)) // Icon
                .column(Column::exact(25.0))     // Status
                .column(Column::initial(column_widths.game).resizable(true).clip(true));     // Game name
                
                // Add optional columns based on preferences
                if visible_columns.play_count {
                    table_builder = table_builder.column(Column::initial(column_widths.play_count).resizable(true));   // Play Count
                }
                if visible_columns.manufacturer {
                    table_builder = table_builder.column(Column::initial(column_widths.manufacturer).resizable(true).clip(true));  // Manufacturer
                }
                if visible_columns.year {
                    table_builder = table_builder.column(Column::initial(column_widths.year).resizable(true));   // Year
                }
                if visible_columns.driver {
                    table_builder = table_builder.column(Column::initial(column_widths.driver).resizable(true));   // Driver
                }
                if visible_columns.category {
                    table_builder = table_builder.column(Column::initial(column_widths.category).resizable(true));  // Category
                }
                if visible_columns.rom {
                    table_builder = table_builder.column(Column::initial(column_widths.rom).resizable(true));  // ROM
                }
                
                table_builder = table_builder.column(Column::initial(column_widths.status).resizable(true))   // Status text
                .min_scrolled_height(available_height - 40.0);  // Use most of available height minus header
                
                // Add CHD column only if enabled
                if visible_columns.chd {
                    table_builder = table_builder.column(Column::initial(60.0).resizable(true));   // CHD status
                }
                
                table_builder
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.label(""); });
                    header.col(|ui| { ui.label("★"); });
                    if show_icons {
                        header.col(|ui| { ui.label("Icon"); });
                    }
                    header.col(|ui| { ui.label("St"); });
                    header.col(|ui| {
                        let sort_indicator = if self.sort_column == SortColumn::Name {
                            if self.sort_ascending { " ▲" } else { " ▼" }
                        } else { "" };
                        if ui.button(format!("Game{}", sort_indicator)).clicked() {
                            self.toggle_sort(SortColumn::Name);
                        }
                    });
                    if visible_columns.play_count {
                        header.col(|ui| { ui.label("Plays"); });
                    }
                    if visible_columns.manufacturer {
                        header.col(|ui| {
                            let sort_indicator = if self.sort_column == SortColumn::Manufacturer {
                                if self.sort_ascending { " ▲" } else { " ▼" }
                            } else { "" };
                            if ui.button(format!("Manufacturer{}", sort_indicator)).clicked() {
                                self.toggle_sort(SortColumn::Manufacturer);
                            }
                        });
                    }
                    if visible_columns.year {
                        header.col(|ui| {
                            let sort_indicator = if self.sort_column == SortColumn::Year {
                                if self.sort_ascending { " ▲" } else { " ▼" }
                            } else { "" };
                            if ui.button(format!("Year{}", sort_indicator)).clicked() {
                                self.toggle_sort(SortColumn::Year);
                            }
                        });
                    }
                    if visible_columns.driver {
                        header.col(|ui| { ui.label("Driver"); });
                    }
                    if visible_columns.category {
                        header.col(|ui| { ui.label("Category"); });
                    }
                    if visible_columns.rom {
                        header.col(|ui| { ui.label("ROM"); });
                    }
                    header.col(|ui| { ui.label("Status"); });
                    // Add CHD column header only if enabled
                    if visible_columns.chd {
                        header.col(|ui| { ui.label("CHD"); });
                    }
                })
                .body(|mut body| {
                    // CRITICAL: Hanya render visible rows!
                    for row_idx in row_range {
                        if let Some(row_data) = self.expanded_rows_cache.get(row_idx).cloned() {
                            if let Some(game) = games.get(row_data.game_idx) {
                                body.row(self.row_height, |mut row| {
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
                                });
                            }
                        }
                    }
                });
                
                // After the table is rendered, try to capture column width changes
                // Note: This is a workaround since egui_extras doesn't provide direct access to column widths
                // We'll use a different approach with persistent IDs
            }
        );
        
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
        visible_columns: &crate::models::VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, crate::models::GameStats>,
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

        // Status icon
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

        // Status text
        row.col(|ui| {
            match game.status {
                RomStatus::Available => {
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), "Available");
                }
                RomStatus::Missing => {
                    ui.colored_label(egui::Color32::from_rgb(200, 100, 100), "Missing");
                }
                RomStatus::ChdRequired => {
                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "CHD Required");
                }
                RomStatus::ChdMissing => {
                    ui.colored_label(egui::Color32::from_rgb(255, 0, 0), "CHD Missing");
                }
                _ => {
                    ui.label("Unknown");
                }
            }
        });

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
    ) {
        let start = Instant::now();

        // Step 1: Get filtered game indices
        self.filtered_indices_cache = if let Some(index) = game_index {
            self.filter_with_index(games, filters, favorites, index, category)
        } else {
            self.filter_manual(games, filters, favorites, category)
        };

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

                // Add clone rows jika parent expanded
                if !game.is_clone && expanded_parents.get(&game.name).copied().unwrap_or(false) {
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
        if elapsed.as_millis() > 50 {
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

        if !filters.show_clones {
            result.retain(|&idx| !games[idx].is_clone);
        }

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
                        }
                    } else {
                        false
                    }
                });
            }
        }

        result
    }

    /// Manual filtering fallback (tanpa GameIndex)
    fn filter_manual(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        category: FilterCategory,
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
                _ => {}
            }

            // Additional filters
            if filters.show_favorites_only && !favorites.contains(&game.name) {
                return false;
            }

            if filters.hide_non_games && (game.is_device || game.is_bios) {
                return false;
            }

            if !filters.show_clones && game.is_clone {
                return false;
            }

            // Search filter
            if !search_lower.is_empty() {
                let matches = match filters.search_mode {
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
                };
                if !matches {
                    return false;
                }
            }

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
        filters.show_clones.hash(&mut hasher);

        // Hash ukuran collections (cheaper than hashing contents)
        favorites.len().hash(&mut hasher);
        expanded.len().hash(&mut hasher);

        // Hash sort state
        self.sort_column.hash(&mut hasher);
        self.sort_ascending.hash(&mut hasher);

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
}
