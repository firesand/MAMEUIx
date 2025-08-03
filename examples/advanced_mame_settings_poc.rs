//! Proof of Concept for Advanced MAME Settings Dialog
//! 
//! This example demonstrates the UI design for the Advanced MAME Settings dialog
//! with dark theme, sidebar navigation, tabs, and various input controls.

use eframe::egui;
use egui::{Color32, CornerRadius, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};

#[derive(Default)]
pub struct AdvancedMameSettingsPOC {
    is_open: bool,
    selected_category: String,
    active_tab: String,
    search_query: String,
    
    // Sample data for POC
    rom_paths: Vec<String>,
    sample_path: String,
    artwork_path: String,
    snapshot_path: String,
    flyer_path: String,
    marquees_path: String,
    titles_path: String,
    
    // Output directories
    snapshot_dir: String,
    config_dir: String,
    nvram_dir: String,
    
    // History & Information files
    catver_path: String,
    history_path: String,
    mameinfo_path: String,
    command_path: String,
}

impl AdvancedMameSettingsPOC {
    pub fn new() -> Self {
        let mut poc = Self::default();
        poc.selected_category = "paths".to_string();
        poc.active_tab = "search-paths".to_string();
        
        // Sample data
        poc.rom_paths = vec![
            "/home/user/roms".to_string(),
            "/home/user/chds".to_string(),
        ];
        poc.sample_path = "/home/user/samples".to_string();
        poc.artwork_path = "/home/user/artwork".to_string();
        poc.snapshot_path = "/home/user/snap".to_string();
        poc.flyer_path = "/home/user/flyers".to_string();
        poc.marquees_path = "/home/user/marquees".to_string();
        poc.titles_path = "/home/user/titles".to_string();
        
        poc.snapshot_dir = "/home/user/snap".to_string();
        poc.config_dir = "/home/user/.mame/cfg".to_string();
        poc.nvram_dir = "/home/user/.mame/nvram".to_string();
        
        poc.catver_path = "/home/user/.mame/catver.ini".to_string();
        poc.history_path = "/home/user/.mame/history.xml".to_string();
        poc.mameinfo_path = "/home/user/.mame/mameinfo.dat".to_string();
        poc.command_path = "/home/user/.mame/command.dat".to_string();
        
        poc
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
                    CornerRadius::ZERO,
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
                corner_radius: CornerRadius::same(12),
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
                        .rounding(CornerRadius::same(6))
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
        
        // Use a vertical layout with fixed width
        ui.allocate_ui_with_layout(
            Vec2::new(sidebar_width, ui.available_height() - 120.0),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
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
                    ("scripting", "📝", "Scripting & Plugins"),
                    ("opengl", "🎯", "OpenGL & Shaders"),
                    ("sdl", "🖥️", "SDL Options"),
                ];
                
                for (id, icon, label) in categories {
                    let is_active = self.selected_category == id;
                    
                    // Allocate space for the category item
                    let (rect, response) = ui.allocate_exact_size(
                        Vec2::new(sidebar_width, 44.0),
                        Sense::click()
                    );
                    
                    // Hover effect
                    if response.hovered() && !is_active {
                        ui.painter().rect_filled(
                            rect,
                            CornerRadius::ZERO,
                            Color32::from_rgb(26, 26, 26),
                        );
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    
                    // Active state
                    if is_active {
                        ui.painter().rect_filled(
                            rect,
                            CornerRadius::ZERO,
                            Color32::from_rgb(42, 42, 42),
                        );
                        
                        // Active indicator
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
                    
                    // Icon and text
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
                        
                        // Reset tab to first tab of the category
                        match id {
                            "paths" => self.active_tab = "search-paths".to_string(),
                            _ => {}
                        }
                    }
                }
            }
        );
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
                CornerRadius::ZERO,
                Color32::from_rgb(10, 10, 10),
            );
            
            ui.add_space(24.0);
            
            // Render content based on selected category
            match self.selected_category.as_str() {
                "paths" => self.render_paths_settings(ui),
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
                self.render_tab_bar(ui);
                
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
    
    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Tab bar background
            let tab_bar_rect = ui.available_rect_before_wrap();
            let tab_bar_rect = Rect::from_min_size(
                tab_bar_rect.min,
                Vec2::new(500.0, 40.0),
            );
            
            ui.painter().rect_filled(
                tab_bar_rect,
                CornerRadius::same(8),
                Color32::from_rgb(26, 26, 26),
            );
            
            ui.add_space(4.0);
            
            let tabs = vec![
                ("search-paths", "Search Paths"),
                ("output-dirs", "Output Directories"),
                ("history-info", "History & Information Files"),
            ];
            
            for (id, label) in tabs {
                let is_active = self.active_tab == id;
                
                let tab_response = ui.add_sized(
                    Vec2::new(150.0, 32.0),
                    egui::Button::new(
                        RichText::new(label)
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
                    .rounding(CornerRadius::same(6))
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
        // Setting group
        let group_rect = ui.available_rect_before_wrap();
        let group_rect = Rect::from_min_size(
            group_rect.min,
            Vec2::new(group_rect.width().min(800.0), 0.0),
        );
        
        egui::Frame {
            fill: Color32::from_rgb(22, 22, 22),
            stroke: Stroke::new(1.0, Color32::from_rgb(42, 42, 42)),
            corner_radius: CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        }
        .show(ui, |ui| {
            // ROM Paths
            let rom_paths = &mut self.rom_paths;
            ui.vertical(|ui| {
                // Label
                ui.label(
                    RichText::new("ROM Paths")
                        .size(14.0)
                        .color(Color32::from_rgb(255, 255, 255))
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Paths to ROM sets (supports multiple directories for regular ROMs and CHDs)")
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );
                
                ui.add_space(12.0);
                
                // Path items
                let mut remove_index = None;
                
                for (index, path) in rom_paths.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // Path input
                        ui.add_sized(
                            Vec2::new(400.0, 36.0),
                            egui::TextEdit::singleline(path)
                                .font(egui::TextStyle::Monospace)
                        );
                        
                        ui.add_space(8.0);
                        
                        // Browse button
                        if ui.add_sized(
                            Vec2::new(80.0, 36.0),
                            egui::Button::new("Browse")
                                .fill(Color32::from_rgb(42, 42, 42))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                        ).clicked() {
                            println!("Browse clicked for path {}", index);
                        }
                        
                        ui.add_space(8.0);
                        
                        // Remove button
                        if ui.add_sized(
                            Vec2::new(36.0, 36.0),
                            egui::Button::new(
                                RichText::new("×")
                                    .size(20.0)
                                    .color(Color32::from_rgb(255, 95, 86))
                            )
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                        ).clicked() {
                            remove_index = Some(index);
                        }
                    });
                    
                    ui.add_space(8.0);
                }
                
                // Remove path if requested
                if let Some(index) = remove_index {
                    rom_paths.remove(index);
                }
                
                // Add path button
                if ui.add_sized(
                    Vec2::new(200.0, 36.0),
                    egui::Button::new("+ Add another ROM path")
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                ).clicked() {
                    rom_paths.push(String::new());
                }
            });
            
            ui.add_space(16.0);
            
            // Artwork Path
            let artwork_path = &mut self.artwork_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Artwork Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to artwork files (bezels, overlays)")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Artwork Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(artwork_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Snapshot Path
            let snapshot_path = &mut self.snapshot_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Snapshot/snap Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to game screenshots")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Snapshot Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(snapshot_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Flyer Path
            let flyer_path = &mut self.flyer_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Flyer Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to game flyer images")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Flyer Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(flyer_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Marquees Path
            let marquees_path = &mut self.marquees_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Marquees Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to marquee images")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Marquees Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(marquees_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Titles Path
            let titles_path = &mut self.titles_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Titles Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to title screen images")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Titles Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(titles_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
        });
    }
    
    fn render_output_dirs_tab(&mut self, ui: &mut egui::Ui) {
        let group_rect = ui.available_rect_before_wrap();
        let group_rect = Rect::from_min_size(
            group_rect.min,
            Vec2::new(group_rect.width().min(800.0), 0.0),
        );
        
        egui::Frame {
            fill: Color32::from_rgb(22, 22, 22),
            stroke: Stroke::new(1.0, Color32::from_rgb(42, 42, 42)),
            corner_radius: CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        }
        .show(ui, |ui| {
            // Snapshot Directory
            let snapshot_dir = &mut self.snapshot_dir;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Snapshot Directory")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Directory to save/load screenshots")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Snapshot Directory");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(snapshot_dir)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Config Directory
            let config_dir = &mut self.config_dir;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Config Directory")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Directory for configuration files")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Config Directory");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(config_dir)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // NVRAM Directory
            let nvram_dir = &mut self.nvram_dir;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("NVRAM Directory")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Directory to save NVRAM contents")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for NVRAM Directory");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(nvram_dir)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
        });
    }
    
    fn render_history_info_tab(&mut self, ui: &mut egui::Ui) {
        let group_rect = ui.available_rect_before_wrap();
        let group_rect = Rect::from_min_size(
            group_rect.min,
            Vec2::new(group_rect.width().min(800.0), 0.0),
        );
        
        egui::Frame {
            fill: Color32::from_rgb(22, 22, 22),
            stroke: Stroke::new(1.0, Color32::from_rgb(42, 42, 42)),
            corner_radius: CornerRadius::same(8),
            inner_margin: egui::Margin::same(20),
            ..Default::default()
        }
        .show(ui, |ui| {
            // Catver Path
            let catver_path = &mut self.catver_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Catver Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to catver.ini file for game categorization")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Catver Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(catver_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // History Path
            let history_path = &mut self.history_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("History (history.xml) Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to history.xml file with game information")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for History Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(history_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // MAMEinfo Path
            let mameinfo_path = &mut self.mameinfo_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("MAMEinfo (mameinfo.dat) Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to mameinfo.dat file")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for MAMEinfo Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(mameinfo_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
            
            ui.add_space(16.0);
            
            // Command Path
            let command_path = &mut self.command_path;
            ui.horizontal(|ui| {
                // Label column
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Command (command.dat) Path")
                            .size(14.0)
                            .color(Color32::from_rgb(255, 255, 255))
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Path to command.dat file with control information")
                            .size(12.0)
                            .color(Color32::from_rgb(136, 136, 136))
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Browse button
                    let browse_btn = ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    );
                    
                    if browse_btn.clicked() {
                        println!("Browse button clicked for Command Path");
                    }
                    
                    ui.add_space(8.0);
                    
                    // Path input
                    ui.add_sized(
                        Vec2::new(280.0, 36.0),
                        egui::TextEdit::singleline(command_path)
                            .font(egui::TextStyle::Monospace)
                    );
                });
            });
        });
    }
    
    
    fn render_path_setting(&mut self, ui: &mut egui::Ui, name: &str, description: &str, value: &mut String) {
        ui.horizontal(|ui| {
            // Label column
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(name)
                        .size(14.0)
                        .color(Color32::from_rgb(255, 255, 255))
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(description)
                        .size(12.0)
                        .color(Color32::from_rgb(136, 136, 136))
                );
            });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Browse button
                let browse_btn = ui.add_sized(
                    Vec2::new(80.0, 36.0),
                    egui::Button::new("Browse")
                        .fill(Color32::from_rgb(42, 42, 42))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                );
                
                if browse_btn.clicked() {
                    // In a real application, this would open a file dialog
                    println!("Browse button clicked for {}", name);
                }
                
                ui.add_space(8.0);
                
                // Path input
                ui.add_sized(
                    Vec2::new(280.0, 36.0),
                    egui::TextEdit::singleline(value)
                        .font(egui::TextStyle::Monospace)
                );
            });
        });
    }
    
    fn render_multi_path_setting(&mut self, ui: &mut egui::Ui, name: &str, description: &str, paths: &mut Vec<String>) {
        ui.vertical(|ui| {
            // Label
            ui.label(
                RichText::new(name)
                    .size(14.0)
                    .color(Color32::from_rgb(255, 255, 255))
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(description)
                    .size(12.0)
                    .color(Color32::from_rgb(136, 136, 136))
            );
            
            ui.add_space(12.0);
            
            // Path items
            let mut remove_index = None;
            
            for (index, path) in paths.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    // Path input
                    ui.add_sized(
                        Vec2::new(400.0, 36.0),
                        egui::TextEdit::singleline(path)
                            .font(egui::TextStyle::Monospace)
                    );
                    
                    ui.add_space(8.0);
                    
                    // Browse button
                    if ui.add_sized(
                        Vec2::new(80.0, 36.0),
                        egui::Button::new("Browse")
                            .fill(Color32::from_rgb(42, 42, 42))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    ).clicked() {
                        println!("Browse clicked for path {}", index);
                    }
                    
                    ui.add_space(8.0);
                    
                    // Remove button
                    if ui.add_sized(
                        Vec2::new(36.0, 36.0),
                        egui::Button::new(
                            RichText::new("×")
                                .size(20.0)
                                .color(Color32::from_rgb(255, 95, 86))
                        )
                        .fill(Color32::from_rgb(42, 42, 42))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
                    ).clicked() {
                        remove_index = Some(index);
                    }
                });
                
                ui.add_space(8.0);
            }
            
            // Remove path if requested
            if let Some(index) = remove_index {
                paths.remove(index);
            }
            
            // Add path button
            if ui.add_sized(
                Vec2::new(200.0, 36.0),
                egui::Button::new("+ Add another ROM path")
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(68, 68, 68)))
            ).clicked() {
                paths.push(String::new());
            }
        });
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
                        // Apply settings
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
                        // Reset settings to defaults
                        println!("Resetting to defaults...");
                    }
                });
            });
        });
    }
    
    pub fn open(&mut self) {
        self.is_open = true;
    }
}

// Example usage
pub struct DemoApp {
    settings_dialog: AdvancedMameSettingsPOC,
}

impl Default for DemoApp {
    fn default() -> Self {
        Self {
            settings_dialog: AdvancedMameSettingsPOC::new(),
        }
    }
}

impl eframe::App for DemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MAME Frontend - POC Demo");
            
            if ui.button("Open Advanced Settings").clicked() {
                self.settings_dialog.open();
            }
        });
        
        // Show the settings dialog
        self.settings_dialog.show(ctx);
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([1024.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Advanced MAME Settings POC",
        options,
        Box::new(|_cc| Ok(Box::<DemoApp>::default())),
    )
}