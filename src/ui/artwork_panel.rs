use eframe::egui;
use crate::models::Game;
use std::path::PathBuf;

pub struct ArtworkPanel {
    current_texture: Option<egui::TextureHandle>,
}

impl ArtworkPanel {
    pub fn new() -> Self {
        Self {
            current_texture: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, selected_game: &Option<usize>, games: &[Game], asset_dirs: &[PathBuf]) {
        ui.heading("Game Details");
        ui.separator();

        if let Some(idx) = selected_game {
            if let Some(game) = games.get(*idx) {
                ui.label(&game.description);
                ui.label(format!("Year: {}", game.year));
                ui.label(format!("Manufacturer: {}", game.manufacturer));
                ui.label(format!("Status: {:?}", game.status));

                ui.separator();

                // Placeholder for artwork
                ui.label("Artwork would appear here");
            }
        } else {
            ui.label("Select a game to see details");
        }
    }
}
