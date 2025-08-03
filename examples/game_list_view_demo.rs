// examples/game_list_view_demo.rs
// Demo aplikasi untuk menunjukkan GameListView yang mirip dengan mockup

use eframe::egui;
use egui::Color32;
// Import using crate path for examples
use crate::models::{Game, RomStatus, GameIndex};
use crate::ui::panels::GameListView;

// Add necessary module declarations
#[path = "../src/models/mod.rs"]
mod models;

#[path = "../src/ui/mod.rs"]
mod ui;
use std::collections::{HashMap, HashSet};

/// Demo app untuk GameListView
pub struct GameListViewDemo {
    games: Vec<Game>,
    game_index: GameIndex,
    list_view: GameListView,
    selected: Option<usize>,
    expanded_parents: HashMap<String, bool>,
    favorites: HashSet<String>,
    icons: HashMap<String, egui::TextureHandle>,
    show_icons: bool,
}

impl Default for GameListViewDemo {
    fn default() -> Self {
        // Create sample games
        let mut games = vec![
            Game {
                name: "sf2".to_string(),
                description: "Street Fighter II".to_string(),
                manufacturer: "Capcom".to_string(),
                year: "1991".to_string(),
                is_clone: false,
                parent: None,
                driver: "cps1".to_string(),
                driver_status: "good".to_string(),
                category: "Fighting".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "sf2ce".to_string(),
                description: "Street Fighter II - Champion Edition".to_string(),
                manufacturer: "Capcom".to_string(),
                year: "1992".to_string(),
                is_clone: true,
                parent: Some("sf2".to_string()),
                driver: "cps1".to_string(),
                driver_status: "good".to_string(),
                category: "Fighting".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "sf2hf".to_string(),
                description: "Street Fighter II - Hyper Fighting".to_string(),
                manufacturer: "Capcom".to_string(),
                year: "1992".to_string(),
                is_clone: true,
                parent: Some("sf2".to_string()),
                driver: "cps1".to_string(),
                driver_status: "good".to_string(),
                category: "Fighting".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "pacman".to_string(),
                description: "Pac-Man".to_string(),
                manufacturer: "Namco".to_string(),
                year: "1980".to_string(),
                is_clone: false,
                parent: None,
                driver: "pacman".to_string(),
                driver_status: "good".to_string(),
                category: "Maze".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "mslug".to_string(),
                description: "Metal Slug".to_string(),
                manufacturer: "SNK".to_string(),
                year: "1996".to_string(),
                is_clone: false,
                parent: None,
                driver: "neogeo".to_string(),
                driver_status: "imperfect".to_string(),
                category: "Platform/Shooter".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "mslug2".to_string(),
                description: "Metal Slug 2".to_string(),
                manufacturer: "SNK".to_string(),
                year: "1998".to_string(),
                is_clone: true,
                parent: Some("mslug".to_string()),
                driver: "neogeo".to_string(),
                driver_status: "imperfect".to_string(),
                category: "Platform/Shooter".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "mk".to_string(),
                description: "Mortal Kombat".to_string(),
                manufacturer: "Midway".to_string(),
                year: "1992".to_string(),
                is_clone: false,
                parent: None,
                driver: "midyunit".to_string(),
                driver_status: "good".to_string(),
                category: "Fighting".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "outrun".to_string(),
                description: "Out Run".to_string(),
                manufacturer: "Sega".to_string(),
                year: "1986".to_string(),
                is_clone: false,
                parent: None,
                driver: "outrun".to_string(),
                driver_status: "good".to_string(),
                category: "Racing".to_string(),
                status: RomStatus::Missing,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
            Game {
                name: "galaga".to_string(),
                description: "Galaga".to_string(),
                manufacturer: "Namco".to_string(),
                year: "1981".to_string(),
                is_clone: false,
                parent: None,
                driver: "galaga".to_string(),
                driver_status: "preliminary".to_string(),
                category: "Shooter".to_string(),
                status: RomStatus::Available,
                requires_chd: false,
                chd_name: None,
                verification_status: None,
            },
        ];

        // Build game index
        let mut game_index = GameIndex::new();
        for (idx, game) in games.iter().enumerate() {
            game_index.add_game(game.clone(), idx);
        }

        // Add some favorites
        let mut favorites = HashSet::new();
        favorites.insert("sf2".to_string());
        favorites.insert("pacman".to_string());

        Self {
            games,
            game_index,
            list_view: GameListView::new(),
            selected: None,
            expanded_parents: HashMap::new(),
            favorites,
            icons: HashMap::new(),
            show_icons: true,
        }
    }
}

impl eframe::App for GameListViewDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set dark theme
        let visuals = egui::Visuals::dark();
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(15, 15, 15)))
            .show(ctx, |ui| {
                // Header
                egui::Frame::new()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.heading(
                                egui::RichText::new("KMameUI - List View Demo")
                                    .size(24.0)
                                    .color(Color32::from_rgb(224, 224, 224))
                            );
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new("Modern MAME Frontend")
                                        .size(14.0)
                                        .color(Color32::from_rgb(136, 136, 136))
                                );
                            });
                        });
                    });
                
                // Main content area
                egui::Frame::none()
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        // Show the list view
                        let (double_clicked, favorite_toggled) = self.list_view.show(
                            ui,
                            &self.games,
                            &mut self.selected,
                            &mut self.expanded_parents,
                            &self.favorites,
                            &self.icons,
                            self.show_icons,
                            Some(&self.game_index),
                            None, // No default icon for demo
                        );

                        // Handle double click
                        if double_clicked {
                            if let Some(selected_idx) = self.selected {
                                if let Some(game) = self.games.get(selected_idx) {
                                    println!("Launching game: {}", game.description);
                                }
                            }
                        }

                        // Handle favorite toggle
                        if let Some(game_name) = favorite_toggled {
                            if self.favorites.contains(&game_name) {
                                self.favorites.remove(&game_name);
                                println!("Removed {} from favorites", game_name);
                            } else {
                                self.favorites.insert(game_name.clone());
                                println!("Added {} to favorites", game_name);
                            }
                        }
                    });

                // Status bar
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 26))
                    .inner_margin(egui::vec2(20.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Total games: {}", self.games.len()))
                                    .size(12.0)
                                    .color(Color32::from_rgb(136, 136, 136))
                            );
                            
                            ui.separator();
                            
                            if let Some(selected_idx) = self.selected {
                                if let Some(game) = self.games.get(selected_idx) {
                                    ui.label(
                                        egui::RichText::new(format!("Selected: {}", game.description))
                                            .size(12.0)
                                            .color(Color32::from_rgb(136, 136, 136))
                                    );
                                }
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!("Favorites: {}", self.favorites.len()))
                                        .size(12.0)
                                        .color(Color32::from_rgb(255, 200, 50))
                                );
                            });
                        });
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
        Box::new(|_cc| {
            Ok(Box::new(GameListViewDemo::default()))
        }),
    )
}