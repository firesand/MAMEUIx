use eframe::egui;
use crate::models::AppConfig;
use std::path::PathBuf;

pub struct DirectoriesDialog;

impl DirectoriesDialog {
    pub fn show(ctx: &egui::Context, config: &mut AppConfig, open: &mut bool) {
        let mut close = false;

        egui::Window::new("Directories Selection")
        .default_size([600.0, 400.0])
        .open(open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_label(true, "MAME Main Paths");
                ui.selectable_label(false, "MAME Paths");
                ui.selectable_label(false, "MAME Support Files");
                ui.selectable_label(false, "User resources");
            });

            ui.separator();

            ui.group(|ui| {
                ui.label("MAME Executables");
                Self::path_list(ui, &mut config.mame_executables, "mame_exe");
            });

            ui.group(|ui| {
                ui.label("ROMs");
                Self::path_list(ui, &mut config.rom_paths, "roms");
            });

            ui.group(|ui| {
                ui.label("Samples");
                Self::path_list(ui, &mut config.sample_paths, "samples");
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    close = true;
                }
                if ui.button("OK").clicked() {
                    close = true;
                }
            });
        });

        if close {
            *open = false;
        }
    }

    fn path_list(ui: &mut egui::Ui, paths: &mut Vec<PathBuf>, _id: &str) {
        egui::ScrollArea::vertical()
        .max_height(100.0)
        .show(ui, |ui| {
            let mut to_remove = None;

            for (idx, path) in paths.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(path.display().to_string());
                    if ui.small_button("Remove").clicked() {
                        to_remove = Some(idx);
                    }
                });
            }

            if let Some(idx) = to_remove {
                paths.remove(idx);
            }
        });

        if ui.button("➕ Add").clicked() {
            paths.push(PathBuf::from("/path/to/directory"));
        }
    }
}
