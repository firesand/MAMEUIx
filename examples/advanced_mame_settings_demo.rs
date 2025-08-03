// examples/advanced_mame_settings_demo.rs
// Demo-only: Advanced MAME Settings dialog (standalone) using egui 0.32
// This file is not integrated into MAMEUIx runtime; run with: cargo run --example advanced_mame_settings_demo

use eframe::egui;
use eframe::egui::{Color32, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Vec2};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Advanced MAME Settings Demo",
        options,
        Box::new(|_cc| Ok(Box::<AdvancedMameSettingsDemo>::default())),
    )
}

#[derive(Default)]
struct AdvancedMameSettingsDemo {
    settings_dialog: AdvancedMameSettings,
}

impl eframe::App for AdvancedMameSettingsDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Advanced MAME Settings Demo");
            ui.add_space(20.0);
            
            ui.label("This demo shows the Advanced MAME Settings dialog implementation.");
            ui.add_space(10.0);
            
            if ui.button("Open Advanced MAME Settings").clicked() {
                self.settings_dialog.open();
            }
            
            ui.add_space(20.0);
            
            ui.label("Features implemented:");
            ui.label("• Dark theme with modern UI design");
            ui.label("• Categories sidebar with icons");
            ui.label("• Search functionality");
            ui.label("• Tab navigation");
            ui.label("• Responsive layout");
            ui.label("• Footer with action buttons");
        });

        // Show the settings dialog
        self.settings_dialog.show(ctx);
    }
}

#[derive(Default)]
struct AdvancedMameSettings {
    is_open: bool,
    selected_category: String,
    active_tab: String,
    search_query: String,
}

impl AdvancedMameSettings {
    pub fn new() -> Self {
        let mut settings = Self::default();
        settings.selected_category = "paths".to_string();
        settings.active_tab = "search-paths".to_string();
        settings
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        // Dark overlay
        let screen_rect = ctx.screen_rect();
        egui::Area::new(egui::Id::new("settings_overlay"))
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen_rect,
                    Rounding::ZERO,
                    Color32::from_black_alpha(200),
                );
            });

        // Main dialog
        egui::Window::new("Advanced MAME Settings")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_size(Vec2::new(1100.0, 720.0))
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(egui::Frame {
                fill: Color32::from_rgb(22, 22, 22),
                corner_radius: egui::epaint::CornerRadius::same(12),
                stroke: Stroke::new(1.0, Color32::from_rgb(51, 51, 51)),
                ..Default::default()
            })
            .show(ctx, |ui| {
                self.render_dialog_content(ui);
            });
    }

    fn render_dialog_content(&mut self, ui: &mut egui::Ui) {
        // Header
        self.render_header(ui);
        ui.add_space(1.0);

        // Main content area
        ui.horizontal(|ui| {
            // Categories sidebar
            self.render_categories_sidebar(ui);

            // Settings panel
            self.render_settings_panel(ui);
        });

        ui.add_space(1.0);

        // Footer
        self.render_footer(ui);
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        let header_rect = ui.available_rect_before_wrap();
        let header_rect = Rect::from_min_size(header_rect.min, Vec2::new(header_rect.width(), 56.0));

        ui.allocate_ui_at_rect(header_rect, |ui| {
            ui.painter().rect_filled(
                header_rect,
                Rounding {
                    nw: 12,
                    ne: 12,
                    sw: 0,
                    se: 0,
                },
                Color32::from_rgb(30, 30, 30),
            );

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                ui.add_space(20.0);

                ui.label(
                    RichText::new("⚙️ Advanced MAME Settings")
                        .size(18.0)
                        .color(Color32::from_rgb(255, 255, 255))
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.0);

                    let close_button = ui.add_sized(
                        Vec2::new(32.0, 32.0),
                        egui::Button::new(
                            RichText::new("×")
                                .size(20.0)
                                .color(Color32::from_rgb(136, 136, 136))
                        )
                        .fill(Color32::from_rgb(42, 42, 42))
                        .stroke(Stroke::NONE)
                        .rounding(Rounding::same(6))
                    );

                    if close_button.clicked() {
                        self.is_open = false;
                    }

                    if close_button.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                });
            });
        });

        ui.add_space(56.0);
    }

    fn render_categories_sidebar(&mut self, ui: &mut egui::Ui) {
        let sidebar_width = 260.0;
        let sidebar_rect = ui.available_rect_before_wrap();
        let sidebar_rect = Rect::from_min_size(
            sidebar_rect.min,
            Vec2::new(sidebar_width, sidebar_rect.height() - 120.0),
        );

        ui.allocate_ui_at_rect(sidebar_rect, |ui| {
            // Background
            ui.painter().rect_filled(
                sidebar_rect,
                Rounding::ZERO,
                Color32::from_rgb(15, 15, 15),
            );

            // Border
            ui.painter().line_segment(
                [
                    Pos2::new(sidebar_rect.max.x, sidebar_rect.min.y),
                    Pos2::new(sidebar_rect.max.x, sidebar_rect.max.y),
                ],
                Stroke::new(1.0, Color32::from_rgb(51, 51, 51)),
            );

            ui.add_space(16.0);

            // Search box
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let search_width = sidebar_width - 40.0;

                ui.add_sized(
                    Vec2::new(search_width, 36.0),
                    egui::TextEdit::singleline(&mut self.search_query)
                        .desired_width(search_width)
                        .hint_text("Search settings...")
                        .frame(true)
                );
            });

            ui.add_space(16.0);

            // Categories
            let categories = vec![
                ("paths", "📁", "Paths & Directories"),
                ("input", "🎮", "Input & Controls"),
                ("video", "🖼️", "Video & Display"),
                ("audio", "🔊", "Audio"),
                ("performance", "⚡", "Performance"),
                ("artwork", "🎨", "Artwork & Effects"),
                ("state", "💾", "State & Recording"),
                ("network", "🌐", "Network & Server"),
                ("debugging", "🔧", "Debugging"),
                ("scripting", "📝", "Scripting & Plugins"),
                ("opengl", "🎯", "OpenGL & Shaders"),
                ("sdl", "🖥️", "SDL Options"),
                ("misc", "⚙️", "Miscellaneous"),
            ];

            for (id, icon, label) in categories {
                let is_active = self.selected_category == id;

                let item_rect = ui.available_rect_before_wrap();
                let item_rect = Rect::from_min_size(
                    item_rect.min,
                    Vec2::new(sidebar_width, 44.0),
                );

                let response = ui.allocate_rect(item_rect, Sense::click());

                // Hover effect
                if response.hovered() && !is_active {
                    ui.painter().rect_filled(
                        item_rect,
                        Rounding::ZERO,
                        Color32::from_rgb(26, 26, 26),
                    );
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                // Active state
                if is_active {
                    ui.painter().rect_filled(
                        item_rect,
                        Rounding::ZERO,
                        Color32::from_rgb(42, 42, 42),
                    );

                    // Active indicator
                    let indicator_rect = Rect::from_min_size(
                        item_rect.min,
                        Vec2::new(3.0, item_rect.height()),
                    );
                    ui.painter().rect_filled(
                        indicator_rect,
                        Rounding::ZERO,
                        Color32::from_rgb(74, 158, 255),
                    );
                }

                // Icon and text
                ui.painter().text(
                    Pos2::new(item_rect.min.x + 20.0, item_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    FontId::proportional(16.0),
                    Color32::from_rgb(255, 255, 255),
                );

                ui.painter().text(
                    Pos2::new(item_rect.min.x + 50.0, item_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(14.0),
                    if is_active {
                        Color32::from_rgb(74, 158, 255)
                    } else {
                        Color32::from_rgb(224, 224, 224)
                    },
                );

                if response.clicked() {
                    self.selected_category = id.to_string();
                    match id {
                        "paths" => self.active_tab = "search-paths".to_string(),
                        _ => {}
                    }
                }

                ui.add_space(44.0);
            }
        });
    }

    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(260.0); // Account for sidebar width

        let panel_rect = ui.available_rect_before_wrap();
        let panel_rect = Rect::from_min_size(
            panel_rect.min,
            Vec2::new(panel_rect.width() - 20.0, panel_rect.height() - 120.0),
        );

        ui.allocate_ui_at_rect(panel_rect, |ui| {
            // Background
            ui.painter().rect_filled(
                panel_rect,
                Rounding::ZERO,
                Color32::from_rgb(10, 10, 10),
            );

            ui.add_space(24.0);

            // Render content based on selected category
            match self.selected_category.as_str() {
                "paths" => self.render_paths_settings(ui),
                "input" => self.render_input_settings(ui),
                "video" => self.render_video_settings(ui),
                "audio" => self.render_audio_settings(ui),
                "performance" => self.render_performance_settings(ui),
                _ => {
                    ui.horizontal(|ui| {
                        ui.add_space(24.0);
                        ui.label("Settings for this category are not yet implemented.");
                    });
                }
            }
        });
    }

    fn render_paths_settings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(24.0);
            ui.vertical(|ui| {
                // Section title
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Paths & Directories")
                            .size(20.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                });

                ui.add_space(8.0);

                ui.label(
                    RichText::new("Configure where MAME looks for and saves various files")
                        .size(14.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );

                ui.add_space(20.0);

                // Tab bar
                self.render_tab_bar(ui, &[
                    ("search-paths", "Search Paths"),
                    ("output-dirs", "Output Directories"),
                    ("history-info", "History & Info"),
                ]);

                ui.add_space(20.0);

                // Tab content
                match self.active_tab.as_str() {
                    "search-paths" => self.render_search_paths_tab(ui),
                    "output-dirs" => self.render_output_dirs_tab(ui),
                    "history-info" => self.render_history_info_tab(ui),
                    _ => {}
                }
            });
        });
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui, tabs: &[(&str, &str)]) {
        ui.horizontal(|ui| {
            // Tab bar background
            let tab_bar_rect = ui.available_rect_before_wrap();
            let tab_bar_rect = Rect::from_min_size(
                tab_bar_rect.min,
                Vec2::new(400.0, 40.0),
            );

            ui.painter().rect_filled(
                tab_bar_rect,
                Rounding::same(8),
                Color32::from_rgb(26, 26, 26),
            );

            ui.add_space(4.0);

            for (id, label) in tabs {
                let is_active = self.active_tab == *id;

                let tab_response = ui.add_sized(
                    Vec2::new(120.0, 32.0),
                    egui::Button::new(
                        RichText::new(*label)
                            .size(14.0)
                            .color(if is_active {
                                Color32::from_rgb(74, 158, 255)
                            } else {
                                Color32::from_rgb(136, 136, 136)
                            })
                    )
                    .fill(if is_active {
                        Color32::from_rgb(42, 42, 42)
                    } else {
                        Color32::TRANSPARENT
                    })
                    .stroke(Stroke::NONE)
                    .rounding(Rounding::same(6))
                );

                if tab_response.clicked() {
                    self.active_tab = id.to_string();
                }

                if tab_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.add_space(2.0);
            }
        });
    }

    fn render_search_paths_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Search Paths Configuration");
        ui.add_space(10.0);
        ui.label("Configure where MAME looks for ROMs, artwork, and other files");
        ui.add_space(20.0);
        
        // ROM Paths example
        ui.label("ROM Paths (multiple directories supported):");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter ROM directory path...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select ROM Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected ROM folder: {}", folder.display());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // Artwork Path example
        ui.label("Artwork Path:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter artwork directory path...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select Artwork Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected artwork folder: {}", folder.display());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // Snapshot Path example
        ui.label("Snapshot/snap Path:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter snapshot directory path...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select Snapshot Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected snapshot folder: {}", folder.display());
                }
            }
        });
    }

    fn render_output_dirs_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Output Directories Configuration");
        ui.add_space(10.0);
        ui.label("Configure where MAME saves various files");
        ui.add_space(20.0);
        
        // Snapshot Directory example
        ui.label("Snapshot Directory:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter snapshot output directory...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select Snapshot Output Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected snapshot output folder: {}", folder.display());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // Config Directory example
        ui.label("Config Directory:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter config directory...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select Config Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected config folder: {}", folder.display());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // State Directory example
        ui.label("State Directory:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter state directory...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select State Directory");
                
                if let Some(folder) = dialog.pick_folder() {
                    println!("Selected state folder: {}", folder.display());
                }
            }
        });
    }

    fn render_history_info_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("History & Information Files Configuration");
        ui.add_space(10.0);
        ui.label("Configure where MAME saves history and information files.");
        ui.add_space(20.0);
        
        // History File example
        ui.label("History File:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter history file path...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select History File");
                
                if let Some(file) = dialog.pick_file() {
                    println!("Selected history file: {}", file.display());
                }
            }
        });
        
        ui.add_space(16.0);
        
        // Info File example
        ui.label("Info File:");
        ui.horizontal(|ui| {
            let mut path_str = String::new();
            ui.add_sized(
                Vec2::new(300.0, 36.0),
                egui::TextEdit::singleline(&mut path_str)
                    .hint_text("Enter info file path...")
            );
            
            if ui.button("Browse...").clicked() {
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Select Info File");
                
                if let Some(file) = dialog.pick_file() {
                    println!("Selected info file: {}", file.display());
                }
            }
        });
    }

    fn render_input_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Input & Controls settings - to be implemented");
    }

    fn render_video_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Video & Display settings - to be implemented");
    }

    fn render_audio_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Audio settings - to be implemented");
    }

    fn render_performance_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Performance settings - to be implemented");
    }

    fn render_footer(&mut self, ui: &mut egui::Ui) {
        let footer_rect = ui.available_rect_before_wrap();
        let footer_rect = Rect::from_min_size(
            Pos2::new(footer_rect.min.x, footer_rect.max.y - 64.0),
            Vec2::new(footer_rect.width(), 64.0),
        );

        ui.allocate_ui_at_rect(footer_rect, |ui| {
            ui.painter().rect_filled(
                footer_rect,
                Rounding {
                    nw: 0,
                    ne: 0,
                    sw: 12,
                    se: 12,
                },
                Color32::from_rgb(30, 30, 30),
            );

            ui.painter().line_segment(
                [
                    Pos2::new(footer_rect.min.x, footer_rect.min.y),
                    Pos2::new(footer_rect.max.x, footer_rect.min.y),
                ],
                Stroke::new(1.0, Color32::from_rgb(51, 51, 51)),
            );

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                ui.add_space(24.0);

                // Footer info
                ui.label(
                    RichText::new("MAME 0.264 Configuration")
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);

                    // Apply button
                    let apply_btn = ui.add_sized(
                        Vec2::new(140.0, 36.0),
                        egui::Button::new("Apply Changes")
                            .fill(Color32::from_rgb(74, 158, 255))
                            .stroke(Stroke::NONE)
                    );

                    if apply_btn.clicked() {
                        println!("Applying settings...");
                        self.is_open = false;
                    }

                    ui.add_space(12.0);

                    // Cancel button
                    let cancel_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Cancel")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );

                    if cancel_btn.clicked() {
                        self.is_open = false;
                    }

                    ui.add_space(12.0);

                    // Reset button
                    let reset_btn = ui.add_sized(
                        Vec2::new(140.0, 36.0),
                        egui::Button::new("Reset to Defaults")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );

                    if reset_btn.clicked() {
                        println!("Resetting to defaults...");
                    }
                });
            });
        });
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}
