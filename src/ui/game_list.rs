use eframe::egui;
use crate::models::{Game, RomStatus, FilterSettings, FilterCategory};
use std::collections::{HashMap, HashSet};

pub struct GameList {
    sort_column: SortColumn,
    sort_ascending: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortColumn {
    Name,
    Manufacturer,
    Year,
    Status,
}

impl GameList {
    pub fn new() -> Self {
        Self {
            sort_column: SortColumn::Name,
            sort_ascending: true,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        games: &[Game],
        filters: &FilterSettings,
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &mut HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
    ) {
        let filtered_games = Self::filter_games(games, filters, favorites);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("game_list")
            .striped(true)
            .show(ui, |ui| {
                self.show_header(ui, show_icons, icon_size);
                ui.end_row();

                for (idx, game) in filtered_games.iter().enumerate() {
                    let is_selected = selected.map_or(false, |s| s == idx);
                    let has_clones = games.iter().any(|g| g.parent.as_ref() == Some(&game.name));
                    let is_expanded = *expanded_parents.get(&game.name).unwrap_or(&false);

                    self.show_game_row(ui, game, idx, is_selected, has_clones, is_expanded, selected, expanded_parents, favorites, icons, show_icons, icon_size);
                    ui.end_row();

                    if is_expanded && has_clones {
                        for clone in games.iter().filter(|g| g.parent.as_ref() == Some(&game.name)) {
                            self.show_clone_row(ui, clone, icons, show_icons, icon_size);
                            ui.end_row();
                        }
                    }
                }
            });
        });
    }

    fn filter_games(games: &[Game], filters: &FilterSettings, favorites: &HashSet<String>) -> Vec<&Game> {
        games.iter()
        .filter(|game| {
            if filters.show_favorites_only && !favorites.contains(&game.name) {
                return false;
            }

            if !filters.search_text.is_empty() {
                let search_lower = filters.search_text.to_lowercase();
                if !game.description.to_lowercase().contains(&search_lower) &&
                    !game.name.to_lowercase().contains(&search_lower) {
                        return false;
                    }
            }

            if filters.hide_non_games && (game.is_device || game.is_bios) {
                return false;
            }

            if !filters.show_clones && game.is_clone {
                return false;
            }

            true
        })
        .collect()
    }

    fn show_header(&mut self, ui: &mut egui::Ui, show_icons: bool, icon_size: u32) {
        ui.label("");
        ui.label("");

        if show_icons {
            ui.add_sized([icon_size as f32 + 8.0, 24.0], egui::Label::new("Icon"));
        }

        ui.label("Status");

        if ui.button("Game").clicked() {
            self.toggle_sort(SortColumn::Name);
        }
        if ui.button("Manufacturer").clicked() {
            self.toggle_sort(SortColumn::Manufacturer);
        }
        if ui.button("Year").clicked() {
            self.toggle_sort(SortColumn::Year);
        }
        if ui.button("Status").clicked() {
            self.toggle_sort(SortColumn::Status);
        }
    }

    fn show_game_row(
        &self,
        ui: &mut egui::Ui,
        game: &Game,
        idx: usize,
        is_selected: bool,
        has_clones: bool,
        is_expanded: bool,
        selected: &mut Option<usize>,
        expanded_parents: &mut HashMap<String, bool>,
        favorites: &HashSet<String>,
        icons: &mut HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
    ) {
        if has_clones {
            if ui.button(if is_expanded { "▼" } else { "▶" }).clicked() {
                expanded_parents.insert(game.name.clone(), !is_expanded);
            }
        } else {
            ui.label("");
        }

        let is_favorite = favorites.contains(&game.name);
        if ui.button(if is_favorite { "⭐" } else { "☆" }).clicked() {
            // Toggle favorite handled in main window
        }

        if show_icons {
            if let Some(icon) = icons.get(&game.name) {
                ui.add(egui::Image::from_texture(icon)
                .fit_to_exact_size(egui::Vec2::splat(icon_size as f32)));
            } else {
                ui.add_sized([icon_size as f32, icon_size as f32], egui::Label::new(""));
            }
        }

        ui.label(game.status.to_icon());

        let response = ui.selectable_label(is_selected, &game.description);
        if response.clicked() {
            *selected = Some(idx);
        }

        ui.label(&game.manufacturer);
        ui.label(&game.year);
        ui.label(format!("{:?}", game.status));
    }

    fn show_clone_row(
        &self,
        ui: &mut egui::Ui,
        clone: &Game,
        icons: &mut HashMap<String, egui::TextureHandle>,
        show_icons: bool,
        icon_size: u32,
    ) {
        ui.label("");
        ui.label("");

        if show_icons {
            if let Some(icon) = icons.get(&clone.name) {
                ui.add(egui::Image::from_texture(icon)
                .fit_to_exact_size(egui::Vec2::splat(icon_size as f32)));
            } else {
                ui.add_sized([icon_size as f32, icon_size as f32], egui::Label::new(""));
            }
        }

        ui.label(clone.status.to_icon());
        ui.label(format!("    └─ {}", clone.description));
        ui.label(&clone.manufacturer);
        ui.label(&clone.year);
        ui.label(format!("{:?}", clone.status));
    }

    fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = true;
        }
    }
}
