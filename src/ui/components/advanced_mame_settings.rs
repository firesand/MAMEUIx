use eframe::egui;
use crate::models::game_properties::*;
use crate::models::AppConfig;

pub struct AdvancedMameSettingsDialog {
    properties: GameProperties,
    original_properties: GameProperties,
    selected_category: SettingsCategory,
    selected_subcategory: String,
    is_dirty: bool,
    search_query: String,
    command_preview: String,
}

#[derive(Debug, Clone, PartialEq)]
enum SettingsCategory {
    CoreConfiguration,
    DisplayGraphics,
    AudioInput,
    Advanced,
    OSDSettings,
}

struct CategoryInfo {
    name: &'static str,
    icon: &'static str,
    subcategories: Vec<SubcategoryInfo>,
}

struct SubcategoryInfo {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
}

impl AdvancedMameSettingsDialog {
    pub fn new(config: &AppConfig) -> Self {
        let properties = config.default_game_properties.clone();
        let mut dialog = Self {
            original_properties: properties.clone(),
            properties,
            selected_category: SettingsCategory::CoreConfiguration,
            selected_subcategory: "core-config".to_string(),
            is_dirty: false,
            search_query: String::new(),
            command_preview: String::new(),
        };
        dialog.update_command_preview();
        dialog
    }

    fn get_categories() -> Vec<(SettingsCategory, CategoryInfo)> {
        vec![
            (SettingsCategory::CoreConfiguration, CategoryInfo {
                name: "Core Configuration",
                icon: "",
                subcategories: vec![
                    SubcategoryInfo { id: "core-config", name: "Configuration", icon: "⚙️" },
                    SubcategoryInfo { id: "core-state", name: "State/Playback", icon: "🎮" },
                    SubcategoryInfo { id: "core-performance", name: "Performance", icon: "⚡" },
                ],
            }),
            (SettingsCategory::DisplayGraphics, CategoryInfo {
                name: "Display & Graphics",
                icon: "",
                subcategories: vec![
                    SubcategoryInfo { id: "core-render", name: "Render Options", icon: "🖼️" },
                    SubcategoryInfo { id: "core-rotation", name: "Rotation", icon: "🔄" },
                    SubcategoryInfo { id: "core-artwork", name: "Artwork", icon: "🎨" },
                    SubcategoryInfo { id: "core-screen", name: "Screen", icon: "📺" },
                    SubcategoryInfo { id: "core-vector", name: "Vector", icon: "📐" },
                ],
            }),
            (SettingsCategory::AudioInput, CategoryInfo {
                name: "Audio & Input",
                icon: "",
                subcategories: vec![
                    SubcategoryInfo { id: "core-sound", name: "Sound", icon: "🔊" },
                    SubcategoryInfo { id: "core-input", name: "Input", icon: "🎯" },
                    SubcategoryInfo { id: "core-input-auto", name: "Auto Enable", icon: "🎮" },
                ],
            }),
            (SettingsCategory::Advanced, CategoryInfo {
                name: "Advanced",
                icon: "",
                subcategories: vec![
                    SubcategoryInfo { id: "core-debug", name: "Debugging", icon: "🐛" },
                    SubcategoryInfo { id: "core-misc", name: "Miscellaneous", icon: "🔧" },
                    SubcategoryInfo { id: "scripting", name: "Scripting", icon: "📜" },
                ],
            }),
            (SettingsCategory::OSDSettings, CategoryInfo {
                name: "OSD Settings",
                icon: "",
                subcategories: vec![
                    SubcategoryInfo { id: "osd-input-mapping", name: "Input Mapping", icon: "⌨️" },
                    SubcategoryInfo { id: "osd-fonts", name: "Fonts", icon: "🔤" },
                    SubcategoryInfo { id: "osd-output", name: "Output", icon: "📤" },
                    SubcategoryInfo { id: "osd-input-providers", name: "Input Providers", icon: "🎹" },
                    SubcategoryInfo { id: "osd-debugging", name: "OSD Debugging", icon: "🔍" },
                    SubcategoryInfo { id: "osd-performance", name: "OSD Performance", icon: "🚀" },
                    SubcategoryInfo { id: "osd-video", name: "Video Options", icon: "🖥️" },
                    SubcategoryInfo { id: "osd-sound", name: "Sound Options", icon: "🎵" },
                    SubcategoryInfo { id: "osd-midi", name: "MIDI Options", icon: "🎹" },
                    SubcategoryInfo { id: "osd-network", name: "Network Options", icon: "🌐" },
                    SubcategoryInfo { id: "opengl", name: "OpenGL", icon: "🎮" },
                    SubcategoryInfo { id: "bgfx", name: "BGFX", icon: "🎨" },
                    SubcategoryInfo { id: "sdl", name: "SDL Options", icon: "🖱️" },
                ],
            }),
        ]
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, config: &mut AppConfig) -> bool {
        let mut apply_changes = false;
        let mut should_close = false;

        // Apply dark theme colors from HTML mockup
        let mut visuals = ctx.style().visuals.clone();
        visuals.window_fill = egui::Color32::from_rgb(22, 22, 22); // --bg-secondary
        visuals.panel_fill = egui::Color32::from_rgb(22, 22, 22);
        visuals.extreme_bg_color = egui::Color32::from_rgb(10, 10, 10); // --bg-primary
        visuals.faint_bg_color = egui::Color32::from_rgb(30, 30, 30); // --bg-tertiary

        // Create widget visuals
        let noninteractive = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 51, 51)), // --border-color
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 240, 250)), // --text-primary
            corner_radius: egui::CornerRadius::same(4),
            expansion: 0.0,
        };

        let inactive = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 51, 51)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(160, 160, 160)), // --text-secondary
            corner_radius: egui::CornerRadius::same(4),
            expansion: 0.0,
        };

        let hovered = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(37, 37, 37),
            bg_fill: egui::Color32::from_rgb(37, 37, 37), // --bg-hover
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(76, 139, 245)), // --accent-primary
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 240, 250)),
            corner_radius: egui::CornerRadius::same(4),
            expansion: 1.0,
        };

        let active = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(76, 139, 245),
            bg_fill: egui::Color32::from_rgb(76, 139, 245), // --accent-primary
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(76, 139, 245)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::WHITE),
            corner_radius: egui::CornerRadius::same(4),
            expansion: 1.0,
        };

        let open_visuals = egui::style::WidgetVisuals {
            weak_bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_fill: egui::Color32::from_rgb(30, 30, 30),
            bg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 51, 51)),
            fg_stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(240, 240, 250)),
            corner_radius: egui::CornerRadius::same(4),
            expansion: 0.0,
        };

        visuals.widgets = egui::style::Widgets {
            noninteractive,
            inactive,
            hovered,
            active,
            open: open_visuals,
        };

        ctx.set_visuals(visuals);

        // Create window matching HTML mockup style
        egui::Window::new("⚙️ Advanced MAME Settings")
        .open(open)
        .default_size([1200.0, 800.0])
        .min_size([900.0, 600.0])
        .resizable(true)
        .show(ctx, |ui| {
            // Add subtitle similar to HTML
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                ui.label(
                    egui::RichText::new("All options from ")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(160, 160, 160))
                );
                ui.label(
                    egui::RichText::new("mame -showusage")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(160, 160, 160))
                    .background_color(egui::Color32::from_rgb(10, 10, 10))
                    .monospace()
                );
            });

            ui.separator();

            // Calculate available height
            let total_height = ui.available_height();
            let footer_height = 60.0;
            let content_height = total_height - footer_height;

            // Main container
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), content_height),
                                       egui::Layout::left_to_right(egui::Align::TOP),
                                       |ui| {
                                           // Left sidebar
                                           self.render_sidebar(ui, content_height);

                                           // Vertical separator
                                           ui.separator();

                                           // Right content area
                                           self.render_content_area(ui, content_height);
                                       }
            );

            // Footer
            ui.separator();
            self.render_footer(ui, &mut should_close, &mut apply_changes, config);
        });

        // Track changes
        self.is_dirty = self.properties != self.original_properties;
        self.update_command_preview();

        // Close window if needed
        if should_close {
            *open = false;
        }

        apply_changes
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui, height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(260.0, height), // Increased width for 4K displays
                                   egui::Layout::top_down(egui::Align::LEFT),
                                   |ui| {
                                       ui.spacing_mut().item_spacing.y = 8.0;

                                       // Search box
                                       ui.add_space(8.0);
                                       ui.horizontal(|ui| {
                                           ui.add_space(8.0);
                                           let response = ui.add(
                                               egui::TextEdit::singleline(&mut self.search_query)
                                               .desired_width(244.0) // Adjusted for new width
                                               .hint_text("Search options...")
                                           );
                                           if response.changed() {
                                               // Implement search functionality
                                           }
                                       });
                                       ui.add_space(8.0);

                                       // Reserve space for command preview at bottom
                                       let available_height = ui.available_height() - 110.0;

                                       // Categories with unique ID and limited height
                                       ui.allocate_ui_with_layout(
                                           egui::vec2(ui.available_width(), available_height),
                                                                  egui::Layout::top_down(egui::Align::LEFT),
                                                                  |ui| {
                                                                      egui::ScrollArea::vertical()
                                                                      .id_salt("sidebar_categories_scroll")
                                                                      .auto_shrink([false, false])
                                                                      .show(ui, |ui| {
                                                                          for (category, info) in Self::get_categories() {
                                                                              self.render_category_group(ui, category, info);
                                                                          }
                                                                      });
                                                                  }
                                       );

                                       // Command preview at bottom
                                       ui.add_space(8.0);
                                       ui.group(|ui| {
                                           ui.set_width(244.0); // Adjusted for new width
                                           ui.label(egui::RichText::new("Command line preview:").strong().size(14.0));
                                           ui.label(
                                               egui::RichText::new(&self.command_preview)
                                               .monospace()
                                               .color(egui::Color32::from_rgb(76, 175, 80)) // --success color
                                               .size(13.0)
                                           );
                                       });
                                   }
        );
    }

    fn render_category_group(&mut self, ui: &mut egui::Ui, category: SettingsCategory, info: CategoryInfo) {
        // Category header
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(info.name.to_uppercase())
            .size(12.0) // Increased from 10.0
            .color(egui::Color32::from_rgb(120, 120, 130))
            .strong()
        );
        ui.add_space(4.0);

        // Subcategory items
        for subcategory in &info.subcategories {
            let is_selected = self.selected_category == category &&
            self.selected_subcategory == subcategory.id;

            let button_color = if is_selected {
                egui::Color32::from_rgb(76, 139, 245)
            } else {
                egui::Color32::TRANSPARENT
            };

            let text_color = if is_selected {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_rgb(240, 240, 250)
            };

            let response = ui.add_sized(
                [244.0, 32.0], // Adjusted width for new sidebar size
                egui::Button::new(
                    egui::RichText::new(format!("{}  {}", subcategory.icon, subcategory.name))
                    .size(15.0) // Increased from 13.0
                    .color(text_color)
                )
                .fill(button_color)
                .stroke(egui::Stroke::NONE)
            );

            if response.hovered() && !is_selected {
                ui.painter().rect_filled(
                    response.rect,
                    egui::CornerRadius::same(4),
                                         egui::Color32::from_rgba_premultiplied(255, 255, 255, 10)
                );
            }

            if response.clicked() {
                self.selected_category = category.clone();
                self.selected_subcategory = subcategory.id.to_string();
            }
        }

        ui.add_space(12.0);
    }

    fn render_content_area(&mut self, ui: &mut egui::Ui, height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
                                   egui::Layout::top_down(egui::Align::LEFT),
                                   |ui| {
                                       ui.spacing_mut().item_spacing.y = 16.0;
                                       ui.add_space(16.0);

                                       // Add both vertical and horizontal scrolling with unique ID
                                       egui::ScrollArea::both()
                                       .id_salt("content_area_scroll")
                                       .auto_shrink([false, false])
                                       .show(ui, |ui| {
                                           // Set a minimum width for content to enable horizontal scrolling when needed
                                           ui.set_min_width(750.0);

                                           // Show content based on selected subcategory
                                           match self.selected_subcategory.as_str() {
                                               "core-config" => self.show_core_config(ui),
                                             "core-state" => self.show_core_state(ui),
                                             "core-performance" => self.show_core_performance(ui),
                                             "core-render" => self.show_core_render(ui),
                                             "core-rotation" => self.show_core_rotation(ui),
                                             "core-artwork" => self.show_core_artwork(ui),
                                             "core-screen" => self.show_core_screen(ui),
                                             "core-vector" => self.show_core_vector(ui),
                                             "core-sound" => self.show_core_sound(ui),
                                             "core-input" => self.show_core_input(ui),
                                             "core-input-auto" => self.show_core_input_auto(ui),
                                             "core-debug" => self.show_core_debug(ui),
                                             "core-misc" => self.show_core_misc(ui),
                                             "scripting" => self.show_scripting(ui),
                                             "osd-input-mapping" => self.show_osd_input_mapping(ui),
                                             "osd-fonts" => self.show_osd_fonts(ui),
                                             "osd-output" => self.show_osd_output(ui),
                                             "osd-input-providers" => self.show_osd_input_providers(ui),
                                             "osd-debugging" => self.show_osd_debugging(ui),
                                             "osd-performance" => self.show_osd_performance(ui),
                                             "osd-video" => self.show_osd_video(ui),
                                             "osd-sound" => self.show_osd_sound(ui),
                                             "osd-midi" => self.show_osd_midi(ui),
                                             "osd-network" => self.show_osd_network(ui),
                                             "opengl" => self.show_opengl(ui),
                                             "bgfx" => self.show_bgfx(ui),
                                             "sdl" => self.show_sdl(ui),
                                             _ => {}
                                           }

                                           // Add extra space at bottom
                                           ui.add_space(50.0);
                                       });
                                   }
        );
    }

    fn render_footer(&mut self, ui: &mut egui::Ui, should_close: &mut bool, apply_changes: &mut bool, config: &mut AppConfig) {
        ui.horizontal(|ui| {
            // Left side buttons
            if ui.button("Reset to Defaults").clicked() {
                self.reset_to_defaults();
            }

            if ui.button("Export Settings").clicked() {
                self.export_settings();
            }

            if ui.button("Import Settings").clicked() {
                self.import_settings();
            }

            // Add space to push right buttons to the right
            ui.add_space(ui.available_width() - 220.0);

            // Right side buttons
            if ui.button("Cancel").clicked() {
                self.properties = self.original_properties.clone();
                *should_close = true;
            }

            if ui.add_enabled(self.is_dirty, egui::Button::new("Save")).clicked() {
                config.default_game_properties = self.properties.clone();
                *should_close = true;
                *apply_changes = true;
            }
        });
    }

    // Helper function to create option groups matching HTML style
    fn render_option_group(ui: &mut egui::Ui, title: Option<&str>, content: impl FnOnce(&mut egui::Ui)) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());

            if let Some(title) = title {
                ui.label(egui::RichText::new(title).strong().size(16.0).color(egui::Color32::from_rgb(100, 181, 246)));
                ui.add_space(12.0);
            }

            content(ui);
        });
    }

    // Helper function to render option item matching HTML style
    fn render_option_item(
        ui: &mut egui::Ui,
        name: &str,
        description: &str,
        content: impl FnOnce(&mut egui::Ui)
    ) {
        // Use a fixed height for consistent vertical alignment
        let row_height = 50.0;
        
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Left side - option name and description (fixed width)
                let left_width = ui.available_width() * 0.6; // 60% for labels
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, ui.available_height()),
                    egui::Layout::top_down_justified(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(name).monospace().size(15.0));
                        ui.label(egui::RichText::new(description)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 160)));
                    }
                );

                // Add some spacing between left and right
                ui.add_space(20.0);

                // Right side - control (remaining width)
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        content(ui);
                    }
                );
            }
        );

        ui.add_space(16.0);
    }

    // Content methods for each subcategory
    fn show_core_config(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Configuration Options").size(20.0));
        ui.label(egui::RichText::new("Control how MAME loads and saves configuration files").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, None, move |ui| {
            Self::render_option_item(
                ui,
                "-readconfig",
                "enable loading of configuration files",
                |ui| {
                    let mut readconfig = true; // Default
                    ui.checkbox(&mut readconfig, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-writeconfig",
                "write configuration to (driver).ini on exit",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.write_config, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_state(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core State/Playback Options").size(20.0));
        ui.label(egui::RichText::new("Configure save states, recording, and playback features").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Save States"), |ui| {
            Self::render_option_item(
                ui,
                "-state",
                "saved state to load",
                |ui| {
                    let mut state_name = String::new();
                    ui.text_edit_singleline(&mut state_name);
                }
            );

            Self::render_option_item(
                ui,
                "-autosave",
                "automatically restore state on start and save on exit for supported systems",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.auto_save, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-rewind",
                "enable rewind savestates",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.rewind, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-rewind_capacity",
                "rewind buffer size in megabytes",
                |ui| {
                    let mut capacity = 100;
                    ui.add(egui::DragValue::new(&mut capacity).range(10..=1000));
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Recording & Playback"), |ui| {
            Self::render_option_item(
                ui,
                "-playback",
                "playback an input file",
                |ui| {
                    let mut playback_file = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut playback_file);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-record",
                "record an input file",
                |ui| {
                    let mut record_file = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut record_file);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-exit_after",
                "exit after recording/playback completes",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.exit_after, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Artwork Options"), |ui| {
            Self::render_option_item(
                ui,
                "-bilinear",
                "use bilinear filtering for artwork",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.bilinear, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-burnin",
                "show burn-in effects",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.burnin, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-crop",
                "crop artwork to game screen",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.crop, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_performance(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Performance Options").size(20.0));
        ui.label(egui::RichText::new("Optimize MAME's performance and emulation speed").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Frame Control"), move |ui| {
            Self::render_option_item(
                ui,
                "-autoframeskip",
                "enable automatic frameskip adjustment to maintain emulation speed",
                |ui| {
                    if ui.checkbox(&mut properties.screen.auto_frameskip, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-frameskip",
                "set frameskip to fixed value, 0-10 (upper limit with autoframeskip)",
                |ui| {
                    if ui.add(egui::DragValue::new(&mut properties.screen.frameskip_value)
                        .range(0..=10)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-seconds_to_run",
                "number of emulated seconds to run before automatically exiting",
                |ui| {
                    let mut seconds = properties.screen.seconds_to_run;
                    if ui.add(egui::DragValue::new(&mut seconds).range(0..=3600)).changed() {
                        properties.screen.seconds_to_run = seconds;
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-throttle",
                "throttle emulation to keep system running in sync with real time",
                |ui| {
                    if ui.checkbox(&mut properties.display.throttle, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-sleep",
                "enable sleeping, which gives time back to other applications when idle",
                |ui| {
                    if ui.checkbox(&mut properties.screen.sleep_when_idle, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-speed",
                "controls the speed of gameplay, relative to realtime; smaller numbers are slower",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.screen.emulation_speed, 0.1..=10.0)
                        .step_by(0.1)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-refreshspeed",
                "automatically adjust emulation speed to keep the emulated refresh rate slower than the host screen",
                |ui| {
                    if ui.checkbox(&mut properties.screen.refresh_speed, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-lowlatency",
                "draws new frame before throttling to reduce input latency",
                |ui| {
                    if ui.checkbox(&mut properties.screen.low_latency, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_render(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Render Options").size(20.0));
        ui.label(egui::RichText::new("Configure rendering and scaling options").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Scaling Options"), move |ui| {
            Self::render_option_item(
                ui,
                "-keepaspect",
                "maintain aspect ratio when scaling to fill output screen/window",
                |ui| {
                    if ui.checkbox(&mut properties.display.enforce_aspect_ratio, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-unevenstretch",
                "allow non-integer ratios when scaling to fill output screen/window horizontally or vertically",
                |ui| {
                    if ui.checkbox(&mut properties.display.use_non_integer_scaling, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-unevenstretchx",
                "allow non-integer ratios when scaling to fill output screen/window horizontally",
                |ui| {
                    if ui.checkbox(&mut properties.display.stretch_only_x_axis, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-unevenstretchy",
                "allow non-integer ratios when scaling to fill output screen/window vertically",
                |ui| {
                    if ui.checkbox(&mut properties.display.stretch_only_y_axis, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-autostretchxy",
                "automatically apply -unevenstretchx/y based on source native orientation",
                |ui| {
                    if ui.checkbox(&mut properties.display.auto_select_stretch_axis, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-intoverscan",
                "allow overscan on integer scaled targets",
                |ui| {
                    if ui.checkbox(&mut properties.display.overscan_on_targets, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-intscalex",
                "set horizontal integer scale factor",
                |ui| {
                    if ui.add(egui::DragValue::new(&mut properties.display.horizontal_scale_factor)
                        .range(0..=10)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-intscaley",
                "set vertical integer scale factor",
                |ui| {
                    if ui.add(egui::DragValue::new(&mut properties.display.vertical_scale_factor)
                        .range(0..=10)).changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_rotation(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Rotation Options").size(20.0));
        ui.label(egui::RichText::new("Configure screen rotation and flipping").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Rotation Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-rotate",
                "rotate the game screen according to the game's orientation when needed",
                |ui| {
                    let mut rotate = properties.display.rotation != RotationMode::Default;
                    if ui.checkbox(&mut rotate, "Enabled").changed() {
                        if rotate {
                            properties.display.rotation = RotationMode::Rotate0;
                        } else {
                            properties.display.rotation = RotationMode::Default;
                        }
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-ror",
                "rotate screen clockwise 90 degrees",
                |ui| {
                    let mut ror = properties.display.rotation == RotationMode::Rotate90;
                    if ui.checkbox(&mut ror, "Enabled").changed() {
                        if ror {
                            properties.display.rotation = RotationMode::Rotate90;
                        } else {
                            properties.display.rotation = RotationMode::Default;
                        }
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-rol",
                "rotate screen counterclockwise 90 degrees",
                |ui| {
                    let mut rol = properties.display.rotation == RotationMode::Rotate270;
                    if ui.checkbox(&mut rol, "Enabled").changed() {
                        if rol {
                            properties.display.rotation = RotationMode::Rotate270;
                        } else {
                            properties.display.rotation = RotationMode::Default;
                        }
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-autoror",
                "automatically rotate screen clockwise 90 degrees if vertical",
                |ui| {
                    if ui.checkbox(&mut properties.display.auto_rotate_right, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-autorol",
                "automatically rotate screen counterclockwise 90 degrees if vertical",
                |ui| {
                    if ui.checkbox(&mut properties.display.auto_rotate_left, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-flipx",
                "flip screen left-right",
                |ui| {
                    if ui.checkbox(&mut properties.display.flip_screen_left_right, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-flipy",
                "flip screen upside-down",
                |ui| {
                    if ui.checkbox(&mut properties.display.flip_screen_upside_down, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_artwork(&mut self, ui: &mut egui::Ui) {
        ui.heading("Core Artwork Options");
        ui.label("Configure artwork display settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Artwork Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-artwork_crop",
                "crop artwork so emulated screen image fills output screen/window in one axis",
                |ui| {
                    let mut crop = false;
                    ui.checkbox(&mut crop, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-fallback_artwork",
                "fallback artwork if no external artwork or internal driver layout defined",
                |ui| {
                    let mut fallback = String::new();
                    ui.text_edit_singleline(&mut fallback);
                }
            );

            Self::render_option_item(
                ui,
                "-override_artwork",
                "override artwork for external artwork and internal driver layout",
                |ui| {
                    let mut override_art = String::new();
                    ui.text_edit_singleline(&mut override_art);
                }
            );
        });
    }

    fn show_core_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Screen Options").size(20.0));
        ui.label(egui::RichText::new("Configure screen display settings").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Display Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-brightness",
                "default game screen brightness correction",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.display.brightness_correction, 0.1..=2.0)
                        .step_by(0.1)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-contrast",
                "default game screen contrast correction",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.display.contrast_correction, 0.1..=2.0)
                        .step_by(0.1)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-gamma",
                "default game screen gamma correction",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.display.gamma_correction, 0.1..=3.0)
                        .step_by(0.1)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-pause_brightness",
                "amount to scale the screen brightness when paused",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.display.pause_brightness, 0.0..=1.0)
                        .step_by(0.05)).changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-effect",
                "name of a PNG file to use for visual effects, or 'none'",
                |ui| {
                    let mut effect = String::from("none");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut effect);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );
        });
    }

    fn show_core_vector(&mut self, ui: &mut egui::Ui) {
        ui.heading("Core Vector Options");
        ui.label("Configure vector display settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Vector Display"), |ui| {
            Self::render_option_item(
                ui,
                "-beam_width_min",
                "set vector beam width minimum",
                |ui| {
                    let mut beam_min = 1.0;
                    ui.add(egui::Slider::new(&mut beam_min, 0.1..=5.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-beam_width_max",
                "set vector beam width maximum",
                |ui| {
                    let mut beam_max = 1.0;
                    ui.add(egui::Slider::new(&mut beam_max, 0.1..=5.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-beam_dot_size",
                "set vector beam size for dots",
                |ui| {
                    let mut dot_size = 1.0;
                    ui.add(egui::Slider::new(&mut dot_size, 0.1..=5.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-beam_intensity_weight",
                "set vector beam intensity weight",
                |ui| {
                    let mut intensity = 0.0;
                    ui.add(egui::Slider::new(&mut intensity, 0.0..=1.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-flicker",
                "set vector flicker effect",
                |ui| {
                    let mut flicker = 0;
                    ui.add(egui::Slider::new(&mut flicker, 0..=100));
                }
            );
        });
    }

    fn show_core_sound(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Sound Options").size(20.0));
        ui.label(egui::RichText::new("Configure audio output settings").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Audio Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-samplerate",
                "set sound output sample rate",
                |ui| {
                    let mut sample_rate = properties.sound.sample_rate;
                    egui::ComboBox::from_id_salt("samplerate")
                        .selected_text(format!("{} Hz", sample_rate))
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut sample_rate, 11025, "11025 Hz").clicked() {
                                properties.sound.sample_rate = sample_rate;
                                *is_dirty = true;
                            }
                            if ui.selectable_value(&mut sample_rate, 22050, "22050 Hz").clicked() {
                                properties.sound.sample_rate = sample_rate;
                                *is_dirty = true;
                            }
                            if ui.selectable_value(&mut sample_rate, 44100, "44100 Hz").clicked() {
                                properties.sound.sample_rate = sample_rate;
                                *is_dirty = true;
                            }
                            if ui.selectable_value(&mut sample_rate, 48000, "48000 Hz").clicked() {
                                properties.sound.sample_rate = sample_rate;
                                *is_dirty = true;
                            }
                        });
                }
            );

            Self::render_option_item(
                ui,
                "-samples",
                "enable the use of external samples if available",
                |ui| {
                    if ui.checkbox(&mut properties.sound.use_samples, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-volume",
                "sound volume in decibels (-32 to 0)",
                |ui| {
                    if ui.add(egui::Slider::new(&mut properties.sound.volume_attenuation, -32..=0)).changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_input(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Input Options").size(20.0));
        ui.label(egui::RichText::new("Configure input device settings and mappings").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Keyboard Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-multikeyboard",
                "enable separate input from each keyboard device (if present)",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.multi_keyboard, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-multimouse",
                "enable separate input from each mouse device (if present)",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.multi_mouse, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-steadykey",
                "enable steadykey support",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.steady_key, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-ui_active",
                "enable user interface on top of emulated keyboard (if present)",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.ui_active, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-offscreen_reload",
                "convert lightgun button 2 into offscreen reload",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.offscreen_reload, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Joystick Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-joystick_map",
                "explicit joystick map, or auto to auto-select",
                |ui| {
                    let mut map = String::from("auto");
                    ui.text_edit_singleline(&mut map);
                }
            );

            Self::render_option_item(
                ui,
                "-joystick_deadzone",
                "center deadzone range for joystick where change is ignored (0.0 center, 1.0 end)",
                |ui| {
                    let mut deadzone = 0.3;
                    ui.add(egui::Slider::new(&mut deadzone, 0.0..=1.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-joystick_saturation",
                "end of axis saturation range for joystick where change is ignored (0.0 center, 1.0 end)",
                |ui| {
                    let mut saturation = 0.85;
                    ui.add(egui::Slider::new(&mut saturation, 0.0..=1.0).step_by(0.05));
                }
            );

            Self::render_option_item(
                ui,
                "-joystick_threshold",
                "threshold for joystick to be considered active as a switch (0.0 center, 1.0 end)",
                |ui| {
                    let mut threshold = 0.3;
                    ui.add(egui::Slider::new(&mut threshold, 0.0..=1.0).step_by(0.1));
                }
            );

            Self::render_option_item(
                ui,
                "-joystick_contradictory",
                "enable contradictory direction digital joystick input at the same time",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.contradictory, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Other Input Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-natural",
                "specifies whether to use a natural keyboard or not",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.natural, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-coin_impulse",
                "set coin impulse time (n<0 disable impulse, n==0 obey driver, 0<n set time n)",
                |ui| {
                    let mut impulse = 0;
                    ui.add(egui::DragValue::new(&mut impulse).range(-1..=100));
                }
            );
        });
    }

    fn show_core_input_auto(&mut self, ui: &mut egui::Ui) {
        ui.heading("Core Input Automatic Enable Options");
        ui.label("Configure automatic input device selection based on game controls");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Auto Device Selection"), |ui| {
            let device_options = ["keyboard", "mouse", "lightgun", "joystick", "none"];

            Self::render_option_item(
                ui,
                "-paddle_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a paddle control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("paddle_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-adstick_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if an analog joystick control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("adstick_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-pedal_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a pedal control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("pedal_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-dial_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a dial control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("dial_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-trackball_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a trackball control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("trackball_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-lightgun_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a lightgun control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("lightgun_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-positional_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a positional control is present",
                                     |ui| {
                                         let mut selected = "keyboard";
                                         egui::ComboBox::from_id_salt("positional_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );

            Self::render_option_item(
                ui,
                "-mouse_device",
                "enable (none|keyboard|mouse|lightgun|joystick) if a mouse control is present",
                                     |ui| {
                                         let mut selected = "mouse";
                                         egui::ComboBox::from_id_salt("mouse_device")
                                         .selected_text(selected)
                                         .show_ui(ui, |ui| {
                                             for option in device_options {
                                                 ui.selectable_value(&mut selected, option, option);
                                             }
                                         });
                                     }
            );
        });
    }

    fn show_core_debug(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Debugging Options").size(20.0));
        ui.label(egui::RichText::new("Configure debugging and logging features").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("Debugging"), |ui| {
            Self::render_option_item(
                ui,
                "-verbose",
                "display additional diagnostic information",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.verbose, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-log",
                "generate an error.log file",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.log, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-oslog",
                "output error.log data to the system debugger",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.oslog, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-debug",
                "enable/disable debugger",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.debug, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-update_in_pause",
                "keep calling video updates while in pause",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.update_pause, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-debugscript",
                "script for debugger",
                |ui| {
                    let mut script_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut script_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-debuglog",
                "write debug console output to debug.log",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.debuglog, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_core_misc(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("Core Miscellaneous Options").size(20.0));
        ui.label(egui::RichText::new("Configure various system and UI settings").size(15.0));
        ui.add_space(20.0);

        let properties = &mut self.properties;
        let is_dirty = &mut self.is_dirty;

        Self::render_option_group(ui, Some("System Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-drc",
                "enable DRC CPU core if available",
                |ui| {
                    let mut drc = true;
                    ui.checkbox(&mut drc, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-drc_use_c",
                "force DRC to use C backend",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.drc_c, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-drc_log_uml",
                "write DRC UML disassembly log",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.log_uml, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-drc_log_native",
                "write DRC native disassembly log",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.log_native, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-bios",
                "select the system BIOS to use",
                |ui| {
                    let mut bios = String::new();
                    ui.text_edit_singleline(&mut bios);
                }
            );

            Self::render_option_item(
                ui,
                "-cheat",
                "enable cheat subsystem",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.cheat, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-skip_gameinfo",
                "skip displaying the system information screen at startup",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.skip, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-ramsize",
                "size of RAM (if supported by driver)",
                |ui| {
                    let mut ramsize = String::new();
                    ui.text_edit_singleline(&mut ramsize);
                }
            );

            Self::render_option_item(
                ui,
                "-nvram_save",
                "save NVRAM data on exit",
                |ui| {
                    let mut nvram_save = "save";
                    egui::ComboBox::from_id_salt("nvram_save")
                    .selected_text(nvram_save)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut nvram_save, "none", "None");
                        ui.selectable_value(&mut nvram_save, "save", "Save");
                        ui.selectable_value(&mut nvram_save, "diff", "Diff");
                    });
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("UI Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-uifont",
                "specify a font to use",
                |ui| {
                    let mut font = String::new();
                    ui.text_edit_singleline(&mut font);
                }
            );

            Self::render_option_item(
                ui,
                "-ui",
                "type of UI (simple|cabinet)",
                |ui| {
                    let mut ui_type = "simple";
                    egui::ComboBox::from_id_salt("ui_type")
                    .selected_text(ui_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut ui_type, "simple", "Simple");
                        ui.selectable_value(&mut ui_type, "cabinet", "Cabinet");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-confirm_quit",
                "ask for confirmation before exiting",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.confirm, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-ui_mouse",
                "display UI mouse cursor",
                |ui| {
                    let mut ui_mouse = true;
                    ui.checkbox(&mut ui_mouse, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-language",
                "specify the language to use",
                |ui| {
                    let mut language = String::new();
                    ui.text_edit_singleline(&mut language);
                }
            );

            Self::render_option_item(
                ui,
                "-console",
                "enable console output",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.console, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-switchres",
                "enable switchres support",
                |ui| {
                    if ui.checkbox(&mut properties.miscellaneous.switchres, "Enabled").changed() {
                        *is_dirty = true;
                    }
                }
            );
        });
    }

    fn show_scripting(&mut self, ui: &mut egui::Ui) {
        ui.heading("Scripting Options");
        ui.label("Configure Lua scripting and plugin settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Scripting Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-autoboot_command",
                "command to execute after machine boot",
                |ui| {
                    let mut command = String::new();
                    ui.text_edit_singleline(&mut command);
                }
            );

            Self::render_option_item(
                ui,
                "-autoboot_delay",
                "delay before executing autoboot command (seconds)",
                                     |ui| {
                                         let mut delay = 0;
                                         ui.add(egui::DragValue::new(&mut delay).range(0..=60));
                                     }
            );

            Self::render_option_item(
                ui,
                "-autoboot_script",
                "Lua script to execute after machine boot",
                |ui| {
                    let mut script_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut script_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-console",
                "enable emulator Lua console",
                |ui| {
                    let mut console = false;
                    ui.checkbox(&mut console, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-plugins",
                "enable Lua plugin support",
                |ui| {
                    let mut plugins = true;
                    ui.checkbox(&mut plugins, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-plugin",
                "list of plugins to enable",
                |ui| {
                    let mut plugin_list = String::new();
                    ui.text_edit_singleline(&mut plugin_list);
                }
            );

            Self::render_option_item(
                ui,
                "-noplugin",
                "list of plugins to disable",
                |ui| {
                    let mut noplugin_list = String::new();
                    ui.text_edit_singleline(&mut noplugin_list);
                }
            );
        });
    }

    fn show_osd_input_mapping(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Input Mapping Options");
        ui.label("Configure input mapping and controller settings");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("Input Mapping"), move |ui| {
            Self::render_option_item(
                ui,
                "-uimodekey",
                "key to enable/disable MAME controls when emulated system has keyboard inputs",
                |ui| {
                    let mut ui_mode_key = properties.osd_options.ui_mode_key.clone().unwrap_or_default();
                    if ui.text_edit_singleline(&mut ui_mode_key).changed() {
                        properties.osd_options.ui_mode_key = if ui_mode_key.is_empty() {
                            None
                        } else {
                            Some(ui_mode_key)
                        };
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-controller_map",
                "game controller mapping file",
                |ui| {
                    let mut map_file = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut map_file);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-background_input",
                "don't ignore input when losing UI focus",
                |ui| {
                    ui.checkbox(&mut properties.osd_options.background_input, "Enabled");
                }
            );
        });
    }

    fn show_osd_fonts(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Font Options");
        ui.label("Configure font provider settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Font Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-uifontprovider",
                "provider for UI font: sdl or none",
                |ui| {
                    let mut provider = "sdl";
                    egui::ComboBox::from_id_salt("uifontprovider")
                    .selected_text(provider)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut provider, "sdl", "SDL");
                        ui.selectable_value(&mut provider, "none", "None");
                    });
                }
            );
        });
    }

    fn show_osd_output(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Output Options");
        ui.label("Configure output notification settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Output Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-output",
                "provider for output notifications: none, console or network",
                |ui| {
                    let mut output = "none";
                    egui::ComboBox::from_id_salt("output")
                    .selected_text(output)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut output, "none", "None");
                        ui.selectable_value(&mut output, "console", "Console");
                        ui.selectable_value(&mut output, "network", "Network");
                    });
                }
            );
        });
    }

    fn show_osd_input_providers(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Input Provider Options");
        ui.label("Configure input device providers");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("Input Providers"), move |ui| {
            Self::render_option_item(
                ui,
                "-keyboardprovider",
                "provider for keyboard input: sdl or none",
                |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut properties.osd_options.keyboard_provider, OSDProvider::SDL, "SDL");
                        ui.radio_value(&mut properties.osd_options.keyboard_provider, OSDProvider::None, "None");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-mouseprovider",
                "provider for mouse input: sdl or none",
                |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut properties.osd_options.mouse_provider, OSDProvider::SDL, "SDL");
                        ui.radio_value(&mut properties.osd_options.mouse_provider, OSDProvider::None, "None");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-lightgunprovider",
                "provider for lightgun input: sdl, x11 or none",
                |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut properties.osd_options.lightgun_provider, LightgunProvider::SDL, "SDL");
                        ui.radio_value(&mut properties.osd_options.lightgun_provider, LightgunProvider::X11, "X11");
                        ui.radio_value(&mut properties.osd_options.lightgun_provider, LightgunProvider::None, "None");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-joystickprovider",
                "provider for joystick input: sdlgame, sdljoy or none",
                |ui| {
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut properties.osd_options.joystick_provider, JoystickProvider::SDLGame, "SDL Game");
                        ui.radio_value(&mut properties.osd_options.joystick_provider, JoystickProvider::SDLJoy, "SDL Joy");
                        ui.radio_value(&mut properties.osd_options.joystick_provider, JoystickProvider::None, "None");
                    });
                }
            );
        });
    }

    fn show_osd_debugging(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Debugging Options");
        ui.label("Configure OSD-specific debugging features");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("OSD Debug Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-debugger",
                "debugger used: qt, imgui, gdbstub or none",
                |ui| {
                    let mut debugger = "none";
                    egui::ComboBox::from_id_salt("debugger")
                    .selected_text(debugger)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut debugger, "none", "None");
                        ui.selectable_value(&mut debugger, "qt", "Qt");
                        ui.selectable_value(&mut debugger, "imgui", "ImGui");
                        ui.selectable_value(&mut debugger, "gdbstub", "GDB Stub");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-debugger_host",
                "address to bind to for gdbstub debugger",
                |ui| {
                    let mut host = String::from("127.0.0.1");
                    ui.text_edit_singleline(&mut host);
                }
            );

            Self::render_option_item(
                ui,
                "-debugger_port",
                "port to use for gdbstub debugger",
                |ui| {
                    let mut port = 23946;
                    ui.add(egui::DragValue::new(&mut port).range(1024..=65535));
                }
            );

            Self::render_option_item(
                ui,
                "-debugger_font",
                "font to use for debugger views",
                |ui| {
                    let mut font = String::new();
                    ui.text_edit_singleline(&mut font);
                }
            );

            Self::render_option_item(
                ui,
                "-debugger_font_size",
                "font size to use for debugger views",
                |ui| {
                    let mut font_size = 12;
                    ui.add(egui::DragValue::new(&mut font_size).range(8..=24));
                }
            );

            Self::render_option_item(
                ui,
                "-watchdog",
                "force the program to terminate if no updates within specified number of seconds",
                |ui| {
                    let mut watchdog = 0;
                    ui.add(egui::DragValue::new(&mut watchdog).range(0..=300));
                }
            );
        });
    }

    fn show_osd_performance(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Performance Options");
        ui.label("Configure OSD performance settings");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("Performance Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-numprocessors",
                "number of processors; this overrides the number the system reports",
                |ui| {
                    if let Some(num_procs) = &mut properties.miscellaneous.num_processors {
                        ui.add(egui::DragValue::new(num_procs).range(1..=64));
                    } else {
                        let mut num = 0;
                        ui.add(egui::DragValue::new(&mut num).range(0..=64));
                    }
                }
            );

            Self::render_option_item(
                ui,
                "-bench",
                "benchmark for the given number of emulated seconds; implies -video none -sound none -nothrottle",
                |ui| {
                    let mut bench = 0;
                    ui.add(egui::DragValue::new(&mut bench).range(0..=300));
                }
            );
        });
    }

    fn show_osd_video(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Video Options");
        ui.label("Configure video output and display settings");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("Video Output"), move |ui| {
            Self::render_option_item(
                ui,
                "-video",
                "video output method: opengl, bgfx, accel, soft or none",
                |ui| {
                    egui::ComboBox::from_id_salt("video")
                    .selected_text(properties.display.video_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut properties.display.video_mode, VideoMode::OpenGL, "OpenGL");
                        ui.selectable_value(&mut properties.display.video_mode, VideoMode::BGFX, "BGFX");
                        ui.selectable_value(&mut properties.display.video_mode, VideoMode::Software, "Software");
                        ui.selectable_value(&mut properties.display.video_mode, VideoMode::Auto, "Auto");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-numscreens",
                "number of output screens/windows to create; usually, you want just one",
                |ui| {
                    ui.add(egui::DragValue::new(&mut properties.miscellaneous.num_screens).range(1..=4));
                }
            );

            Self::render_option_item(
                ui,
                "-window",
                "enable window mode; otherwise, full screen mode is assumed",
                |ui| {
                    ui.checkbox(&mut properties.display.run_in_window, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-maximize",
                "default to maximized windows",
                |ui| {
                    ui.checkbox(&mut properties.display.start_out_maximized, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-waitvsync",
                "enable waiting for the start of VBLANK before flipping screens (reduces tearing effects)",
                                     |ui| {
                                         ui.checkbox(&mut properties.screen.wait_for_vertical_sync, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-syncrefresh",
                "enable using the start of VBLANK for throttling instead of the game time",
                |ui| {
                    ui.checkbox(&mut properties.screen.sync_to_monitor_refresh, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-monitorprovider",
                "monitor discovery method: sdl",
                |ui| {
                    let mut provider = String::from("sdl");
                    ui.text_edit_singleline(&mut provider);
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Per-Window Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-screen",
                "explicit name of all screens; 'auto' here will try to make a best guess",
                |ui| {
                    let mut screen = String::from("auto");
                    ui.text_edit_singleline(&mut screen);
                }
            );

            Self::render_option_item(
                ui,
                "-aspect",
                "aspect ratio for all screens; 'auto' here will try to make a best guess",
                |ui| {
                    let mut aspect = String::from("auto");
                    ui.text_edit_singleline(&mut aspect);
                }
            );

            Self::render_option_item(
                ui,
                "-resolution",
                "preferred resolution for all screens; format is <width>x<height>[@<refreshrate>] or 'auto'",
                |ui| {
                    let mut resolution = String::from("auto");
                    ui.text_edit_singleline(&mut resolution);
                }
            );

            Self::render_option_item(
                ui,
                "-view",
                "preferred view for all screens",
                |ui| {
                    let mut view = String::new();
                    ui.text_edit_singleline(&mut view);
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Full Screen Options"), |ui| {
            Self::render_option_item(
                ui,
                "-switchres",
                "enable resolution switching",
                |ui| {
                    let mut switchres = false;
                    ui.checkbox(&mut switchres, "Enabled");
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Accelerated Video Options"), |ui| {
            Self::render_option_item(
                ui,
                "-filter",
                "use bilinear filtering when scaling emulated video",
                |ui| {
                    let mut filter = true;
                    ui.checkbox(&mut filter, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-prescale",
                "scale emulated video by this factor before applying filters/shaders",
                |ui| {
                    let mut prescale = 1;
                    ui.add(egui::DragValue::new(&mut prescale).range(1..=3));
                }
            );
        });
    }

    fn show_osd_sound(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Sound Options");
        ui.label("Configure audio output backend settings");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("Sound Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-sound",
                "sound output method: sdl, portaudio, pulse, pipewire or none",
                |ui| {
                    egui::ComboBox::from_id_salt("sound")
                    .selected_text(properties.sound.sound_mode.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut properties.sound.sound_mode, SoundMode::SDL, "SDL");
                        ui.selectable_value(&mut properties.sound.sound_mode, SoundMode::PortAudio, "PortAudio");
                        ui.selectable_value(&mut properties.sound.sound_mode, SoundMode::PulseAudio, "PulseAudio");
                        ui.selectable_value(&mut properties.sound.sound_mode, SoundMode::None, "None");
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-audio_latency",
                "audio latency, 0 for default (increase to reduce glitches, decrease for responsiveness)",
                                     |ui| {
                                         ui.add(egui::Slider::new(&mut properties.sound.audio_latency, 0.0..=5.0));
                                     }
            );
        });
    }

    fn show_osd_midi(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD MIDI Options");
        ui.label("Configure MIDI I/O settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("MIDI Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-midiprovider",
                "MIDI I/O method: pm or none",
                |ui| {
                    let mut provider = "none";
                    egui::ComboBox::from_id_salt("midiprovider")
                    .selected_text(provider)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut provider, "pm", "PortMIDI");
                        ui.selectable_value(&mut provider, "none", "None");
                    });
                }
            );
        });
    }

    fn show_osd_network(&mut self, ui: &mut egui::Ui) {
        ui.heading("OSD Emulated Networking Options");
        ui.label("Configure emulated networking settings");
        ui.add_space(16.0);

        Self::render_option_group(ui, Some("Network Settings"), |ui| {
            Self::render_option_item(
                ui,
                "-networkprovider",
                "Emulated networking provider: taptun or none",
                |ui| {
                    let mut provider = "none";
                    egui::ComboBox::from_id_salt("networkprovider")
                    .selected_text(provider)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut provider, "taptun", "TAP/TUN");
                        ui.selectable_value(&mut provider, "none", "None");
                    });
                }
            );
        });
    }

    fn show_opengl(&mut self, ui: &mut egui::Ui) {
        ui.heading("OpenGL-Specific Options");
        ui.label("Configure OpenGL rendering settings");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("OpenGL Features"), move |ui| {
            Self::render_option_item(
                ui,
                "-gl_forcepow2texture",
                "force power-of-two texture sizes (default no)",
                                     |ui| {
                                         ui.checkbox(&mut properties.advanced.force_power_of_two_textures, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_notexturerect",
                "don't use OpenGL GL_ARB_texture_rectangle (default on)",
                                     |ui| {
                                         ui.checkbox(&mut properties.advanced.dont_use_gl_arb_texture_rectangle, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_vbo",
                "enable OpenGL VBO if available (default on)",
                                     |ui| {
                                         ui.checkbox(&mut properties.advanced.enable_vbo, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_pbo",
                "enable OpenGL PBO if available (default on)",
                                     |ui| {
                                         ui.checkbox(&mut properties.advanced.enable_pbo, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_glsl",
                "enable OpenGL GLSL if available (default off)",
                                     |ui| {
                                         ui.checkbox(&mut properties.advanced.enable_glsl, "Enabled");
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_glsl_filter",
                "enable OpenGL GLSL filtering instead of FF filtering",
                |ui| {
                    egui::ComboBox::from_id_salt("gl_glsl_filter")
                    .selected_text(match properties.advanced.glsl_filter {
                        GLSLFilter::Plain => "Plain",
                        GLSLFilter::Bilinear => "Bilinear",
                        GLSLFilter::Bicubic => "Bicubic",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut properties.advanced.glsl_filter, GLSLFilter::Plain, "Plain");
                        ui.selectable_value(&mut properties.advanced.glsl_filter, GLSLFilter::Bilinear, "Bilinear");
                        ui.selectable_value(&mut properties.advanced.glsl_filter, GLSLFilter::Bicubic, "Bicubic");
                    });
                }
            );
        });

        ui.add_space(16.0);

        Self::render_option_group(ui, Some("GLSL Shaders"), |ui| {
            Self::render_option_item(
                ui,
                "-glsl_shader_mame0",
                "custom OpenGL GLSL shader set mame bitmap 0",
                |ui| {
                    let mut shader_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut shader_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-glsl_shader_mame1",
                "custom OpenGL GLSL shader set mame bitmap 1",
                |ui| {
                    let mut shader_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut shader_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-glsl_shader_screen0",
                "custom OpenGL GLSL shader screen bitmap 0",
                |ui| {
                    let mut shader_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut shader_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-glsl_shader_screen1",
                "custom OpenGL GLSL shader screen bitmap 1",
                |ui| {
                    let mut shader_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut shader_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );
        });
    }

    fn show_bgfx(&mut self, ui: &mut egui::Ui) {
        ui.heading("BGFX Post-Processing Options");
        ui.label("Configure BGFX rendering backend and effects");
        ui.add_space(16.0);

        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("BGFX Settings"), move |ui| {
            Self::render_option_item(
                ui,
                "-bgfx_path",
                "path to BGFX-related files",
                |ui| {
                    let mut bgfx_path = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut bgfx_path);
                        if ui.button("Browse").clicked() {
                            // TODO: Open directory dialog
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_backend",
                "BGFX backend to use",
                |ui| {
                    egui::ComboBox::from_id_salt("bgfx_backend")
                    .selected_text(properties.advanced.bgfx_settings.backend.to_string())
                    .show_ui(ui, |ui| {
                        for backend in BGFXBackend::available_backends() {
                            ui.selectable_value(&mut properties.advanced.bgfx_settings.backend, backend, backend.to_string());
                        }
                    });
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_debug",
                "enable BGFX debugging statistics",
                |ui| {
                    ui.checkbox(&mut properties.advanced.bgfx_settings.enable_debug, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_screen_chains",
                "comma-delimited list of screen chain JSON names, colon-delimited per-window",
                |ui| {
                    ui.text_edit_singleline(&mut properties.advanced.bgfx_settings.screen_chains);
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_shadow_mask",
                "shadow mask texture name",
                |ui| {
                    let mut shadow_mask = String::new();
                    ui.text_edit_singleline(&mut shadow_mask);
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_lut",
                "LUT texture name",
                |ui| {
                    let mut lut = String::new();
                    ui.text_edit_singleline(&mut lut);
                }
            );

            Self::render_option_item(
                ui,
                "-bgfx_avi_name",
                "filename for BGFX output logging",
                |ui| {
                    let mut avi_name = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut avi_name);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );
        });
    }

    fn show_sdl(&mut self, ui: &mut egui::Ui) {
        ui.heading("SDL-Specific Options");
        ui.label("Configure SDL-specific settings");
        ui.add_space(16.0);

        // Get properties reference for first group
        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("SDL Performance"), move |ui| {
            Self::render_option_item(
                ui,
                "-sdlvideofps",
                "show SDL video performance",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.show_video_fps, "Enabled");
                }
            );
        });

        ui.add_space(16.0);

        // Get fresh properties reference for second group
        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("SDL Video"), move |ui| {
            Self::render_option_item(
                ui,
                "-centerh",
                "center horizontally within the view area",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.center_horizontal, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-centerv",
                "center vertically within the view area",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.center_vertical, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-scalemode",
                "Scale mode: none, hwblit, hwbest, yv12, yuy2, yv12x2, yuy2x2 (-video soft only)",
                                     |ui| {
                                         egui::ComboBox::from_id_salt("scalemode")
                                         .selected_text(properties.sdl_options.scale_mode.to_string())
                                         .show_ui(ui, |ui| {
                                             ui.selectable_value(&mut properties.sdl_options.scale_mode, SDLScaleMode::None, "None");
                                             ui.selectable_value(&mut properties.sdl_options.scale_mode, SDLScaleMode::HWBlit, "Hardware Blit");
                                             ui.selectable_value(&mut properties.sdl_options.scale_mode, SDLScaleMode::HWBest, "Hardware Best");
                                             ui.selectable_value(&mut properties.sdl_options.scale_mode, SDLScaleMode::YV12, "YV12");
                                             ui.selectable_value(&mut properties.sdl_options.scale_mode, SDLScaleMode::YUY2, "YUY2");
                                         });
                                     }
            );
        });

        ui.add_space(16.0);

        // SDL Full Screen options
        Self::render_option_group(ui, Some("SDL Full Screen"), |ui| {
            Self::render_option_item(
                ui,
                "-useallheads",
                "split full screen image across monitors",
                |ui| {
                    let mut useallheads = false;
                    ui.checkbox(&mut useallheads, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-attach_window",
                "attach to arbitrary window",
                |ui| {
                    let mut window_handle = String::new();
                    ui.text_edit_singleline(&mut window_handle);
                }
            );
        });

        ui.add_space(16.0);

        // SDL Keyboard Mapping
        Self::render_option_group(ui, Some("SDL Keyboard Mapping"), |ui| {
            Self::render_option_item(
                ui,
                "-keymap",
                "enable keymap",
                |ui| {
                    let mut keymap = false;
                    ui.checkbox(&mut keymap, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-keymap_file",
                "keymap filename",
                |ui| {
                    let mut keymap_file = String::new();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut keymap_file);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );
        });

        ui.add_space(16.0);

        // Get fresh properties reference for SDL Input group
        let properties = &mut self.properties;

        Self::render_option_group(ui, Some("SDL Input"), move |ui| {
            Self::render_option_item(
                ui,
                "-enable_touch",
                "enable touch input support",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.enable_touch, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-sixaxis",
                "use special handling for PS3 Sixaxis controllers",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.sixaxis_support, "Enabled");
                }
            );

            Self::render_option_item(
                ui,
                "-dual_lightgun",
                "enable dual lightgun input",
                |ui| {
                    ui.checkbox(&mut properties.sdl_options.dual_lightgun, "Enabled");
                }
            );
        });

        ui.add_space(16.0);

        // SDL Lightgun Mapping
        Self::render_option_group(ui, Some("SDL Lightgun Mapping"), |ui| {
            Self::render_option_item(
                ui,
                "-lightgun_index1",
                "name of lightgun mapped to lightgun #1",
                |ui| {
                    let mut lightgun1 = String::new();
                    ui.text_edit_singleline(&mut lightgun1);
                }
            );

            Self::render_option_item(
                ui,
                "-lightgun_index2",
                "name of lightgun mapped to lightgun #2",
                |ui| {
                    let mut lightgun2 = String::new();
                    ui.text_edit_singleline(&mut lightgun2);
                }
            );
        });

        ui.add_space(16.0);

        // SDL Low-Level Drivers
        Self::render_option_group(ui, Some("SDL Low-Level Drivers"), |ui| {
            Self::render_option_item(
                ui,
                "-videodriver",
                "SDL video driver to use ('x11', 'directfb', ... or 'auto' for SDL default",
                                     |ui| {
                                         let mut videodriver = String::from("auto");
                                         ui.text_edit_singleline(&mut videodriver);
                                     }
            );

            Self::render_option_item(
                ui,
                "-renderdriver",
                "SDL render driver to use ('software', 'opengl', 'directfb' ... or 'auto' for SDL default",
                                     |ui| {
                                         let mut renderdriver = String::from("auto");
                                         ui.text_edit_singleline(&mut renderdriver);
                                     }
            );

            Self::render_option_item(
                ui,
                "-audiodriver",
                "SDL audio driver to use ('alsa', 'arts', ... or 'auto' for SDL default",
                                     |ui| {
                                         let mut audiodriver = String::from("auto");
                                         ui.text_edit_singleline(&mut audiodriver);
                                     }
            );

            Self::render_option_item(
                ui,
                "-gl_lib",
                "alternative libGL.so to use; 'auto' for system default",
                |ui| {
                    let mut gl_lib = String::from("auto");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut gl_lib);
                        if ui.button("Browse").clicked() {
                            // TODO: Open file dialog
                        }
                    });
                }
            );
        });
    }

    // Helper methods
    fn update_command_preview(&mut self) {
        let mut parts = vec!["mame".to_string()];

        // Add relevant command line options based on settings
        if !self.properties.display.run_in_window {
            parts.push("-window 0".to_string());
        }

        if self.properties.display.start_out_maximized {
            parts.push("-maximize".to_string());
        }

        if !self.properties.display.throttle {
            parts.push("-nothrottle".to_string());
        }

        if self.properties.display.video_mode != VideoMode::Auto {
            parts.push(format!("-video {}", self.properties.display.video_mode.to_string().to_lowercase()));
        }

        // Rotation options
        match self.properties.display.rotation {
            RotationMode::Rotate90 => parts.push("-ror".to_string()),
            RotationMode::Rotate180 => {
                parts.push("-ror".to_string());
                parts.push("-ror".to_string());
            }
            RotationMode::Rotate270 => parts.push("-rol".to_string()),
            _ => {}
        }

        if self.properties.display.flip_screen_upside_down {
            parts.push("-flipy".to_string());
        }

        if self.properties.display.flip_screen_left_right {
            parts.push("-flipx".to_string());
        }

        if self.properties.display.auto_rotate_right {
            parts.push("-autoror".to_string());
        }

        if self.properties.display.auto_rotate_left {
            parts.push("-autorol".to_string());
        }

        // Core Configuration options
        if self.properties.miscellaneous.write_config {
            parts.push("-writeconfig".to_string());
        }

        if self.properties.miscellaneous.auto_save {
            parts.push("-autosave".to_string());
        }

        if self.properties.miscellaneous.rewind {
            parts.push("-rewind".to_string());
        }

        if self.properties.miscellaneous.exit_after {
            parts.push("-exit_after".to_string());
        }

        if self.properties.miscellaneous.bilinear {
            parts.push("-bilinear".to_string());
        }

        if self.properties.miscellaneous.burnin {
            parts.push("-burnin".to_string());
        }

        if self.properties.miscellaneous.crop {
            parts.push("-crop".to_string());
        }

        // Core Input options
        if self.properties.miscellaneous.multi_keyboard {
            parts.push("-multikeyboard".to_string());
        }

        if self.properties.miscellaneous.multi_mouse {
            parts.push("-multimouse".to_string());
        }

        if self.properties.miscellaneous.steady_key {
            parts.push("-steadykey".to_string());
        }

        if self.properties.miscellaneous.ui_active {
            parts.push("-ui_active".to_string());
        }

        if self.properties.miscellaneous.offscreen_reload {
            parts.push("-offscreen_reload".to_string());
        }

        if self.properties.miscellaneous.contradictory {
            parts.push("-joystick_contradictory".to_string());
        }

        if self.properties.miscellaneous.natural {
            parts.push("-natural".to_string());
        }

        // Core Debug options
        if self.properties.miscellaneous.verbose {
            parts.push("-verbose".to_string());
        }

        if self.properties.miscellaneous.log {
            parts.push("-log".to_string());
        }

        if self.properties.miscellaneous.oslog {
            parts.push("-oslog".to_string());
        }

        if self.properties.miscellaneous.debug {
            parts.push("-debug".to_string());
        }

        if self.properties.miscellaneous.update_pause {
            parts.push("-update_in_pause".to_string());
        }

        if self.properties.miscellaneous.debuglog {
            parts.push("-debuglog".to_string());
        }

        if self.properties.miscellaneous.drc_c {
            parts.push("-drc_use_c".to_string());
        }

        if self.properties.miscellaneous.log_uml {
            parts.push("-drc_log_uml".to_string());
        }

        if self.properties.miscellaneous.log_native {
            parts.push("-drc_log_native".to_string());
        }

        if self.properties.miscellaneous.cheat {
            parts.push("-cheat".to_string());
        }

        if self.properties.miscellaneous.skip {
            parts.push("-skip_gameinfo".to_string());
        }

        if self.properties.miscellaneous.confirm {
            parts.push("-confirm_quit".to_string());
        }

        if self.properties.miscellaneous.console {
            parts.push("-console".to_string());
        }

        if self.properties.miscellaneous.switchres {
            parts.push("-switchres".to_string());
        }

        self.command_preview = parts.join(" ");
    }

    fn reset_to_defaults(&mut self) {
        self.properties = GameProperties::default();
        self.is_dirty = true;
    }

    fn export_settings(&self) {
        // TODO: Implement export functionality
        // This would serialize the properties to JSON and save to file
    }

    fn import_settings(&mut self) {
        // TODO: Implement import functionality
        // This would load JSON from file and deserialize to properties
    }
}

// Implement PartialEq for GameProperties to track changes
impl PartialEq for GameProperties {
    fn eq(&self, other: &Self) -> bool {
        // Compare all relevant fields for proper change detection
        self.display.run_in_window == other.display.run_in_window &&
        self.display.start_out_maximized == other.display.start_out_maximized &&
        self.display.throttle == other.display.throttle &&
        self.display.video_mode == other.display.video_mode &&
        self.display.rotation == other.display.rotation &&
        self.display.flip_screen_left_right == other.display.flip_screen_left_right &&
        self.display.flip_screen_upside_down == other.display.flip_screen_upside_down &&
        self.display.enforce_aspect_ratio == other.display.enforce_aspect_ratio &&
        self.display.use_non_integer_scaling == other.display.use_non_integer_scaling &&
        self.display.stretch_only_x_axis == other.display.stretch_only_x_axis &&
        self.display.stretch_only_y_axis == other.display.stretch_only_y_axis &&
        self.display.auto_select_stretch_axis == other.display.auto_select_stretch_axis &&
        self.display.overscan_on_targets == other.display.overscan_on_targets &&
        self.display.horizontal_scale_factor == other.display.horizontal_scale_factor &&
        self.display.vertical_scale_factor == other.display.vertical_scale_factor &&
        self.display.brightness_correction == other.display.brightness_correction &&
        self.display.contrast_correction == other.display.contrast_correction &&
        self.display.gamma_correction == other.display.gamma_correction &&
        self.display.pause_brightness == other.display.pause_brightness &&
        self.display.auto_rotate_right == other.display.auto_rotate_right &&
        self.display.auto_rotate_left == other.display.auto_rotate_left &&
        self.screen.auto_frameskip == other.screen.auto_frameskip &&
        self.screen.frameskip_value == other.screen.frameskip_value &&
        self.screen.seconds_to_run == other.screen.seconds_to_run &&
        self.screen.sleep_when_idle == other.screen.sleep_when_idle &&
        self.screen.emulation_speed == other.screen.emulation_speed &&
        self.screen.refresh_speed == other.screen.refresh_speed &&
        self.screen.low_latency == other.screen.low_latency &&
        self.sound.sample_rate == other.sound.sample_rate &&
        self.sound.use_samples == other.sound.use_samples &&
        self.sound.volume_attenuation == other.sound.volume_attenuation &&
        // Core Configuration options
        self.miscellaneous.write_config == other.miscellaneous.write_config &&
        self.miscellaneous.auto_save == other.miscellaneous.auto_save &&
        self.miscellaneous.rewind == other.miscellaneous.rewind &&
        self.miscellaneous.exit_after == other.miscellaneous.exit_after &&
        self.miscellaneous.bilinear == other.miscellaneous.bilinear &&
        self.miscellaneous.burnin == other.miscellaneous.burnin &&
        self.miscellaneous.crop == other.miscellaneous.crop &&
        // Core Input options
        self.miscellaneous.multi_keyboard == other.miscellaneous.multi_keyboard &&
        self.miscellaneous.multi_mouse == other.miscellaneous.multi_mouse &&
        self.miscellaneous.steady_key == other.miscellaneous.steady_key &&
        self.miscellaneous.ui_active == other.miscellaneous.ui_active &&
        self.miscellaneous.offscreen_reload == other.miscellaneous.offscreen_reload &&
        self.miscellaneous.contradictory == other.miscellaneous.contradictory &&
        self.miscellaneous.natural == other.miscellaneous.natural &&
        // Core Debug options
        self.miscellaneous.verbose == other.miscellaneous.verbose &&
        self.miscellaneous.log == other.miscellaneous.log &&
        self.miscellaneous.oslog == other.miscellaneous.oslog &&
        self.miscellaneous.debug == other.miscellaneous.debug &&
        self.miscellaneous.update_pause == other.miscellaneous.update_pause &&
        self.miscellaneous.debuglog == other.miscellaneous.debuglog &&
        self.miscellaneous.drc_c == other.miscellaneous.drc_c &&
        self.miscellaneous.log_uml == other.miscellaneous.log_uml &&
        self.miscellaneous.log_native == other.miscellaneous.log_native &&
        self.miscellaneous.cheat == other.miscellaneous.cheat &&
        self.miscellaneous.skip == other.miscellaneous.skip &&
        self.miscellaneous.confirm == other.miscellaneous.confirm &&
        self.miscellaneous.console == other.miscellaneous.console &&
        self.miscellaneous.switchres == other.miscellaneous.switchres
    }
}
