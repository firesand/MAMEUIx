// src/ui/panels/game_list_view.rs
// Modern list view implementation matching the mockup design

use crate::models::{
    FilterCategory, FilterSettings, Game, GameIndex, RomSetType, RomStatus, SortColumn,
};
use crate::utils::hardware_filter::HardwareFilter;
use eframe::egui;
use egui::{Color32, FontId, RichText, Sense, Ui, Vec2};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

const GAME_CARD_HEIGHT: f32 = 72.0;
const CLONE_CARD_HEIGHT: f32 = 64.0;
const CARD_GAP: f32 = 8.0;

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
    pub scroll_to_row: Option<usize>,
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
            scroll_to_row: None,
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

    pub fn row_count(&self) -> usize {
        self.filtered_indices_cache.len()
    }

    pub fn game_idx_at_row(&self, row: usize) -> Option<usize> {
        self.filtered_indices_cache.get(row).copied()
    }

    pub fn row_for_game_idx(&self, game_idx: usize) -> Option<usize> {
        self.filtered_indices_cache
            .iter()
            .position(|&idx| idx == game_idx)
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
        _icon_size: u32,
        game_index: Option<&GameIndex>,
        category: FilterCategory,
        _column_widths: &mut crate::models::ColumnWidths,
        _visible_columns: &crate::models::VisibleColumns,
        default_icon: Option<&egui::TextureHandle>,
        game_stats: &HashMap<String, crate::models::GameStats>,
        hardware_filter: Option<&HardwareFilter>, // Placeholder for hardware filter
        _has_catver: bool,
        pre_filtered_indices: Option<&[usize]>,
        theme_colors: Option<&crate::models::GameListColors>,
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        let mut play_requested = None;
        let mut favorite_toggled = None;
        let mut properties_requested = None;

        let default_colors = crate::models::GameListColors::default();
        let colors = theme_colors.unwrap_or(&default_colors);
        ui.style_mut().animation_time = 0.15;

        // Check if filter changed
        let current_filter_hash =
            self.calculate_filter_hash(filters, favorites, expanded_parents, category);

        if current_filter_hash != self.last_filter_hash
            || filters.search_text != self.last_search_text
        {
            self.cache_valid = false;
            self.last_filter_hash = current_filter_hash;
            self.last_search_text = filters.search_text.clone();
        }

        // Update cache if needed
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

        let filtered_count = self.filtered_indices_cache.len();
        let total_count = games.len();

        // Games count - show filtered vs total
        ui.horizontal(|ui| {
            if filtered_count < total_count {
                ui.label(
                    RichText::new(format!(
                        "Showing {} of {} games",
                        filtered_count, total_count
                    ))
                    .size(12.0)
                    .color(ui.visuals().weak_text_color()),
                );
            } else {
                ui.label(
                    RichText::new(format!("Showing {} games", total_count))
                        .size(12.0)
                        .color(ui.visuals().weak_text_color()),
                );
            }
        });

        ui.add_space(8.0);

        // Clean up finished animations
        self.state
            .expansion_animations
            .retain(|_, anim| !anim.is_finished());

        // Request repaint if animations are running
        if !self.state.expansion_animations.is_empty() {
            ui.ctx().request_repaint();
        }

        // Virtual scrolling implementation
        let item_height = GAME_CARD_HEIGHT + CARD_GAP;
        let clone_item_height = CLONE_CARD_HEIGHT;

        // Calculate total height considering expanded items
        let mut total_height = 0.0;
        let mut item_positions = Vec::new();

        for &game_idx in &self.filtered_indices_cache {
            if let Some(game) = games.get(game_idx) {
                item_positions.push((game_idx, total_height, false)); // (index, y_position, is_clone)
                total_height += item_height;

                // Add height for clones if expanded
                let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false)
                    || filters.auto_expand_clones;
                if is_expanded
                    && !game.is_clone
                    && let Some(index) = game_index
                {
                    let clone_count =
                        Self::visible_clone_indices(games, index, &game.name, filters).len();
                    total_height += clone_count as f32 * clone_item_height;
                }
            }
        }

        // Main scroll area with virtual scrolling
        let mut scroll_target = self.scroll_to_row.take();
        ui.scope(|ui| {
            // Card heights already include their explicit gap. The dialog's
            // 10 px widget spacing must not also enter show_rows' row stride.
            ui.spacing_mut().item_spacing.y = 0.0;
            egui::ScrollArea::vertical()
                .id_salt("game_list_view_scroll")
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show_rows(
                    ui,
                    item_height,
                    self.filtered_indices_cache.len(),
                    |ui, row_range| {
                        if let Some(target_row) = scroll_target.take() {
                            let y = target_row as f32 * item_height;
                            ui.scroll_to_rect(
                                egui::Rect::from_min_size(
                                    egui::pos2(0.0, y),
                                    egui::vec2(1.0, item_height),
                                ),
                                Some(egui::Align::Center),
                            );
                        }

                        let available_width = ui.available_width();

                        // Only render visible items
                        for row in row_range {
                            if let Some(&game_idx) = self.filtered_indices_cache.get(row)
                                && let Some(game) = games.get(game_idx)
                            {
                                // Allocate space for the item
                                let (rect, _response) = ui.allocate_exact_size(
                                    Vec2::new(available_width, GAME_CARD_HEIGHT),
                                    Sense::hover(),
                                );

                                // Only render if visible
                                if ui.is_rect_visible(rect) {
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                                        let (clicked, fav_toggled, props_requested) = self
                                            .render_game_item(
                                                ui,
                                                game,
                                                game_idx,
                                                games,
                                                filters,
                                                selected,
                                                expanded_parents,
                                                favorites,
                                                icons,
                                                show_icons,
                                                game_index,
                                                default_icon,
                                                game_stats,
                                                colors,
                                            );

                                        if clicked.is_some() {
                                            play_requested = clicked;
                                        }
                                        if let Some(name) = fav_toggled {
                                            favorite_toggled = Some(name);
                                        }
                                        if props_requested.is_some() {
                                            properties_requested = props_requested;
                                        }
                                    });

                                    // Show clones if expanded (inline, not in virtual scroll)
                                    let is_expanded =
                                        expanded_parents.get(&game.name).copied().unwrap_or(false)
                                            || filters.auto_expand_clones;
                                    if is_expanded
                                        && !game.is_clone
                                        && let Some(index) = game_index
                                    {
                                        for clone_idx in Self::visible_clone_indices(
                                            games, index, &game.name, filters,
                                        ) {
                                            if let Some(clone_game) = games.get(clone_idx) {
                                                // Allocate space for clone
                                                let (clone_rect, _) = ui.allocate_exact_size(
                                                    Vec2::new(available_width, clone_item_height),
                                                    Sense::hover(),
                                                );

                                                if ui.is_rect_visible(clone_rect) {
                                                    ui.scope_builder(
                                                        egui::UiBuilder::new().max_rect(clone_rect),
                                                        |ui| {
                                                            let (
                                                                clone_clicked,
                                                                clone_fav_toggled,
                                                                clone_props_requested,
                                                            ) = self.render_clone_item(
                                                                ui,
                                                                clone_game,
                                                                clone_idx,
                                                                selected,
                                                                favorites,
                                                                icons,
                                                                show_icons,
                                                                default_icon,
                                                                colors,
                                                            );

                                                            if clone_clicked.is_some() {
                                                                play_requested = clone_clicked;
                                                            }
                                                            if let Some(name) = clone_fav_toggled {
                                                                favorite_toggled = Some(name);
                                                            }
                                                            if clone_props_requested.is_some() {
                                                                properties_requested =
                                                                    clone_props_requested;
                                                            }
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                ui.add_space(8.0);
                            }
                        }
                    },
                );
        });

        (play_requested, favorite_toggled, properties_requested)
    }

    fn visible_clone_indices(
        games: &[Game],
        index: &GameIndex,
        parent: &str,
        filters: &FilterSettings,
    ) -> Vec<usize> {
        index
            .get_clones(parent)
            .into_iter()
            .filter(|&idx| {
                games
                    .get(idx)
                    .is_some_and(|game| filters.rom_requirement_matches(game.requires_roms))
            })
            .collect()
    }

    /// Render a single game item
    fn render_game_item(
        &mut self,
        ui: &mut Ui,
        game: &Game,
        game_idx: usize,
        games: &[Game],
        filters: &FilterSettings,
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        game_index: Option<&GameIndex>,
        default_icon: Option<&egui::TextureHandle>,
        _game_stats: &HashMap<String, crate::models::GameStats>,
        colors: &crate::models::GameListColors,
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        let mut play_requested = None;
        let mut favorite_toggled = None;
        let mut properties_requested = None;

        let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false);
        let is_hovered = self.state.hovered_item.as_ref() == Some(&game.name);
        let is_selected = selected.is_some_and(|s| s == game_idx);
        let is_favorite = favorites.contains(&game.name);

        let available_width = ui.available_width();

        // Calculate expansion height for clones
        let clone_count = if let Some(index) = game_index {
            Self::visible_clone_indices(games, index, &game.name, filters).len()
        } else {
            0
        };

        // Create frame for the game item
        let frame = egui::Frame::NONE
            .fill(if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else {
                colors.row_bg_even
            })
            .stroke(egui::Stroke::new(
                1.0_f32,
                if is_selected {
                    ui.visuals().selection.stroke.color
                } else {
                    colors.row_separator
                },
            ))
            .corner_radius(8.0)
            .shadow(if is_hovered || is_selected {
                egui::epaint::Shadow {
                    offset: [0, 2],
                    blur: 4,
                    spread: 0,
                    color: ui.visuals().window_shadow.color,
                }
            } else {
                egui::epaint::Shadow::default()
            });

        let (card_rect, item_response) =
            ui.allocate_exact_size(Vec2::new(available_width, GAME_CARD_HEIGHT), Sense::click());
        ui.painter().add(frame.paint(card_rect.shrink(1.0)));

        // Reserve the action column before laying out text. A long title may
        // truncate, but it must never move the badges or favorite button.
        let mut content = ui.new_child(egui::UiBuilder::new().max_rect(card_rect));
        content.set_clip_rect(ui.clip_rect().intersect(card_rect));
        Self::compact_card_style(&mut content);
        let inner = card_rect.shrink2(Vec2::new(8.0, 6.0));
        let center_y = inner.center().y;
        let arrow_rect = egui::Rect::from_center_size(
            egui::pos2(inner.left() + 10.0, center_y),
            Vec2::new(20.0, 24.0),
        );
        if clone_count > 0 && !game.is_clone {
            let arrow = if is_expanded { "▼" } else { "▶" };
            if content
                .put(
                    arrow_rect,
                    egui::Button::new(
                        RichText::new(arrow)
                            .color(content.visuals().weak_text_color())
                            .size(14.0),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(egui::Stroke::NONE),
                )
                .clicked()
            {
                expanded_parents.insert(game.name.clone(), !is_expanded);
                self.invalidate_cache();
            }
        }
        let preview_rect = egui::Rect::from_center_size(
            egui::pos2(arrow_rect.right() + 6.0 + 32.0, center_y),
            Vec2::new(64.0, 48.0),
        );
        Self::render_preview(
            &mut content,
            preview_rect,
            icons.get(&game.name).or(default_icon),
            show_icons,
            "🎮",
        );

        let star_rect = egui::Rect::from_center_size(
            egui::pos2(inner.right() - 14.0, center_y),
            Vec2::splat(28.0),
        );
        let badges_right = star_rect.left() - 6.0;
        let badges_left = badges_right - 96.0;
        let has_clones = clone_count > 0 && !game.is_clone;
        let status_y = if has_clones {
            center_y - 23.0
        } else {
            center_y - 10.0
        };
        self.render_status_badge(
            &mut content,
            egui::Rect::from_min_size(egui::pos2(badges_left, status_y), Vec2::new(96.0, 20.0)),
            game,
            colors,
        );
        if has_clones {
            self.render_clone_badge(
                &mut content,
                egui::Rect::from_min_size(
                    egui::pos2(badges_left, center_y + 3.0),
                    Vec2::new(96.0, 20.0),
                ),
                clone_count,
                colors,
            );
        }

        let text_left = preview_rect.right() + 12.0;
        let text_width = (badges_left - 10.0 - text_left).max(0.0);
        Self::render_card_text(
            &mut content,
            egui::Rect::from_min_size(
                egui::pos2(text_left, center_y - 22.0),
                Vec2::new(text_width, 22.0),
            ),
            RichText::new(&game.description)
                .size(16.0)
                .color(ui.visuals().strong_text_color())
                .strong(),
            &game.description,
        );
        let metadata = if game.category.is_empty() {
            format!("{} • {}", game.manufacturer, game.year)
        } else {
            format!("{} • {} • {}", game.manufacturer, game.year, game.category)
        };
        let mut metadata_left = text_left;
        if has_clones {
            Self::render_badge(
                &mut content,
                egui::Rect::from_min_size(
                    egui::pos2(text_left, center_y + 4.0),
                    Vec2::new(48.0, 18.0),
                ),
                "PARENT",
                10.0,
                colors.row_bg_selected,
            );
            metadata_left += 54.0;
        }
        Self::render_card_text(
            &mut content,
            egui::Rect::from_min_size(
                egui::pos2(metadata_left, center_y + 3.0),
                Vec2::new((text_left + text_width - metadata_left).max(0.0), 20.0),
            ),
            RichText::new(&metadata)
                .size(13.0)
                .color(ui.visuals().weak_text_color()),
            &metadata,
        );
        if Self::render_favorite(&mut content, star_rect, is_favorite, colors).clicked() {
            favorite_toggled = Some(game.name.clone());
        }

        // Handle interactions
        if item_response.clicked() {
            *selected = Some(game_idx);
        }

        if item_response.double_clicked() {
            play_requested = Some(game_idx);
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
                play_requested = Some(game_idx);
                ui.close();
            }

            ui.separator();

            if ui.button("⚙️ Properties...").clicked() {
                properties_requested = Some(game_idx);
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

        (play_requested, favorite_toggled, properties_requested)
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
        colors: &crate::models::GameListColors,
    ) -> (Option<usize>, Option<String>, Option<usize>) {
        let mut play_requested = None;
        let mut favorite_toggled = None;
        let mut properties_requested = None;

        let is_hovered = self.state.hovered_item.as_ref() == Some(&clone.name);
        let is_selected = selected.is_some_and(|s| s == clone_idx);
        let is_favorite = favorites.contains(&clone.name);

        // Keep the clone's complete painted and interactive bounds inside the
        // same 64 px row that the scrolling code reserves for it.
        let (row_rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), CLONE_CARD_HEIGHT),
            Sense::hover(),
        );
        let card_rect = egui::Rect::from_min_max(row_rect.min + Vec2::new(32.0, 0.0), row_rect.max);
        let response = ui.interact(
            card_rect,
            ui.id().with(("clone_card", clone_idx)),
            Sense::click(),
        );
        let frame = egui::Frame::NONE
            .fill(if is_selected {
                colors.row_bg_selected
            } else if is_hovered {
                colors.row_bg_hover
            } else {
                colors.row_bg_odd
            })
            .stroke(egui::Stroke::new(
                1.0_f32,
                if is_selected {
                    ui.visuals().selection.stroke.color
                } else {
                    colors.row_separator
                },
            ))
            .corner_radius(6.0);
        ui.painter().add(frame.paint(card_rect.shrink(1.0)));

        let mut content = ui.new_child(egui::UiBuilder::new().max_rect(card_rect));
        content.set_clip_rect(ui.clip_rect().intersect(card_rect));
        Self::compact_card_style(&mut content);
        let inner = card_rect.shrink2(Vec2::new(8.0, 6.0));
        let center_y = inner.center().y;
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(inner.left() + 32.0, center_y),
            Vec2::new(64.0, 48.0),
        );
        Self::render_preview(
            &mut content,
            icon_rect,
            icons.get(&clone.name).or(default_icon),
            show_icons,
            "🎯",
        );
        let star_rect = egui::Rect::from_center_size(
            egui::pos2(inner.right() - 14.0, center_y),
            Vec2::splat(28.0),
        );
        let text_left = icon_rect.right() + 12.0;
        let text_width = (star_rect.left() - 10.0 - text_left).max(0.0);
        Self::render_card_text(
            &mut content,
            egui::Rect::from_min_size(
                egui::pos2(text_left, center_y - 22.0),
                Vec2::new(text_width, 22.0),
            ),
            RichText::new(&clone.description)
                .size(14.0)
                .color(colors.clone_text),
            &clone.description,
        );
        let metadata = format!("Clone • {} • {}", clone.year, clone.name);
        Self::render_card_text(
            &mut content,
            egui::Rect::from_min_size(
                egui::pos2(text_left, center_y + 3.0),
                Vec2::new(text_width, 20.0),
            ),
            RichText::new(&metadata)
                .size(12.0)
                .color(ui.visuals().weak_text_color()),
            &metadata,
        );
        if Self::render_favorite(&mut content, star_rect, is_favorite, colors).clicked() {
            favorite_toggled = Some(clone.name.clone());
        }

        if response.clicked() {
            *selected = Some(clone_idx);
        }
        if response.double_clicked() {
            play_requested = Some(clone_idx);
        }
        if response.hovered() {
            self.state.hovered_item = Some(clone.name.clone());
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response.context_menu(|ui| {
            if ui.button("🎮 Play Clone").clicked() {
                play_requested = Some(clone_idx);
                ui.close();
            }
            ui.separator();
            if ui.button("⚙️ Properties...").clicked() {
                properties_requested = Some(clone_idx);
                ui.close();
            }
        });

        (play_requested, favorite_toggled, properties_requested)
    }

    fn compact_card_style(ui: &mut Ui) {
        ui.style_mut().interaction.selectable_labels = false;
        ui.spacing_mut().item_spacing = Vec2::new(6.0, 4.0);
        ui.spacing_mut().button_padding = Vec2::new(3.0, 2.0);
        ui.spacing_mut().interact_size.y = 20.0;
    }

    fn render_preview(
        ui: &mut Ui,
        rect: egui::Rect,
        texture: Option<&egui::TextureHandle>,
        show_icons: bool,
        placeholder: &str,
    ) {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
        if let Some(texture) = texture.filter(|_| show_icons) {
            ui.put(
                rect.shrink(4.0),
                egui::Image::new(texture)
                    .fit_to_exact_size(Vec2::new(56.0, 40.0))
                    .corner_radius(4.0),
            );
        } else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                placeholder,
                FontId::proportional(24.0),
                ui.visuals().strong_text_color(),
            );
        }
    }

    fn render_card_text(ui: &mut Ui, rect: egui::Rect, text: RichText, full_text: &str) {
        if rect.width() <= 0.0 {
            return;
        }
        let mut text_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        text_ui.set_clip_rect(ui.clip_rect().intersect(rect));
        text_ui
            .add(
                egui::Label::new(text)
                    .truncate()
                    .halign(egui::Align::Min)
                    .show_tooltip_when_elided(false),
            )
            .on_hover_text(full_text);
    }

    fn render_favorite(
        ui: &mut Ui,
        rect: egui::Rect,
        is_favorite: bool,
        colors: &crate::models::GameListColors,
    ) -> egui::Response {
        ui.put(
            rect,
            egui::Button::new(
                RichText::new(if is_favorite { "★" } else { "☆" })
                    .color(if is_favorite {
                        colors.favorite_active
                    } else {
                        colors.favorite_inactive
                    })
                    .size(18.0),
            )
            .fill(Color32::TRANSPARENT)
            .stroke(egui::Stroke::NONE),
        )
    }

    fn render_badge(ui: &mut Ui, rect: egui::Rect, text: &str, size: f32, fill: Color32) {
        // Fixed bounds and modest padding keep pills readable at dialog-scale
        // text sizes, without inheriting the large default button padding.
        ui.painter().rect_filled(rect, 4.0, fill);
        ui.put(
            rect.shrink2(Vec2::new(4.0, 1.0)),
            egui::Label::new(
                RichText::new(text)
                    .size(size)
                    .color(ui.visuals().strong_text_color())
                    .strong(),
            )
            .truncate(),
        );
    }

    fn render_status_badge(
        &self,
        ui: &mut Ui,
        rect: egui::Rect,
        game: &Game,
        colors: &crate::models::GameListColors,
    ) {
        let (text, bg_color) = match game.driver_status.as_str() {
            "good" => ("WORKING", colors.status_available.gamma_multiply(0.2)),
            "imperfect" => ("ISSUES", ui.visuals().warn_fg_color.gamma_multiply(0.2)),
            _ => ("NOT WORKING", colors.status_missing.gamma_multiply(0.2)),
        };
        Self::render_badge(ui, rect, text, 11.0, bg_color);
    }

    fn render_clone_badge(
        &self,
        ui: &mut Ui,
        rect: egui::Rect,
        count: usize,
        colors: &crate::models::GameListColors,
    ) {
        Self::render_badge(
            ui,
            rect,
            &format!("{} versions", count),
            12.0,
            colors.row_bg_selected,
        );
    }

    /// Update cache with filtered games
    fn update_cache(
        &mut self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        _expanded_parents: &HashMap<String, bool>,
        game_index: Option<&GameIndex>,
        category: FilterCategory,
        hardware_filter: Option<&HardwareFilter>, // Placeholder for hardware filter
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

        // Re-apply manufacturer filter (pre_filtered may lag one frame; also covers cache misses)
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

        self.cache_valid = true;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > 500 {
            println!(
                "Warning: Cache update took {}ms for {} games",
                elapsed.as_millis(),
                self.filtered_indices_cache.len()
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

    /// Fast filtering using GameIndex with new multi-selection filters
    fn filter_with_index(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        index: &GameIndex,
        _category: FilterCategory,
        _hardware_filter: Option<&HardwareFilter>, // Placeholder for hardware filter
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

            // Use parallel search for large datasets
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

    /// Manual filtering fallback (without GameIndex) with new multi-selection filters
    fn filter_manual(
        &self,
        games: &[Game],
        filters: &FilterSettings,
        favorites: &HashSet<String>,
        _category: FilterCategory,
        _hardware_filter: Option<&HardwareFilter>, // Placeholder for hardware filter
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
        filters.hide_romless_systems.hash(&mut hasher);

        // Hash AVAILABILITY filters
        filters
            .availability_filters
            .show_available
            .hash(&mut hasher);
        filters
            .availability_filters
            .show_unavailable
            .hash(&mut hasher);

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

        filters.cpu_filter.hash(&mut hasher);
        filters.device_filter.hash(&mut hasher);
        filters.sound_filter.hash(&mut hasher);
        let mut manufacturers: Vec<_> = filters.selected_manufacturers.iter().collect();
        manufacturers.sort();
        for m in manufacturers {
            m.hash(&mut hasher);
        }

        hasher.finish()
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    fn game(name: &str, description: &str) -> Game {
        Game {
            name: name.into(),
            description: description.into(),
            manufacturer: "A manufacturer with a very long company name".into(),
            year: "1999".into(),
            driver: "test.cpp".into(),
            driver_status: "preliminary".into(),
            status: RomStatus::Available,
            parent: None,
            category: "A long category that must not push the favorite button away".into(),
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

    fn text_shapes(shapes: &[egui::epaint::ClippedShape]) -> Vec<&egui::epaint::TextShape> {
        fn collect<'a>(shape: &'a egui::Shape, texts: &mut Vec<&'a egui::epaint::TextShape>) {
            match shape {
                egui::Shape::Text(text) => texts.push(text),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, texts);
                    }
                }
                _ => {}
            }
        }
        let mut texts = Vec::new();
        for shape in shapes {
            collect(&shape.shape, &mut texts);
        }
        texts
    }

    #[test]
    fn narrow_cards_reserve_actions_and_truncate_titles_with_dialog_spacing() {
        let context = egui::Context::default();
        crate::ui::components::steam_ui::SteamUi::apply(&context);
        let parent = game(
            "parent",
            "A very long parent game title that would previously cover the status and favorite controls",
        );
        let mut clone = game(
            "clone",
            "A very long clone title that would previously extend beyond the edge of the list panel",
        );
        clone.is_clone = true;
        clone.parent = Some(parent.name.clone());
        let games = vec![parent, clone];
        let index = GameIndex::build(games.clone(), HashSet::new());
        let colors = crate::models::GameListColors::default();
        let mut list = GameListView::new();
        let mut card_rects = Vec::new();
        let output = context.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // CentralPanel preallocates its entire min_rect. Use a
                    // bounded child so both width and measured content height
                    // describe the cards rather than the surrounding panel.
                    let mut card_ui =
                        ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                            ui.next_widget_position(),
                            Vec2::new(480.0, 250.0),
                        )));
                    let ui = &mut card_ui;
                    let origin = ui.next_widget_position();
                    card_rects.push(egui::Rect::from_min_size(
                        origin,
                        Vec2::new(480.0, GAME_CARD_HEIGHT),
                    ));
                    list.render_game_item(
                        ui,
                        &games[0],
                        0,
                        &games,
                        &FilterSettings::default(),
                        &mut None,
                        &mut HashMap::new(),
                        &HashSet::new(),
                        &HashMap::new(),
                        false,
                        Some(&index),
                        None,
                        &HashMap::new(),
                        &colors,
                    );
                    assert!(
                        (ui.min_rect().bottom() - origin.y - GAME_CARD_HEIGHT).abs() < 0.1,
                        "origin={origin:?}; panel_min={:?}; cursor={:?}",
                        ui.min_rect(),
                        ui.next_widget_position()
                    );
                    let origin = ui.next_widget_position();
                    card_rects.push(egui::Rect::from_min_size(
                        origin + Vec2::new(32.0, 0.0),
                        Vec2::new(448.0, CLONE_CARD_HEIGHT),
                    ));
                    list.render_clone_item(
                        ui,
                        &games[1],
                        1,
                        &mut None,
                        &HashSet::new(),
                        &HashMap::new(),
                        false,
                        None,
                        &colors,
                    );
                    assert!((ui.min_rect().bottom() - origin.y - CLONE_CARD_HEIGHT).abs() < 0.1);
                });
            },
        );
        let texts = text_shapes(&output.shapes);
        let parent_title = texts
            .iter()
            .find(|text| text.galley.job.text == games[0].description)
            .unwrap();
        let clone_title = texts
            .iter()
            .find(|text| text.galley.job.text == games[1].description)
            .unwrap();
        let status = texts
            .iter()
            .find(|text| text.galley.job.text == "NOT WORKING")
            .unwrap();
        let versions = texts
            .iter()
            .find(|text| text.galley.job.text == "1 versions")
            .unwrap();
        assert!(parent_title.galley.elided && clone_title.galley.elided);
        assert!(!status.galley.elided && !versions.galley.elided);
        assert!(parent_title.visual_bounding_rect().right() < status.visual_bounding_rect().left());
        assert!(
            parent_title.visual_bounding_rect().right() < versions.visual_bounding_rect().left()
        );
        let stars: Vec<_> = texts
            .iter()
            .filter(|text| text.galley.job.text == "☆")
            .collect();
        assert_eq!(stars.len(), 2);
        assert!(
            clone_title.visual_bounding_rect().right() < stars[1].visual_bounding_rect().left()
        );
        for text in texts {
            assert!(
                card_rects
                    .iter()
                    .any(|rect| rect.expand(1.0).contains_rect(text.visual_bounding_rect())),
                "Text outside cards: {:?}",
                text.galley.job.text
            );
        }
    }
}
