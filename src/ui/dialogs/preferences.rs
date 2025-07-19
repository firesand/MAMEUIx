// src/ui/dialogs/preferences.rs
use eframe::egui;
use crate::models::{Preferences, Theme};

pub struct PreferencesDialog {
    selected_tab: PreferencesTab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreferencesTab {
    General,
    Display,
}

impl Default for PreferencesDialog {
    fn default() -> Self {
        Self {
            selected_tab: PreferencesTab::General,
        }
    }
}

impl PreferencesDialog {
    /// Display the preferences dialog with tabs
    /// Note: This now takes a Preferences struct and Theme
    pub fn show(ctx: &egui::Context, prefs: &mut Preferences, theme: &mut Theme, open: &mut bool, has_catver_ini: bool) {
        let mut close = false;
        
        // Use persistent ID to maintain tab state
        let id = egui::Id::new("preferences_dialog_state");
        let mut selected_tab = ctx.data_mut(|d| d.get_temp::<PreferencesTab>(id).unwrap_or(PreferencesTab::General));

        egui::Window::new("Preferences")
        .default_size([600.0, 650.0])
        .open(open)
        .show(ctx, |ui| {
            // Tab selector
            ui.horizontal(|ui| {
                if ui.selectable_value(&mut selected_tab, PreferencesTab::General, "General").clicked() {
                    ctx.data_mut(|d| d.insert_temp(id, selected_tab));
                }
                if ui.selectable_value(&mut selected_tab, PreferencesTab::Display, "Display").clicked() {
                    ctx.data_mut(|d| d.insert_temp(id, selected_tab));
                }
            });
            
            ui.separator();
            
            // Show content based on selected tab
            match selected_tab {
                PreferencesTab::General => {
                    Self::show_general_tab_static(ui, prefs, has_catver_ini);
                }
                PreferencesTab::Display => {
                    Self::show_display_tab_static(ui, prefs, theme);
                }
            }

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
    
    fn show_general_tab_static(ui: &mut egui::Ui, prefs: &mut Preferences, has_catver_ini: bool) {
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
                columns[1].checkbox(&mut prefs.visible_columns.driver_status, "Driver Status");
                
                // Category checkbox - disabled if no catver.ini configured
                columns[1].add_enabled_ui(has_catver_ini, |ui| {
                    ui.checkbox(&mut prefs.visible_columns.category, "Category");
                });
                
                columns[1].checkbox(&mut prefs.visible_columns.rom, "ROM");
                columns[1].checkbox(&mut prefs.visible_columns.chd, "CHD");
            });
            
            // Add note about category column requirement
            ui.add_space(5.0);
            if !has_catver_ini {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    "⚠ Category column is disabled. Configure catver.ini path in Directories settings to enable it."
                );
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 255, 100),
                    "✓ Category column is available (catver.ini configured)."
                );
            }
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
    }
    
    fn show_display_tab_static(ui: &mut egui::Ui, prefs: &mut Preferences, theme: &mut Theme) {
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

        // Theme selection
        ui.group(|ui| {
            ui.label("Theme Selection");
            ui.label("Choose your preferred visual theme:");
            
            // Create a grid layout for theme selection
            ui.columns(2, |columns| {
                let themes = [
                    Theme::DarkBlue,
                    Theme::DarkGrey,
                    Theme::ArcadePurple,
                    Theme::LightClassic,
                    Theme::NeonGreen,
                    Theme::SunsetOrange,
                    Theme::OceanBlue,
                    Theme::MidnightBlack,
                    Theme::ForestGreen,
                    Theme::RetroAmber,
                ];

                for (i, theme_option) in themes.iter().enumerate() {
                    let column = i % 2;
                    let is_selected = *theme_option == *theme;
                    
                    columns[column].horizontal(|ui| {
                        if ui.radio(is_selected, theme_option.display_name()).clicked() {
                            *theme = theme_option.clone();
                        }
                    });
                }
            });
        });
    }
    
}
