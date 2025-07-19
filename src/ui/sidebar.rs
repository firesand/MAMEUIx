// src/ui/sidebar.rs
use eframe::egui;
use crate::models::{FilterCategory, FilterSettings, RomSetType, filters::SearchMode};

pub struct Sidebar {
    search_text: String,
    category_search_text: String,
    previous_category: Option<String>,  // Add this
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            search_text: String::new(),
            category_search_text: String::new(),
            previous_category: None,  // Initialize
        }
    }

    // Add getter for previous category
    pub fn get_previous_category(&self) -> Option<String> {
        self.previous_category.clone()
    }

    /// Display the sidebar with filter options
    /// The selected_filter parameter allows the sidebar to modify which filter is active
    pub fn show(
        &mut self, 
        ui: &mut egui::Ui, 
        selected_filter: &mut FilterCategory, 
        filter_settings: &mut FilterSettings, 
        category_manager: Option<&crate::models::filters::CategoryManager>,
        hidden_categories: &mut std::collections::HashSet<String>,
        show_hidden_categories_dialog: &mut bool,
    ) {
        // Store the current category before any changes
        self.previous_category = filter_settings.catver_category.clone();
        ui.heading("Filters");

        ui.separator();

        // Search section with mode selector
        ui.label("Search:");

        // Search mode selector
        ui.horizontal(|ui| {
            ui.label("Search by:");
            egui::ComboBox::from_id_salt("search_mode_combo")
            .selected_text(match filter_settings.search_mode {
                SearchMode::GameTitle => "Game Title",
                SearchMode::Manufacturer => "Manufacturer",
                SearchMode::RomFileName => "ROM File Name",
                SearchMode::Year => "Year",
                SearchMode::Status => "Status",
                SearchMode::Cpu => "CPU",
                SearchMode::Device => "Device",
                SearchMode::Sound => "Sound",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::GameTitle, "Game Title");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Manufacturer, "Manufacturer");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::RomFileName, "ROM File Name");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Year, "Year");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Status, "Status");
                ui.separator();
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Cpu, "CPU");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Device, "Device");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Sound, "Sound");
            });
        });

        // Search box - now properly connected to filter settings
        ui.horizontal(|ui| {
            ui.label("Text:");
            if ui.text_edit_singleline(&mut filter_settings.search_text).changed() {
                // Search text changed, will trigger filter update in main window
            }
        });

        ui.separator();

        // Filter category selection
        ui.label("Show:");

        // Radio buttons for filter categories
        ui.radio_value(selected_filter, FilterCategory::All, "All Games");
        ui.radio_value(selected_filter, FilterCategory::Available, "Available");
        ui.radio_value(selected_filter, FilterCategory::Missing, "Missing");
        ui.radio_value(selected_filter, FilterCategory::Favorites, "Favorites");
        
        // Make Parent ROMs more prominent
        ui.radio_value(selected_filter, FilterCategory::Parents, "Parent ROMs (no duplicates)");
        
        ui.radio_value(selected_filter, FilterCategory::Working, "Working");
        ui.radio_value(selected_filter, FilterCategory::NotWorking, "Not Working");
        ui.radio_value(selected_filter, FilterCategory::ChdGames, "CHD Games");

        ui.separator();

        // Clone display options
        ui.collapsing("🔽 Clone Options", |ui| {
            ui.checkbox(&mut filter_settings.auto_expand_clones, "Auto expand clones");
            ui.label("ℹ When enabled, clone games will be automatically expanded under their parent games");
        });

        ui.separator();

        // Category filter section
        if let Some(category_manager) = category_manager {
            ui.collapsing("📁 Categories", |ui| {
                // Show total categories and games
                ui.label(format!(
                    "Total: {} categories, {} games",
                    category_manager.categories.len(),
                                 category_manager.total_games
                ));
                ui.separator();

                // Current filter display
                ui.horizontal(|ui| {
                    ui.label("Current:");
                    if let Some(ref selected_cat) = filter_settings.catver_category {
                        ui.label(selected_cat);
                        if ui.small_button("✖").clicked() {
                            filter_settings.catver_category = None;
                        }
                    } else {
                        ui.label("All Categories");
                    }
                });
                ui.separator();

                // Quick picks for popular categories
                ui.label("Popular Categories:");
                ui.horizontal_wrapped(|ui| {
                    for popular_cat in Self::get_popular_categories() {
                        let exists = category_manager.categories.values()
                        .any(|cat| cat.display_name == popular_cat);
                        if exists {
                            let is_selected = filter_settings.catver_category
                            .as_ref()
                            .map(|c| {
                                category_manager.categories.get(c)
                                .map(|cat| cat.display_name == popular_cat)
                                .unwrap_or(false)
                            })
                            .unwrap_or(false);
                            if ui.selectable_label(is_selected, popular_cat).clicked() {
                                if let Some(cat) = category_manager.categories.values()
                                    .find(|cat| cat.display_name == popular_cat) {
                                        filter_settings.catver_category = Some(cat.name.clone());
                                    }
                            }
                        }
                    }
                });
                ui.separator();

                // Letter navigation bar
                ui.horizontal_wrapped(|ui| {
                    if ui.small_button("All").clicked() {
                        filter_settings.catver_category = None;
                    }
                    ui.separator();
                    for letter in 'A'..='Z' {
                        if category_manager.has_categories_for_letter(letter) {
                            ui.small_button(letter.to_string());
                        } else {
                            ui.add_enabled(false, egui::Button::new(letter.to_string()).small());
                        }
                    }
                });
                ui.separator();

                // Category search - FIXED: Use persistent field instead of temporary data
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.category_search_text);

                    // Add clear button when there's text
                    if !self.category_search_text.is_empty() {
                        if ui.small_button("✖").clicked() {
                            self.category_search_text.clear();
                        }
                    }
                });

                // Show search results count
                if !self.category_search_text.is_empty() {
                    let search_lower = self.category_search_text.to_lowercase();
                    let matching_count = category_manager.categories.values()
                    .filter(|cat| cat.display_name.to_lowercase().contains(&search_lower))
                    .count();
                    ui.label(format!("Found: {} categories", matching_count));
                }

                // Scrollable category list
                egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    let search_lower = self.category_search_text.to_lowercase();

                    for letter in category_manager.get_letter_groups() {
                        if let Some(categories) = category_manager.categories_by_letter.get(&letter) {
                            let mut has_visible_categories = false;

                            // Check if any categories in this letter group match the search
                            if !search_lower.is_empty() {
                                has_visible_categories = categories.iter().any(|cat_name| {
                                    if let Some(cat) = category_manager.categories.get(cat_name) {
                                        cat.display_name.to_lowercase().contains(&search_lower)
                                    } else {
                                        false
                                    }
                                });
                            } else {
                                has_visible_categories = !categories.is_empty();
                            }

                            if has_visible_categories {
                                // Show letter group header
                                ui.label(egui::RichText::new(format!("─── {} ───", letter))
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .small());

                                // Show categories in this letter group
                                for cat_name in categories {
                                    if let Some(category) = category_manager.categories.get(cat_name) {
                                        // Filter by search text
                                        if search_lower.is_empty() ||
                                            category.display_name.to_lowercase().contains(&search_lower) {
                                                let is_selected = filter_settings.catver_category
                                                .as_ref()
                                                .map(|c| c == &category.name)
                                                .unwrap_or(false);

                                                let label = Self::format_category_display(category);

                                                if ui.selectable_label(is_selected, label).clicked() {
                                                    filter_settings.catver_category = Some(category.name.clone());
                                                }
                                            }
                                    }
                                }
                                ui.add_space(5.0);
                            }
                        }
                    }
                });
            });
        } else {
            ui.collapsing("📁 Categories", |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                                 "No catver.ini loaded"
                );
                ui.label("Configure catver.ini path in");
                ui.label("Options → Directories → History/DAT Files");
            });
        }

        ui.separator();

        // Hidden Categories section
        if let Some(category_manager) = category_manager {
            ui.collapsing("🚫 Hidden Categories", |ui| {
                ui.label("Categories to hide from game list:");
                ui.separator();
                
                // Toggle to apply hidden categories filter
                ui.checkbox(&mut filter_settings.apply_hidden_categories, "Apply hidden categories filter");
                
                if filter_settings.apply_hidden_categories {
                    // Show current hidden categories count
                    ui.label(format!("Currently hiding: {} categories", hidden_categories.len()));
                    
                    // Quick add common categories to hide
                    ui.label("Quick hide:");
                    ui.horizontal_wrapped(|ui| {
                        let common_hide = vec![
                            "Casino", "Casino / Cards", "Casino / Reels",
                            "Gambling / Bingo", "Gambling / Lottery", 
                            "Mahjong", "Mature / Cards", "Mature / Mahjong"
                        ];
                        
                        for cat_name in common_hide {
                            if category_manager.categories.values()
                                .any(|cat| cat.display_name == cat_name) {
                                let is_hidden = hidden_categories.contains(cat_name);
                                if ui.selectable_label(is_hidden, cat_name).clicked() {
                                    if is_hidden {
                                        hidden_categories.remove(cat_name);
                                    } else {
                                        hidden_categories.insert(cat_name.to_string());
                                    }
                                }
                            }
                        }
                    });
                    
                    ui.separator();
                    
                    // Button to manage all hidden categories
                    if ui.button("Manage Hidden Categories...").clicked() {
                        // This will open a dialog - we'll implement it next
                        *show_hidden_categories_dialog = true;
                    }
                    
                    // Show list of currently hidden categories
                    if !hidden_categories.is_empty() {
                        ui.separator();
                        ui.label("Currently hidden:");
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                let mut to_remove = Vec::new();
                                for hidden_cat in hidden_categories.iter() {
                                    ui.horizontal(|ui| {
                                        ui.label(hidden_cat);
                                        if ui.small_button("✖").clicked() {
                                            to_remove.push(hidden_cat.clone());
                                        }
                                    });
                                }
                                for cat in to_remove {
                                    hidden_categories.remove(&cat);
                                }
                            });
                        
                        // Clear all button
                        if ui.button("Clear All Hidden").clicked() {
                            hidden_categories.clear();
                        }
                    }
                }
            });
        }
        
        ui.separator();

        // Additional filter options
        ui.label("Options:");

        // Checkbox for hiding non-games (devices and BIOS)
        ui.checkbox(&mut filter_settings.hide_non_games, "Hide non-games (devices/BIOS)");

        self.show_rom_set_info(ui, filter_settings);
    }

    /// Get the current search text
    pub fn search_text(&self) -> &str {
        &self.search_text
    }

    /// Format category display for sidebar
    fn format_category_display(category: &crate::models::filters::CatverCategory) -> String {
        match category.game_count {
            0 => category.display_name.clone(),
            1 => format!("{} (1 game)", category.display_name),
            n => format!("{} ({} games)", category.display_name, n),
        }
    }

    /// Popular categories for quick access
    fn get_popular_categories() -> Vec<&'static str> {
        vec![
            "Arcade",
            "Ball & Paddle",
            "Casino",
            "Driving",
            "Fighter",
            "Maze",
            "Platform",
            "Puzzle",
            "Shooter",
            "Sports",
        ]
    }

    fn show_rom_set_info(&self, ui: &mut egui::Ui, filter_settings: &mut FilterSettings) {
        ui.separator();
        
        // ROM Set Type Detection
        ui.collapsing("🔍 ROM Set Type", |ui| {
            match filter_settings.rom_set_type {
                RomSetType::NonMerged => {
                    ui.label("📦 Non-Merged Set Detected");
                    ui.label("ℹ Each game is independent with all required files");
                    ui.label("💡 Tip: Use 'Parents Only' to avoid duplicates");
                },
                RomSetType::Split => {
                    ui.label("✂️ Split Set Detected");
                    ui.label("ℹ Parent contains normal data, clones contain only changes");
                    ui.label("💡 Tip: Use 'Parents Only' to see main games");
                },
                RomSetType::Merged => {
                    ui.label("🔗 Merged Set Detected");
                    ui.label("ℹ All clones are included in parent archives");
                    ui.label("💡 Tip: Only parent games are needed");
                },
                RomSetType::Unknown => {
                    ui.label("❓ ROM Set Type Unknown");
                    ui.label("ℹ Detection in progress...");
                }
            }
        });

        // ROM Set Type Specific Controls
        match filter_settings.rom_set_type {
            RomSetType::NonMerged => {
                ui.collapsing("📦 Non-Merged Options", |ui| {
                    ui.checkbox(&mut filter_settings.show_clones_in_split, "Show all games (parents + clones)");
                    ui.label("ℹ When unchecked, shows only parent games to avoid duplicates");
                });
            },
            RomSetType::Split => {
                ui.collapsing("✂️ Split Set Options", |ui| {
                    ui.checkbox(&mut filter_settings.show_clones_in_split, "Show clones under parents");
                    ui.label("ℹ When checked, clone games will be shown");
                });
            },
            RomSetType::Merged => {
                ui.collapsing("🔗 Merged Set Options", |ui| {
                    ui.checkbox(&mut filter_settings.show_clones_in_merged, "Show clone entries (not recommended)");
                    ui.label("ℹ Usually not needed as clones are included in parent archives");
                });
            },
            RomSetType::Unknown => {
                // No specific controls for unknown type
            }
        }
    }
}
