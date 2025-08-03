// examples/game_list_view_demo_standalone.rs
// Standalone demo untuk GameListView - tidak memerlukan import dari crate utama

use eframe::egui;
use egui::{Color32, FontId, RichText, Vec2, Pos2, Rect, Response, Sense, Ui, FontFamily, TextStyle};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

// Simplified Game struct for demo
#[derive(Clone)]
struct Game {
    name: String,
    description: String,
    manufacturer: String,
    year: String,
    driver_status: String,
    is_clone: bool,
    parent: Option<String>,
    category: String,
}

// Simplified GameIndex for demo
struct GameIndex {
    parent_to_clones: HashMap<String, Vec<usize>>,
}

impl GameIndex {
    fn new() -> Self {
        Self {
            parent_to_clones: HashMap::new(),
        }
    }

    fn add_game(&mut self, game: &Game, idx: usize) {
        if let Some(parent) = &game.parent {
            self.parent_to_clones.entry(parent.clone()).or_insert_with(Vec::new).push(idx);
        }
    }

    fn get_clones(&self, parent: &str) -> Vec<usize> {
        self.parent_to_clones.get(parent).cloned().unwrap_or_default()
    }
}

// Copy of AnimationState from game_list_view.rs
struct AnimationState {
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
        let t = 1.0 - (1.0 - t).powi(3);
        self.from + (self.to - self.from) * t
    }

    fn is_finished(&self) -> bool {
        self.start_time.elapsed().as_secs_f32() >= self.duration
    }
}

// Simplified ListViewState
struct ListViewState {
    expanded_items: HashMap<String, bool>,
    expansion_animations: HashMap<String, AnimationState>,
    hovered_item: Option<String>,
    selected_item: Option<String>,
    search_query: String,
}

impl Default for ListViewState {
    fn default() -> Self {
        Self {
            expanded_items: HashMap::new(),
            expansion_animations: HashMap::new(),
            hovered_item: None,
            selected_item: None,
            search_query: String::new(),
        }
    }
}

// Simplified GameListView
struct GameListView {
    state: ListViewState,
    filtered_indices: Vec<usize>,
}

impl GameListView {
    fn new() -> Self {
        Self {
            state: ListViewState::default(),
            filtered_indices: Vec::new(),
        }
    }

    fn show(
        &mut self,
        ui: &mut Ui,
        games: &[Game],
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        game_index: &GameIndex,
    ) -> (bool, Option<String>) {
        let mut double_clicked = false;
        let mut favorite_toggled = None;

        // Apply dark theme
        ui.style_mut().visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(15, 15, 15);
        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 26, 26);
        ui.style_mut().visuals.widgets.hovered.bg_fill = Color32::from_rgb(34, 34, 34);

        // Update filtered indices
        self.update_filtered_indices(games, &self.state.search_query.clone());

        // Search bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍").size(16.0));
            ui.add(
                egui::TextEdit::singleline(&mut self.state.search_query)
                    .desired_width(ui.available_width() - 40.0)
                    .hint_text("Search games...")
            );
        });

        ui.separator();
        ui.add_space(8.0);

        // Games count
        ui.label(
            RichText::new(format!("Showing {} games", self.filtered_indices.len()))
                .size(12.0)
                .color(Color32::from_rgb(136, 136, 136))
        );

        // Clean up animations
        self.state.expansion_animations.retain(|_, anim| !anim.is_finished());
        if !self.state.expansion_animations.is_empty() {
            ui.ctx().request_repaint();
        }

        // Main scroll area
        egui::ScrollArea::vertical().show(ui, |ui| {
            for &game_idx in &self.filtered_indices.clone() {
                if let Some(game) = games.get(game_idx) {
                    let (clicked, fav) = self.render_game_item(
                        ui,
                        game,
                        game_idx,
                        games,
                        selected,
                        expanded_parents,
                        favorites,
                        game_index,
                    );
                    if clicked {
                        double_clicked = true;
                    }
                    if let Some(name) = fav {
                        favorite_toggled = Some(name);
                    }
                }
            }
        });

        (double_clicked, favorite_toggled)
    }

    fn render_game_item(
        &mut self,
        ui: &mut Ui,
        game: &Game,
        game_idx: usize,
        games: &[Game],
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        game_index: &GameIndex,
    ) -> (bool, Option<String>) {
        let mut double_clicked = false;
        let mut favorite_toggled = None;

        let is_expanded = expanded_parents.get(&game.name).copied().unwrap_or(false);
        let is_hovered = self.state.hovered_item.as_ref() == Some(&game.name);
        let is_selected = selected.map_or(false, |s| s == game_idx);
        let is_favorite = favorites.contains(&game.name);

        let clone_count = game_index.get_clones(&game.name).len();
        let item_height = 72.0;

        let expansion_height = if let Some(anim) = self.state.expansion_animations.get(&game.name) {
            anim.value()
        } else if is_expanded && clone_count > 0 {
            (clone_count as f32 * 56.0) + 8.0
        } else {
            0.0
        };

        // Main item frame
        let frame = egui::Frame::new()
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
            .rounding(8.0);

        frame.show(ui, |ui| {
            // Header
            let header_response = ui.allocate_response(
                Vec2::new(ui.available_width(), item_height),
                Sense::click()
            );

            if header_response.clicked() {
                if clone_count > 0 && !game.is_clone {
                    let current = expanded_parents.get(&game.name).copied().unwrap_or(false);
                    expanded_parents.insert(game.name.clone(), !current);
                    
                    let target_height = (clone_count as f32 * 56.0) + 8.0;
                    if !current {
                        self.state.expansion_animations.insert(
                            game.name.clone(),
                            AnimationState::new(0.2, 0.0, target_height)
                        );
                    } else {
                        self.state.expansion_animations.insert(
                            game.name.clone(),
                            AnimationState::new(0.2, target_height, 0.0)
                        );
                    }
                } else {
                    *selected = Some(game_idx);
                }
            }

            if header_response.double_clicked() {
                double_clicked = true;
            }

            if header_response.hovered() {
                self.state.hovered_item = Some(game.name.clone());
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            // Draw header content
            ui.allocate_ui_at_rect(header_response.rect, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(16.0);

                    // Expand arrow
                    if clone_count > 0 && !game.is_clone {
                        let arrow = if is_expanded { "▼" } else { "▶" };
                        ui.label(RichText::new(arrow).size(12.0).color(Color32::from_rgb(102, 102, 102)));
                    } else {
                        ui.add_space(20.0);
                    }

                    // Game icon placeholder
                    let preview_rect = ui.allocate_space(Vec2::new(64.0, 48.0)).1;
                    ui.painter().rect_filled(preview_rect, 4.0, Color32::from_rgb(34, 34, 34));
                    ui.painter().text(
                        preview_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🎮",
                        FontId::proportional(24.0),
                        Color32::WHITE
                    );

                    ui.add_space(16.0);

                    // Game info
                    ui.vertical(|ui| {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&game.description)
                                    .size(16.0)
                                    .color(Color32::from_rgb(224, 224, 224))
                                    .strong()
                            );
                            
                            if clone_count > 0 && !game.is_clone {
                                ui.add_space(8.0);
                                egui::Frame::new()
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
                        
                        ui.label(
                            RichText::new(format!("{} • {} • {}", game.manufacturer, game.year, game.category))
                                .size(13.0)
                                .color(Color32::from_rgb(136, 136, 136))
                        );
                    });

                    // Right side
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(16.0);

                        // Removed play button - double click is enough

                        // Clone badge
                        if clone_count > 0 && !game.is_clone {
                            egui::Frame::new()
                                .fill(Color32::from_rgba_premultiplied(74, 158, 255, 51))
                                .rounding(6.0)
                                .inner_margin(12.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("{} versions", clone_count))
                                            .size(12.0)
                                            .color(Color32::from_rgb(255, 255, 255))
                                            .strong()
                                    );
                                });
                            ui.add_space(8.0);
                        }

                        // Status badge
                        let (text, bg_color, text_color) = match game.driver_status.as_str() {
                            "good" => ("WORKING", Color32::from_rgba_premultiplied(39, 201, 63, 51), Color32::from_rgb(255, 255, 255)),
                            "imperfect" => ("ISSUES", Color32::from_rgba_premultiplied(255, 189, 46, 51), Color32::from_rgb(255, 255, 255)),
                            _ => ("NOT WORKING", Color32::from_rgba_premultiplied(255, 95, 86, 51), Color32::from_rgb(255, 255, 255)),
                        };

                        egui::Frame::new()
                            .fill(bg_color)
                            .rounding(6.0)
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(text).size(11.0).color(text_color).strong());
                            });

                        ui.add_space(8.0);

                        // Favorite star
                        let star = if is_favorite { "★" } else { "☆" };
                        let star_color = if is_favorite {
                            Color32::from_rgb(255, 200, 50)
                        } else {
                            Color32::from_rgb(100, 100, 110)
                        };
                        
                        if ui.add(
                            egui::Button::new(RichText::new(star).color(star_color).size(18.0))
                                .fill(Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE)
                        ).clicked() {
                            favorite_toggled = Some(game.name.clone());
                        }
                    });
                });
            });

            // Clone expansion
            if expansion_height > 1.0 && clone_count > 0 {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                let clone_indices = game_index.get_clones(&game.name);
                for &clone_idx in &clone_indices {
                    if let Some(clone_game) = games.get(clone_idx) {
                        self.render_clone_item(ui, clone_game);
                    }
                }
            }
        });

        (double_clicked, favorite_toggled)
    }

    fn render_clone_item(&mut self, ui: &mut Ui, clone: &Game) {
        let is_hovered = self.state.hovered_item.as_ref() == Some(&clone.name);

        let response = ui.allocate_response(Vec2::new(ui.available_width(), 52.0), Sense::click());

        if response.hovered() {
            self.state.hovered_item = Some(clone.name.clone());
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        ui.allocate_ui_at_rect(response.rect, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(60.0);

                // Clone icon
                let icon_rect = ui.allocate_space(Vec2::new(32.0, 32.0)).1;
                ui.painter().rect_filled(icon_rect, 4.0, Color32::from_rgb(34, 34, 34));
                ui.painter().text(
                    icon_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎯",
                    FontId::proportional(18.0),
                    Color32::WHITE
                );

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(&clone.description)
                            .size(14.0)
                            .color(Color32::from_rgb(224, 224, 224))
                    );
                    ui.label(
                        RichText::new(format!("Clone • {} • {}", clone.year, clone.name))
                            .size(12.0)
                            .color(Color32::from_rgb(102, 102, 102))
                    );
                });

                // Removed play button for clones too
            });
        });
    }

    fn update_filtered_indices(&mut self, games: &[Game], query: &str) {
        self.filtered_indices.clear();
        
        if query.is_empty() {
            self.filtered_indices = (0..games.len()).collect();
        } else {
            let query_lower = query.to_lowercase();
            self.filtered_indices = games.iter()
                .enumerate()
                .filter(|(_, game)| {
                    game.description.to_lowercase().contains(&query_lower) ||
                    game.manufacturer.to_lowercase().contains(&query_lower) ||
                    game.name.to_lowercase().contains(&query_lower)
                })
                .map(|(idx, _)| idx)
                .collect();
        }
    }
}

// Demo app
struct GameListViewDemo {
    games: Vec<Game>,
    game_index: GameIndex,
    list_view: GameListView,
    selected: Option<usize>,
    expanded_parents: HashMap<String, bool>,
    favorites: HashSet<String>,
}

impl Default for GameListViewDemo {
    fn default() -> Self {
        let games = vec![
            Game {
                name: "sf2".to_string(),
                description: "Street Fighter II".to_string(),
                manufacturer: "Capcom".to_string(),
                year: "1991".to_string(),
                driver_status: "good".to_string(),
                is_clone: false,
                parent: None,
                category: "Fighting".to_string(),
            },
            Game {
                name: "sf2ce".to_string(),
                description: "Street Fighter II - Champion Edition".to_string(),
                manufacturer: "Capcom".to_string(),
                year: "1992".to_string(),
                driver_status: "good".to_string(),
                is_clone: true,
                parent: Some("sf2".to_string()),
                category: "Fighting".to_string(),
            },
            Game {
                name: "pacman".to_string(),
                description: "Pac-Man".to_string(),
                manufacturer: "Namco".to_string(),
                year: "1980".to_string(),
                driver_status: "good".to_string(),
                is_clone: false,
                parent: None,
                category: "Maze".to_string(),
            },
            Game {
                name: "mslug".to_string(),
                description: "Metal Slug".to_string(),
                manufacturer: "SNK".to_string(),
                year: "1996".to_string(),
                driver_status: "imperfect".to_string(),
                is_clone: false,
                parent: None,
                category: "Platform/Shooter".to_string(),
            },
        ];

        let mut game_index = GameIndex::new();
        for (idx, game) in games.iter().enumerate() {
            game_index.add_game(game, idx);
        }

        let mut favorites = HashSet::new();
        favorites.insert("sf2".to_string());

        Self {
            games,
            game_index,
            list_view: GameListView::new(),
            selected: None,
            expanded_parents: HashMap::new(),
            favorites,
        }
    }
}

impl eframe::App for GameListViewDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(15, 15, 15)))
            .show(ctx, |ui| {
                // Header
                egui::Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.heading(
                            RichText::new("KMameUI - List View Demo")
                                .size(24.0)
                                .color(Color32::from_rgb(224, 224, 224))
                        );
                    });
                
                // Main content
                egui::Frame::new()
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        let (double_clicked, favorite_toggled) = self.list_view.show(
                            ui,
                            &self.games,
                            &mut self.selected,
                            &mut self.expanded_parents,
                            &self.favorites,
                            &self.game_index,
                        );

                        if double_clicked {
                            if let Some(idx) = self.selected {
                                if let Some(game) = self.games.get(idx) {
                                    println!("Launching: {}", game.description);
                                }
                            }
                        }

                        if let Some(name) = favorite_toggled {
                            if self.favorites.contains(&name) {
                                self.favorites.remove(&name);
                            } else {
                                self.favorites.insert(name);
                            }
                        }
                    });
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "KMameUI List View Demo",
        options,
        Box::new(|_cc| Ok(Box::new(GameListViewDemo::default()))),
    )
}