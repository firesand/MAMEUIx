use eframe::egui;
use crate::models::Preferences;

pub struct PreferencesDialog;

impl PreferencesDialog {
    pub fn show(ctx: &egui::Context, prefs: &mut Preferences, open: &mut bool) {
        let mut close = false;

        egui::Window::new("GMAMEUI Preferences")
        .default_size([400.0, 500.0])
        .open(open)
        .show(ctx, |ui| {
            ui.heading("Startup Options");
            ui.checkbox(&mut prefs.search_new_games, "Search for new games");
            ui.checkbox(&mut prefs.version_mismatch_warning, "Enable version mismatch warning");
            ui.checkbox(&mut prefs.use_mame_defaults, "Use MAME default options");
            ui.checkbox(&mut prefs.joystick_selection, "Allow game selection with a joystick");

            ui.separator();
            ui.heading("Visible Columns");

            ui.columns(2, |columns| {
                columns[0].checkbox(&mut prefs.visible_columns.game_name, "Game Name");
                columns[0].checkbox(&mut prefs.visible_columns.directory, "Directory");
                columns[0].checkbox(&mut prefs.visible_columns.play_count, "Playcount");
                columns[0].checkbox(&mut prefs.visible_columns.manufacturer, "Manufacturer");
                columns[0].checkbox(&mut prefs.visible_columns.year, "Year");

                columns[1].checkbox(&mut prefs.visible_columns.driver, "Driver");
                columns[1].checkbox(&mut prefs.visible_columns.clone_of, "Clone of");
                columns[1].checkbox(&mut prefs.visible_columns.version, "Version");
                columns[1].checkbox(&mut prefs.visible_columns.category, "Category");
            });

            ui.separator();
            ui.heading("Miscellaneous Options");

            ui.horizontal(|ui| {
                ui.label("Clone Colour:");
                ui.color_edit_button_rgb(&mut prefs.clone_color);
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        });

        if close {
            *open = false;
        }
    }
}
