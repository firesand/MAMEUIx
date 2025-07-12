// src/ui/sidebar.rs
use eframe::egui;
use crate::models::{FilterCategory, SearchMode};  // Import SearchMode

pub struct Sidebar {
    search_text: String,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            search_text: String::new(),
        }
    }

    /// Display the sidebar with filter options
    /// The selected_filter parameter allows the sidebar to modify which filter is active
    pub fn show(&mut self, ui: &mut egui::Ui, selected_filter: &mut FilterCategory, filter_settings: &mut crate::models::FilterSettings) {
        ui.heading("Filters");

        ui.separator();

        // Search section with mode selector
        ui.label("Search:");
        
        // Search mode selector
        ui.horizontal(|ui| {
            ui.label("Search by:");
            egui::ComboBox::from_label("")
                .selected_text(match filter_settings.search_mode {
                    SearchMode::GameTitle => "Game Title",
                    SearchMode::Manufacturer => "Manufacturer",
                    SearchMode::RomFileName => "ROM File Name",
                    SearchMode::Year => "Year",
                    SearchMode::Status => "Status",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut filter_settings.search_mode, SearchMode::GameTitle, "Game Title");
                    ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Manufacturer, "Manufacturer");
                    ui.selectable_value(&mut filter_settings.search_mode, SearchMode::RomFileName, "ROM File Name");
                    ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Year, "Year");
                    ui.selectable_value(&mut filter_settings.search_mode, SearchMode::Status, "Status");
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
        ui.radio_value(selected_filter, FilterCategory::Parents, "Parent ROMs");
        ui.radio_value(selected_filter, FilterCategory::Clones, "Clones");
        ui.radio_value(selected_filter, FilterCategory::Working, "Working");
        ui.radio_value(selected_filter, FilterCategory::NotWorking, "Not Working");

        ui.separator();

        // Additional options could go here
        ui.label("Additional filters and options can be added here");
    }

    /// Get the current search text
    pub fn search_text(&self) -> &str {
        &self.search_text
    }
}
