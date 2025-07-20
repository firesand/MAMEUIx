use eframe::egui;
use crate::models::Game;

pub struct RomInfoDialog;

impl RomInfoDialog {
    pub fn show(ctx: &egui::Context, game: &Game, open: &mut bool) {
        let mut close = false;

        egui::Window::new(&game.description)
        .default_size([350.0, 400.0])
        .open(open)
        .show(ctx, |ui| {
            ui.heading("About this ROM");

            ui.horizontal(|ui| {
                ui.label("Year:");
                ui.label(&game.year);
            });

            ui.horizontal(|ui| {
                ui.label("Manufacturer:");
                ui.label(&game.manufacturer);
            });

            ui.separator();
            ui.heading("Technical Details");

            ui.horizontal(|ui| {
                ui.label("CPU:");
                ui.label("68000 12.000000 MHz\nZ80 4.000000 MHz");
            });

            ui.horizontal(|ui| {
                ui.label("Sound:");
                ui.label("YM2610 8.000000 MHz");
            });

            ui.separator();
            ui.heading("Screen Details");

            ui.horizontal(|ui| {
                ui.label("Resolution:");
                ui.label("224 × 320 Vertical 59.00 Hz");
            });

            ui.horizontal(|ui| {
                ui.label("Colors:");
                ui.label("0 colors");
            });

            ui.separator();
            ui.heading("Audit Details");

            ui.horizontal(|ui| {
                ui.label("Rom check:");
                ui.label("Passed");
            });

            ui.horizontal(|ui| {
                ui.label("Sample check:");
                ui.label("None required");
            });

            ui.group(|ui| {
                ui.label("Details");
                ui.label("1 romsets found, 1 were OK.");
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }
            });
        });

        if close {
            *open = false;
        }
    }
}
