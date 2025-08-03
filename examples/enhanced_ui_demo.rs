// examples/enhanced_ui_demo.rs
// Demo application showcasing enhanced egui features

use eframe::egui;
use egui_dock::{DockArea, DockState, Node, NodeIndex, TabViewer};
use egui_toast::{Toast, ToastOptions, Toasts};
use std::collections::HashMap;

/// Demo application showcasing enhanced UI features
pub struct EnhancedUIDemo {
    /// Dock state for panel management
    dock_state: DockState<String>,
    
    /// Toast notification system
    toasts: Toasts,
    
    /// Demo data
    demo_games: Vec<DemoGame>,
    selected_game: Option<usize>,
    
    /// UI state
    show_notifications: bool,
    notification_counter: u32,
}

#[derive(Debug, Clone)]
struct DemoGame {
    name: String,
    year: u32,
    manufacturer: String,
    category: String,
    available: bool,
    favorite: bool,
}

impl Default for EnhancedUIDemo {
    fn default() -> Self {
        let mut dock_state = DockState::new(vec![
            Node::Leaf {
                name: "Game List".to_string(),
                tabs: vec!["game_list".to_string()],
                active: 0,
            },
        ]);
        
        // Add sidebar panel
        dock_state.add_node(
            NodeIndex::root(),
            Node::Leaf {
                name: "Sidebar".to_string(),
                tabs: vec!["sidebar".to_string()],
                active: 0,
            },
            egui_dock::InsertBehavior::Left,
        );
        
        // Add info panel
        dock_state.add_node(
            NodeIndex::root(),
            Node::Leaf {
                name: "Info Panel".to_string(),
                tabs: vec!["info_panel".to_string()],
                active: 0,
            },
            egui_dock::InsertBehavior::Right,
        );

        let demo_games = vec![
            DemoGame {
                name: "Street Fighter II".to_string(),
                year: 1991,
                manufacturer: "Capcom".to_string(),
                category: "Fighting".to_string(),
                available: true,
                favorite: true,
            },
            DemoGame {
                name: "Pac-Man".to_string(),
                year: 1980,
                manufacturer: "Namco".to_string(),
                category: "Maze".to_string(),
                available: true,
                favorite: false,
            },
            DemoGame {
                name: "Donkey Kong".to_string(),
                year: 1981,
                manufacturer: "Nintendo".to_string(),
                category: "Platform".to_string(),
                available: true,
                favorite: true,
            },
            DemoGame {
                name: "Galaga".to_string(),
                year: 1981,
                manufacturer: "Namco".to_string(),
                category: "Shooter".to_string(),
                available: false,
                favorite: false,
            },
            DemoGame {
                name: "Mortal Kombat".to_string(),
                year: 1992,
                manufacturer: "Midway".to_string(),
                category: "Fighting".to_string(),
                available: true,
                favorite: false,
            },
        ];

        Self {
            dock_state,
            toasts: Toasts::new()
                .anchor(egui_toast::Anchor::TopRight, egui::vec2(16.0, 16.0))
                .direction(egui_toast::Direction::BottomUp)
                .max_toasts(5)
                .max_width(300.0),
            demo_games,
            selected_game: None,
            show_notifications: true,
            notification_counter: 0,
        }
    }
}

impl eframe::App for EnhancedUIDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Show toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎮 Enhanced UI Demo");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔔 Toggle Notifications").clicked() {
                        self.show_notifications = !self.show_notifications;
                        self.add_demo_notification();
                    }
                    
                    if ui.button("➕ Add Panel").clicked() {
                        self.add_demo_panel();
                    }
                    
                    if ui.button("📊 Performance").clicked() {
                        self.add_performance_panel();
                    }
                });
            });
        });

        // Show status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Games: {}", self.demo_games.len()));
                ui.separator();
                ui.label(format!("Available: {}", self.demo_games.iter().filter(|g| g.available).count()));
                ui.separator();
                ui.label(format!("Favorites: {}", self.demo_games.iter().filter(|g| g.favorite).count()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Enhanced UI Demo • egui_dock + egui_toast");
                });
            });
        });

        // Show main content with docking
        let mut tab_viewer = DemoTabViewer::new(self);
        DockArea::new(&mut self.dock_state)
            .show(ctx, &mut tab_viewer);
        
        // Show toasts if enabled
        if self.show_notifications {
            self.toasts.show(ctx);
        }
    }
}

impl EnhancedUIDemo {
    fn add_demo_notification(&mut self) {
        self.notification_counter += 1;
        
        let notifications = [
            ("Success", "Demo notification sent successfully!", Toast::success),
            ("Warning", "This is a demo warning message", Toast::warning),
            ("Error", "Demo error occurred", Toast::error),
            ("Info", "This is a demo info message", Toast::info),
        ];
        
        let (title, message, toast_fn) = notifications[self.notification_counter as usize % notifications.len()];
        let toast = toast_fn(title, message);
        
        let options = ToastOptions::default()
            .duration_in_seconds(3.0)
            .show_progress(false);
        
        self.toasts.add(toast.with_options(options));
    }
    
    fn add_demo_panel(&mut self) {
        let panel_id = format!("demo_panel_{}", self.notification_counter);
        
        // Find a suitable node to add the panel to
        if let Some(node_index) = self.find_suitable_node() {
            self.dock_state.add_tab_to_node(
                node_index,
                panel_id.clone(),
            );
        }
    }
    
    fn add_performance_panel(&mut self) {
        let panel_id = "performance_panel".to_string();
        
        // Find a suitable node to add the panel to
        if let Some(node_index) = self.find_suitable_node() {
            self.dock_state.add_tab_to_node(
                node_index,
                panel_id.clone(),
            );
        }
    }
    
    fn find_suitable_node(&self) -> Option<NodeIndex> {
        Some(NodeIndex::root())
    }
}

/// Tab viewer for the demo
struct DemoTabViewer<'a> {
    demo: &'a mut EnhancedUIDemo,
}

impl<'a> DemoTabViewer<'a> {
    fn new(demo: &'a mut EnhancedUIDemo) -> Self {
        Self { demo }
    }
}

impl<'a> TabViewer for DemoTabViewer<'a> {
    type Tab = String;
    
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "game_list" => {
                self.show_game_list(ui);
            }
            "sidebar" => {
                self.show_sidebar(ui);
            }
            "info_panel" => {
                self.show_info_panel(ui);
            }
            "performance_panel" => {
                self.show_performance_panel(ui);
            }
            _ => {
                if tab.starts_with("demo_panel_") {
                    self.show_demo_panel(ui, tab);
                } else {
                    ui.label(format!("Unknown panel: {}", tab));
                }
            }
        }
    }
    
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab.as_str() {
            "game_list" => "Game List".into(),
            "sidebar" => "Sidebar".into(),
            "info_panel" => "Info Panel".into(),
            "performance_panel" => "Performance".into(),
            _ => {
                if tab.starts_with("demo_panel_") {
                    "Demo Panel".into()
                } else {
                    tab.clone().into()
                }
            }
        }
    }
}

impl<'a> DemoTabViewer<'a> {
    fn show_game_list(&mut self, ui: &mut egui::Ui) {
        ui.heading("Game List");
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, game) in self.demo.demo_games.iter().enumerate() {
                let is_selected = self.demo.selected_game == Some(index);
                let mut selected = is_selected;
                
                ui.horizontal(|ui| {
                    // Selection checkbox
                    if ui.checkbox(&mut selected, "").clicked() {
                        self.demo.selected_game = if selected { Some(index) } else { None };
                    }
                    
                    // Game info
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&game.name).heading());
                        ui.label(format!("{} • {} • {}", game.year, game.manufacturer, game.category));
                        
                        ui.horizontal(|ui| {
                            if game.available {
                                ui.label("✅ Available");
                            } else {
                                ui.label("❌ Not Available");
                            }
                            
                            if game.favorite {
                                ui.label("⭐ Favorite");
                            }
                        });
                    });
                });
                
                if index < self.demo.demo_games.len() - 1 {
                    ui.separator();
                }
            }
        });
    }
    
    fn show_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sidebar");
        ui.separator();
        
        ui.label("Filters:");
        ui.add_space(8.0);
        
        // Availability filter
        ui.collapsing("Availability", |ui| {
            ui.checkbox(&mut true, "Available");
            ui.checkbox(&mut false, "Not Available");
        });
        
        // Category filter
        ui.collapsing("Category", |ui| {
            ui.checkbox(&mut true, "Fighting");
            ui.checkbox(&mut true, "Platform");
            ui.checkbox(&mut true, "Shooter");
            ui.checkbox(&mut true, "Maze");
        });
        
        // Manufacturer filter
        ui.collapsing("Manufacturer", |ui| {
            ui.checkbox(&mut true, "Capcom");
            ui.checkbox(&mut true, "Namco");
            ui.checkbox(&mut true, "Nintendo");
            ui.checkbox(&mut true, "Midway");
        });
        
        ui.separator();
        
        // Actions
        ui.label("Actions:");
        ui.add_space(8.0);
        
        if ui.button("🔍 Search Games").clicked() {
            self.demo.add_demo_notification();
        }
        
        if ui.button("⭐ Toggle Favorites").clicked() {
            self.demo.add_demo_notification();
        }
        
        if ui.button("📊 Show Statistics").clicked() {
            self.demo.add_demo_notification();
        }
    }
    
    fn show_info_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Info Panel");
        ui.separator();
        
        if let Some(selected_idx) = self.demo.selected_game {
            if let Some(game) = self.demo.demo_games.get(selected_idx) {
                ui.label(egui::RichText::new(&game.name).heading());
                ui.add_space(8.0);
                
                ui.label(format!("Year: {}", game.year));
                ui.label(format!("Manufacturer: {}", game.manufacturer));
                ui.label(format!("Category: {}", game.category));
                ui.label(format!("Available: {}", if game.available { "Yes" } else { "No" }));
                ui.label(format!("Favorite: {}", if game.favorite { "Yes" } else { "No" }));
                
                ui.add_space(16.0);
                
                ui.horizontal(|ui| {
                    if ui.button("▶ Play").clicked() {
                        self.demo.add_demo_notification();
                    }
                    
                    if ui.button("⭐ Toggle Favorite").clicked() {
                        self.demo.add_demo_notification();
                    }
                    
                    if ui.button("ℹ Info").clicked() {
                        self.demo.add_demo_notification();
                    }
                });
            }
        } else {
            ui.label("No game selected");
            ui.label("Select a game from the list to see details");
        }
    }
    
    fn show_performance_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance Monitor");
        ui.separator();
        
        ui.label("System Information:");
        ui.label("• CPU: 8 cores @ 3.2 GHz");
        ui.label("• Memory: 16 GB DDR4");
        ui.label("• GPU: NVIDIA RTX 3080");
        ui.label("• Storage: 1 TB NVMe SSD");
        
        ui.add_space(16.0);
        
        ui.label("Performance Metrics:");
        ui.label("• FPS: 60.0");
        ui.label("• Frame Time: 16.67 ms");
        ui.label("• Memory Usage: 512 MB");
        ui.label("• CPU Usage: 15%");
        
        ui.add_space(16.0);
        
        // Demo progress bar
        ui.label("Demo Progress:");
        ui.add(egui::ProgressBar::new(0.75).text("75% Complete"));
        
        ui.add_space(16.0);
        
        if ui.button("🔄 Refresh Metrics").clicked() {
            self.demo.add_demo_notification();
        }
    }
    
    fn show_demo_panel(&mut self, ui: &mut egui::Ui, tab: &str) {
        ui.heading(format!("Demo Panel: {}", tab));
        ui.separator();
        
        ui.label("This is a dynamically added demo panel.");
        ui.label("You can add as many panels as you want!");
        
        ui.add_space(16.0);
        
        ui.label("Features demonstrated:");
        ui.label("• Dynamic panel creation");
        ui.label("• Docking system");
        ui.label("• Toast notifications");
        ui.label("• Enhanced UI components");
        
        ui.add_space(16.0);
        
        if ui.button("🎉 Demo Action").clicked() {
            self.demo.add_demo_notification();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    
    eframe::run_native(
        "Enhanced UI Demo",
        options,
        Box::new(|_cc| Box::new(EnhancedUIDemo::default())),
    )
} 