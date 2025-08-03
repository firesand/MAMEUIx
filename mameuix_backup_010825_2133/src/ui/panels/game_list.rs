// src/ui/game_list.rs
// Optimized untuk handle 48,000+ games dengan virtual scrolling yang benar
// Kunci: hanya render yang terlihat, gunakan index untuk O(1) lookups

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use crate::models::{Game, FilterSettings, FilterCategory, GameIndex, RomStatus, SortColumn, RomSetType, GameStats, VisibleColumns, ColumnWidths};
use crate::utils::hardware_filter::HardwareFilter;
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
    pub indent_level: u32,    // Indentation level for hierarchy
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

    /// Main show function - entry point untuk rendering
    /// Returns (double_clicked, favorite_toggled_game, properties_requested)
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
    ) -> (bool, Option<String>, bool) {
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
            
            return (false, None, false);
        }

        // Show stats untuk large collections
        self.show_stats(ui, games.len());

        // Allocate the full available height for the table container
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), available_height),
            egui::Sense::hover()
        );

        // Create a child UI with the allocated rect to ensure full height usage
        let (double_clicked, favorite_toggled, properties_requested) = ui.allocate_new_ui(
            egui::UiBuilder::new().max_rect(rect),
            |ui| {
                // Use ScrollArea for better scrollbar visibility
                let scroll_output = egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .drag_to_scroll(true)
                    .show(ui, |ui| {
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
                            theme_colors, // Pass theme_colors
                        )
                    });
                scroll_output.inner
            }
        ).inner;
        
        (double_clicked, favorite_toggled, properties_requested)
    }

    /// Render table dengan TRUE virtual scrolling
    /// Returns (double_clicked, favorite_toggled_game, properties_requested)
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
        theme_colors: Option<&crate::models::GameListColors>, // Add theme_colors parameter
    ) -> (bool, Option<String>, bool) {
        let mut double_clicked = false;
        let mut favorite_toggled: Option<String> = None;
        let mut properties_requested = false;

        let total_rows = self.expanded_rows_cache.len();

        // Enhanced color scheme
        let header_bg_color = egui::Color32::from_rgb(42, 42, 48);
        let header_text_color = egui::Color32::from_rgb(180, 180, 200);
        let row_separator_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 15);
        
        // Track hovered row for visual feedback
        let mut hovered_row: Option<usize> = None;
        
        // Create custom table builder with enhanced styling
        let mut table = egui_extras::TableBuilder::new(ui)
            .striped(false) // Disable default striping, we'll handle it manually
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .min_scrolled_height(0.0)
            .max_scroll_height(f32::INFINITY)
            .vscroll(true);

        // Define columns with better spacing
        table = table.column(Column::initial(column_widths.expand)
            .clip(true)
            .at_least(30.0));
            
        table = table.column(Column::initial(column_widths.favorite)
            .clip(true)
            .at_least(40.0));

        if show_icons {
            table = table.column(Column::initial(column_widths.icon)
                .clip(true)
                .at_least(50.0));
        }

        // Status column is always shown
        table = table.column(Column::initial(column_widths.status)
            .clip(true)
            .at_least(40.0));

        table = table.column(Column::initial(column_widths.game.max(200.0))
            .clip(true)
            .at_least(100.0));

        // Add remaining columns...
        if visible_columns.play_count {
            table = table.column(Column::initial(column_widths.play_count)
                .clip(true)
                .at_least(60.0));
        }
        
        if visible_columns.manufacturer {
            table = table.column(Column::initial(column_widths.manufacturer)
                .clip(true)
                .at_least(80.0));
        }
        
        if visible_columns.year {
            table = table.column(Column::initial(column_widths.year)
                .clip(true)
                .at_least(50.0));
        }
        
        if visible_columns.driver {
            table = table.column(Column::initial(column_widths.driver)
                .clip(true)
                .at_least(60.0));
        }
        
        if visible_columns.driver_status {
            table = table.column(Column::initial(column_widths.driver_status)
                .clip(true)
                .at_least(80.0));
        }
        
        if visible_columns.category {
            table = table.column(Column::initial(column_widths.category)
                .clip(true)
                .at_least(80.0));
        }
        
        if visible_columns.rom {
            table = table.column(Column::initial(column_widths.rom)
                .clip(true)
                .at_least(60.0));
        }
        
        if visible_columns.chd {
            table = table.column(Column::initial(column_widths.chd)
                .clip(true)
                .at_least(60.0));
        }

        // Render the table with enhanced header
        let response = table
            .header(36.0, |mut header| {
                // Custom header rendering with gradient background
                let render_header = |ui: &mut egui::Ui, text: &str| {
                    let rect = ui.available_rect_before_wrap();
                    
                    // Draw gradient background
                    ui.painter().rect_filled(
                        rect,
                        4.0,
                        header_bg_color,
                    );
                    
                    // Draw header text with shadow effect
                    ui.label(
                        egui::RichText::new(text)
                            .strong()
                            .color(header_text_color)
                            .size(14.0)
                    );
                };

                header.col(|ui| { render_header(ui, ""); });
                header.col(|ui| { render_header(ui, "★"); });
                if show_icons {
                    header.col(|ui| { render_header(ui, "Icon"); });
                }
                header.col(|ui| { render_header(ui, "St"); });
                header.col(|ui| { render_header(ui, "Game"); });
                
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
                body.rows(self.row_height.max(36.0), total_rows, |mut row| {
                    let row_idx = row.index();
                    
                    if let Some(row_data) = self.expanded_rows_cache.get(row_idx).cloned() {
                        if let Some(game) = games.get(row_data.game_idx) {
                            let (row_double_clicked, row_favorite_toggled, row_properties_requested) = self.render_single_row(
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

                            if row_double_clicked {
                                double_clicked = true;
                            }
                            if let Some(game_name) = row_favorite_toggled {
                                favorite_toggled = Some(game_name);
                            }
                            if row_properties_requested {
                                properties_requested = true;
                            }
                        }
                    }
                });
            });

        (double_clicked, favorite_toggled, properties_requested)
    }

    /// Render single row - dipanggil HANYA untuk visible rows
    /// Returns (double_clicked, favorite_toggled_game, properties_requested)
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
    ) -> (bool, Option<String>, bool) {
        let is_selected = selected.map_or(false, |s| s == row_data.game_idx);
        let is_favorite = favorites.contains(&game.name);
        let mut double_clicked = false;
        let mut favorite_toggled = None;
        let mut properties_requested = false;

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
                egui::Color32::from_rgb(45, 65, 95)
            } else if is_hovered {
                egui::Color32::from_rgb(40, 40, 48)
            } else if row_idx % 2 == 0 {
                egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
            } else {
                egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
            };
            
            // Draw background
            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            
            // Draw selection/hover effects
            if is_selected {
                // Left edge highlight for selected row
                let highlight_rect = egui::Rect::from_min_size(
                    cell_rect.min,
                    egui::vec2(4.0, cell_rect.height())
                );
                ui.painter().rect_filled(
                    highlight_rect,
                    0.0,
                    egui::Color32::from_rgb(100, 150, 255),
                );
            } else if is_hovered {
                // Subtle left edge highlight for hover
                let highlight_rect = egui::Rect::from_min_size(
                    cell_rect.min,
                    egui::vec2(2.0, cell_rect.height())
                );
                ui.painter().rect_filled(
                    highlight_rect,
                    0.0,
                    egui::Color32::from_rgba_premultiplied(100, 150, 255, 60),
                );
            }
            
            ui.add_space(12.0); // Increased padding inside block
            if !row_data.is_clone {
                if let Some(index) = game_index {
                    if index.has_clones(&game.name) {
                        let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false);
                        let arrow = if is_expanded { "▼" } else { "▶" };
                        
                        let arrow_response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(arrow)
                                    .color(egui::Color32::from_rgb(150, 150, 150))
                                    .size(12.0)
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                            .small()
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
                egui::Color32::from_rgb(45, 65, 95)
            } else if is_hovered {
                egui::Color32::from_rgb(40, 40, 48)
            } else if row_idx % 2 == 0 {
                egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
            } else {
                egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
            };
            
            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            let star = if is_favorite { "★" } else { "☆" };
            let star_color = if is_favorite {
                egui::Color32::from_rgb(255, 200, 50)
            } else {
                egui::Color32::from_rgb(100, 100, 110)
            };
            
            ui.add_space(4.0);
            let star_response = ui.add(
                egui::Button::new(
                    egui::RichText::new(star)
                        .color(star_color)
                        .size(18.0)
                )
                .fill(egui::Color32::TRANSPARENT)
                .stroke(egui::Stroke::NONE)
            );
            
            if star_response.clicked() {
                favorite_toggled = Some(game.name.clone());
            }
            
            // Add glow effect on hover
            if star_response.hovered() && is_favorite {
                ui.painter().circle(
                    star_response.rect.center(),
                    12.0,
                    egui::Color32::from_rgba_premultiplied(255, 200, 50, 30),
                    egui::Stroke::NONE
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                
                // Add more padding around the icon for breathing room
                ui.add_space(6.0);
                
                if let Some(texture) = icons.get(&game.name).or(default_icon) {
                    // Add padding container around the icon
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0); // Top padding
                        let icon_response = ui.add(
                            egui::Image::new(texture)
                                .fit_to_exact_size(egui::Vec2::splat((icon_size - 4) as f32)) // Slightly smaller to account for padding
                                .rounding(4.0)
                        );
                        ui.add_space(2.0); // Bottom padding
                    });
                } else {
                    // Placeholder with background and padding
                    ui.vertical_centered(|ui| {
                        ui.add_space(2.0); // Top padding
                        let rect = ui.allocate_space(egui::Vec2::splat((icon_size - 4) as f32)).1;
                        ui.painter().rect_filled(
                            rect,
                            4.0,
                            egui::Color32::from_rgb(40, 40, 45)
                        );
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
                egui::Color32::from_rgb(45, 65, 95)
            } else if is_hovered {
                egui::Color32::from_rgb(40, 40, 48)
            } else if row_idx % 2 == 0 {
                egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
            } else {
                egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
            };
            
            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            let (icon, color) = if let Some(_verification_status) = &game.verification_status {
                game.get_verification_display()
            } else {
                (game.status.to_icon(), match game.status {
                    RomStatus::Available => egui::Color32::from_rgb(50, 200, 100),
                    RomStatus::Missing => egui::Color32::from_rgb(200, 50, 50),
                    _ => egui::Color32::from_rgb(150, 150, 150),
                })
            };
            
            ui.add_space(4.0);
            let status_label = ui.label(
                egui::RichText::new(icon)
                    .color(color)
                    .size(16.0)
            );
            
            // Add glow effect for available games
            if matches!(game.status, RomStatus::Available) {
                ui.painter().circle(
                    status_label.rect.center(),
                    8.0,
                    egui::Color32::from_rgba_premultiplied(50, 200, 100, 20),
                    egui::Stroke::NONE
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
                egui::Color32::from_rgb(45, 65, 95)
            } else if is_hovered {
                egui::Color32::from_rgb(40, 40, 48)
            } else if row_idx % 2 == 0 {
                egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
            } else {
                egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
            };
            
            ui.painter().rect_filled(cell_rect, 0.0, bg_color);
            
            ui.add_space(8.0);
            
            let text_color = if is_selected {
                egui::Color32::from_rgb(255, 255, 255)
            } else if is_hovered {
                // Brighter text on hover for better readability
                egui::Color32::from_rgb(240, 240, 255)
            } else if row_data.is_clone {
                egui::Color32::from_rgb(180, 180, 200)
            } else {
                egui::Color32::from_rgb(220, 220, 240)
            };
            
            let game_text = if row_data.is_clone {
                format!("  └─ {}", game.description)
            } else {
                game.description.clone()
            };
            
            let response = ui.add(
                egui::Label::new(
                    egui::RichText::new(game_text)
                        .color(text_color)
                        .size(14.0)
                )
                .sense(egui::Sense::click())
            );
            
            if response.clicked() {
                *selected = Some(row_data.game_idx);
            }
            
            if response.double_clicked() {
                double_clicked = true;
            }
            
            // Context menu
            response.context_menu(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);
                
                if ui.button("🎮 Play Game").clicked() {
                    double_clicked = true;
                    ui.close_menu();
                }
                
                ui.separator();
                
                if ui.button("⚙️ Properties...").clicked() {
                    properties_requested = true;
                    ui.close_menu();
                }
                
                let star_text = if is_favorite {
                    "★ Remove from Favorites"
                } else {
                    "☆ Add to Favorites"
                };
                
                if ui.button(star_text).clicked() {
                    favorite_toggled = Some(game.name.clone());
                    ui.close_menu();
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let play_count = game_stats.get(&game.name)
                    .map(|stats| stats.play_count)
                    .unwrap_or(0);
                
                if play_count > 0 {
                    // Badge style for play count
                    let text = play_count.to_string();
                    let galley = ui.painter().layout_no_wrap(
                        text.clone(),
                        egui::FontId::new(12.0, egui::FontFamily::Proportional),
                        egui::Color32::from_rgb(140, 180, 255)
                    );
                    
                    let rect = egui::Rect::from_min_size(
                        ui.cursor().min,
                        galley.size() + egui::vec2(16.0, 4.0)
                    );
                    
                    ui.painter().rect_filled(
                        rect,
                        12.0,
                        egui::Color32::from_rgba_premultiplied(100, 149, 255, 30)
                    );
                    
                    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, egui::Color32::WHITE);
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                ui.label(
                    egui::RichText::new(&game.manufacturer)
                        .color(text_color)
                        .size(13.0)
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                ui.label(
                    egui::RichText::new(&game.year)
                        .color(text_color)
                        .size(13.0)
                );
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                ui.label(
                    egui::RichText::new(&game.driver)
                        .color(text_color)
                        .size(13.0)
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let (icon, text) = game.get_driver_status_display();
                let color = game.get_driver_status_color();
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                ui.label(
                    egui::RichText::new(&game.category)
                        .color(text_color)
                        .size(13.0)
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
                };
                
                ui.painter().rect_filled(cell_rect, 0.0, bg_color);
                let text_color = if is_hovered || is_selected {
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                ui.label(
                    egui::RichText::new(&game.name)
                        .color(text_color)
                        .size(13.0)
                );
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
                    egui::Color32::from_rgb(45, 65, 95)
                } else if is_hovered {
                    egui::Color32::from_rgb(40, 40, 48)
                } else if row_idx % 2 == 0 {
                    egui::Color32::from_rgb(26, 26, 30)  // Darker for even rows
                } else {
                    egui::Color32::from_rgb(32, 32, 38)  // Lighter for odd rows - more contrast
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
                    egui::Color32::from_rgb(220, 220, 240)
                } else {
                    egui::Color32::from_rgb(200, 200, 220)
                };
                
                ui.label(
                    egui::RichText::new(chd_text)
                        .color(text_color)
                        .size(13.0)
                );
            });
        }
        
        (double_clicked, favorite_toggled, properties_requested)
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
                        for clone_idx in index.get_clones(&game.name) {
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

    /// Fast filtering menggunakan GameIndex with new multi-selection filters
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
                            // Enhanced search modes are handled by GameIndexManager
                            crate::models::filters::SearchMode::FuzzySearch |
                            crate::models::filters::SearchMode::FullText |
                            crate::models::filters::SearchMode::Regex => {
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
                            // Enhanced search modes are handled by GameIndexManager
                            crate::models::filters::SearchMode::FuzzySearch |
                            crate::models::filters::SearchMode::FullText |
                            crate::models::filters::SearchMode::Regex => {
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
        indices.into_iter()
            .filter(|&idx| {
                if let Some(game) = games.get(idx) {
                    // AVAILABILITY check (OR within category)
                    let availability_match = {
                        let avail = &filters.availability_filters;
                        // If no filters selected, show all
                        if !avail.show_available && !avail.show_unavailable {
                            true
                        } else {
                            (avail.show_available && matches!(game.status, RomStatus::Available)) ||
                            (avail.show_unavailable && !matches!(game.status, RomStatus::Available))
                        }
                    };

                    // STATUS check (OR within category)
                    let status_match = {
                        let status = &filters.status_filters;
                        // If no filters selected, show all
                        if !status.show_working && !status.show_not_working {
                            true
                        } else {
                            let is_working = matches!(game.driver_status.as_str(), "good" | "imperfect");
                            (status.show_working && is_working) ||
                            (status.show_not_working && !is_working)
                        }
                    };

                    // OTHERS check (OR within category)
                    let others_match = {
                        let others = &filters.other_filters;
                        // If no filters selected, show all
                        if !others.show_favorites && !others.show_parents_only && !others.show_chd_games {
                            true
                        } else {
                            (others.show_favorites && favorites.contains(&game.name)) ||
                            (others.show_parents_only && !game.is_clone) ||
                            (others.show_chd_games && game.requires_chd)
                        }
                    };

                    // AND logic between categories
                    availability_match && status_match && others_match
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

    /// Manual filtering fallback (tanpa GameIndex) with new multi-selection filters
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
        .filter(|(idx, game)| {
            // Apply new categorized filters
            let indices = vec![*idx];
            let filtered = self.apply_categorized_filters(games, filters, favorites, indices);
            if filtered.is_empty() {
                return false;
            }

            // Apply catver category filter if set
            if let Some(ref catver_category) = filters.catver_category {
                if &game.category != catver_category {
                    return false;
                }
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
                    // Enhanced search modes are handled by GameIndexManager
                    crate::models::filters::SearchMode::FuzzySearch |
                    crate::models::filters::SearchMode::FullText |
                    crate::models::filters::SearchMode::Regex => {
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


}
