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



    /// Display the sidebar with filter options
    /// The selected_filter parameter is now deprecated but kept for compatibility
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        _selected_filter: &mut crate::models::FilterCategory,
        filter_settings: &mut FilterSettings,
        _category_manager: Option<&crate::models::filters::CategoryManager>,
        _hidden_categories: &mut std::collections::HashSet<String>,
        _dialog_manager: &mut crate::ui::DialogManager,
    ) {
        ui.heading("Filters");

        ui.separator();
        
        // Quick action buttons
        ui.horizontal(|ui| {
            if ui.button("Clear All").clicked() {
                self.clear_all_filters(filter_settings);
            }
            if ui.button("Select All").clicked() {
                self.select_all_filters(filter_settings);
            }
        });
        
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
                SearchMode::FuzzySearch => "🔍 Fuzzy Search",
                SearchMode::FullText => "📄 Full-Text Search",
                SearchMode::Regex => "🔤 Regex Search",
            })
            .show_ui(ui, |ui| {
                ui.label("🔸 Basic Search");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::GameTitle, "Game Title");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Manufacturer, "Manufacturer");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::RomFileName, "ROM File Name");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Year, "Year");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Status, "Status");
                ui.separator();
                ui.label("🔧 Hardware");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Cpu, "CPU");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Device, "Device");
                ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Sound, "Sound");
                ui.separator();
                ui.label("⚡ Enhanced Search");
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

        // Search box - now properly connected to filter settings
        ui.horizontal(|ui| {
            ui.label("Text:");
            if ui.text_edit_singleline(&mut filter_settings.search_text).changed() {
                // Search text changed, will trigger filter update in main window
            }
        });

        // Search performance info (only show for enhanced modes)
        match filter_settings.search_mode {
            SearchMode::FuzzySearch | SearchMode::FullText | SearchMode::Regex => {
                ui.horizontal(|ui| {
                    ui.label("💡");
                    ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "Enhanced search active");
                });
                
                // Search tips based on mode
                match filter_settings.search_mode {
                    SearchMode::FuzzySearch => {
                        ui.horizontal(|ui| {
                            ui.label("💬");
                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Try: 'strt fgtr' for 'Street Fighter'");
                        });
                    }
                    SearchMode::FullText => {
                        ui.horizontal(|ui| {
                            ui.label("💬");
                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Searches all fields simultaneously");
                        });
                    }
                    SearchMode::Regex => {
                        ui.horizontal(|ui| {
                            ui.label("💬");
                            ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "Try: '^Street.*Fighter$'");
                        });
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        ui.separator();

        // Filter category selection with checkboxes for multi-selection
        ui.label("Show:");
        
        // AVAILABILITY section
        ui.label(egui::RichText::new("📋 AVAILABILITY").heading().color(egui::Color32::from_rgb(100, 150, 255)));
        ui.checkbox(
            &mut filter_settings.availability_filters.show_available,
            "Available"
        ).on_hover_text("Show games with all required ROM files present");
        ui.checkbox(
            &mut filter_settings.availability_filters.show_unavailable,
            "Unavailable"
        ).on_hover_text("Show games that MAME knows about but ROM files are not found");
        
        ui.separator();

        // STATUS section
        ui.label(egui::RichText::new("⚙️ STATUS").heading().color(egui::Color32::from_rgb(255, 150, 100)));
        ui.checkbox(
            &mut filter_settings.status_filters.show_working,
            "Working"
        ).on_hover_text("Show games that are playable (includes both perfect and imperfect emulation)");
        ui.checkbox(
            &mut filter_settings.status_filters.show_not_working,
            "Not Working"
        ).on_hover_text("Show games with significant emulation issues making them unplayable");
        
        ui.separator();

        // OTHERS section
        ui.label(egui::RichText::new("📁 OTHERS").heading().color(egui::Color32::from_rgb(150, 255, 100)));
        ui.checkbox(
            &mut filter_settings.other_filters.show_favorites,
            "Favorites"
        ).on_hover_text("Show only games marked as favorites");
        ui.checkbox(
            &mut filter_settings.other_filters.show_parents_only,
            "Parent ROMs (no duplicates)"
        ).on_hover_text("Show only parent games, hiding all clones");
        ui.checkbox(
            &mut filter_settings.other_filters.show_chd_games,
            "CHD Games"
        ).on_hover_text("Show only games that require CHD disk images");

        ui.separator();
        
        // Filter status display
        let active_count = filter_settings.count_active_filters();
        if active_count > 0 {
            ui.horizontal(|ui| {
                ui.label(format!("Active Filters: {}", active_count));
                if ui.small_button("Clear").clicked() {
                    self.clear_all_filters(filter_settings);
                }
            });
        }
        
        ui.separator();

        // Clone display options
        // ui.collapsing("🔽 Clone Options", |ui| {
            ui.checkbox(&mut filter_settings.auto_expand_clones, "Auto expand clones");
            ui.label("ℹ When enabled, clone games will be automatically expanded under their parent games");
        // });

        ui.separator();

        // Additional filter options
        ui.label("Options:");

        self.show_rom_set_info(ui, filter_settings);
    }
    
    /// Clear all filters
    fn clear_all_filters(&self, filters: &mut FilterSettings) {
        // Reset to show all games
        filters.availability_filters.show_available = true;
        filters.availability_filters.show_unavailable = true;
        filters.status_filters.show_working = true;
        filters.status_filters.show_not_working = true;
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
