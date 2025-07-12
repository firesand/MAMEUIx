// src/ui/artwork_panel.rs
use eframe::egui;
use crate::models::Game;
use std::path::PathBuf;
use super::artwork_loader::{ArtworkLoader, ArtworkType};

pub struct ArtworkPanel {
    // Any state the artwork panel needs can go here
    selected_artwork_type: ArtworkType,
    artwork_loader: ArtworkLoader,
}

impl ArtworkPanel {
    pub fn new() -> Self {
        Self {
            selected_artwork_type: ArtworkType::Screenshot,
            artwork_loader: ArtworkLoader::new(),
        }
    }

    /// Display the artwork panel
    /// This is now a method (takes &mut self) so it can be called on an instance
    /// The selected_game parameter is an Option<usize> representing the index
    pub fn show(
        &mut self,  // This makes it a method instead of an associated function
        ui: &mut egui::Ui,
        selected_game: &Option<usize>,  // Index into the games array
        games: &[Game],
        _asset_dirs: &[PathBuf],
        config: &crate::models::config::AppConfig,
    ) {
        ui.heading("Artwork");

        ui.separator();

        // Check if a game is selected
        if let Some(idx) = selected_game {
            // Use the index to get the game - this is the correct way
            if let Some(game) = games.get(*idx) {
                println!("ArtworkPanel: Selected game: {} ({})", game.name, game.description);
                
                // Display game information
                ui.label(format!("Game: {}", game.description));
                ui.label(format!("Year: {}", game.year));
                ui.label(format!("Manufacturer: {}", game.manufacturer));

                ui.separator();

                // Artwork type selection
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_artwork_type, ArtworkType::Screenshot, "Screenshot");
                    ui.selectable_value(&mut self.selected_artwork_type, ArtworkType::Cabinet, "Cabinet");
                    ui.selectable_value(&mut self.selected_artwork_type, ArtworkType::Marquee, "Marquee");
                    ui.selectable_value(&mut self.selected_artwork_type, ArtworkType::Title, "Title");
                    ui.selectable_value(&mut self.selected_artwork_type, ArtworkType::Flyer, "Flyer");
                });

                ui.separator();

                // Try to load and display artwork
                let artwork_frame = egui::Frame::dark_canvas(ui.style())
                    .inner_margin(egui::Vec2::splat(10.0));
                
                artwork_frame.show(ui, |ui| {
                    // Try to load the artwork
                    if let Some(texture) = self.artwork_loader.load_artwork(
                        ui.ctx(),
                        &game.name,
                        self.selected_artwork_type,
                        config,
                    ) {
                        // Calculate size to fit the available space while maintaining aspect ratio
                        let available_size = ui.available_size();
                        let texture_size = texture.size_vec2();
                        
                        let scale = (available_size.x / texture_size.x)
                            .min(available_size.y / texture_size.y)
                            .min(1.0); // Don't scale up beyond original size
                        
                        let display_size = texture_size * scale;
                        
                        // Center the image
                        ui.centered_and_justified(|ui| {
                            ui.add(egui::Image::new(&texture).fit_to_exact_size(display_size));
                        });
                    } else {
                        // No artwork available
                        ui.centered_and_justified(|ui| {
                            ui.label(format!("No {} available for {}",
                                             match self.selected_artwork_type {
                                                 ArtworkType::Screenshot => "screenshot",
                                                 ArtworkType::Cabinet => "cabinet art",
                                                 ArtworkType::Marquee => "marquee",
                                                 ArtworkType::Title => "title screen",
                                                 ArtworkType::Flyer => "flyer",
                                             },
                                             game.name
                            ));
                        });
                    }
                });
            } else {
                ui.label("Invalid game selection");
            }
        } else {
            // No game selected
            ui.centered_and_justified(|ui| {
                ui.label("Select a game to view artwork");
            });
        }
    }
}
