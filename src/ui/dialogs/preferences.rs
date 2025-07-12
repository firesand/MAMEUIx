// src/ui/dialogs/preferences.rs
use eframe::egui;
use crate::models::Preferences;

pub struct PreferencesDialog;

impl PreferencesDialog {
    /// Display the preferences dialog
    /// Note: This now takes a Preferences struct, not the entire AppConfig
    /// The calling code needs to pass config.preferences or similar
    pub fn show(ctx: &egui::Context, prefs: &mut Preferences, open: &mut bool) {
        let mut close = false;

        egui::Window::new("Preferences")
        .default_size([500.0, 600.0])
        .open(open)
        .show(ctx, |ui| {
            // General preferences section
            ui.group(|ui| {
                ui.label("General Settings");
                ui.checkbox(&mut prefs.search_new_games, "Search for new games at startup");
                ui.checkbox(&mut prefs.version_mismatch_warning, "Enable version mismatch warning");
                ui.checkbox(&mut prefs.use_mame_defaults, "Use MAME default options");
                ui.checkbox(&mut prefs.joystick_selection, "Allow game selection with a joystick");
                ui.checkbox(&mut prefs.auto_save, "Auto-save configuration changes");
                ui.checkbox(&mut prefs.confirm_exit, "Confirm before exiting");
            });

            ui.add_space(10.0);

            // Visible columns configuration
            ui.group(|ui| {
                ui.label("Visible Columns in Game List");
                ui.label("Choose which columns to display:");

                // Create two columns for better layout
                ui.columns(2, |columns| {
                    // First column
                    columns[0].checkbox(&mut prefs.visible_columns.game_name, "Game Name");
                    columns[0].checkbox(&mut prefs.visible_columns.play_count, "Play Count");
                    columns[0].checkbox(&mut prefs.visible_columns.manufacturer, "Manufacturer");
                    columns[0].checkbox(&mut prefs.visible_columns.year, "Year");

                    // Second column
                    columns[1].checkbox(&mut prefs.visible_columns.driver, "Driver");
                    columns[1].checkbox(&mut prefs.visible_columns.category, "Category");
                    columns[1].checkbox(&mut prefs.visible_columns.rom, "ROM");
                });
            });

            ui.add_space(10.0);

            // Display preferences
            ui.group(|ui| {
                ui.label("Display Settings");

                ui.horizontal(|ui| {
                    ui.label("Clone color:");
                    ui.color_edit_button_rgb(&mut prefs.clone_color);
                });

                ui.checkbox(&mut prefs.show_fps, "Show FPS counter");
                ui.checkbox(&mut prefs.fullscreen, "Start in fullscreen mode");
                ui.checkbox(&mut prefs.vsync, "Enable V-Sync");

                ui.horizontal(|ui| {
                    ui.label("Window Width:");
                    ui.add(egui::DragValue::new(&mut prefs.window_width)
                    .speed(10.0)
                    .range(640.0..=3840.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Window Height:");
                    ui.add(egui::DragValue::new(&mut prefs.window_height)
                    .speed(10.0)
                    .range(480.0..=2160.0));
                });
            });

            ui.add_space(10.0);

            // Language selection
            ui.horizontal(|ui| {
                ui.label("Language:");
                egui::ComboBox::from_label("")
                .selected_text(&prefs.language)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut prefs.language, "English".to_string(), "English");
                    ui.selectable_value(&mut prefs.language, "Spanish".to_string(), "Spanish");
                    ui.selectable_value(&mut prefs.language, "French".to_string(), "French");
                    ui.selectable_value(&mut prefs.language, "German".to_string(), "German");
                    ui.selectable_value(&mut prefs.language, "Italian".to_string(), "Italian");
                    ui.selectable_value(&mut prefs.language, "Japanese".to_string(), "Japanese");
                });
            });

            ui.separator();

            // Dialog buttons
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    close = true;
                    // In a real app, you'd save preferences here
                }
                if ui.button("Cancel").clicked() {
                    close = true;
                    // In a real app, you might revert changes here
                }
                if ui.button("Apply").clicked() {
                    // Apply changes without closing
                }
            });
        });

        if close {
            *open = false;
        }
    }
}
