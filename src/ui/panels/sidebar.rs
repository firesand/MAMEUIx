// src/ui/sidebar.rs
use crate::models::{FilterSettings, filters::SearchMode};
use crate::ui::components::steam_ui::SteamUi;
use crate::utils::hardware_filter::HardwareFilter;
use eframe::egui;

pub(crate) const ROMLESS_FILTER_LABEL: &str = "Hide systems that don't require ROMs";

/// Both shells edit the same persisted option, with wrapping for narrow sidebars.
pub(crate) fn romless_filter_checkbox(
    ui: &mut egui::Ui,
    filters: &mut FilterSettings,
) -> egui::Response {
    // Earlier rows can expand the layout beyond the sidebar's visible width.
    // Keep the vertical coordinates so scrolling does not reposition the checkbox.
    let mut bounds = ui.available_rect_before_wrap();
    let visible_bounds = bounds.intersect(ui.clip_rect());
    bounds.min.x = visible_bounds.min.x;
    bounds.max.x = visible_bounds.max.x.max(bounds.min.x);
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(bounds)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.checkbox(&mut filters.hide_romless_systems, ROMLESS_FILTER_LABEL)
        },
    )
        .inner
        .on_hover_text(
            "Uses MAME metadata about required ROM or disk media, including BIOS and devices. \
             This is independent of whether files are installed. Games that need no ROMs are hidden too.",
        )
}

pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Self {}
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
        hardware_filter: Option<&HardwareFilter>,
        all_manufacturers: &[String],
    ) {
        let mut bounds = ui.available_rect_before_wrap();
        let visible_bounds = bounds.intersect(ui.clip_rect());
        bounds.min.x = visible_bounds.min.x;
        bounds.max.x = visible_bounds.max.x.max(bounds.min.x);
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(bounds)
                .layout(egui::Layout::top_down(egui::Align::Min)),
            |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                Self::filter_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(Self::section_title(ui, "Search"));
                    Self::search_field(ui, &mut filter_settings.search_text, "Search games...");
                });

                // Search mode container with precise alignment
                ui.add_space(8.0);
                Self::filter_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(Self::section_title(ui, "Search Mode:"));

                    // Search mode dropdown with same width as search bar
                    egui::ComboBox::from_id_salt("search_mode_combo")
                        .width(ui.available_width())
                        .wrap_mode(egui::TextWrapMode::Truncate)
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
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::GameTitle,
                                "🎯 Game Title",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Manufacturer,
                                "🏭 Manufacturer",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::RomFileName,
                                "📁 ROM File Name",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Year,
                                "📅 Year",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Status,
                                "⚙️ Status",
                            );
                            ui.separator();
                            ui.label(egui::RichText::new("🔧 Hardware").strong());
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Cpu,
                                "🖥️ CPU",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Device,
                                "🔧 Device",
                            );
                            ui.selectable_value(
                                &mut filter_settings.search_mode,
                                SearchMode::Sound,
                                "🔊 Sound",
                            );
                            ui.separator();
                            ui.label(egui::RichText::new("⚡ Enhanced Search").strong());
                            if ui
                                .selectable_value(
                                    &mut filter_settings.search_mode,
                                    SearchMode::FuzzySearch,
                                    "🔍 Fuzzy Search",
                                )
                                .on_hover_text("Finds matches even with typos or partial spelling")
                                .clicked()
                            {
                                // Fuzzy search selected
                            }
                            if ui
                                .selectable_value(
                                    &mut filter_settings.search_mode,
                                    SearchMode::FullText,
                                    "📄 Full-Text Search",
                                )
                                .on_hover_text("Search across all game information simultaneously")
                                .clicked()
                            {
                                // Full-text search selected
                            }
                            if ui
                                .selectable_value(
                                    &mut filter_settings.search_mode,
                                    SearchMode::Regex,
                                    "🔤 Regex Search",
                                )
                                .on_hover_text(
                                    "Use regular expressions for advanced pattern matching",
                                )
                                .clicked()
                            {
                                // Regex search selected
                            }
                        });
                });

                if let Some(tip) = match filter_settings.search_mode {
                    SearchMode::FuzzySearch => Some("Try: 'strt fgtr' for 'Street Fighter'"),
                    SearchMode::FullText => Some("Searches all fields simultaneously"),
                    SearchMode::Regex => Some("Try: '^Street.*Fighter$'"),
                    _ => None,
                } {
                    ui.add_space(8.0);
                    Self::filter_frame(ui).show(ui, |ui| {
                        ui.label(Self::section_title(ui, "Enhanced search active"));
                        ui.label(
                            egui::RichText::new(tip)
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        );
                    });
                }

                ui.add_space(12.0);
                Self::filter_frame(ui).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(Self::section_title(ui, "Filters"));
                    if ui
                        .checkbox(
                            &mut filter_settings.select_all_mode,
                            egui::RichText::new("Select / Clear All")
                                .color(ui.visuals().weak_text_color()),
                        )
                        .changed()
                    {
                        if filter_settings.select_all_mode {
                            self.select_all_filters(filter_settings);
                        } else {
                            self.clear_all_filters(filter_settings);
                        }
                    }
                    romless_filter_checkbox(ui, filter_settings);
                });
                ui.add_space(8.0);

                Self::filter_header(ui, "📋 Availability", true).show(ui, |ui| {
                    ui.checkbox(
                        &mut filter_settings.availability_filters.show_available,
                        egui::RichText::new("Available").color(SteamUi::SUCCESS),
                    );
                    ui.checkbox(
                        &mut filter_settings.availability_filters.show_unavailable,
                        egui::RichText::new("Unavailable").color(SteamUi::DANGER),
                    );
                });

                Self::filter_header(ui, "⚙️ Status", true).show(ui, |ui| {
                    ui.checkbox(
                        &mut filter_settings.status_filters.show_working,
                        egui::RichText::new("Working").color(SteamUi::SUCCESS),
                    );
                    ui.checkbox(
                        &mut filter_settings.status_filters.show_not_working,
                        egui::RichText::new("Not Working").color(SteamUi::DANGER),
                    );
                });

                Self::filter_header(ui, "📁 Others", true).show(ui, |ui| {
                    let secondary = ui.visuals().weak_text_color();
                    ui.checkbox(
                        &mut filter_settings.other_filters.show_favorites,
                        egui::RichText::new("Favorites").color(secondary),
                    );
                    ui.checkbox(
                        &mut filter_settings.other_filters.show_parents_only,
                        egui::RichText::new("Parent ROMs").color(secondary),
                    );
                    ui.checkbox(
                        &mut filter_settings.other_filters.show_chd_games,
                        egui::RichText::new("CHD Games").color(secondary),
                    );
                });

                ui.add_space(16.0);

                self.show_manufacturer_filters(ui, filter_settings, all_manufacturers);

                ui.add_space(16.0);

                self.show_hardware_filters(ui, filter_settings, hardware_filter);

                ui.add_space(16.0);

                let active_count = filter_settings.count_active_filters();
                if active_count > 0 {
                    Self::filter_frame(ui).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(format!("Active Filters: {active_count}"))
                                    .small()
                                    .color(ui.visuals().weak_text_color()),
                            );
                            if ui.button("Clear").clicked() {
                                self.clear_all_filters(filter_settings);
                            }
                        });
                    });
                }
                ui.add_space(8.0);
            },
        );
    }

    fn section_title(ui: &egui::Ui, text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .size(14.5)
            .strong()
            .color(ui.visuals().hyperlink_color)
    }

    fn filter_header(
        ui: &egui::Ui,
        text: impl Into<String>,
        default_open: bool,
    ) -> egui::CollapsingHeader {
        // CollapsingHeader otherwise forces Extend, even in a wrapping Ui.
        let width =
            (ui.available_width() - ui.spacing().indent - ui.spacing().button_padding.x).max(1.0);
        let text = egui::WidgetText::from(Self::section_title(ui, text)).into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            width,
            egui::TextStyle::Button,
        );
        egui::CollapsingHeader::new(text).default_open(default_open)
    }

    fn filter_frame(ui: &egui::Ui) -> egui::Frame {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::same(8))
    }

    fn search_field(ui: &mut egui::Ui, value: &mut String, hint: &str) {
        let has_query = !value.is_empty();
        let clear_button_width = ui.spacing().interact_size.x.max(
            ui.spacing().button_padding.x * 2.0 + ui.text_style_height(&egui::TextStyle::Button),
        );
        let clear_width = clear_button_width + ui.spacing().item_spacing.x;
        let inline_clear = has_query && ui.available_width() >= clear_width + 72.0;
        ui.horizontal(|ui| {
            let input_width =
                (ui.available_width() - if inline_clear { clear_width } else { 0.0 }).max(0.0);
            ui.add_sized(
                [input_width, ui.spacing().interact_size.y],
                egui::TextEdit::singleline(value)
                    .desired_width(input_width)
                    .hint_text(hint),
            );
            if inline_clear
                && ui
                    .add_sized(
                        [clear_button_width, ui.spacing().interact_size.y],
                        egui::Button::new("✕"),
                    )
                    .on_hover_text("Clear search")
                    .clicked()
            {
                value.clear();
            }
        });
        if has_query && !inline_clear && ui.button("Clear search").clicked() {
            value.clear();
        }
    }

    /// Clear all filters
    fn clear_all_filters(&self, filters: &mut FilterSettings) {
        // Reset to show all games
        filters.hide_romless_systems = false;
        filters.availability_filters.show_available = false;
        filters.availability_filters.show_unavailable = false;
        filters.status_filters.show_working = false;
        filters.status_filters.show_not_working = false;
        filters.other_filters.show_favorites = false;
        filters.other_filters.show_parents_only = false;
        filters.other_filters.show_chd_games = false;
        filters.cpu_filter.clear();
        filters.device_filter.clear();
        filters.sound_filter.clear();
        filters.manufacturer.clear();
        filters.selected_manufacturers.clear();
    }

    /// Select all filters (might result in no games shown due to conflicting criteria)
    fn select_all_filters(&self, filters: &mut FilterSettings) {
        filters.hide_romless_systems = true;
        filters.availability_filters.show_available = true;
        filters.availability_filters.show_unavailable = true;
        filters.status_filters.show_working = true;
        filters.status_filters.show_not_working = true;
        filters.other_filters.show_favorites = true;
        filters.other_filters.show_parents_only = true;
        filters.other_filters.show_chd_games = true;
    }

    fn show_manufacturer_filters(
        &self,
        ui: &mut egui::Ui,
        filter_settings: &mut FilterSettings,
        all_manufacturers: &[String],
    ) {
        let selected_count = filter_settings.selected_manufacturers.len();
        let header = if selected_count > 0 {
            format!("🏭 Manufacturer ({selected_count})")
        } else {
            "🏭 Manufacturer".to_string()
        };

        Self::filter_header(ui, header, false).show(ui, |ui| {
            ui.add_space(8.0);

            if all_manufacturers.is_empty() {
                ui.label(
                    egui::RichText::new("Load games first to see manufacturers")
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                );
                return;
            }

            Self::search_field(
                ui,
                &mut filter_settings.manufacturer,
                "Search manufacturers...",
            );

            ui.add_space(6.0);

            ui.horizontal_wrapped(|ui| {
                if ui.small_button("Select visible").clicked() {
                    for name in Self::filtered_manufacturers(all_manufacturers, filter_settings) {
                        filter_settings.selected_manufacturers.insert(name);
                    }
                }
                if ui.small_button("Clear").clicked() {
                    filter_settings.selected_manufacturers.clear();
                }
            });

            ui.add_space(6.0);

            let search = filter_settings.manufacturer.to_lowercase();
            let visible: Vec<&String> = all_manufacturers
                .iter()
                .filter(|m| search.is_empty() || m.to_lowercase().contains(&search))
                .collect();

            ui.label(egui::RichText::new(format!("{} manufacturers", visible.len())).small());

            egui::ScrollArea::vertical()
                .id_salt("manufacturer_filter_list")
                .max_height(220.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for name in visible {
                        let mut selected = filter_settings.selected_manufacturers.contains(name);
                        if ui.checkbox(&mut selected, name.as_str()).changed() {
                            if selected {
                                filter_settings.selected_manufacturers.insert(name.clone());
                            } else {
                                filter_settings.selected_manufacturers.remove(name);
                            }
                        }
                    }
                });
        });
    }

    fn filtered_manufacturers(
        all_manufacturers: &[String],
        filter_settings: &FilterSettings,
    ) -> Vec<String> {
        let search = filter_settings.manufacturer.to_lowercase();
        all_manufacturers
            .iter()
            .filter(|m| search.is_empty() || m.to_lowercase().contains(&search))
            .cloned()
            .collect()
    }

    fn show_hardware_filters(
        &self,
        ui: &mut egui::Ui,
        filter_settings: &mut FilterSettings,
        hardware_filter: Option<&HardwareFilter>,
    ) {
        Self::filter_header(ui, "🔧 Hardware Filter", false).show(ui, |ui| {
            ui.add_space(8.0);

            if let Some(hw) = hardware_filter {
                ui.label(format!(
                    "Loaded: {} CPUs, {} devices, {} sound chips",
                    hw.cpu_count(),
                    hw.device_count(),
                    hw.sound_count()
                ));
            } else {
                ui.colored_label(
                    ui.visuals().weak_text_color(),
                    "No hardware INI files loaded. Set INI directory in Options → Directories.",
                );
            }

            ui.add_space(8.0);
            self.hardware_filter_field(
                ui,
                "cpu_filter",
                "CPU",
                &mut filter_settings.cpu_filter,
                hardware_filter.map(|hw| hw.get_all_cpus()),
            );
            self.hardware_filter_field(
                ui,
                "device_filter",
                "Device",
                &mut filter_settings.device_filter,
                hardware_filter.map(|hw| hw.get_all_devices()),
            );
            self.hardware_filter_field(
                ui,
                "sound_filter",
                "Sound",
                &mut filter_settings.sound_filter,
                hardware_filter.map(|hw| hw.get_all_sounds()),
            );

            if ui.button("Clear hardware filters").clicked() {
                filter_settings.cpu_filter.clear();
                filter_settings.device_filter.clear();
                filter_settings.sound_filter.clear();
            }
        });
    }

    fn hardware_filter_field(
        &self,
        ui: &mut egui::Ui,
        id: &str,
        label: &str,
        value: &mut String,
        options: Option<Vec<String>>,
    ) {
        ui.label(egui::RichText::new(format!("{label}:")).color(ui.visuals().weak_text_color()));
        ui.add(
            egui::TextEdit::singleline(value)
                .id_salt(id)
                .desired_width(ui.available_width())
                .hint_text(format!("Filter by {label}")),
        );

        if let Some(items) = options {
            let preview: Vec<String> = items.into_iter().take(12).collect();
            if !preview.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new("Quick:").small().weak());
                    for item in preview {
                        if ui.small_button(&item).clicked() {
                            *value = item;
                        }
                    }
                });
            }
        }

        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_sidebar_keeps_filter_labels_inside_a_narrow_panel() {
        let context = egui::Context::default();
        SteamUi::apply(&context);
        let mut sidebar = Sidebar::new();
        let mut filters = FilterSettings {
            search_text: "fighter".into(),
            selected_manufacturers: std::collections::HashSet::from(["Example".into()]),
            ..Default::default()
        };
        let mut selected_filter = crate::models::FilterCategory::All;
        let mut hidden_categories = std::collections::HashSet::new();
        let mut dialogs = crate::ui::DialogManager::new();
        let mut clip = egui::Rect::NOTHING;
        let mut output = egui::FullOutput::default();
        for frame in 0..2 {
            output = context.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 1600.0),
                    )),
                    time: Some(frame as f64 * 0.05),
                    ..Default::default()
                },
                |ctx| {
                    egui::SidePanel::left("narrow_sidebar")
                        .exact_width(180.0)
                        .show(ctx, |ui| {
                            clip = ui.clip_rect();
                            sidebar.show(
                                ui,
                                &mut selected_filter,
                                &mut filters,
                                None,
                                &mut hidden_categories,
                                &mut dialogs,
                                None,
                                &[],
                            );
                        });
                },
            );
        }
        for label in [
            "✕",
            "Select / Clear All",
            ROMLESS_FILTER_LABEL,
            "📋 Availability",
            "⚙️ Status",
            "📁 Others",
            "🏭 Manufacturer (1)",
            "🔧 Hardware Filter",
            "Clear",
        ] {
            let text = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::Shape::Text(text) if text.galley.job.text == label => Some(text),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing label: {label}"));
            let rect = egui::Rect::from_min_size(text.pos, text.galley.size());
            assert!(
                clip.contains_rect(rect),
                "{label}: {rect:?} outside {clip:?}"
            );
            assert!(!text.galley.elided, "label was truncated: {label}");
        }
    }

    #[test]
    fn romless_checkbox_wraps_within_clipped_sidebar() {
        let ctx = egui::Context::default();
        let mut filters = FilterSettings::default();
        let clip = egui::Rect::from_min_max(egui::pos2(12.0, 12.0), egui::pos2(195.0, 400.0));
        let mut time = 0.0;
        let mut frame = |events: Vec<egui::Event>| {
            time += 0.05;
            let mut response = None;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800.0, 600.0),
                    )),
                    time: Some(time),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.scope_builder(
                            egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                                clip.min,
                                egui::pos2(500.0, clip.max.y),
                            )),
                            |ui| {
                                ui.set_clip_rect(clip);
                                // Reproduce an oversized preceding row and a nonwrapping style.
                                ui.set_min_width(480.0);
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                                assert!(ui.available_width() > clip.width());
                                response = Some(romless_filter_checkbox(ui, &mut filters));
                                assert_eq!(ui.wrap_mode(), egui::TextWrapMode::Extend);
                            },
                        );
                    });
                },
            );
            (output, response.unwrap(), filters.hide_romless_systems)
        };

        let (output, response, checked) = frame(Vec::new());
        assert!(checked);
        assert!(clip.contains_rect(response.rect), "{:?}", response.rect);
        let label = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == ROMLESS_FILTER_LABEL => {
                    Some(text)
                }
                _ => None,
            })
            .expect("the full checkbox label is rendered");
        assert!(label.galley.rows.len() > 1);
        assert!(!label.galley.elided);
        let text_rect = egui::Rect::from_min_size(label.pos, label.galley.size());
        assert!(clip.contains_rect(text_rect), "{text_rect:?}");

        // The wrapped label remains an actual checkbox hit target in both directions.
        let pointer = text_rect.center();
        for expected in [false, true] {
            frame(vec![
                egui::Event::PointerMoved(pointer),
                egui::Event::PointerButton {
                    pos: pointer,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ]);
            let (_, response, checked) = frame(vec![egui::Event::PointerButton {
                pos: pointer,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }]);
            assert!(response.changed());
            assert_eq!(checked, expected);
            assert!(clip.contains_rect(response.rect));
        }

        // A scrolled-off row must keep its original Y, rather than move into the clip.
        let _ = egui::Context::default().run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.scope_builder(
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                        egui::pos2(clip.min.x, -100.0),
                        egui::pos2(500.0, clip.max.y),
                    )),
                    |ui| {
                        ui.set_clip_rect(clip);
                        let response = romless_filter_checkbox(ui, &mut filters);
                        assert_eq!(response.rect.top(), -100.0);
                        assert!(response.rect.bottom() < clip.top());
                        assert!(response.rect.right() <= clip.right());
                    },
                );
            });
        });
    }
}
