//! Standalone Advanced MAME Settings Viewport Demo
//! 
//! This example demonstrates the Advanced MAME Settings dialog
//! using the viewport API to create a separate native window.
//! This is a standalone version that includes all necessary code.

use eframe::egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2, ViewportId, ViewportBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MameSettings {
    paths: PathSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PathSettings {
    // Search Paths tab
    rom_paths: Vec<String>,
    artwork_path: String,
    snapshot_path: String,
    flyer_path: String,
    marquees_path: String,
    titles_path: String,
    
    // Output Directories tab
    snapshot_dir: String,
    config_dir: String,
    nvram_dir: String,
    
    // History & Information Files tab
    catver_path: String,
    history_path: String,
    mameinfo_path: String,
    command_path: String,
}

impl Default for MameSettings {
    fn default() -> Self {
        Self {
            paths: PathSettings {
                // Search Paths
                rom_paths: vec![
                    String::from("/home/user/roms"),
                    String::from("/home/user/chds"),
                ],
                artwork_path: String::from("/home/user/mame/artwork"),
                snapshot_path: String::from("/home/user/mame/snap"),
                flyer_path: String::from("/home/user/mame/flyers"),
                marquees_path: String::from("/home/user/mame/marquees"),
                titles_path: String::from("/home/user/mame/titles"),
                
                // Output Directories
                snapshot_dir: String::from("/home/user/mame/snapshots"),
                config_dir: String::from("/home/user/.mame/cfg"),
                nvram_dir: String::from("/home/user/.mame/nvram"),
                
                // History & Information Files
                catver_path: String::from("/home/user/mame/catver.ini"),
                history_path: String::from("/home/user/mame/history.xml"),
                mameinfo_path: String::from("/home/user/mame/mameinfo.dat"),
                command_path: String::from("/home/user/mame/command.dat"),
            },
        }
    }
}

/// Main structure for the Advanced MAME Settings dialog with viewport support
struct AdvancedMameSettingsViewport {
    is_open: bool,
    viewport_id: ViewportId,
    selected_category: String,
    active_tab: String,
    settings: MameSettings,
}

impl Default for AdvancedMameSettingsViewport {
    fn default() -> Self {
        Self {
            is_open: false,
            viewport_id: ViewportId::from_hash_of("advanced_mame_settings_window"),
            selected_category: "paths".to_string(),
            active_tab: "search-paths".to_string(),
            settings: MameSettings::default(),
        }
    }
}

impl AdvancedMameSettingsViewport {
    fn new() -> Self {
        Self::default()
    }
    
    fn open(&mut self) {
        self.is_open = true;
    }
    
    fn close(&mut self) {
        self.is_open = false;
    }
    
    fn is_open(&self) -> bool {
        self.is_open
    }
    
    fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }
        
        // Create a separate native window
        ctx.show_viewport_immediate(
            self.viewport_id,
            ViewportBuilder::default()
                .with_title("⚙️ Advanced MAME Settings")
                .with_inner_size([1200.0, 900.0])
                .with_min_inner_size([1000.0, 800.0])
                .with_resizable(true)
                .with_decorations(true) // Native window decorations
                .with_close_button(true),
            |ctx, _class| {
                // Check if window should be closed
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.is_open = false;
                }
                
                egui::CentralPanel::default()
                    .frame(egui::Frame {
                        fill: Color32::from_rgb(22, 22, 22),
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        self.render_dialog_content(ui);
                    });
            },
        );
    }
    
    fn render_dialog_content(&mut self, ui: &mut egui::Ui) {
        let available_rect = ui.available_rect_before_wrap();
        
        // Header height
        let header_height = 56.0;
        let footer_height = 64.0;
        let content_height = available_rect.height() - header_height - footer_height - 2.0;
        
        // Header
        self.render_header(ui);
        ui.add_space(1.0);
        
        // Main content area with proper layout
        let content_rect = Rect::from_min_size(
            Pos2::new(available_rect.min.x, available_rect.min.y + header_height + 1.0),
            Vec2::new(available_rect.width(), content_height),
        );
        
        ui.scope_builder(egui::UiBuilder::new().max_rect(content_rect), |ui| {
            ui.horizontal(|ui| {
                // Sidebar
                let sidebar_width = 260.0;
                ui.allocate_ui_with_layout(
                    Vec2::new(sidebar_width, content_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_categories_sidebar_content(ui);
                    }
                );
                
                // Settings panel
                let panel_width = available_rect.width() - sidebar_width;
                ui.allocate_ui_with_layout(
                    Vec2::new(panel_width, content_height),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        self.render_settings_panel(ui);
                    }
                );
            });
        });
        
        ui.add_space(1.0);
        
        // Footer
        self.render_footer(ui);
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        let header_rect = ui.available_rect_before_wrap();
        let header_rect = Rect::from_min_size(header_rect.min, Vec2::new(header_rect.width(), 56.0));
        
        ui.scope_builder(egui::UiBuilder::new().max_rect(header_rect), |ui| {
            ui.painter().rect_filled(
                header_rect,
                CornerRadius {
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
                        .corner_radius(CornerRadius::same(6))
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
    
    fn render_categories_sidebar_content(&mut self, ui: &mut egui::Ui) {
        let sidebar_rect = ui.max_rect();
        
        // Background
        ui.painter().rect_filled(
            sidebar_rect,
            CornerRadius::ZERO,
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
        
        ui.add_space(20.0);
        
        // Categories
        let categories = vec![
            ("paths", "📁", "Paths & Directories"),
            ("input", "🎮", "Input & Controls"),
            ("video", "🖼️", "Video & Display"),
            ("audio", "🔊", "Audio"),
            ("scripting", "📝", "Scripting & Plugins"),
            ("opengl", "🎯", "OpenGL & Shaders"),
            ("sdl", "🖥️", "SDL Options"),
        ];
        
        for (id, icon, label) in categories {
            let is_active = self.selected_category == id;
            
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(260.0, 44.0),
                Sense::click()
            );
            
            if response.hovered() && !is_active {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::ZERO,
                    Color32::from_rgb(26, 26, 26),
                );
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            
            if is_active {
                ui.painter().rect_filled(
                    rect,
                    CornerRadius::ZERO,
                    Color32::from_rgb(42, 42, 42),
                );
                
                let indicator_rect = Rect::from_min_size(
                    rect.min,
                    Vec2::new(3.0, rect.height()),
                );
                ui.painter().rect_filled(
                    indicator_rect,
                    CornerRadius::ZERO,
                    Color32::from_rgb(74, 158, 255),
                );
            }
            
            ui.painter().text(
                Pos2::new(rect.min.x + 20.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                icon,
                FontId::proportional(16.0),
                Color32::from_rgb(255, 255, 255),
            );
            
            ui.painter().text(
                Pos2::new(rect.min.x + 50.0, rect.center().y),
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
                    _ => self.active_tab = "general".to_string(),
                }
            }
        }
    }
    
    fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        let panel_rect = ui.max_rect();
        
        // Background
        ui.painter().rect_filled(
            panel_rect,
            CornerRadius::ZERO,
            Color32::from_rgb(10, 10, 10),
        );
        
        match self.selected_category.as_str() {
            "paths" => self.render_paths_settings(ui),
            _ => {
                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    ui.add_space(40.0);
                    ui.label("Settings for this category are not yet implemented.");
                });
            }
        }
    }
    
    fn render_paths_settings(&mut self, ui: &mut egui::Ui) {
        ui.add_space(24.0);
        
        ui.horizontal(|ui| {
            ui.add_space(40.0);
            
            ui.vertical(|ui| {
                ui.set_max_width(800.0);
                
                ui.label(
                    RichText::new("Paths & Directories")
                        .size(20.0)
                        .color(Color32::from_rgb(255, 255, 255))
                );
                
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
                    ("history-info", "History & Information Files"),
                ]);
                
                ui.add_space(20.0);
                
                // Tab content without scroll area for more visible content
                match self.active_tab.as_str() {
                    "search-paths" => self.render_search_paths_tab(ui),
                    "output-dirs" => self.render_output_directories_tab(ui),
                    "history-info" => self.render_history_info_tab(ui),
                    _ => {}
                }
            });
        });
    }
    
    fn render_tab_bar(&mut self, ui: &mut egui::Ui, tabs: &[(&str, &str)]) {
        ui.horizontal(|ui| {
            for (id, label) in tabs {
                let is_active = self.active_tab == *id;
                
                let tab_response = ui.add_sized(
                    Vec2::new(150.0, 32.0),
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
                    .corner_radius(CornerRadius::same(6))
                );
                
                if tab_response.clicked() {
                    self.active_tab = id.to_string();
                }
                
                ui.add_space(2.0);
            }
        });
    }
    
    fn render_search_paths_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // ROM Paths section
            ui.label(
                RichText::new("ROM Paths")
                    .size(14.0)
                    .color(Color32::from_rgb(255, 255, 255))
            );
            ui.label(
                RichText::new("Paths to ROM sets (supports multiple directories for regular ROMs and CHDs)")
                    .size(12.0)
                    .color(Color32::from_rgb(136, 136, 136))
            );
            
            ui.add_space(10.0);
            
            // Display existing ROM paths
            let mut paths_to_remove = Vec::new();
            for (idx, path) in self.settings.paths.rom_paths.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(500.0, 36.0),
                        egui::TextEdit::singleline(path)
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.add_space(8.0);
                    
                    if ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(51, 51, 51))
                    ).clicked() {
                        println!("Browse for ROM path {}", idx);
                    }
                    
                    ui.add_space(8.0);
                    
                    if ui.add_sized(
                        Vec2::new(36.0, 36.0),
                        egui::Button::new("×")
                            .fill(Color32::from_rgb(51, 51, 51))
                    ).clicked() {
                        paths_to_remove.push(idx);
                    }
                });
                ui.add_space(4.0);
            }
            
            // Remove marked paths
            for idx in paths_to_remove.iter().rev() {
                self.settings.paths.rom_paths.remove(*idx);
            }
            
            // Add another ROM path button
            ui.add_space(8.0);
            if ui.add_sized(
                Vec2::new(624.0, 36.0),
                egui::Button::new("+ Add another ROM path")
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
            ).clicked() {
                self.settings.paths.rom_paths.push(String::new());
            }
            
            ui.add_space(20.0);
            
            // Artwork Path
            Self::render_single_path_setting(
                ui,
                "Artwork Path",
                "Path to artwork files (bezels, overlays)",
                &mut self.settings.paths.artwork_path
            );
            
            ui.add_space(20.0);
            
            // Snapshot/snap Path
            Self::render_single_path_setting(
                ui,
                "Snapshot/snap Path",
                "Path to game screenshots",
                &mut self.settings.paths.snapshot_path
            );
            
            ui.add_space(20.0);
            
            // Flyer Path
            Self::render_single_path_setting(
                ui,
                "Flyer Path",
                "Path to game flyer images",
                &mut self.settings.paths.flyer_path
            );
            
            ui.add_space(20.0);
            
            // Marquees Path
            Self::render_single_path_setting(
                ui,
                "Marquees Path",
                "Path to marquee images",
                &mut self.settings.paths.marquees_path
            );
            
            ui.add_space(20.0);
            
            // Titles Path
            Self::render_single_path_setting(
                ui,
                "Titles Path",
                "Path to title screen images",
                &mut self.settings.paths.titles_path
            );
        });
    }
    
    fn render_output_directories_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Snapshot Directory
            Self::render_single_path_setting(
                ui,
                "Snapshot Directory",
                "Directory to save screenshots",
                &mut self.settings.paths.snapshot_dir
            );
            
            ui.add_space(20.0);
            
            // Config Directory
            Self::render_single_path_setting(
                ui,
                "Config Directory",
                "Directory to save configuration files",
                &mut self.settings.paths.config_dir
            );
            
            ui.add_space(20.0);
            
            // NVRAM Directory
            Self::render_single_path_setting(
                ui,
                "NVRAM Directory",
                "Directory to save NVRAM contents",
                &mut self.settings.paths.nvram_dir
            );
        });
    }
    
    fn render_history_info_tab(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Catver Path
            Self::render_single_path_setting(
                ui,
                "Catver Path",
                "Path to category version file",
                &mut self.settings.paths.catver_path
            );
            
            ui.add_space(20.0);
            
            // History (history.xml) Path
            Self::render_single_path_setting(
                ui,
                "History (history.xml) Path",
                "Path to MAME history XML file",
                &mut self.settings.paths.history_path
            );
            
            ui.add_space(20.0);
            
            // mameinfo (mameinfo.dat) Path
            Self::render_single_path_setting(
                ui,
                "mameinfo (mameinfo.dat) Path",
                "Path to MAME info DAT file",
                &mut self.settings.paths.mameinfo_path
            );
            
            ui.add_space(20.0);
            
            // Command (command.dat) Path
            Self::render_single_path_setting(
                ui,
                "Command (command.dat) Path",
                "Path to command DAT file",
                &mut self.settings.paths.command_path
            );
        });
    }
    
    fn render_single_path_setting(ui: &mut egui::Ui, label: &str, description: &str, value: &mut String) {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(label)
                    .size(14.0)
                    .color(Color32::from_rgb(255, 255, 255))
            );
            ui.label(
                RichText::new(description)
                    .size(12.0)
                    .color(Color32::from_rgb(136, 136, 136))
            );
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.add_sized(
                    Vec2::new(500.0, 36.0),
                    egui::TextEdit::singleline(value)
                        .font(egui::TextStyle::Monospace)
                );
                
                ui.add_space(8.0);
                
                if ui.add_sized(
                    Vec2::new(80.0, 36.0),
                    egui::Button::new("Browse")
                        .fill(Color32::from_rgb(51, 51, 51))
                ).clicked() {
                    println!("Browse for: {}", label);
                }
            });
        });
    }
    
    fn render_footer(&mut self, ui: &mut egui::Ui) {
        let footer_rect = ui.available_rect_before_wrap();
        let footer_rect = Rect::from_min_size(
            Pos2::new(footer_rect.min.x, footer_rect.max.y - 64.0),
            Vec2::new(footer_rect.width(), 64.0),
        );
        
        ui.scope_builder(egui::UiBuilder::new().max_rect(footer_rect), |ui| {
            ui.painter().rect_filled(
                footer_rect,
                CornerRadius {
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
                
                ui.label(
                    RichText::new("MAME 0.264 Configuration")
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(24.0);
                    
                    let apply_btn = ui.add_sized(
                        Vec2::new(140.0, 36.0),
                        egui::Button::new("Apply Changes")
                            .fill(Color32::from_rgb(74, 158, 255))
                            .stroke(Stroke::NONE)
                    );
                    
                    if apply_btn.clicked() {
                        println!("Applying settings: {:?}", self.settings);
                        self.is_open = false;
                    }
                    
                    ui.add_space(12.0);
                    
                    let cancel_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Cancel")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if cancel_btn.clicked() {
                        self.is_open = false;
                    }
                });
            });
        });
    }
}

struct DemoApp {
    settings_dialog: AdvancedMameSettingsViewport,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            settings_dialog: AdvancedMameSettingsViewport::new(),
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply dark theme
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = egui::Color32::from_rgb(22, 22, 22);
        style.visuals.panel_fill = egui::Color32::from_rgb(22, 22, 22);
        style.visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(51, 51, 51));
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 30);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(42, 42, 42);
        style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(51, 51, 51);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 60);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(74, 158, 255);
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 224, 224));
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 224, 224));
        style.visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 255, 255));
        ctx.set_style(style);
        
        // Show the settings dialog as a separate window
        self.settings_dialog.show(ctx);
        
        // Main window
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                
                ui.heading("Advanced MAME Settings Viewport Demo");
                
                ui.add_space(20.0);
                
                ui.label("This demo shows the Advanced MAME Settings dialog as a separate window.");
                ui.label("The dialog uses egui's viewport API to create a native window that can be moved independently.");
                
                ui.add_space(40.0);
                
                let button_text = if self.settings_dialog.is_open() {
                    "Settings Window is Open"
                } else {
                    "Open Advanced MAME Settings"
                };
                
                let button = ui.add_sized(
                    egui::Vec2::new(250.0, 40.0),
                    egui::Button::new(button_text)
                        .fill(if self.settings_dialog.is_open() {
                            egui::Color32::from_rgb(60, 120, 60)
                        } else {
                            egui::Color32::from_rgb(74, 158, 255)
                        })
                );
                
                if button.clicked() && !self.settings_dialog.is_open() {
                    self.settings_dialog.open();
                }
                
                ui.add_space(20.0);
                
                if self.settings_dialog.is_open() {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 100),
                        "✓ Settings window is open. You can drag it around!"
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::from_rgb(136, 136, 136),
                        "Settings window is closed."
                    );
                }
                
                ui.add_space(40.0);
                
                ui.group(|ui| {
                    ui.label("Instructions:");
                    ui.label("• Click the button to open the Advanced MAME Settings window");
                    ui.label("• The settings window can be moved independently from this main window");
                    ui.label("• Close the settings window using the X button or the Cancel/Apply buttons");
                    ui.label("• The window remembers its position while the app is running");
                });
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Advanced MAME Settings Viewport Demo"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Advanced MAME Settings Viewport Demo",
        options,
        Box::new(|_cc| Ok(Box::new(DemoApp::default()))),
    )
}