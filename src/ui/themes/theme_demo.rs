// src/ui/theme_demo.rs
// Demo untuk menampilkan warna-warna UI yang diminta

use crate::ui::themes::theme::{ThemeHelper, UiTheme};
use eframe::egui;

pub struct ThemeDemo {
    theme: UiTheme,
}

impl ThemeDemo {
    pub fn new() -> Self {
        Self {
            theme: UiTheme::default(),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("🎨 MAMEUIx Color Theme Demo");
        ui.add_space(20.0);

        // Folder icons - kuning
        ui.label("📁 Folder Icons (Yellow):");
        ui.horizontal(|ui| {
            ThemeHelper::folder_icon(ui, &self.theme);
            ui.label("Categories");
        });
        ui.horizontal(|ui| {
            ThemeHelper::folder_icon(ui, &self.theme);
            ui.label("ROM Directories");
        });
        ui.add_space(10.0);

        // Hidden category symbols - merah
        ui.label("🔒 Hidden Category Symbols (Red):");
        ui.horizontal(|ui| {
            ThemeHelper::hidden_category_symbol(ui, &self.theme);
            ui.label("Hidden Category");
        });
        ui.horizontal(|ui| {
            ThemeHelper::hidden_category_symbol(ui, &self.theme);
            ui.label("Locked Category");
        });
        ui.add_space(10.0);

        // Play game buttons - hijau dengan gamepad merah
        ui.label("🎮 Play Game Buttons (Green with Red Gamepad):");
        ui.horizontal(|ui| {
            let play_button = egui::Button::new("🎮 Play Game").fill(self.theme.play_button_color);
            if ui.add(play_button).clicked() {
                // Demo action
            }
            ThemeHelper::gamepad_icon(ui, &self.theme);
        });
        ui.horizontal(|ui| {
            let play_button =
                egui::Button::new("🎮 Launch Game").fill(self.theme.play_button_color);
            if ui.add(play_button).clicked() {
                // Demo action
            }
            ThemeHelper::gamepad_icon(ui, &self.theme);
        });
        ui.add_space(10.0);

        // Properties icons - kuning
        ui.label("⚙️ Properties Icons (Yellow):");
        ui.horizontal(|ui| {
            ThemeHelper::properties_icon(ui, &self.theme);
            ui.label("Game Properties");
        });
        ui.horizontal(|ui| {
            ThemeHelper::warning_icon(ui, &self.theme);
            ui.label("Warning Messages");
        });
        ui.horizontal(|ui| {
            ThemeHelper::info_icon(ui, &self.theme);
            ui.label("Information");
        });
        ui.add_space(10.0);

        // Refresh icons - biru
        ui.label("🔄 Refresh Icons (Blue):");
        ui.horizontal(|ui| {
            let refresh_button =
                egui::Button::new("🔄 Refresh").fill(self.theme.refresh_button_color);
            if ui.add(refresh_button).clicked() {
                // Demo action
            }
        });
        ui.horizontal(|ui| {
            ThemeHelper::refresh_icon(ui, &self.theme);
            ui.label("Refresh Game List");
        });
        ui.add_space(10.0);

        // Favorite stars - kuning
        ui.label("★ Favorite Stars (Yellow):");
        ui.horizontal(|ui| {
            ThemeHelper::favorite_star(ui, &self.theme, true);
            ui.label("Favorite Game (Active)");
        });
        ui.horizontal(|ui| {
            ThemeHelper::favorite_star(ui, &self.theme, false);
            ui.label("Favorite Game (Inactive)");
        });
        ui.add_space(10.0);

        // Status colors
        ui.label("📊 Game Status Colors:");
        ui.horizontal(|ui| {
            ThemeHelper::status_text(ui, &self.theme, &crate::models::RomStatus::Available);
            ui.label(" - Available games");
        });
        ui.horizontal(|ui| {
            ThemeHelper::status_text(ui, &self.theme, &crate::models::RomStatus::NotWorking);
            ui.label(" - Not working games");
        });
        ui.horizontal(|ui| {
            ThemeHelper::status_text(ui, &self.theme, &crate::models::RomStatus::Missing);
            ui.label(" - Missing ROMs");
        });
        ui.horizontal(|ui| {
            ThemeHelper::status_text(ui, &self.theme, &crate::models::RomStatus::Available);
            ui.label(" - Available games");
        });
        ui.add_space(10.0);

        // Category text colors
        ui.label("📂 Category Text Colors:");
        ui.horizontal(|ui| {
            ThemeHelper::category_text(ui, &self.theme, "Action Games", false);
            ui.label(" - Normal category");
        });
        ui.horizontal(|ui| {
            ThemeHelper::category_text(ui, &self.theme, "Hidden Category", true);
            ui.label(" - Hidden category (red)");
        });
        ui.add_space(10.0);

        // Directory text colors
        ui.label("📁 Directory Text Colors (Yellow):");
        ui.horizontal(|ui| {
            ThemeHelper::directory_text(ui, &self.theme, "/home/user/roms");
        });
        ui.horizontal(|ui| {
            ThemeHelper::directory_text(ui, &self.theme, "/usr/local/share/mame/roms");
        });
        ui.add_space(10.0);

        // Filter active indicator
        ui.label("🔍 Filter Active Indicator (Blue):");
        ui.horizontal(|ui| {
            ThemeHelper::filter_active_indicator(ui, &self.theme, "Filter: Action Games");
        });
        ui.horizontal(|ui| {
            ThemeHelper::filter_active_indicator(ui, &self.theme, "Search: pacman");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Color palette display
        ui.heading("🎨 Color Palette");
        ui.label("These are the exact colors used in the theme:");

        let colors = [
            ("Folder Icon (Yellow)", self.theme.folder_icon_color),
            ("Hidden Category (Red)", self.theme.hidden_category_color),
            ("Play Button (Green)", self.theme.play_button_color),
            ("Gamepad Icon (Red)", self.theme.gamepad_icon_color),
            ("Properties Icon (Yellow)", self.theme.properties_icon_color),
            ("Refresh Icon (Blue)", self.theme.refresh_icon_color),
            ("Favorite Star (Gold)", self.theme.favorite_star_color),
            ("Working Status (Green)", self.theme.working_status_color),
            (
                "Not Working Status (Red)",
                self.theme.not_working_status_color,
            ),
            ("Missing Status (Yellow)", self.theme.missing_status_color),
        ];

        for (name, color) in colors {
            ui.horizontal(|ui| {
                ui.colored_label(color, "■");
                ui.label(name);
                ui.label(format!("RGB({}, {}, {})", color.r(), color.g(), color.b()));
            });
        }
    }
}
