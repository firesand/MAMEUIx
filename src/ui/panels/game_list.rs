// src/ui/game_list.rs
// Optimized untuk handle 48,000+ games dengan virtual scrolling yang benar
// Kunci: hanya render yang terlihat, gunakan index untuk O(1) lookups

use crate::models::{
    ColumnWidths, FilterCategory, FilterSettings, Game, GameIndex, GameStats, RomSetType,
    RomStatus, SortColumn, VisibleColumns,
};
use crate::utils::hardware_filter::HardwareFilter;
use eframe::egui;
use egui_extras::Column;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const TABLE_HEADER_HEIGHT: f32 = 36.0;
const STATS_BAR_HEIGHT: f32 = 44.0;

/// Solid, always-visible scrollbar — Table mode needs reserved width; floating bars
/// are drawn over the columns and end up hidden/clipped with wide tables.
fn table_scroll_style() -> egui::style::ScrollStyle {
    let mut style = egui::style::ScrollStyle::solid();
    style.foreground_color = true;
    style.bar_width = 12.0;
    style.handle_min_length = 32.0;
    style.bar_inner_margin = 4.0;
    style
}

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
    filtered_indices_cache: Vec<usize>, // Games yang pass filter
    pub expanded_rows_cache: Vec<RowData>, // Actual rows to display (includes clones)
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
    pub game_idx: usize,           // Index di games array
    pub is_clone: bool,            // Apakah ini clone row
    pub indent_level: u32,         // Indentation level for hierarchy
    pub parent_idx: Option<usize>, // Parent row index if this is a clone
}

impl GameList {
    pub fn new() -> Self {
        Self {
            sort_column: SortColumn::Name,
            sort_ascending: true,
            visible_start: 0,
            visible_end: 50,
            row_height: 36.0, // Increased pixels per row for better spacing
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

    pub fn row_count(&self) -> usize {
        self.expanded_rows_cache.len()
    }

    pub fn game_idx_at_row(&self, row: usize) -> Option<usize> {
        self.expanded_rows_cache
            .get(row)
            .map(|row_data| row_data.game_idx)
    }

    pub fn row_for_game_idx(&self, game_idx: usize) -> Option<usize> {
        self.expanded_rows_cache
            .iter()
            .position(|row| row.game_idx == game_idx)
    }

    /// Main show function - entry point untuk rendering
    /// Returns (play_game_index, favorite_toggled_game, properties_game_index)
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
        theme_colors: Option<&crate::models::GameListColors>, // Add theme colors parameter
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        // Remove aggressive frame skipping - it's causing glitches
        // Let egui handle frame pacing instead

        // Check apakah filter berubah
        let current_filter_hash =
            self.calculate_filter_hash(filters, favorites, expanded_parents, category);

        if current_filter_hash != self.last_filter_hash
            || filters.search_text != self.last_search_text
        {
            self.cache_valid = false;
            self.last_filter_hash = current_filter_hash;
            self.last_search_text = filters.search_text.clone();
        }

        // Update cache jika perlu
        if !self.cache_valid {
            self.update_cache(
                games,
                filters,
                favorites,
                expanded_parents,
                game_index,
                category,
                hardware_filter,
                pre_filtered_indices,
            );
        }

        let total_rows = self.expanded_rows_cache.len();

        // Calculate available height first
        let available_height = ui.available_height();

        if total_rows == 0 {
            // Use all available height for empty state to match main list behavior

            // Allocate space for the empty state using all available height
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), available_height),
                egui::Sense::hover(),
            );

            // Draw centered content in the allocated space
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
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

            return (None, None, None);
        }

        // Show stats untuk large collections
        self.show_stats(ui, games.len());

        let table_height = (available_height - STATS_BAR_HEIGHT).max(100.0);
        let body_scroll_height = (table_height - TABLE_HEADER_HEIGHT).max(80.0);

        // Allocate the full available height for the table container
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), table_height),
            egui::Sense::hover(),
        );

        let (play_requested, favorite_toggled, properties_requested) = ui
            .scope_builder(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
                |ui| {
                    ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    egui::ScrollArea::horizontal()
                        .id_salt("game_list_columns")
                        .auto_shrink([false, false])
                        .max_width(rect.width())
                        .show(ui, |ui| {
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
                                (body_scroll_height - 16.0).max(64.0),
                                column_widths,
                                visible_columns,
                                default_icon,
                                game_stats,
                                has_catver,
                                theme_colors,
                            )
                        })
                        .inner
                },
            )
            .inner;

        (play_requested, favorite_toggled, properties_requested)
    }

    /// Render table dengan TRUE virtual scrolling.
    /// Returns (play_game_index, favorite_toggled_game, properties_game_index).
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
        body_scroll_height: f32,
        column_widths: &mut ColumnWidths,
        visible_columns: &VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, GameStats>,
        has_catver: bool,
        theme_colors: Option<&crate::models::GameListColors>, // Add theme_colors parameter
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        let mut play_requested = None;
        let mut favorite_toggled: Option<String> = None;
        let mut properties_requested = None;

        let total_rows = self.expanded_rows_cache.len();

        let default_colors = crate::models::GameListColors::default();
        let colors = theme_colors.unwrap_or(&default_colors);
        let header_bg_color = colors.header_bg;
        let header_text_color = colors.header_text;

        // Track hovered row for visual feedback
        let _hovered_row: Option<usize> = None;

        let previous_spacing = ui.spacing().clone();
        ui.spacing_mut().scroll = table_scroll_style();
        // Dense table cells use compact controls within the shared dialog theme.
        ui.spacing_mut().button_padding = egui::vec2(4.0, 2.0);
        ui.spacing_mut().item_spacing.x = 6.0;

        // Match List mode: reserve scrollbar width and keep the thumb visible/draggable.
        let mut table = egui_extras::TableBuilder::new(ui)
            .id_salt("game_list_table")
            .striped(false)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .min_scrolled_height(0.0)
            .max_scroll_height(body_scroll_height)
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .drag_to_scroll(true)
            .vscroll(true);

        if let Some(target_row) = self.scroll_to_row.take() {
            table = table.scroll_to_row(target_row, Some(egui::Align::Center));
        }

        // Define columns with better spacing
        table = table.column(
            Column::initial(column_widths.expand)
                .clip(true)
                .at_least(30.0),
        );

        table = table.column(
            Column::initial(column_widths.favorite)
                .clip(true)
                .at_least(40.0),
        );

        if show_icons {
            table = table.column(
                Column::initial(column_widths.icon)
                    .clip(true)
                    .at_least(50.0),
            );
        }

        // Status column is always shown
        table = table.column(
            Column::initial(column_widths.status)
                .clip(true)
                .at_least(40.0),
        );

        table = table.column(
            Column::initial(column_widths.game.max(200.0))
                .clip(true)
                .at_least(100.0),
        );

        // Add remaining columns...
        if visible_columns.play_count {
            table = table.column(
                Column::initial(column_widths.play_count)
                    .clip(true)
                    .at_least(60.0),
            );
        }

        if visible_columns.manufacturer {
            table = table.column(
                Column::initial(column_widths.manufacturer)
                    .clip(true)
                    .at_least(80.0),
            );
        }

        if visible_columns.year {
            table = table.column(
                Column::initial(column_widths.year)
                    .clip(true)
                    .at_least(50.0),
            );
        }

        if visible_columns.driver {
            table = table.column(
                Column::initial(column_widths.driver)
                    .clip(true)
                    .at_least(60.0),
            );
        }

        if visible_columns.driver_status {
            table = table.column(
                Column::initial(column_widths.driver_status)
                    .clip(true)
                    .at_least(80.0),
            );
        }

        if visible_columns.category {
            table = table.column(
                Column::initial(column_widths.category)
                    .clip(true)
                    .at_least(80.0),
            );
        }

        if visible_columns.rom {
            table = table.column(Column::initial(column_widths.rom).clip(true).at_least(60.0));
        }

        if visible_columns.chd {
            table = table.column(Column::initial(column_widths.chd).clip(true).at_least(60.0));
        }

        // Render the table with enhanced header
        let _response = table
            .header(TABLE_HEADER_HEIGHT, |mut header| {
                // Custom header rendering with gradient background
                let render_header = |ui: &mut egui::Ui, text: &str| {
                    let rect = ui.available_rect_before_wrap();

                    // Draw gradient background
                    ui.painter().rect_filled(rect, 4.0, header_bg_color);

                    // Draw header text with shadow effect
                    ui.label(
                        egui::RichText::new(text)
                            .strong()
                            .color(header_text_color)
                            .size(14.0),
                    );
                };

                header.col(|ui| {
                    render_header(ui, "");
                });
                header.col(|ui| {
                    render_header(ui, "★");
                });
                if show_icons {
                    header.col(|ui| {
                        render_header(ui, "Icon");
                    });
                }
                header.col(|ui| {
                    render_header(ui, "St");
                });
                header.col(|ui| {
                    render_header(ui, "Game");
                });

                if visible_columns.play_count {
                    header.col(|ui| {
                        render_header(ui, "Plays");
                    });
                }
                if visible_columns.manufacturer {
                    header.col(|ui| {
                        render_header(ui, "Manufacturer");
                    });
                }
                if visible_columns.year {
                    header.col(|ui| {
                        render_header(ui, "Year");
                    });
                }
                if visible_columns.driver {
                    header.col(|ui| {
                        render_header(ui, "Driver");
                    });
                }
                if visible_columns.driver_status {
                    header.col(|ui| {
                        render_header(ui, "Driver Status");
                    });
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
                    header.col(|ui| {
                        render_header(ui, "ROM");
                    });
                }
                if visible_columns.chd {
                    header.col(|ui| {
                        render_header(ui, "CHD");
                    });
                }
            })
            .body(|body| {
                body.rows(self.row_height.max(36.0), total_rows, |mut row| {
                    let row_idx = row.index();

                    if let Some(row_data) = self.expanded_rows_cache.get(row_idx).cloned()
                        && let Some(game) = games.get(row_data.game_idx)
                    {
                        let (row_play_requested, row_favorite_toggled, row_properties_requested) =
                            self.render_single_row(
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
                                theme_colors, // Pass theme_colors
                            );

                        if row_play_requested.is_some() {
                            play_requested = row_play_requested;
                        }
                        if let Some(game_name) = row_favorite_toggled {
                            favorite_toggled = Some(game_name);
                        }
                        if row_properties_requested.is_some() {
                            properties_requested = row_properties_requested;
                        }
                    }
                });
            });

        *ui.spacing_mut() = previous_spacing;

        (play_requested, favorite_toggled, properties_requested)
    }

    /// Render single row - dipanggil HANYA untuk visible rows
    /// Returns (play_game_index, favorite_toggled_game, properties_game_index)
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
        theme_colors: Option<&crate::models::GameListColors>,
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        let is_selected = selected.is_some_and(|s| s == row_data.game_idx);
        let is_favorite = favorites.contains(&game.name);
        let mut play_requested = None;
        let mut favorite_toggled = None;
        let mut properties_requested = None;

        // Get row index for alternating colors
        let row_idx = row.index();

        // Track if row is hovered
        let mut is_hovered = false;

        // Get theme colors or use defaults
        let colors = if let Some(theme_colors) = theme_colors {
            theme_colors
        } else {
            &crate::models::GameListColors::default()
        };

        // Expand/collapse button
        row.col(|ui| {
            // Check if this cell is hovered
            let cell_rect = ui.max_rect();
            if ui.rect_contains_pointer(cell_rect) {
                is_hovered = true;
            }

            // Use consistent background colors for all columns
            let bg_color = if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else if row_idx.is_multiple_of(2) {
                colors.row_bg_even // Darker for even rows
            } else {
                colors.row_bg_odd // Lighter for odd rows - more contrast
            };

            // Draw background
            ui.painter().rect_filled(cell_rect, 0.0, bg_color);

            // Draw selection/hover effects
            if is_selected {
                // Left edge highlight for selected row
                let highlight_rect =
                    egui::Rect::from_min_size(cell_rect.min, egui::vec2(4.0, cell_rect.height()));
                ui.painter()
                    .rect_filled(highlight_rect, 0.0, ui.visuals().selection.stroke.color);
            } else if is_hovered {
                // Subtle left edge highlight for hover
                let highlight_rect =
                    egui::Rect::from_min_size(cell_rect.min, egui::vec2(2.0, cell_rect.height()));
                ui.painter().rect_filled(
                    highlight_rect,
                    0.0,
                    ui.visuals().selection.stroke.color.gamma_multiply(0.35),
                );
            }

            ui.add_space(12.0); // Increased padding inside block
            if !row_data.is_clone {
                if let Some(index) = game_index {
                    if index.has_clones(&game.name) {
                        let is_expanded =
                            expanded_parents.get(&game.name).copied().unwrap_or(false);
                        let arrow = if is_expanded { "▼" } else { "▶" };

                        let arrow_response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(arrow)
                                    .color(colors.status_unknown)
                                    .size(12.0),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .small(),
                        );

                        if arrow_response.clicked() {
                            expanded_parents.insert(game.name.clone(), !is_expanded);
                            self.invalidate_cache();
                        }
                    } else {
                        ui.add_space(20.0);
                    }
                } else {
                    ui.add_space(20.0);
                }
            } else {
                ui.add_space(20.0);
            }
        });

        // Favorite star with animation
        row.col(|ui| {
            // Check hover and draw background
            let cell_rect = ui.max_rect();
            if ui.rect_contains_pointer(cell_rect) {
                is_hovered = true;
            }

            let bg_color = if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else if row_idx.is_multiple_of(2) {
                colors.row_bg_even // Darker for even rows
            } else {
                colors.row_bg_odd // Lighter for odd rows - more contrast
            };

            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            let star = if is_favorite { "★" } else { "☆" };
            let star_color = if is_favorite {
                colors.favorite_active
            } else {
                colors.favorite_inactive
            };

            ui.add_space(4.0);
            let star_response = ui.add(
                egui::Button::new(egui::RichText::new(star).color(star_color).size(18.0))
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
            );

            if star_response.clicked() {
                favorite_toggled = Some(game.name.clone());
            }

            // Add glow effect on hover
            if star_response.hovered() && is_favorite {
                ui.painter().circle(
                    star_response.rect.center(),
                    12.0,
                    colors.favorite_active.gamma_multiply(0.12),
                    egui::Stroke::NONE,
                );
            }
        });

        // Game icon with rounded corners
        if show_icons {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);

                // Add more padding around the icon for breathing room
                ui.add_space(6.0);

                if let Some(texture) = icons.get(&game.name).or(default_icon) {
                    // Add padding container around the icon
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0); // Top padding
                        let _icon_response = ui.add(
                            egui::Image::new(texture)
                                .fit_to_exact_size(egui::Vec2::splat((icon_size - 4) as f32)) // Slightly smaller to account for padding
                                .corner_radius(4.0),
                        );
                        ui.add_space(2.0); // Bottom padding
                    });
                } else {
                    // Placeholder with background and padding
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0); // Top padding
                        let rect = ui
                            .allocate_space(egui::Vec2::splat((icon_size - 4) as f32))
                            .1;
                        ui.painter()
                            .rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
                        ui.add_space(2.0); // Bottom padding
                    });
                }

                ui.add_space(6.0); // Right padding
            });
        }

        // Status indicator with glow
        row.col(|ui| {
            // Check hover and draw background
            let cell_rect = ui.max_rect();
            if ui.rect_contains_pointer(cell_rect) {
                is_hovered = true;
            }

            let bg_color = if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else if row_idx.is_multiple_of(2) {
                colors.row_bg_even // Darker for even rows
            } else {
                colors.row_bg_odd // Lighter for odd rows - more contrast
            };

            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            let (icon, color) = if let Some(status) = game.verification_status {
                let color = match status {
                    crate::models::VerificationStatus::Verified => colors.status_available,
                    crate::models::VerificationStatus::Failed => colors.status_missing,
                    crate::models::VerificationStatus::Warning => ui.visuals().warn_fg_color,
                    crate::models::VerificationStatus::NotFound
                    | crate::models::VerificationStatus::NotVerified => colors.status_unknown,
                };
                (status.to_icon(), color)
            } else {
                (
                    game.status.to_icon(),
                    match game.status {
                        RomStatus::Available => colors.status_available,
                        RomStatus::Missing => colors.status_missing,
                        _ => colors.status_unknown,
                    },
                )
            };

            ui.add_space(4.0);
            let status_label = ui.label(egui::RichText::new(icon).color(color).size(16.0));

            // Add glow effect for available games
            if matches!(game.status, RomStatus::Available) {
                ui.painter().circle(
                    status_label.rect.center(),
                    8.0,
                    colors.status_available.gamma_multiply(0.08),
                    egui::Stroke::NONE,
                );
            }
        });

        // Game name with enhanced text styling
        row.col(|ui| {
            // Check hover and draw background
            let cell_rect = ui.max_rect();
            if ui.rect_contains_pointer(cell_rect) {
                is_hovered = true;
            }

            let bg_color = if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else if row_idx.is_multiple_of(2) {
                colors.row_bg_even // Darker for even rows
            } else {
                colors.row_bg_odd // Lighter for odd rows - more contrast
            };

            ui.painter().rect_filled(cell_rect, 0.0, bg_color);

            ui.add_space(8.0);

            let text_color = if is_selected {
                ui.visuals().strong_text_color()
            } else if is_hovered {
                // Brighter text on hover for better readability
                ui.visuals().strong_text_color()
            } else if row_data.is_clone {
                colors.clone_text
            } else {
                ui.visuals().text_color()
            };

            let game_text = if row_data.is_clone {
                format!("  └─ {}", game.description)
            } else {
                game.description.clone()
            };

            let response = ui.add(
                egui::Label::new(egui::RichText::new(game_text).color(text_color).size(14.0))
                    .sense(egui::Sense::click()),
            );

            if response.clicked() {
                *selected = Some(row_data.game_idx);
            }

            if response.double_clicked() {
                play_requested = Some(row_data.game_idx);
            }

            // Context menu
            response.context_menu(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);

                if ui.button("🎮 Play Game").clicked() {
                    play_requested = Some(row_data.game_idx);
                    ui.close();
                }

                ui.separator();

                if ui.button("⚙️ Properties...").clicked() {
                    properties_requested = Some(row_data.game_idx);
                    ui.close();
                }

                let star_text = if is_favorite {
                    "★ Remove from Favorites"
                } else {
                    "☆ Add to Favorites"
                };

                if ui.button(star_text).clicked() {
                    favorite_toggled = Some(game.name.clone());
                    ui.close();
                }
            });
        });

        // Play Count with badge styling
        if visible_columns.play_count {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let play_count = game_stats
                    .get(&game.name)
                    .map(|stats| stats.play_count)
                    .unwrap_or(0);

                if play_count > 0 {
                    // Badge style for play count
                    let text = play_count.to_string();
                    let galley = ui.painter().layout_no_wrap(
                        text.clone(),
                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                        ui.visuals().strong_text_color(),
                    );

                    let rect = egui::Rect::from_min_size(
                        ui.cursor().min,
                        galley.size() + egui::vec2(16.0, 4.0),
                    );

                    ui.painter().rect_filled(rect, 12.0, colors.row_bg_selected);

                    ui.painter().galley(
                        rect.center() - galley.size() / 2.0,
                        galley,
                        ui.visuals().strong_text_color(),
                    );
                    ui.allocate_rect(rect, egui::Sense::hover());
                } else {
                    ui.label("-");
                }
            });
        }

        // Manufacturer
        if visible_columns.manufacturer {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.label(
                    egui::RichText::new(&game.manufacturer)
                        .color(text_color)
                        .size(13.0),
                );
            });
        }

        // Year
        if visible_columns.year {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.label(egui::RichText::new(&game.year).color(text_color).size(13.0));
            });
        }

        // Driver
        if visible_columns.driver {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.label(
                    egui::RichText::new(&game.driver)
                        .color(text_color)
                        .size(13.0),
                );
            });
        }

        // Driver Status
        if visible_columns.driver_status {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let (icon, text) = game.get_driver_status_display();
                let color = match game.driver_status.as_str() {
                    "good" => colors.status_available,
                    "imperfect" => ui.visuals().warn_fg_color,
                    _ => colors.status_missing,
                };
                let display = format!("{} {}", icon, text);
                ui.colored_label(color, display);
            });
        }

        // Category
        if visible_columns.category {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.label(
                    egui::RichText::new(&game.category)
                        .color(text_color)
                        .size(13.0),
                );
            });
        }

        // ROM
        if visible_columns.rom {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };
                ui.label(egui::RichText::new(&game.name).color(text_color).size(13.0));
            });
        }

        // CHD
        if visible_columns.chd {
            row.col(|ui| {
                // Check hover and draw background
                let cell_rect = ui.max_rect();
                if ui.rect_contains_pointer(cell_rect) {
                    is_hovered = true;
                }

                let bg_color = if is_selected {
                    colors.row_bg_selected
                } else if is_hovered {
                    colors.row_bg_hover
                } else if row_idx.is_multiple_of(2) {
                    colors.row_bg_even // Darker for even rows
                } else {
                    colors.row_bg_odd // Lighter for odd rows - more contrast
                };

                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let chd_text = if game.requires_chd {
                    if let Some(chd_name) = &game.chd_name {
                        chd_name.clone()
                    } else {
                        "Required".to_string()
                    }
                } else {
                    "None".to_string()
                };

                let text_color = if is_hovered || is_selected {
                    ui.visuals().text_color()
                } else {
                    ui.visuals().weak_text_color()
                };

                ui.label(egui::RichText::new(chd_text).color(text_color).size(13.0));
            });
        }

        (play_requested, favorite_toggled, properties_requested)
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

        if !filters.selected_manufacturers.is_empty() {
            filtered_indices.retain(|&idx| {
                games
                    .get(idx)
                    .is_some_and(|game| filters.manufacturer_matches(&game.manufacturer))
            });
        }

        // The manager/pre-filtered candidates may lag a toggle by one frame.
        filtered_indices.retain(|&idx| {
            games
                .get(idx)
                .is_some_and(|game| filters.rom_requirement_matches(game.requires_roms))
        });

        // Step 1.5: Apply ROM set type specific filtering to prevent duplicates
        filtered_indices =
            self.apply_rom_set_filtering(games, filtered_indices, filters, game_index);

        self.filtered_indices_cache = filtered_indices;

        // Step 2: Apply sorting to the filtered indices
        self.apply_sorting(games);

        // Step 3: Build expanded rows dengan clones
        self.expanded_rows_cache.clear();
        self.expanded_rows_cache
            .reserve(self.filtered_indices_cache.len() * 2); // Reserve space

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
                let should_expand = expanded_parents.get(&game.name).copied().unwrap_or(false)
                    || filters.auto_expand_clones;
                if !game.is_clone
                    && should_expand
                    && let Some(index) = game_index
                {
                    // O(1) clone lookup thanks to GameIndex!
                    for clone_idx in index.get_clones(&game.name) {
                        if !games.get(clone_idx).is_some_and(|clone| {
                            filters.rom_requirement_matches(clone.requires_roms)
                        }) {
                            continue;
                        }
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

        self.cache_valid = true;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            println!(
                "Warning: Cache update took {}ms for {} games",
                elapsed.as_millis(),
                self.expanded_rows_cache.len()
            );
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
                SortColumn::Status => game_a.status.description().cmp(game_b.status.description()),
                SortColumn::Category => game_a.category.cmp(&game_b.category),
            };

            if sort_ascending {
                ordering
            } else {
                ordering.reverse()
            }
        });
    }

    /// Fast filtering menggunakan GameIndex with new multi-selection filters
    fn filter_with_index(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        index: &GameIndex,
        _category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>,
    ) -> Vec<usize> {
        // Check search cache first
        if !filters.search_text.is_empty()
            && let Some(cached) = index.get_cached_search(&filters.search_text)
        {
            // Cached text matches are candidates; exclusions still apply.
            return self.apply_categorized_filters(games, filters, favorites, cached.to_vec());
        }

        // Start with all games
        let mut result: Vec<usize> = (0..games.len()).collect();

        // Apply new multi-selection filters
        result = self.apply_categorized_filters(games, filters, favorites, result);

        // Apply catver category filter if set
        if let Some(ref catver_category) = filters.catver_category {
            result.retain(|&idx| {
                if let Some(game) = games.get(idx) {
                    &game.category == catver_category
                } else {
                    false
                }
            });
        }

        // Apply text search last
        if !filters.search_text.is_empty() {
            let search_lower = filters.search_text.to_lowercase();

            // Use parallel search untuk large datasets
            if result.len() > 1000 {
                use rayon::prelude::*;
                result = result
                    .into_par_iter()
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
                                crate::models::filters::SearchMode::Status => game
                                    .status
                                    .description()
                                    .to_lowercase()
                                    .contains(&search_lower),
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
                                // Enhanced search modes are handled by GameIndexManager
                                crate::models::filters::SearchMode::FuzzySearch
                                | crate::models::filters::SearchMode::FullText
                                | crate::models::filters::SearchMode::Regex => {
                                    // These are handled by enhanced search in GameIndexManager
                                    game.description.to_lowercase().contains(&search_lower)
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
                            crate::models::filters::SearchMode::Status => game
                                .status
                                .description()
                                .to_lowercase()
                                .contains(&search_lower),
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
                            // Enhanced search modes are handled by GameIndexManager
                            crate::models::filters::SearchMode::FuzzySearch
                            | crate::models::filters::SearchMode::FullText
                            | crate::models::filters::SearchMode::Regex => {
                                // These are handled by enhanced search in GameIndexManager
                                game.description.to_lowercase().contains(&search_lower)
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

    /// Apply the new categorized multi-selection filters
    fn apply_categorized_filters(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        indices: Vec<usize>,
    ) -> Vec<usize> {
        indices
            .into_iter()
            .filter(|&idx| {
                if let Some(game) = games.get(idx) {
                    // AVAILABILITY check (OR within category)
                    let availability_match = {
                        let avail = &filters.availability_filters;
                        // If no filters selected, show all
                        if !avail.show_available && !avail.show_unavailable {
                            true
                        } else {
                            (avail.show_available && matches!(game.status, RomStatus::Available))
                                || (avail.show_unavailable
                                    && !matches!(game.status, RomStatus::Available))
                        }
                    };

                    // STATUS check (OR within category)
                    let status_match = {
                        let status = &filters.status_filters;
                        // If no filters selected, show all
                        if !status.show_working && !status.show_not_working {
                            true
                        } else {
                            let is_working =
                                matches!(game.driver_status.as_str(), "good" | "imperfect");
                            (status.show_working && is_working)
                                || (status.show_not_working && !is_working)
                        }
                    };

                    // OTHERS check (OR within category)
                    let others_match = {
                        let others = &filters.other_filters;
                        // If no filters selected, show all
                        if !others.show_favorites
                            && !others.show_parents_only
                            && !others.show_chd_games
                        {
                            true
                        } else {
                            (others.show_favorites && favorites.contains(&game.name))
                                || (others.show_parents_only && !game.is_clone)
                                || (others.show_chd_games && game.requires_chd)
                        }
                    };

                    // AND logic between categories
                    filters.rom_requirement_matches(game.requires_roms)
                        && availability_match
                        && status_match
                        && others_match
                        && filters.manufacturer_matches(&game.manufacturer)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Apply ROM set type specific filtering to prevent duplicates
    fn apply_rom_set_filtering(
        &self,
        games: &[Game],
        mut filtered_indices: Vec<usize>,
        filters: &FilterSettings,
        game_index: Option<&GameIndex>,
    ) -> Vec<usize> {
        // A ROM-bearing child must remain reachable when this exclusion hides
        // its ROM-less parent. Only promote children already in the filtered
        // candidates, so search/manufacturer/other filters retain their meaning.
        let hidden_romless_parents: HashSet<&str> = if filters.hide_romless_systems {
            games
                .iter()
                .filter(|game| !game.requires_roms)
                .map(|game| game.name.as_str())
                .collect()
        } else {
            HashSet::new()
        };
        let standalone_child = |game: &Game| {
            game.is_clone
                && game.requires_roms
                && game
                    .parent
                    .as_deref()
                    .is_some_and(|parent| hidden_romless_parents.contains(parent))
        };

        // Special handling for "All Games" filter with auto expand clones
        // When auto expand is enabled, we want to show parent games and their clones
        // but avoid showing standalone clones (clones without parents in the list)

        if filters.auto_expand_clones {
            // For auto expand mode, we need to:
            // 1. Keep all parent games
            // 2. Keep clones that have their parent in the filtered list
            // 3. Remove standalone clones (clones whose parent is not in the list)

            let parent_names: HashSet<String> = filtered_indices
                .iter()
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
                    if !game.is_clone || standalone_child(game) {
                        // Keep normal parents and children whose parent is hidden above.
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
                                !game.is_clone || standalone_child(game)
                            } else {
                                false
                            }
                        });
                    }
                }
                RomSetType::Split => {
                    // For split sets, show parent games and optionally clones
                    if !filters.show_clones_in_split {
                        filtered_indices.retain(|&idx| {
                            if let Some(game) = games.get(idx) {
                                !game.is_clone || standalone_child(game)
                            } else {
                                false
                            }
                        });
                    }
                }
                RomSetType::Merged => {
                    // For merged sets, show only parent games (clones are merged into parent)
                    filtered_indices.retain(|&idx| {
                        if let Some(game) = games.get(idx) {
                            !game.is_clone || standalone_child(game)
                        } else {
                            false
                        }
                    });
                }
                RomSetType::Unknown => {
                    // If type is unknown, try to detect based on clone ratio
                    let total_games = filtered_indices.len();
                    let clone_count = filtered_indices
                        .iter()
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
                                !game.is_clone || standalone_child(game)
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
    fn get_parent_name(
        &self,
        _games: &[Game],
        clone_game: &Game,
        _index: &GameIndex,
    ) -> Option<String> {
        // Use the parent field directly from the Game struct
        clone_game.parent.clone()
    }

    /// Manual filtering fallback (tanpa GameIndex) with new multi-selection filters
    fn filter_manual(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        _category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>,
    ) -> Vec<usize> {
        let search_lower = filters.search_text.to_lowercase();

        games
            .iter()
            .enumerate()
            .filter(|(idx, game)| {
                // Apply new categorized filters
                let indices = vec![*idx];
                let filtered = self.apply_categorized_filters(games, filters, favorites, indices);
                if filtered.is_empty() {
                    return false;
                }

                // Apply catver category filter if set
                if let Some(ref catver_category) = filters.catver_category
                    && &game.category != catver_category
                {
                    return false;
                }

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
                        crate::models::filters::SearchMode::Status => game
                            .status
                            .description()
                            .to_lowercase()
                            .contains(&search_lower),
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
                        // Enhanced search modes are handled by GameIndexManager
                        crate::models::filters::SearchMode::FuzzySearch
                        | crate::models::filters::SearchMode::FullText
                        | crate::models::filters::SearchMode::Regex => {
                            // These are handled by enhanced search in GameIndexManager
                            game.description.to_lowercase().contains(&search_lower)
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
        });
        ui.separator();
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
        filters.hide_romless_systems.hash(&mut hasher);
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

        filters.cpu_filter.hash(&mut hasher);
        filters.device_filter.hash(&mut hasher);
        filters.sound_filter.hash(&mut hasher);
        let mut manufacturers: Vec<_> = filters.selected_manufacturers.iter().collect();
        manufacturers.sort();
        for m in manufacturers {
            m.hash(&mut hasher);
        }

        // Hash INI filter state - CRITICAL for cache invalidation
        // INI filter removed from hash

        hasher.finish()
    }

    /// Show column width management context menu
    fn show_column_width_menu(
        &self,
        ui: &mut egui::Ui,
        column_widths: &mut crate::models::ColumnWidths,
    ) {
        ui.label("Adjust Column Widths:");
        ui.separator();

        let columns = [
            ("Game", &mut column_widths.game, 100.0, 500.0),
            ("Manufacturer", &mut column_widths.manufacturer, 80.0, 400.0),
            ("Year", &mut column_widths.year, 40.0, 100.0),
            ("Driver", &mut column_widths.driver, 60.0, 200.0),
            (
                "Driver Status",
                &mut column_widths.driver_status,
                80.0,
                200.0,
            ),
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

#[cfg(test)]
mod interaction_tests {
    use super::*;
    use crate::models::AppConfig;
    use crate::ui::panels::GameListView;

    fn game(name: &str, title: &str) -> Game {
        Game {
            name: name.into(),
            description: title.into(),
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
            requires_roms: true,
            requires_chd: false,
            chd_name: None,
            verification_status: None,
        }
    }

    fn text_position(shapes: &[egui::epaint::ClippedShape], label: &str) -> egui::Pos2 {
        fn find(shape: &egui::Shape, label: &str) -> Option<egui::Pos2> {
            match shape {
                egui::Shape::Text(text) if text.galley.job.text.contains(label) => {
                    Some(text.pos + text.galley.size() / 2.0)
                }
                egui::Shape::Vec(shapes) => shapes.iter().find_map(|shape| find(shape, label)),
                _ => None,
            }
        }
        shapes
            .iter()
            .find_map(|shape| find(&shape.shape, label))
            .unwrap_or_else(|| panic!("Missing rendered label {label:?}"))
    }

    fn pointer_event(
        pos: egui::Pos2,
        button: egui::PointerButton,
        pressed: bool,
    ) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button,
                pressed,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }

    fn context_action_targets_clicked_row(use_list: bool, properties: bool, clone: bool) {
        let context = egui::Context::default();
        let mut games = vec![game("alpha", "Audit Alpha"), game("bravo", "Audit Bravo")];
        if clone {
            games[1].is_clone = true;
            games[1].parent = Some("alpha".into());
        }
        let index = GameIndex::build(games.clone(), HashSet::new());
        let mut config = AppConfig::default();
        config.filter_settings.rom_set_type = RomSetType::Merged;
        let mut table = GameList::new();
        let mut list = GameListView::new();
        // The menu target is Bravo, while Alpha remains selected throughout.
        let mut selected = Some(0);
        let mut expanded = HashMap::new();
        if clone {
            expanded.insert("alpha".into(), true);
        }
        let mut icons = HashMap::new();
        let mut time = 0.0;
        let mut frame = |events| {
            time += 0.05;
            let mut actions = (None, None, None);
            let output = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1200.0, 800.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        actions = if use_list {
                            list.show(
                                ui,
                                &games,
                                &config.filter_settings,
                                &mut selected,
                                &mut expanded,
                                &config.favorite_games,
                                &mut icons,
                                false,
                                32,
                                Some(&index),
                                FilterCategory::All,
                                &mut config.column_widths,
                                &config.preferences.visible_columns,
                                None,
                                &config.game_stats,
                                None,
                                false,
                                Some(&[0, 1]),
                                None,
                            )
                        } else {
                            table.show(
                                ui,
                                &games,
                                &config.filter_settings,
                                &mut selected,
                                &mut expanded,
                                &config.favorite_games,
                                &mut icons,
                                false,
                                32,
                                Some(&index),
                                FilterCategory::All,
                                &mut config.column_widths,
                                &config.preferences.visible_columns,
                                None,
                                &config.game_stats,
                                None,
                                false,
                                Some(&[0, 1]),
                                None,
                            )
                        };
                    });
                },
            );
            (actions, output)
        };

        frame(Vec::new());
        let (_, output) = frame(Vec::new());
        let bravo = text_position(&output.shapes, "Audit Bravo");
        frame(pointer_event(bravo, egui::PointerButton::Secondary, true));
        frame(pointer_event(bravo, egui::PointerButton::Secondary, false));
        let (_, output) = frame(Vec::new());
        let command = if properties {
            "Properties..."
        } else if use_list && clone {
            "Play Clone"
        } else {
            "Play Game"
        };
        let menu_item = text_position(&output.shapes, command);
        frame(pointer_event(menu_item, egui::PointerButton::Primary, true));
        let (actions, _) = frame(pointer_event(
            menu_item,
            egui::PointerButton::Primary,
            false,
        ));
        assert_eq!(
            selected,
            Some(0),
            "right-click must not depend on changing selection"
        );
        if properties {
            assert_eq!(actions.2, Some(1));
            assert_eq!(actions.0, None);
        } else {
            assert_eq!(actions.0, Some(1));
            assert_eq!(actions.2, None);
        }
    }

    fn rendered_game_titles(shapes: &[egui::epaint::ClippedShape]) -> HashSet<String> {
        fn collect(shape: &egui::Shape, titles: &mut HashSet<String>) {
            match shape {
                egui::Shape::Text(text) => {
                    titles.insert(text.galley.job.text.clone());
                }
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, titles);
                    }
                }
                _ => {}
            }
        }
        let mut titles = HashSet::new();
        for shape in shapes {
            collect(&shape.shape, &mut titles);
        }
        titles
    }

    fn romless_rows_stay_hidden_after_cache_and_clone_expansion(use_list: bool, source: &str) {
        let context = egui::Context::default();
        let mut games = vec![
            game("parent", "Audit ROM Parent"),
            game("romless", "Audit ROM-less System"),
            game("emptyclone", "Audit ROM-less Clone"),
            game("romclone", "Audit ROM Clone"),
            game("emptyparent", "Audit ROM-less Parent"),
            game("requiredchild", "Audit ROM Child of Empty Parent"),
        ];
        games[1].requires_roms = false;
        games[2].requires_roms = false;
        for game in &mut games[2..4] {
            game.is_clone = true;
            game.parent = Some("parent".into());
        }
        games[4].requires_roms = false;
        games[5].is_clone = true;
        games[5].parent = Some("emptyparent".into());
        let mut index = GameIndex::build(games.clone(), HashSet::new());
        // A legacy text cache may contain every matching row, including hidden systems.
        index.cache_search("Audit".into(), vec![0, 1, 2, 3, 4, 5]);
        let mut config = AppConfig::default();
        config.filter_settings.search_text = "Audit".into();
        config.filter_settings.rom_set_type = if source == "manual" {
            RomSetType::NonMerged
        } else {
            RomSetType::Merged
        };
        config.filter_settings.show_clones_in_split = true;
        let mut table = GameList::new();
        let mut list = GameListView::new();
        let mut selected = None;
        let mut expanded = HashMap::from([("parent".into(), true)]);
        let mut icons = HashMap::new();
        let mut time = 0.0;
        for hide in [true, false, true, false] {
            config.filter_settings.hide_romless_systems = hide;
            // Render twice for font/layout settling, keeping both widget and query caches.
            for frame in 0..2 {
                time += 0.1;
                let output = context.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1400.0, 1000.0),
                        )),
                        time: Some(time),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            let game_index = (source != "manual").then_some(&index);
                            let prefiltered =
                                (source == "prefiltered").then_some(&[0, 1, 2, 3, 4, 5][..]);
                            if use_list {
                                list.show(
                                    ui,
                                    &games,
                                    &config.filter_settings,
                                    &mut selected,
                                    &mut expanded,
                                    &config.favorite_games,
                                    &mut icons,
                                    false,
                                    32,
                                    game_index,
                                    FilterCategory::All,
                                    &mut config.column_widths,
                                    &config.preferences.visible_columns,
                                    None,
                                    &config.game_stats,
                                    None,
                                    false,
                                    prefiltered,
                                    None,
                                );
                            } else {
                                table.show(
                                    ui,
                                    &games,
                                    &config.filter_settings,
                                    &mut selected,
                                    &mut expanded,
                                    &config.favorite_games,
                                    &mut icons,
                                    false,
                                    32,
                                    game_index,
                                    FilterCategory::All,
                                    &mut config.column_widths,
                                    &config.preferences.visible_columns,
                                    None,
                                    &config.game_stats,
                                    None,
                                    false,
                                    prefiltered,
                                    None,
                                );
                            }
                        });
                    },
                );
                if frame == 0 {
                    continue;
                }
                let titles = rendered_game_titles(&output.shapes);
                // Table clone labels include a tree prefix; inspect the title within it.
                let rendered = |title: &str| titles.iter().any(|text| text.contains(title));
                assert!(rendered("Audit ROM Parent"), "{source} list={use_list}");
                assert!(rendered("Audit ROM Clone"), "{source} list={use_list}");
                assert_eq!(
                    rendered("Audit ROM-less System"),
                    !hide,
                    "{source} list={use_list}"
                );
                assert_eq!(
                    rendered("Audit ROM-less Clone"),
                    !hide,
                    "{source} list={use_list}"
                );
                assert_eq!(
                    rendered("Audit ROM-less Parent"),
                    !hide,
                    "{source} list={use_list}"
                );
                let child_visible = hide || source == "manual";
                assert_eq!(
                    rendered("Audit ROM Child of Empty Parent"),
                    child_visible,
                    "{source} list={use_list}"
                );
                // A visible child has its own selectable/actionable row index.
                let child_row = if use_list {
                    list.row_for_game_idx(5)
                } else {
                    table.row_for_game_idx(5)
                };
                assert_eq!(
                    child_row.is_some(),
                    child_visible,
                    "{source} list={use_list}"
                );
            }
        }
    }

    #[test]
    fn required_child_of_romless_parent_keeps_its_own_filter_contract() {
        let mut games = vec![
            game("emptyparent", "Empty Parent"),
            game("child", "Required Child"),
        ];
        games[0].requires_roms = false;
        games[0].manufacturer = "Parent Maker".into();
        games[1].manufacturer = "Child Maker".into();
        games[1].is_clone = true;
        games[1].parent = Some("emptyparent".into());
        let index = GameIndex::build(games.clone(), HashSet::new());
        let table = GameList::new();
        let mut filters = FilterSettings {
            rom_set_type: RomSetType::Merged,
            ..FilterSettings::default()
        };
        for auto_expand in [false, true] {
            filters.auto_expand_clones = auto_expand;
            filters.search_text = "Child".into();
            filters.selected_manufacturers = HashSet::from(["Child Maker".into()]);
            let candidates =
                table.filter_manual(&games, &filters, &HashSet::new(), FilterCategory::All, None);
            assert_eq!(
                table.apply_rom_set_filtering(&games, candidates, &filters, Some(&index)),
                vec![1]
            );

            filters.selected_manufacturers = HashSet::from(["Parent Maker".into()]);
            let candidates =
                table.filter_manual(&games, &filters, &HashSet::new(), FilterCategory::All, None);
            assert!(
                table
                    .apply_rom_set_filtering(&games, candidates, &filters, Some(&index))
                    .is_empty()
            );

            filters.selected_manufacturers.clear();
            filters.search_text = "Parent".into();
            let candidates =
                table.filter_manual(&games, &filters, &HashSet::new(), FilterCategory::All, None);
            assert!(
                table
                    .apply_rom_set_filtering(&games, candidates, &filters, Some(&index))
                    .is_empty()
            );
        }
    }

    #[test]
    fn romless_exclusion_survives_legacy_cache_and_clone_expansion_in_both_widgets() {
        for use_list in [false, true] {
            for source in ["cached", "prefiltered", "manual"] {
                romless_rows_stay_hidden_after_cache_and_clone_expansion(use_list, source);
            }
        }
    }

    #[test]
    fn table_context_menu_targets_the_right_clicked_game() {
        for properties in [false, true] {
            context_action_targets_clicked_row(false, properties, false);
        }
    }

    #[test]
    fn list_context_menu_targets_the_right_clicked_game() {
        for properties in [false, true] {
            context_action_targets_clicked_row(true, properties, false);
        }
    }

    #[test]
    fn expanded_clone_context_menus_target_the_clone_in_both_views() {
        for use_list in [false, true] {
            for properties in [false, true] {
                context_action_targets_clicked_row(use_list, properties, true);
            }
        }
    }
}
