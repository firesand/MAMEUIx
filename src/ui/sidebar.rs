use eframe::egui;
use crate::models::FilterCategory;

pub struct Sidebar;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterCategory {
    All,
    Available,
    Unavailable,
    Working,
    NotWorking,
    Favorites,
}

impl Sidebar {
    pub fn new() -> Self {
        Self
    }

    pub fn show(&mut self, ui: &mut egui::Ui, selected_filter: &mut FilterCategory) {
        ui.vertical(|ui| {
            ui.heading("Filters");
            ui.separator();

            ui.collapsing("Status", |ui| {
                self.filter_button(ui, "All ROMs", FilterCategory::All, selected_filter);
                self.filter_button(ui, "Available", FilterCategory::Available, selected_filter);
                self.filter_button(ui, "Unavailable", FilterCategory::Unavailable, selected_filter);
                self.filter_button(ui, "Working", FilterCategory::Working, selected_filter);
                self.filter_button(ui, "Not Working", FilterCategory::NotWorking, selected_filter);
            });

            ui.collapsing("Custom", |ui| {
                self.filter_button(ui, "❤ Favorites", FilterCategory::Favorites, selected_filter);
            });
        });
    }

    fn filter_button(&self, ui: &mut egui::Ui, text: &str, filter: FilterCategory, selected: &mut FilterCategory) {
        if ui.selectable_label(*selected == filter, text).clicked() {
            *selected = filter;
        }
    }
}
