// src/ui/dialogs/preferences.rs
use crate::models::{Preferences, Theme, UiShellMode};
use crate::ui::components::steam_ui::SteamUi;
use eframe::egui;

#[derive(Default)]
pub struct PreferencesDialog {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PreferencesTab {
    General,
    Display,
}

impl PreferencesDialog {
    /// Display the preferences dialog with tabs
    /// Note: This now takes a Preferences struct and Theme
    pub fn show(
        ctx: &egui::Context,
        prefs: &mut Preferences,
        theme: &mut Theme,
        open: &mut bool,
        has_catver_ini: bool,
    ) {
        let mut close = false;

        let previous_style = (*ctx.style()).clone();
        let initial_theme = theme.clone();
        SteamUi::apply(ctx);

        // Use persistent ID to maintain tab state
        let id = egui::Id::new("preferences_dialog_state");
        let mut selected_tab = ctx.data_mut(|d| {
            d.get_temp::<PreferencesTab>(id)
                .unwrap_or(PreferencesTab::General)
        });

        egui::Window::new("Preferences")
            .default_size([860.0, 680.0])
            .min_size([740.0, 560.0])
            .frame(SteamUi::window_frame())
            .open(open)
            .show(ctx, |ui| {
                let body_height = (ui.available_height() - SteamUi::FOOTER_HEIGHT).max(420.0);

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), body_height),
                    egui::Layout::left_to_right(egui::Align::TOP),
                    |ui| {
                        SteamUi::sidebar_column(ui, 208.0, body_height, |ui| {
                            ui.label(SteamUi::section_title("Sections"));
                            ui.add_space(10.0);

                            if SteamUi::sidebar_button(
                                ui,
                                "General",
                                selected_tab == PreferencesTab::General,
                            )
                            .clicked()
                            {
                                selected_tab = PreferencesTab::General;
                                ctx.data_mut(|d| d.insert_temp(id, selected_tab));
                            }
                            ui.add_space(6.0);
                            if SteamUi::sidebar_button(
                                ui,
                                "Display",
                                selected_tab == PreferencesTab::Display,
                            )
                            .clicked()
                            {
                                selected_tab = PreferencesTab::Display;
                                ctx.data_mut(|d| d.insert_temp(id, selected_tab));
                            }
                        });

                        ui.add_space(SteamUi::COLUMN_GAP);

                        SteamUi::content_column(ui, body_height, |ui| {
                            let scroll_height = ui.available_height();
                            match selected_tab {
                                PreferencesTab::General => {
                                    SteamUi::page_header(
                                        ui,
                                        "General",
                                        "Startup behavior, game list columns, and language",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        Self::show_general_tab_static(ui, prefs, has_catver_ini);
                                    });
                                }
                                PreferencesTab::Display => {
                                    SteamUi::page_header(
                                        ui,
                                        "Display",
                                        "Theme, window, notification, and rendering preferences",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        Self::show_display_tab_static(ui, prefs, theme, ctx);
                                    });
                                }
                            }
                        });
                    },
                );

                ui.add_space(12.0);
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("OK").clicked() {
                        close = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    if ui.button("Apply").clicked() {
                        theme.apply(ctx);
                        SteamUi::apply(ctx);
                    }
                });
            });

        if *theme != initial_theme {
            theme.apply(ctx);
        } else {
            ctx.set_style(previous_style);
        }

        if close {
            *open = false;
        }
    }

    fn show_general_tab_static(ui: &mut egui::Ui, prefs: &mut Preferences, has_catver_ini: bool) {
        // General preferences section
        SteamUi::panel(ui, |ui| {
            ui.label(SteamUi::section_title("General Settings"));
            ui.add_space(4.0);
            ui.checkbox(
                &mut prefs.search_new_games,
                "Search for new games at startup",
            );
            ui.checkbox(
                &mut prefs.version_mismatch_warning,
                "Enable version mismatch warning",
            );
            ui.checkbox(&mut prefs.use_mame_defaults, "Use MAME default options");
            ui.checkbox(
                &mut prefs.joystick_selection,
                "Allow game selection with a joystick",
            );
            ui.checkbox(&mut prefs.auto_save, "Auto-save configuration changes");
            ui.checkbox(&mut prefs.confirm_exit, "Confirm before exiting");
        });

        ui.add_space(SteamUi::SECTION_GAP);

        // Visible columns configuration
        SteamUi::panel(ui, |ui| {
            ui.label(SteamUi::section_title("Visible Columns in Game List"));
            ui.label(SteamUi::subtitle("Choose which columns to display"));
            ui.add_space(8.0);

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
                    SteamUi::WARNING,
                    "Category column is disabled. Configure catver.ini in Directories to enable it."
                );
            } else {
                ui.colored_label(
                    SteamUi::SUCCESS,
                    "Category column is available because catver.ini is configured.",
                );
            }
        });

        ui.add_space(12.0);

        // Language selection
        SteamUi::panel(ui, |ui| {
            ui.label(SteamUi::section_title("Language"));
            ui.horizontal(|ui| {
                ui.label("Interface language:");
                egui::ComboBox::from_id_salt("preferences_language")
                    .selected_text(&prefs.language)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut prefs.language, "English".to_string(), "English");
                        ui.selectable_value(&mut prefs.language, "Spanish".to_string(), "Spanish");
                        ui.selectable_value(&mut prefs.language, "French".to_string(), "French");
                        ui.selectable_value(&mut prefs.language, "German".to_string(), "German");
                        ui.selectable_value(&mut prefs.language, "Italian".to_string(), "Italian");
                        ui.selectable_value(
                            &mut prefs.language,
                            "Japanese".to_string(),
                            "Japanese",
                        );
                    });
            });
        });
    }

    fn show_display_tab_static(
        ui: &mut egui::Ui,
        prefs: &mut Preferences,
        theme: &mut Theme,
        ctx: &egui::Context,
    ) {
        // Display preferences
        SteamUi::panel(ui, |ui| {
            ui.label(SteamUi::section_title("Display Settings"));
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Clone color:");
                ui.color_edit_button_rgb(&mut prefs.clone_color);
            });

            ui.checkbox(&mut prefs.show_fps, "Show FPS counter");

            ui.add_space(8.0);
            ui.label(SteamUi::section_title("UI shell"));
            ui.label(SteamUi::subtitle(
                "Choose the interface layout. Redesign preview is experimental — switch back anytime.",
            ));
            for mode in [
                UiShellMode::LegacyDock,
                UiShellMode::LegacyClassic,
                UiShellMode::RedesignPreview,
            ] {
                if ui
                    .radio(prefs.ui_shell == mode, mode.display_name())
                    .clicked()
                {
                    prefs.ui_shell = mode;
                    prefs.use_dock_layout = mode == UiShellMode::LegacyDock;
                }
                ui.label(SteamUi::muted(mode.description()));
            }

            ui.checkbox(
                &mut prefs.enable_toast_notifications,
                "Enable toast notifications",
            );
            ui.checkbox(&mut prefs.fullscreen, "Start in fullscreen mode");
            ui.checkbox(&mut prefs.vsync, "Enable V-Sync");

            ui.horizontal(|ui| {
                ui.label("Window Width:");
                ui.add(
                    egui::DragValue::new(&mut prefs.window_width)
                        .speed(10.0)
                        .range(640.0..=3840.0),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Window Height:");
                ui.add(
                    egui::DragValue::new(&mut prefs.window_height)
                        .speed(10.0)
                        .range(480.0..=2160.0),
                );
            });
        });

        ui.add_space(12.0);

        // Theme selection
        SteamUi::panel(ui, |ui| {
            ui.label(SteamUi::section_title("Theme Selection"));
            ui.label(SteamUi::subtitle("Choose your preferred visual theme"));

            // Show current theme preview
            ui.horizontal(|ui| {
                ui.label("Current theme:");
                ui.colored_label(SteamUi::ACCENT, theme.display_name());
            });

            ui.add_space(5.0);

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
                            println!("Theme changed to: {}", theme_option.display_name());
                            *theme = theme_option.clone();
                            // Apply the theme immediately
                            theme_option.apply(ctx);
                            // Force a repaint to see the changes
                            ctx.request_repaint();
                            println!("Theme applied and repaint requested");
                        }
                    });
                }
            });

            // Add a test button to verify theme application
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Test Theme").clicked() {
                    println!("Testing theme application for: {}", theme.display_name());
                    theme.apply(ctx);
                    SteamUi::apply(ctx);
                    ctx.request_repaint();
                }
                if ui.button("Apply Theme").clicked() {
                    println!("Applying current theme: {}", theme.display_name());
                    theme.apply(ctx);
                    SteamUi::apply(ctx);
                    ctx.request_repaint();
                }
            });
        });
    }
}
