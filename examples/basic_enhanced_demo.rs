// examples/basic_enhanced_demo.rs
// Basic enhanced UI demo showcasing egui improvements

use eframe::egui;
use egui_extras::TableBuilder;

/// Basic enhanced UI demo
pub struct BasicEnhancedDemo {
    /// Demo data
    demo_games: Vec<DemoGame>,
    selected_game: Option<usize>,
    
    /// UI state
    view_mode: ViewMode,
    
    /// Enhanced features
    search_text: String,
    filter_category: String,
    sort_by: SortBy,
    
    /// Performance tracking
    frame_count: u32,
    last_fps_update: std::time::Instant,
    fps_history: Vec<f32>,
}

#[derive(Debug, Clone)]
struct DemoGame {
    name: String,
    year: u32,
    manufacturer: String,
    category: String,
    available: bool,
    favorite: bool,
    play_time: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum SortBy {
    Name,
    Year,
    Manufacturer,
    Category,
    PlayTime,
}

#[derive(Debug, Clone, PartialEq)]
enum ViewMode {
    Table,
    Grid,
    List,
}

impl Default for BasicEnhancedDemo {
    fn default() -> Self {
        let demo_games = vec![
            DemoGame {
                name: "Street Fighter II".to_string(),
                year: 1991,
                manufacturer: "Capcom".to_string(),
                category: "Fighting".to_string(),
                available: true,
                favorite: true,
                play_time: 120,
            },
            DemoGame {
                name: "Pac-Man".to_string(),
                year: 1980,
                manufacturer: "Namco".to_string(),
                category: "Maze".to_string(),
                available: true,
                favorite: false,
                play_time: 45,
            },
            DemoGame {
                name: "Donkey Kong".to_string(),
                year: 1981,
                manufacturer: "Nintendo".to_string(),
                category: "Platform".to_string(),
                available: true,
                favorite: true,
                play_time: 90,
            },
            DemoGame {
                name: "Galaga".to_string(),
                year: 1981,
                manufacturer: "Namco".to_string(),
                category: "Shooter".to_string(),
                available: false,
                favorite: false,
                play_time: 0,
            },
            DemoGame {
                name: "Mortal Kombat".to_string(),
                year: 1992,
                manufacturer: "Midway".to_string(),
                category: "Fighting".to_string(),
                available: true,
                favorite: false,
                play_time: 75,
            },
        ];

        Self {
            demo_games,
            selected_game: None,
            view_mode: ViewMode::Table,
            search_text: String::new(),
            filter_category: "All".to_string(),
            sort_by: SortBy::Name,
            frame_count: 0,
            last_fps_update: std::time::Instant::now(),
            fps_history: Vec::new(),
        }
    }
}

impl eframe::App for BasicEnhancedDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update performance tracking
        self.frame_count += 1;
        if self.last_fps_update.elapsed().as_secs() >= 1 {
            let fps = self.frame_count as f32;
            self.fps_history.push(fps);
            if self.fps_history.len() > 60 {
                self.fps_history.remove(0);
            }
            self.frame_count = 0;
            self.last_fps_update = std::time::Instant::now();
        }

        // Show toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🎮 Basic Enhanced UI Demo");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Performance indicator
                    if let Some(avg_fps) = self.fps_history.last() {
                        let color = if *avg_fps < 30.0 {
                            egui::Color32::RED
                        } else if *avg_fps < 50.0 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::GREEN
                        };
                        ui.colored_label(color, format!("FPS: {:.1}", avg_fps));
                    }
                    
                    ui.separator();
                    
                    // View mode buttons
                    ui.selectable_value(&mut self.view_mode, ViewMode::Table, "📊 Table");
                    ui.selectable_value(&mut self.view_mode, ViewMode::Grid, "⊞ Grid");
                    ui.selectable_value(&mut self.view_mode, ViewMode::List, "☰ List");
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
                    ui.label("Basic Enhanced UI Demo • egui_extras");
                });
            });
        });

        // Show main content
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left sidebar
                ui.vertical(|ui| {
                    ui.heading("Filters");
                    ui.separator();
                    
                    // Search
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.search_text);
                    
                    ui.add_space(8.0);
                    
                    // Category filter
                    ui.label("Category:");
                    egui::ComboBox::from_id_salt("category_filter")
                        .selected_text(&self.filter_category)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.filter_category, "All".to_string(), "All");
                            ui.selectable_value(&mut self.filter_category, "Fighting".to_string(), "Fighting");
                            ui.selectable_value(&mut self.filter_category, "Platform".to_string(), "Platform");
                            ui.selectable_value(&mut self.filter_category, "Shooter".to_string(), "Shooter");
                            ui.selectable_value(&mut self.filter_category, "Maze".to_string(), "Maze");
                        });
                    
                    ui.add_space(8.0);
                    
                    // Sort options
                    ui.label("Sort by:");
                    egui::ComboBox::from_id_salt("sort_by")
                        .selected_text(format!("{:?}", self.sort_by))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.sort_by, SortBy::Name, "Name");
                            ui.selectable_value(&mut self.sort_by, SortBy::Year, "Year");
                            ui.selectable_value(&mut self.sort_by, SortBy::Manufacturer, "Manufacturer");
                            ui.selectable_value(&mut self.sort_by, SortBy::Category, "Category");
                            ui.selectable_value(&mut self.sort_by, SortBy::PlayTime, "Play Time");
                        });
                    
                    ui.add_space(16.0);
                    
                    // Actions
                    ui.label("Actions:");
                    if ui.button("🔄 Refresh").clicked() {
                        // Demo refresh action
                    }
                    
                    if ui.button("⭐ Toggle Favorites").clicked() {
                        // Demo favorite toggle
                    }
                    
                    if ui.button("📊 Statistics").clicked() {
                        // Demo statistics
                    }
                });
                
                ui.separator();
                
                // Main content area
                ui.vertical(|ui| {
                    match self.view_mode {
                        ViewMode::Table => self.show_enhanced_table(ui),
                        ViewMode::Grid => self.show_grid_view(ui),
                        ViewMode::List => self.show_list_view(ui),
                    }
                });
                
                ui.separator();
                
                // Right info panel
                ui.vertical(|ui| {
                    ui.heading("Info Panel");
                    ui.separator();
                    
                    if let Some(selected_idx) = self.selected_game {
                        if let Some(game) = self.demo_games.get(selected_idx) {
                            ui.label(egui::RichText::new(&game.name).heading());
                            ui.add_space(8.0);
                            
                            ui.label(format!("Year: {}", game.year));
                            ui.label(format!("Manufacturer: {}", game.manufacturer));
                            ui.label(format!("Category: {}", game.category));
                            ui.label(format!("Available: {}", if game.available { "Yes" } else { "No" }));
                            ui.label(format!("Favorite: {}", if game.favorite { "Yes" } else { "No" }));
                            ui.label(format!("Play Time: {} minutes", game.play_time));
                            
                            ui.add_space(16.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button("▶ Play").clicked() {
                                    // Demo play action
                                }
                                
                                if ui.button("⭐ Toggle Favorite").clicked() {
                                    // Demo favorite toggle
                                }
                                
                                if ui.button("ℹ Info").clicked() {
                                    // Demo info action
                                }
                            });
                        }
                    } else {
                        ui.label("No game selected");
                        ui.label("Select a game from the list to see details");
                    }
                });
            });
        });
    }
}

impl BasicEnhancedDemo {
    fn show_enhanced_table(&mut self, ui: &mut egui::Ui) {
        ui.heading("Enhanced Table View");
        ui.separator();
        
        // Filter and sort games
        let mut filtered_games: Vec<_> = self.demo_games.iter().enumerate().collect();
        
        // Apply search filter
        if !self.search_text.is_empty() {
            filtered_games.retain(|(_, game)| {
                game.name.to_lowercase().contains(&self.search_text.to_lowercase()) ||
                game.manufacturer.to_lowercase().contains(&self.search_text.to_lowercase())
            });
        }
        
        // Apply category filter
        if self.filter_category != "All" {
            filtered_games.retain(|(_, game)| game.category == self.filter_category);
        }
        
        // Apply sorting
        filtered_games.sort_by(|(_, a), (_, b)| {
            match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Year => a.year.cmp(&b.year),
                SortBy::Manufacturer => a.manufacturer.cmp(&b.manufacturer),
                SortBy::Category => a.category.cmp(&b.category),
                SortBy::PlayTime => a.play_time.cmp(&b.play_time),
            }
        });
        
        // Show table using egui_extras
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto().resizable(true).clip(true))
            .column(egui_extras::Column::auto().resizable(true))
            .column(egui_extras::Column::auto().resizable(true))
            .column(egui_extras::Column::auto().resizable(true))
            .column(egui_extras::Column::auto().resizable(true))
            .column(egui_extras::Column::auto().resizable(true))
            .header(20.0, |mut header| {
                header.col(|ui| { ui.strong("Name"); });
                header.col(|ui| { ui.strong("Year"); });
                header.col(|ui| { ui.strong("Manufacturer"); });
                header.col(|ui| { ui.strong("Category"); });
                header.col(|ui| { ui.strong("Available"); });
                header.col(|ui| { ui.strong("Play Time"); });
            })
            .body(|mut body| {
                for (_idx, (original_idx, game)) in filtered_games.iter().enumerate() {
                    let is_selected = self.selected_game == Some(*original_idx);
                    let mut selected = is_selected;
                    
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut selected, "").clicked() {
                                    self.selected_game = if selected { Some(*original_idx) } else { None };
                                }
                                ui.label(&game.name);
                            });
                        });
                        row.col(|ui| { ui.label(game.year.to_string()); });
                        row.col(|ui| { ui.label(&game.manufacturer); });
                        row.col(|ui| { ui.label(&game.category); });
                        row.col(|ui| { 
                            if game.available {
                                ui.label("✅");
                            } else {
                                ui.label("❌");
                            }
                        });
                        row.col(|ui| { ui.label(format!("{} min", game.play_time)); });
                    });
                }
            });
    }
    
    fn show_grid_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("Grid View");
        ui.separator();
        
        // Filter games
        let mut filtered_games: Vec<_> = self.demo_games.iter().enumerate().collect();
        
        if !self.search_text.is_empty() {
            filtered_games.retain(|(_, game)| {
                game.name.to_lowercase().contains(&self.search_text.to_lowercase())
            });
        }
        
        if self.filter_category != "All" {
            filtered_games.retain(|(_, game)| game.category == self.filter_category);
        }
        
        // Sort games
        filtered_games.sort_by(|(_, a), (_, b)| {
            match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Year => a.year.cmp(&b.year),
                SortBy::Manufacturer => a.manufacturer.cmp(&b.manufacturer),
                SortBy::Category => a.category.cmp(&b.category),
                SortBy::PlayTime => a.play_time.cmp(&b.play_time),
            }
        });
        
        // Show grid
        egui::Grid::new("game_grid")
            .num_columns(3)
            .spacing([8.0, 8.0])
            .show(ui, |ui| {
                for (original_idx, game) in filtered_games {
                    let is_selected = self.selected_game == Some(original_idx);
                    let mut selected = is_selected;
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut selected, "").clicked() {
                                self.selected_game = if selected { Some(original_idx) } else { None };
                            }
                            ui.label(egui::RichText::new(&game.name).heading());
                        });
                        
                        ui.label(format!("{} • {}", game.year, game.manufacturer));
                        ui.label(&game.category);
                        
                        ui.horizontal(|ui| {
                            if game.available {
                                ui.label("✅");
                            } else {
                                ui.label("❌");
                            }
                            
                            if game.favorite {
                                ui.label("⭐");
                            }
                            
                            ui.label(format!("{} min", game.play_time));
                        });
                    });
                    
                    ui.end_row();
                }
            });
    }
    
    fn show_list_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("List View");
        ui.separator();
        
        // Filter games
        let mut filtered_games: Vec<_> = self.demo_games.iter().enumerate().collect();
        
        if !self.search_text.is_empty() {
            filtered_games.retain(|(_, game)| {
                game.name.to_lowercase().contains(&self.search_text.to_lowercase())
            });
        }
        
        if self.filter_category != "All" {
            filtered_games.retain(|(_, game)| game.category == self.filter_category);
        }
        
        // Sort games
        filtered_games.sort_by(|(_, a), (_, b)| {
            match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Year => a.year.cmp(&b.year),
                SortBy::Manufacturer => a.manufacturer.cmp(&b.manufacturer),
                SortBy::Category => a.category.cmp(&b.category),
                SortBy::PlayTime => a.play_time.cmp(&b.play_time),
            }
        });
        
        // Show list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (original_idx, game) in filtered_games {
                let is_selected = self.selected_game == Some(original_idx);
                let mut selected = is_selected;
                
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut selected, "").clicked() {
                        self.selected_game = if selected { Some(original_idx) } else { None };
                    }
                    
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&game.name).heading());
                        ui.label(format!("{} • {} • {}", game.year, game.manufacturer, game.category));
                    });
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if game.available {
                            ui.label("✅");
                        } else {
                            ui.label("❌");
                        }
                        
                        if game.favorite {
                            ui.label("⭐");
                        }
                        
                        ui.label(format!("{} min", game.play_time));
                    });
                });
                
                if original_idx < self.demo_games.len() - 1 {
                    ui.separator();
                }
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Basic Enhanced UI Demo",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(BasicEnhancedDemo::default()))),
    )
} 