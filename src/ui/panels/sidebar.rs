// src/ui/sidebar.rs
use eframe::egui;
use crate::models::{FilterSettings, RomSetType, filters::SearchMode};

pub struct Sidebar {
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
        }
    }

    /// Display the sidebar with modern accordion-style filters
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _selected_filter: &mut crate::models::FilterCategory,
        filter_settings: &mut FilterSettings,
        _category_manager: Option<&crate::models::filters::CategoryManager>,
        _hidden_categories: &mut std::collections::HashSet<String>,
        _dialog_manager: &mut crate::ui::DialogManager,
    ) {
        // Search bar container with precise alignment
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            
            // Search bar with magnifying glass icon
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("🔍").size(18.0));
                
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut filter_settings.search_text)
                        .desired_width(ui.available_width() - 40.0)
                        .hint_text("Search games...")
                        .font(egui::TextStyle::Button)
                );
                
                // Animated clear button
                if !filter_settings.search_text.is_empty() {
                    if ui.add(
                        egui::Button::new("✕")
                            .fill(egui::Color32::from_rgba_premultiplied(255, 255, 255, 20))
                            .min_size(egui::Vec2::splat(24.0))
                    ).clicked() {
                        filter_settings.search_text.clear();
                    }
                }
            });
        });

        // Search mode container with precise alignment
        ui.add_space(8.0);
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            
            // Search mode label
            ui.label(egui::RichText::new("Search Mode:").size(14.0).strong());
            
            // Search mode dropdown with same width as search bar
            egui::ComboBox::from_id_salt("search_mode_combo")
            .selected_text(match filter_settings.search_mode {
                SearchMode::GameTitle => "🎯 Game Title",
                SearchMode::Manufacturer => "🏭 Manufacturer",
                SearchMode::RomFileName => "📁 ROM File Name",
                SearchMode::Year => "📅 Year",
                SearchMode::Status => "⚙️ Status",
                SearchMode::Cpu => "🖥️ CPU",
                SearchMode::Device => "🔧 Device",
                SearchMode::Sound => "🔊 Sound",
                SearchMode::FuzzySearch => "🔍 Fuzzy Search",
                SearchMode::FullText => "📄 Full-Text Search",
                SearchMode::Regex => "🔤 Regex Search",
            })
            .show_ui(ui, |ui| {
                ui.label(egui::RichText::new("🔸 Basic Search").strong());
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::GameTitle, "🎯 Game Title");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Manufacturer, "🏭 Manufacturer");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::RomFileName, "📁 ROM File Name");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Year, "📅 Year");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Status, "⚙️ Status");
                ui.separator();
                ui.label(egui::RichText::new("🔧 Hardware").strong());
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Cpu, "🖥️ CPU");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Device, "🔧 Device");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Sound, "🔊 Sound");
                ui.separator();
                ui.label(egui::RichText::new("⚡ Enhanced Search").strong());
                if ui.selectable_value(&mut filter_settings.search_mode, SearchMode::FuzzySearch, "🔍 Fuzzy Search").on_hover_text("Finds matches even with typos or partial spelling").clicked() {
                    // Fuzzy search selected
                }
                if ui.selectable_value(&mut filter_settings.search_mode, SearchMode::FullText, "📄 Full-Text Search").on_hover_text("Search across all game information simultaneously").clicked() {
                    // Full-text search selected
                }
                if ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Regex, "🔤 Regex Search").on_hover_text("Use regular expressions for advanced pattern matching").clicked() {
                    // Regex search selected
                }
            });
        });

        // Search performance info (only show for enhanced modes)
        match filter_settings.search_mode {
            SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                ui.add_space(8.0);
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("💡").size(16.0));
                        ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "Enhanced search active");
                    });
                    
                    // Search tips based on mode
                    match filter_settings.search_mode {
                        SearchMode::FuzzySearch => {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💬").size(14.0));
                                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Try: 'strt fgtr' for 'Street Fighter'");
                            });
                        }
                        SearchMode::FullText => {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💬").size(14.0));
                                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Searches all fields simultaneously");
                            });
                        }
                        SearchMode::Regex => {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("💬").size(14.0));
                                ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Try: '^Street.*Fighter$'");
                            });
                        }
                        _ => {}
                    }
                });
            }
            _ => {}
        }

        ui.add_space(16.0);

        // Horizontal layout with Filters and radio button in one row
        ui.horizontal(|ui| {
            // Filters label with lightning icon
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚡").size(18.0).color(egui::Color32::from_rgb(255, 193, 7))); // Lightning icon in yellow
                ui.heading(
                    egui::RichText::new("Filters")
                        .size(18.0)
                        .color(egui::Color32::from_rgb(100, 150, 255)) // Blue text
                        .strong()
                );
            });
            
            ui.add_space(16.0);
            
            // Checkbox for Select/Clear All
            let old_state = filter_settings.select_all_mode;
            if ui.add(
                egui::Checkbox::new(
                    &mut filter_settings.select_all_mode,
                    egui::RichText::new("Select / Clear All").size(14.0).color(egui::Color32::from_rgb(180, 180, 200))
                )
            ).clicked() {
                // Check if state actually changed
                if filter_settings.select_all_mode != old_state {
                    // Apply the appropriate action based on new state
                    if filter_settings.select_all_mode {
                        self.select_all_filters(filter_settings);
                    } else {
                        self.clear_all_filters(filter_settings);
                    }
                }
            }
        });

        ui.add_space(16.0);

        // Collapsible filter sections dengan animasi
        egui::CollapsingHeader::new(
            egui::RichText::new("📋 Availability")
                .size(16.0)
                .color(egui::Color32::from_rgb(100, 200, 255))
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            // Custom styled checkboxes with badges
            ui.horizontal(|ui| {
                let mut available = filter_settings.availability_filters.show_available;
                if ui.add(
                    egui::Checkbox::new(&mut available, "")
                ).clicked() {
                    filter_settings.availability_filters.show_available = available;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Available")
                        .color(egui::Color32::from_rgb(76, 175, 80))
                );
            });

            ui.horizontal(|ui| {
                let mut unavailable = filter_settings.availability_filters.show_unavailable;
                if ui.add(
                    egui::Checkbox::new(&mut unavailable, "")
                ).clicked() {
                    filter_settings.availability_filters.show_unavailable = unavailable;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Unavailable")
                        .color(egui::Color32::from_rgb(244, 67, 54))
                );
            });
        });

        // STATUS filter section
        egui::CollapsingHeader::new(
            egui::RichText::new("⚙️ Status")
                .size(16.0)
                .color(egui::Color32::from_rgb(255, 193, 7))
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                let mut working = filter_settings.status_filters.show_working;
                if ui.add(
                    egui::Checkbox::new(&mut working, "")
                ).clicked() {
                    filter_settings.status_filters.show_working = working;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Working")
                        .color(egui::Color32::from_rgb(76, 175, 80))
                );
            });

            ui.horizontal(|ui| {
                let mut not_working = filter_settings.status_filters.show_not_working;
                if ui.add(
                    egui::Checkbox::new(&mut not_working, "")
                ).clicked() {
                    filter_settings.status_filters.show_not_working = not_working;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Not Working")
                        .color(egui::Color32::from_rgb(244, 67, 54))
                );
            });
        });

        egui::CollapsingHeader::new(
            egui::RichText::new("📁 Others")
                .size(16.0)
                .color(egui::Color32::from_rgb(150, 255, 100))
        )
        .default_open(true)
        .show(ui, |ui| {
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                let mut favorites = filter_settings.other_filters.show_favorites;
                if ui.add(
                    egui::Checkbox::new(&mut favorites, "")
                ).clicked() {
                    filter_settings.other_filters.show_favorites = favorites;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Favorites")
                        .color(egui::Color32::from_rgb(233, 30, 99))
                );
            });

            ui.horizontal(|ui| {
                let mut parents_only = filter_settings.other_filters.show_parents_only;
                if ui.add(
                    egui::Checkbox::new(&mut parents_only, "")
                ).clicked() {
                    filter_settings.other_filters.show_parents_only = parents_only;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Parent ROMs")
                        .color(egui::Color32::from_rgb(156, 39, 176))
                );
            });

            ui.horizontal(|ui| {
                let mut chd_games = filter_settings.other_filters.show_chd_games;
                if ui.add(
                    egui::Checkbox::new(&mut chd_games, "")
                ).clicked() {
                    filter_settings.other_filters.show_chd_games = chd_games;
                }
                
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("CHD Games")
                        .color(egui::Color32::from_rgb(0, 188, 212))
                );
            });
        });

        ui.add_space(16.0);
        
        // Filter status display with modern styling
        let active_count = filter_settings.count_active_filters();
        if active_count > 0 {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("Active Filters: {}", active_count)).strong());
                    if ui.add(
                        egui::Button::new(egui::RichText::new("Clear").size(12.0))
                            .fill(egui::Color32::from_rgb(244, 67, 54))
                            .min_size(egui::Vec2::new(50.0, 24.0))
                    ).clicked() {
                        self.clear_all_filters(filter_settings);
                    }
                });
            });
        }
        
        ui.add_space(16.0);
    }
    
    /// Clear all filters
    fn clear_all_filters(&self, filters: &mut FilterSettings) {
        // Reset to show all games
        filters.availability_filters.show_available = false;
        filters.availability_filters.show_unavailable = false;
        filters.status_filters.show_working = false;
        filters.status_filters.show_not_working = false;
        filters.other_filters.show_favorites = false;
        filters.other_filters.show_parents_only = false;
        filters.other_filters.show_chd_games = false;
    }
    
    /// Select all filters (might result in no games shown due to conflicting criteria)
    fn select_all_filters(&self, filters: &mut FilterSettings) {
        filters.availability_filters.show_available = true;
        filters.availability_filters.show_unavailable = true;
        filters.status_filters.show_working = true;
        filters.status_filters.show_not_working = true;
        filters.other_filters.show_favorites = true;
        filters.other_filters.show_parents_only = true;
        filters.other_filters.show_chd_games = true;
    }


}
