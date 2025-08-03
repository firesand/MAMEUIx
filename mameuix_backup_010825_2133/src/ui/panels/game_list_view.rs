// src/ui/panels/game_list_view.rs
// Modern list view implementation matching the mockup design

use eframe::egui;
use egui::{Color32, FontId, RichText, Vec2, Pos2, Rect, Response, Sense, Ui, FontFamily, TextStyle};
use crate::models::{Game, FilterSettings, FilterCategory, GameIndex, RomStatus, SortColumn, GameStats, VisibleColumns, RomSetType};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Animation state for smooth transitions
pub struct AnimationState {
    start_time: Instant,
    duration: f32,
    from: f32,
    to: f32,
}

impl AnimationState {
    fn new(duration: f32, from: f32, to: f32) -> Self {
        Self {
            start_time: Instant::now(),
            duration,
            from,
            to,
        }
    }

    fn value(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let t = (elapsed / self.duration).min(1.0);
        
        // Ease-out cubic
        let t = 1.0 - (1.0 - t).powi(3);
        
        self.from + (self.to - self.from) * t
    }

    fn is_finished(&self) -> bool {
        self.start_time.elapsed().as_secs_f32() >= self.duration
    }
}

/// State management for the list view
pub struct ListViewState {
    expansion_animations: HashMap<String, AnimationState>,
    hovered_item: Option<String>,
    selected_item: Option<String>,
    scroll_position: f32,
}

impl Default for ListViewState {
    fn default() -> Self {
        Self {
            expansion_animations: HashMap::new(),
            hovered_item: None,
            selected_item: None,
            scroll_position: 0.0,
        }
    }
}

/// Modern list view widget for games
pub struct GameListView {
    state: ListViewState,
    
    // Caching
    filtered_indices_cache: Vec<usize>,
    cache_valid: bool,
    last_filter_hash: u64,
    last_search_text: String,
    
    // Sorting
    sort_column: SortColumn,
    sort_ascending: bool,
}

impl GameListView {
    pub fn new() -> Self {
        Self {
            state: ListViewState::default(),
            filtered_indices_cache: Vec::new(),
            cache_valid: false,
            last_filter_hash: 0,
            last_search_text: String::new(),
            sort_column: SortColumn::Name,
            sort_ascending: true,
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }

    /// Main show function - renders the list view
    pub fn show(
        &mut self,
        ui: &mut Ui,
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
        hardware_filter: Option<&()>, // Placeholder for hardware filter
        has_catver: bool,
        pre_filtered_indices: Option<&[usize]>,
        theme_colors: Option<&crate::models::GameListColors>,
    ) -> (bool, Option<String>, bool) {
        let mut double_clicked = false;
        let mut favorite_toggled = None;
        let mut properties_requested = false;

        // Apply dark theme colors
        ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(15, 15, 15);
        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 26, 26);
        ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 34, 34);
        ui.style_mut().animation_time = 0.15;

        // Check if filter changed
        let current_filter_hash = self.calculate_filter_hash(filters, favorites, expanded_parents, category);
        
        if current_filter_hash != self.last_filter_hash || filters.search_text != self.last_search_text {
            self.cache_valid = false;
            self.last_filter_hash = current_filter_hash;
            self.last_search_text = filters.search_text.clone();
        }

        // Update cache if needed
        if !self.cache_valid {
            self.update_cache(games, filters, favorites, expanded_parents, game_index, category, hardware_filter, pre_filtered_indices);
        }

        let filtered_count = self.filtered_indices_cache.len();
        let total_count = games.len();

        // Games count - show filtered vs total
        ui.horizontal(|ui| {
            if filtered_count < total_count {
                ui.label(
                    RichText::new(format!("Showing {} of {} games", filtered_count, total_count))
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );
            } else {
                ui.label(
                    RichText::new(format!("Showing {} games", total_count))
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );
            }
        });
        
        ui.add_space(8.0);

        // Clean up finished animations
        self.state.expansion_animations.retain(|_, anim| !anim.is_finished());

        // Request repaint if animations are running
        if !self.state.expansion_animations.is_empty() {
            ui.ctx().request_repaint();
        }

        // Virtual scrolling implementation
        let item_height = 80.0; // Height of each item including spacing
        let clone_item_height = 64.0; // Height of clone items
        
        // Calculate total height considering expanded items
        let mut total_height = 0.0;
        let mut item_positions = Vec::new();
        
        for &game_idx in &self.filtered_indices_cache {
            if let Some(game) = games.get(game_idx) {
                item_positions.push((game_idx, total_height, false)); // (index, y_position, is_clone)
                total_height += item_height;
                
                // Add height for clones if expanded
                let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false) || filters.auto_expand_clones;
                if is_expanded && !game.is_clone {
                    if let Some(index) = game_index {
                        let clone_count = index.get_clones(&game.name).len();
                        total_height += clone_count as f32 * clone_item_height;
                    }
                }
            }
        }
        
        // Main scroll area with virtual scrolling
        egui::ScrollArea::vertical()
            .id_source("game_list_view_scroll")
            .show_rows(
                ui,
                item_height,
                self.filtered_indices_cache.len(),
                |ui, row_range| {
                    let available_width = ui.available_width();
                    
                    // Only render visible items
                    for row in row_range {
                        if let Some(&game_idx) = self.filtered_indices_cache.get(row) {
                            if let Some(game) = games.get(game_idx) {
                                // Allocate space for the item
                                let (rect, response) = ui.allocate_exact_size(
                                    Vec2::new(available_width, item_height - 8.0),
                                    Sense::hover()
                                );
                                
                                // Only render if visible
                                if ui.is_rect_visible(rect) {
                                    ui.allocate_ui_at_rect(rect, |ui| {
                                        let (clicked, fav_toggled, props_requested) = self.render_game_item(
                                            ui,
                                            game,
                                            game_idx,
                                            games,
                                            selected,
                                            expanded_parents,
                                            favorites,
                                            icons,
                                            show_icons,
                                            game_index,
                                            default_icon,
                                            game_stats,
                                        );

                                        if clicked {
                                            double_clicked = true;
                                        }
                                        if let Some(name) = fav_toggled {
                                            favorite_toggled = Some(name);
                                        }
                                        if props_requested {
                                            properties_requested = true;
                                        }
                                    });
                                    
                                    // Show clones if expanded (inline, not in virtual scroll)
                                    let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false) || filters.auto_expand_clones;
                                    if is_expanded && !game.is_clone {
                                        if let Some(index) = game_index {
                                            for clone_idx in index.get_clones(&game.name) {
                                                if let Some(clone_game) = games.get(clone_idx) {
                                                    // Allocate space for clone
                                                    let (clone_rect, _) = ui.allocate_exact_size(
                                                        Vec2::new(available_width, clone_item_height),
                                                        Sense::hover()
                                                    );
                                                    
                                                    if ui.is_rect_visible(clone_rect) {
                                                        ui.allocate_ui_at_rect(clone_rect, |ui| {
                                                            let (clone_clicked, clone_fav_toggled, clone_props_requested) = self.render_clone_item(
                                                                ui,
                                                                clone_game,
                                                                clone_idx,
                                                                selected,
                                                                favorites,
                                                                icons,
                                                                show_icons,
                                                                default_icon,
                                                            );

                                                            if clone_clicked {
                                                                double_clicked = true;
                                                            }
                                                            if let Some(name) = clone_fav_toggled {
                                                                favorite_toggled = Some(name);
                                                            }
                                                            if clone_props_requested {
                                                                properties_requested = true;
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                ui.add_space(8.0);
                            }
                        }
                    }
                }
            );

        (double_clicked, favorite_toggled, properties_requested)
    }

    /// Render a single game item
    fn render_game_item(
        &mut self,
        ui: &mut Ui,
        game: &Game,
        game_idx: usize,
        games: &[Game],
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        game_index: Option<&GameIndex>,
        default_icon: Option<&egui::TextureHandle>,
        _game_stats: &HashMap<String, crate::models::GameStats>,
    ) -> (bool, Option<String>, bool) {
        let mut double_clicked = false;
        let mut favorite_toggled = None;
        let mut properties_requested = false;

        let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false);
        let is_hovered = self.state.hovered_item.as_ref() == Some(&game.name);
        let is_selected = selected.map_or(false, |s| s == game_idx);
        let is_favorite = favorites.contains(&game.name);

        let available_width = ui.available_width();
        let item_height = 72.0;

        // Calculate expansion height for clones
        let clone_count = if let Some(index) = game_index {
            index.get_clones(&game.name).len()
        } else {
            0
        };

        // Create frame for the game item
        let frame = egui::Frame::none()
            .fill(if is_selected {
                Color32::from_rgb(26, 63, 95)
            } else if is_hovered {
                Color32::from_rgb(34, 34, 34)
            } else {
                Color32::from_rgb(26, 26, 26)
            })
            .stroke(egui::Stroke::new(
                1.0,
                if is_selected {
                    Color32::from_rgb(74, 158, 255)
                } else {
                    Color32::from_rgb(51, 51, 51)
                }
            ))
            .rounding(8.0)
            .shadow(if is_hovered || is_selected {
                egui::epaint::Shadow {
                    offset: [0, 2],
                    blur: 4,
                    spread: 0,
                    color: Color32::from_rgba_premultiplied(0, 0, 0, 80),
                }
            } else {
                egui::epaint::Shadow::default()
            });

        // Make the entire frame interactive
        let frame_response = frame.show(ui, |ui| {
            // Create an invisible interactive area that covers the entire item
            let item_response = ui.allocate_response(
                Vec2::new(available_width - 16.0, item_height),
                Sense::click()
            );

            // Draw content on top of the interactive area
            ui.allocate_ui_at_rect(item_response.rect, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);

                    // Fixed width for expand arrow area (always allocate same space)
                    let arrow_width = 24.0;
                    let arrow_rect = ui.allocate_space(Vec2::new(arrow_width, 20.0)).1;
                    
                    // Only draw arrow if this is a parent with clones
                    if clone_count > 0 && !game.is_clone {
                        let arrow = if is_expanded { "▼" } else { "▶" };
                        let arrow_response = ui.allocate_ui_at_rect(arrow_rect, |ui| {
                            ui.add(
                                egui::Button::new(
                                    RichText::new(arrow)
                                        .color(Color32::from_rgb(150, 150, 150))
                                        .size(14.0)
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                                .small()
                            )
                        }).inner;
                        
                        if arrow_response.clicked() {
                            expanded_parents.insert(game.name.clone(), !is_expanded);
                            self.invalidate_cache();
                        }
                    }

                    ui.add_space(8.0);

                    // Fixed size game preview box
                    let preview_size = Vec2::new(64.0, 48.0);
                    let preview_rect = ui.allocate_space(preview_size).1;
                    
                    // Background for preview
                    ui.painter().rect_filled(
                        preview_rect,
                        4.0,
                        Color32::from_rgb(34, 34, 34)
                    );
                    
                    // Icon or placeholder
                    if show_icons {
                        if let Some(texture) = icons.get(&game.name).or(default_icon) {
                            ui.allocate_ui_at_rect(preview_rect, |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.add(
                                        egui::Image::new(texture)
                                            .fit_to_exact_size(Vec2::new(56.0, 40.0))
                                            .rounding(4.0)
                                    );
                                });
                            });
                        } else {
                            // Emoji placeholder
                            ui.painter().text(
                                preview_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🎮",
                                FontId::proportional(24.0),
                                Color32::WHITE
                            );
                        }
                    } else {
                        // Just emoji if no icons
                        ui.painter().text(
                            preview_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "🎮",
                            FontId::proportional(24.0),
                            Color32::WHITE
                        );
                    }

                    ui.add_space(16.0);

                    // Game info
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        
                        // Game name with parent badge
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&game.description)
                                    .size(16.0)
                                    .color(Color32::from_rgb(224, 224, 224))
                                    .strong()
                            );
                            
                            // Parent badge
                            if clone_count > 0 && !game.is_clone {
                                ui.add_space(8.0);
                                egui::Frame::none()
                                    .fill(Color32::from_rgb(42, 63, 95))
                                    .rounding(4.0)
                                    .inner_margin(6.0)
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new("PARENT")
                                                .size(10.0)
                                                .color(Color32::from_rgb(255, 255, 255))
                                                .strong()
                                        );
                                    });
                            }
                        });
                        
                        // Manufacturer, year, category info
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{} • {}", game.manufacturer, game.year))
                                    .size(13.0)
                                    .color(Color32::from_rgb(136, 136, 136))
                            );
                            if !game.category.is_empty() {
                                ui.label(
                                    RichText::new(format!(" • {}", game.category))
                                        .size(13.0)
                                        .color(Color32::from_rgb(102, 102, 102))
                                );
                            }
                        });
                    });

                    // Right side badges and buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);

                        // Clone count badge
                        if clone_count > 0 && !game.is_clone {
                            self.render_clone_badge(ui, clone_count);
                            ui.add_space(8.0);
                        }

                        // Status badge
                        self.render_status_badge(ui, game);
                        ui.add_space(8.0);

                        // Favorite star
                        let star = if is_favorite { "★" } else { "☆" };
                        let star_color = if is_favorite {
                            Color32::from_rgb(255, 200, 50)
                        } else {
                            Color32::from_rgb(100, 100, 110)
                        };
                        
                        let star_response = ui.add(
                            egui::Button::new(
                                RichText::new(star)
                                    .color(star_color)
                                    .size(18.0)
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(egui::Stroke::NONE)
                        );
                        
                        if star_response.clicked() {
                            favorite_toggled = Some(game.name.clone());
                        }
                    });
                });
            });

            // Handle interactions
            if item_response.clicked() {
                *selected = Some(game_idx);
            }

            if item_response.double_clicked() {
                double_clicked = true;
            }

            if item_response.hovered() {
                self.state.hovered_item = Some(game.name.clone());
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            } else if self.state.hovered_item == Some(game.name.clone()) {
                self.state.hovered_item = None;
            }

            // Context menu
            item_response.context_menu(|ui| {
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

            item_response
        });

        (double_clicked, favorite_toggled, properties_requested)
    }

    /// Render a clone item
    fn render_clone_item(
        &mut self,
        ui: &mut Ui,
        clone: &Game,
        clone_idx: usize,
        selected: &mut Option<usize>,
        favorites: &HashSet<String>,
        icons: &HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        default_icon: Option<&egui::TextureHandle>,
    ) -> (bool, Option<String>, bool) {
        let mut double_clicked = false;
        let mut favorite_toggled = None;
        let mut properties_requested = false;

        let is_hovered = self.state.hovered_item.as_ref() == Some(&clone.name);
        let is_selected = selected.map_or(false, |s| s == clone_idx);
        let is_favorite = favorites.contains(&clone.name);

        // Clone item with indentation
        ui.horizontal(|ui| {
            ui.add_space(40.0); // Indent for clone

            let frame = egui::Frame::none()
                .fill(if is_selected {
                    Color32::from_rgb(26, 63, 95)
                } else if is_hovered {
                    Color32::from_rgb(30, 30, 30)
                } else {
                    Color32::from_rgb(20, 20, 20)
                })
                .stroke(egui::Stroke::new(
                    1.0,
                    if is_selected {
                        Color32::from_rgb(74, 158, 255)
                    } else {
                        Color32::from_rgb(40, 40, 40)
                    }
                ))
                .rounding(6.0)
                .inner_margin(8.0);

            frame.show(ui, |ui| {
                let response = ui.allocate_response(
                    Vec2::new(ui.available_width(), 56.0),
                    Sense::click()
                );

                if response.clicked() {
                    *selected = Some(clone_idx);
                }

                if response.double_clicked() {
                    double_clicked = true;
                }

                if response.hovered() {
                    self.state.hovered_item = Some(clone.name.clone());
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                // Context menu
                response.context_menu(|ui| {
                    if ui.button("🎮 Play Clone").clicked() {
                        double_clicked = true;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("⚙️ Properties...").clicked() {
                        properties_requested = true;
                        ui.close_menu();
                    }
                });

                ui.allocate_ui_at_rect(response.rect, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        
                        // Match parent's arrow space (24px) + spacing (8px) = 32px
                        ui.add_space(32.0);

                        // Clone icon with same size as parent preview
                        let icon_size = Vec2::new(64.0, 48.0);
                        let icon_rect = ui.allocate_space(icon_size).1;
                        
                        ui.painter().rect_filled(
                            icon_rect,
                            4.0,
                            Color32::from_rgb(34, 34, 34)
                        );
                        
                        if show_icons {
                            if let Some(texture) = icons.get(&clone.name).or(default_icon) {
                                ui.allocate_ui_at_rect(icon_rect, |ui| {
                                    ui.centered_and_justified(|ui| {
                                        ui.add(
                                            egui::Image::new(texture)
                                                .fit_to_exact_size(Vec2::new(56.0, 40.0))
                                                .rounding(4.0)
                                        );
                                    });
                                });
                            } else {
                                ui.painter().text(
                                    icon_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "🎯",
                                    FontId::proportional(20.0),
                                    Color32::WHITE
                                );
                            }
                        } else {
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🎯",
                                FontId::proportional(20.0),
                                Color32::WHITE
                            );
                        }

                        ui.add_space(16.0);

                        // Clone info
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(&clone.description)
                                    .size(14.0)
                                    .color(Color32::from_rgb(200, 200, 200))
                            );
                            
                            ui.label(
                                RichText::new(format!("Clone • {} • {}", clone.year, clone.name))
                                    .size(12.0)
                                    .color(Color32::from_rgb(102, 102, 102))
                            );
                        });

                        // Right side
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);

                            // Favorite star
                            let star = if is_favorite { "★" } else { "☆" };
                            let star_color = if is_favorite {
                                Color32::from_rgb(255, 200, 50)
                            } else {
                                Color32::from_rgb(80, 80, 90)
                            };
                            
                            let star_response = ui.add(
                                egui::Button::new(
                                    RichText::new(star)
                                        .color(star_color)
                                        .size(16.0)
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                            );
                            
                            if star_response.clicked() {
                                favorite_toggled = Some(clone.name.clone());
                            }
                        });
                    });
                });
            });
        });

        (double_clicked, favorite_toggled, properties_requested)
    }

    /// Render status badge
    fn render_status_badge(&self, ui: &mut Ui, game: &Game) {
        let (text, bg_color, text_color) = match game.driver_status.as_str() {
            "good" => (
                "WORKING",
                Color32::from_rgba_premultiplied(39, 201, 63, 51),
                Color32::from_rgb(255, 255, 255)
            ),
            "imperfect" => (
                "ISSUES",
                Color32::from_rgba_premultiplied(255, 189, 46, 51),
                Color32::from_rgb(255, 255, 255)
            ),
            _ => (
                "NOT WORKING",
                Color32::from_rgba_premultiplied(255, 95, 86, 51),
                Color32::from_rgb(255, 255, 255)
            ),
        };

        egui::Frame::none()
            .fill(bg_color)
            .rounding(6.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(text)
                        .size(11.0)
                        .color(text_color)
                        .strong()
                );
            });
    }

    /// Render clone count badge
    fn render_clone_badge(&self, ui: &mut Ui, count: usize) {
        egui::Frame::none()
            .fill(Color32::from_rgba_premultiplied(74, 158, 255, 51))
            .rounding(6.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} versions", count))
                        .size(12.0)
                        .color(Color32::from_rgb(255, 255, 255))
                        .strong()
                );
            });
    }

    /// Update cache with filtered games
    fn update_cache(
        &mut self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        expanded_parents: &HashMap<String, bool>,
        game_index: Option<&GameIndex>,
        category: FilterCategory,
        hardware_filter: Option<&()>, // Placeholder for hardware filter
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

        self.cache_valid = true;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            println!("Warning: Cache update took {}ms for {} games",
                     elapsed.as_millis(), self.filtered_indices_cache.len());
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

    /// Fast filtering using GameIndex with new multi-selection filters
    fn filter_with_index(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        index: &GameIndex,
        category: FilterCategory,
        hardware_filter: Option<&()>, // Placeholder for hardware filter
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

            // Use parallel search for large datasets
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
                                // Hardware filter not available
                                false
                            }
                            crate::models::filters::SearchMode::Device => {
                                // Hardware filter not available
                                false
                            }
                            crate::models::filters::SearchMode::Sound => {
                                // Hardware filter not available
                                false
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
                                // Hardware filter not available
                                false
                            }
                            crate::models::filters::SearchMode::Device => {
                                // Hardware filter not available
                                false
                            }
                            crate::models::filters::SearchMode::Sound => {
                                // Hardware filter not available
                                false
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

    /// Manual filtering fallback (without GameIndex) with new multi-selection filters
    fn filter_manual(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        category: FilterCategory,
        hardware_filter: Option<&()>, // Placeholder for hardware filter
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
                        // Hardware filter not available
                        false
                    }
                    crate::models::filters::SearchMode::Device => {
                        // Hardware filter not available
                        false
                    }
                    crate::models::filters::SearchMode::Sound => {
                        // Hardware filter not available
                        false
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

    /// Calculate hash for cache invalidation
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
        
        // Hash all filter state
        category.hash(&mut hasher);
        filters.show_favorites_only.hash(&mut hasher);
        
        // Hash AVAILABILITY filters
        filters.availability_filters.show_available.hash(&mut hasher);
        filters.availability_filters.show_unavailable.hash(&mut hasher);
        
        // Hash STATUS filters
        filters.status_filters.show_working.hash(&mut hasher);
        filters.status_filters.show_not_working.hash(&mut hasher);
        
        // Hash OTHERS filters
        filters.other_filters.show_favorites.hash(&mut hasher);
        filters.other_filters.show_parents_only.hash(&mut hasher);
        filters.other_filters.show_chd_games.hash(&mut hasher);
        
        // Hash catver category filter - CRITICAL for cache invalidation
        if let Some(ref catver_category) = filters.catver_category {
            catver_category.hash(&mut hasher);
        } else {
            // Hash None state to distinguish from Some("")
            "NONE".hash(&mut hasher);
        }

        // Hash collection sizes (cheaper than hashing contents)
        favorites.len().hash(&mut hasher);
        expanded.len().hash(&mut hasher);

        // Hash sort state
        self.sort_column.hash(&mut hasher);
        self.sort_ascending.hash(&mut hasher);
        
        // Hash ROM set type and related settings
        filters.rom_set_type.hash(&mut hasher);
        filters.show_clones_in_split.hash(&mut hasher);
        filters.auto_expand_clones.hash(&mut hasher);

        hasher.finish()
    }
}