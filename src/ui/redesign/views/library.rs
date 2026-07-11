use super::super::fonts;
use super::super::state::{RedesignCollection, RedesignState, YearDecade};
use super::super::tokens::RedesignTokens;
use super::super::widgets::{
    checkbox_row, collapsible_header, collection_dot, section_header, sidebar_row, text_link,
};
use crate::app::MameApp;
use crate::models::{FilterSettings, SearchMode, StatusFilter};
use eframe::egui;
use std::collections::HashMap;

// Kept as a switch so the existing manufacturer implementation can be restored
// later without affecting the shared filter engine or the legacy shell.
const SHOW_MANUFACTURER_FILTER: bool = false;

pub struct LibraryAction {
    pub open_detail: Option<usize>,
    pub toggle_favorite: Option<String>,
    pub filters_changed: bool,
}

pub fn show(ctx: &egui::Context, app: &mut MameApp, state: &mut RedesignState) -> LibraryAction {
    let mut action = LibraryAction {
        open_detail: None,
        toggle_favorite: None,
        filters_changed: false,
    };

    enforce_visible_filter_state(state);
    state.ensure_search_buf(&app.config.filter_settings.search_text);
    apply_search_debounce(ctx, app, state, &mut action);

    let narrow = ctx.available_rect().width() < 720.0;
    if !narrow || state.narrow_sidebar_open {
        let sidebar_width = if narrow {
            (ctx.available_rect().width() * 0.42).clamp(200.0, RedesignTokens::SIDEBAR_WIDTH)
        } else {
            RedesignTokens::SIDEBAR_WIDTH
        };
        egui::SidePanel::left("redesign_sidebar")
            .exact_width(sidebar_width)
            .frame(
                egui::Frame::new()
                    .fill(RedesignTokens::BG_PANEL)
                    .stroke(egui::Stroke::new(1.0_f32, RedesignTokens::BORDER))
                    .inner_margin(0.0),
            )
            .show(ctx, |ui| {
                show_sidebar(ui, app, state, &mut action);
            });
    }

    sync_filters_to_config(app, state, &mut action);

    if action.filters_changed || app.game_index_manager.is_cache_dirty() {
        app.update_filtered_games_cache();
        state.mark_table_dirty();
    }

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(RedesignTokens::BG_ROOT)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            if narrow {
                show_compact_toolbar(ui, state);
            }
            super::library_table::show_game_table(ui, app, state, &mut action);
        });

    action
}

fn enforce_visible_filter_state(state: &mut RedesignState) {
    if !SHOW_MANUFACTURER_FILTER {
        state.selected_manufacturer = None;
        state.manufacturer_open = false;
    }
}

fn show_compact_toolbar(ui: &mut egui::Ui, state: &mut RedesignState) {
    egui::Frame::new()
        .fill(RedesignTokens::BG_PANEL)
        .stroke(egui::Stroke::new(1.0_f32, RedesignTokens::BORDER))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let label = if state.narrow_sidebar_open {
                    "× Filters"
                } else {
                    "☰ Filters"
                };
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(label)
                                .font(fonts::semibold(12.0))
                                .color(RedesignTokens::TEXT_PRIMARY),
                        )
                        .selected(state.narrow_sidebar_open),
                    )
                    .clicked()
                {
                    state.narrow_sidebar_open = !state.narrow_sidebar_open;
                }

                let search_width = ui.available_width().max(100.0);
                let search = ui.add_sized(
                    [search_width, 28.0],
                    egui::TextEdit::singleline(&mut state.search_text_buf)
                        .hint_text("Search games…")
                        .font(egui::TextStyle::Body),
                );
                focus_search_if_requested(&search, state);
                if search.changed() {
                    state.search_debounce_deadline = Some(ui.input(|i| i.time) + 0.25);
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(250));
                }
            });
        });
}

fn focus_search_if_requested(search: &egui::Response, state: &mut RedesignState) {
    if state.take_search_focus_request() {
        search.request_focus();
    }
}

fn apply_search_debounce(
    ctx: &egui::Context,
    app: &mut MameApp,
    state: &mut RedesignState,
    action: &mut LibraryAction,
) {
    let Some(deadline) = state.search_debounce_deadline else {
        return;
    };
    let now = ctx.input(|i| i.time);
    if now < deadline {
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        return;
    }
    state.search_debounce_deadline = None;
    if app.config.filter_settings.search_text != state.search_text_buf {
        app.config.filter_settings.search_text = state.search_text_buf.clone();
        app.game_index_manager.mark_cache_dirty();
        app.game_list.invalidate_cache();
        app.game_list_view.invalidate_cache();
        state.mark_table_dirty();
        action.filters_changed = true;
    }
}

fn sync_filters_to_config(
    app: &mut MameApp,
    state: &mut RedesignState,
    action: &mut LibraryAction,
) {
    let fs = &mut app.config.filter_settings;

    let old_search = fs.search_text.clone();
    let old_chd = fs.other_filters.show_chd_games;
    let old_mfr = fs.manufacturer.clone();
    let old_selected_mfrs = fs.selected_manufacturers.clone();
    let old_year_from = fs.year_from.clone();
    let old_year_to = fs.year_to.clone();
    let old_fav = fs.other_filters.show_favorites;
    let old_show_avail = fs.availability_filters.show_available;
    let old_show_unavail = fs.availability_filters.show_unavailable;
    let hidden_filters_changed = normalize_hidden_redesign_filters(fs);

    fs.other_filters.show_chd_games = state.chd_only;

    sync_manufacturer_and_year(
        fs,
        state.selected_manufacturer.as_deref(),
        state.year_decade,
    );

    apply_collection(fs, state.collection);

    if old_search != fs.search_text
        || old_chd != fs.other_filters.show_chd_games
        || old_mfr != fs.manufacturer
        || old_selected_mfrs != fs.selected_manufacturers
        || old_year_from != fs.year_from
        || old_year_to != fs.year_to
        || old_fav != fs.other_filters.show_favorites
        || old_show_avail != fs.availability_filters.show_available
        || old_show_unavail != fs.availability_filters.show_unavailable
        || hidden_filters_changed
    {
        action.filters_changed = true;
        app.game_index_manager.mark_cache_dirty();
        app.game_list.invalidate_cache();
    }

    let collection = state.collection;
    let chd_only = state.chd_only;
    let manufacturer = state.selected_manufacturer.clone();
    let year_decade = state.year_decade;
    let search_text = fs.search_text.clone();
    state.mark_filters_synced(
        collection,
        chd_only,
        &manufacturer,
        year_decade,
        &search_text,
    );
}

/// The redesign only exposes title search, collection, CHD, manufacturer, and
/// decade controls. Clear legacy-only predicates so they cannot silently
/// constrain the visible redesign library.
fn normalize_hidden_redesign_filters(fs: &mut FilterSettings) -> bool {
    let changed = fs.search_mode != SearchMode::GameTitle
        || fs.catver_category.is_some()
        || !fs.cpu_filter.is_empty()
        || !fs.device_filter.is_empty()
        || !fs.sound_filter.is_empty()
        || fs.show_favorites_only
        || fs.status_filter != StatusFilter::All
        || !fs.status_filters.show_working
        || !fs.status_filters.show_not_working
        || fs.other_filters.show_parents_only;

    fs.search_mode = SearchMode::GameTitle;
    fs.catver_category = None;
    fs.cpu_filter.clear();
    fs.device_filter.clear();
    fs.sound_filter.clear();
    fs.show_favorites_only = false;
    fs.status_filter = StatusFilter::All;
    fs.status_filters.show_working = true;
    fs.status_filters.show_not_working = true;
    fs.other_filters.show_parents_only = false;

    changed
}

fn sync_manufacturer_and_year(
    fs: &mut FilterSettings,
    manufacturer: Option<&str>,
    year_decade: Option<YearDecade>,
) {
    fs.manufacturer = manufacturer.unwrap_or_default().to_string();
    fs.selected_manufacturers.clear();
    if let Some(manufacturer) = manufacturer {
        fs.selected_manufacturers.insert(manufacturer.to_string());
    }

    if let Some(decade) = year_decade {
        let (from, to) = decade.range();
        fs.year_from = from.to_string();
        fs.year_to = to.to_string();
    } else {
        // Redesign exposes decade selection only, so do not retain an invisible
        // legacy year range after its visible selection has been cleared.
        fs.year_from.clear();
        fs.year_to.clear();
    }
}

fn apply_collection(fs: &mut FilterSettings, collection: RedesignCollection) {
    fs.availability_filters.show_available = true;
    fs.availability_filters.show_unavailable = true;
    fs.other_filters.show_favorites = false;

    match collection {
        RedesignCollection::AllGames => {}
        RedesignCollection::Available => {
            fs.availability_filters.show_unavailable = false;
        }
        RedesignCollection::Favorites => {
            fs.other_filters.show_favorites = true;
        }
        RedesignCollection::Missing => {
            // Broad pre-filter only. The table applies the collection's exact
            // ROM-status predicate after the shared filters and search.
            fs.availability_filters.show_available = false;
        }
        RedesignCollection::Issues => {
            // Issues and Missing are both unavailable to the shared engine, but
            // their disjoint status predicates are enforced by the redesign table.
            fs.availability_filters.show_available = false;
        }
    }
}

fn show_sidebar(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    state: &mut RedesignState,
    action: &mut LibraryAction,
) {
    if state.sidebar_stats_dirty || state.sidebar_stats.games_len != app.games.len() {
        rebuild_sidebar_stats(app, state);
    }
    let sidebar = state.sidebar_stats.clone();

    ui.vertical(|ui| {
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            let search = ui.add(
                egui::TextEdit::singleline(&mut state.search_text_buf)
                    .desired_width(ui.available_width() - 24.0)
                    .hint_text("Search games…  (Ctrl+F)")
                    .font(egui::TextStyle::Body),
            );
            focus_search_if_requested(&search, state);
            if search.changed() {
                state.search_debounce_deadline = Some(ui.input(|i| i.time) + 0.25);
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(250));
            }
        });
        ui.add_space(10.0);

        let footer_height = 52.0;
        let scroll_height = (ui.available_height() - footer_height).max(0.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), scroll_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        section_header(ui, "COLLECTIONS");
                        ui.add_space(4.0);
                        for (collection, label, dot_color, count) in [
                            (
                                RedesignCollection::AllGames,
                                "All Games",
                                RedesignTokens::STATUS_NEUTRAL,
                                sidebar.all,
                            ),
                            (
                                RedesignCollection::Available,
                                "Available",
                                RedesignTokens::STATUS_OK,
                                sidebar.available,
                            ),
                            (
                                RedesignCollection::Favorites,
                                "Favorites",
                                RedesignTokens::STATUS_WARN,
                                sidebar.favorites,
                            ),
                            (
                                RedesignCollection::Missing,
                                "Missing",
                                RedesignTokens::STATUS_MISSING,
                                sidebar.missing,
                            ),
                            (
                                RedesignCollection::Issues,
                                "Issues",
                                RedesignTokens::STATUS_WARN,
                                sidebar.issues,
                            ),
                        ] {
                            let selected = state.collection == collection;
                            let row = sidebar_row(ui, selected, |ui| {
                                ui.horizontal(|ui| {
                                    collection_dot(ui, dot_color);
                                    ui.label(
                                        egui::RichText::new(label).font(fonts::medium(12.5)).color(
                                            if selected {
                                                RedesignTokens::TEXT_BRIGHT
                                            } else {
                                                RedesignTokens::TEXT_SECONDARY
                                            },
                                        ),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(format!("{count}"))
                                                    .size(11.0)
                                                    .color(RedesignTokens::TEXT_MUTED),
                                            );
                                        },
                                    );
                                });
                            });
                            if row.clicked() {
                                state.collection = collection;
                                action.filters_changed = true;
                                state.mark_table_dirty();
                            }
                        }

                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            section_header(ui, "FILTERS");
                            let has_filters = state.year_decade.is_some() || state.chd_only;
                            if has_filters {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if text_link(
                                            ui,
                                            "× Clear",
                                            fonts::semibold(11.0),
                                            RedesignTokens::STATUS_NEUTRAL,
                                        )
                                        .clicked()
                                        {
                                            state.selected_manufacturer = None;
                                            state.year_decade = None;
                                            state.chd_only = false;
                                            action.filters_changed = true;
                                        }
                                    },
                                );
                            }
                        });
                        ui.add_space(4.0);

                        let chd_count = sidebar.chd_count;
                        let chd = state.chd_only;
                        if checkbox_row(ui, &mut state.chd_only, "CHD games only", Some(chd_count))
                            .clicked()
                        {
                            let _ = chd;
                            action.filters_changed = true;
                        }

                        if SHOW_MANUFACTURER_FILTER {
                            ui.add_space(8.0);
                            let mfr_hint = state.selected_manufacturer.as_deref();
                            if collapsible_header(
                                ui,
                                "MANUFACTURER",
                                &mut state.manufacturer_open,
                                mfr_hint,
                            )
                            .clicked()
                            {
                                state.manufacturer_open = !state.manufacturer_open;
                            }
                            if state.manufacturer_open {
                                const MAX_MFR_ROWS: usize = 120;
                                let total = sidebar.manufacturers.len();
                                for (name, count) in sidebar.manufacturers.iter().take(MAX_MFR_ROWS) {
                                    let selected = state.selected_manufacturer.as_deref()
                                        == Some(name.as_str());
                                    let row = sidebar_row(ui, selected, |ui| {
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("{count}"))
                                                        .font(fonts::regular(11.0))
                                                        .color(RedesignTokens::TEXT_MUTED),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::left_to_right(egui::Align::Center),
                                                    |ui| {
                                                        ui.add(
                                                            egui::Label::new(
                                                                egui::RichText::new(name)
                                                                    .font(fonts::medium(12.5))
                                                                    .color(if selected {
                                                                        RedesignTokens::TEXT_BRIGHT
                                                                    } else {
                                                                        RedesignTokens::TEXT_SECONDARY
                                                                    }),
                                                            )
                                                            .truncate()
                                                            .selectable(false),
                                                        );
                                                    },
                                                );
                                            },
                                        );
                                    });
                                    if row.clicked() {
                                        if selected {
                                            state.selected_manufacturer = None;
                                        } else {
                                            state.selected_manufacturer = Some(name.clone());
                                        }
                                        action.filters_changed = true;
                                    }
                                }
                                if total > MAX_MFR_ROWS {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "+ {} more manufacturers…",
                                            total - MAX_MFR_ROWS
                                        ))
                                        .size(11.0)
                                        .color(RedesignTokens::TEXT_FAINT),
                                    );
                                }
                            }
                        }

                        ui.add_space(8.0);
                        let year_hint = state.year_decade.map(|d| d.label());
                        if collapsible_header(
                            ui,
                            "YEAR",
                            &mut state.year_open,
                            year_hint.as_deref(),
                        )
                        .clicked()
                        {
                            state.year_open = !state.year_open;
                        }
                        if state.year_open {
                            ui.horizontal_wrapped(|ui| {
                                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                                for decade in &sidebar.decades {
                                    let selected = state.year_decade == Some(*decade);
                                    let mut button = egui::Button::new(
                                        egui::RichText::new(decade.label())
                                            .font(fonts::medium(12.0))
                                            .color(if selected {
                                                RedesignTokens::TEXT_BRIGHT
                                            } else {
                                                RedesignTokens::TEXT_SECONDARY
                                            }),
                                    )
                                    .selected(selected)
                                    .corner_radius(egui::CornerRadius::same(
                                        RedesignTokens::RADIUS_MD,
                                    ));
                                    if selected {
                                        button = button
                                            .stroke(egui::Stroke::new(1.0_f32, RedesignTokens::ACCENT));
                                    }
                                    if ui.add(button).clicked() {
                                        state.year_decade =
                                            if selected { None } else { Some(*decade) };
                                        action.filters_changed = true;
                                    }
                                }
                            });
                        }
                    });
            },
        );

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), footer_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.separator();
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "catver.ini {} · Last scan: {}",
                        if app.config.catver_ini_path.is_some() {
                            "loaded"
                        } else {
                            "not set"
                        },
                        "—"
                    ))
                    .size(11.0)
                    .color(RedesignTokens::TEXT_FAINT)
                    .line_height(Some(17.0)),
                );
                ui.add_space(8.0);
            },
        );
    });
}

struct CollectionCounts {
    all: usize,
    available: usize,
    favorites: usize,
    missing: usize,
    issues: usize,
}

fn rebuild_sidebar_stats(app: &MameApp, state: &mut RedesignState) {
    let counts = collection_counts(app);
    let mut mfr_counts: HashMap<String, usize> = HashMap::new();
    let mut chd_count = 0usize;

    for g in &app.games {
        if g.requires_chd {
            chd_count += 1;
        }
        // Facets must describe actionable top-level rows. Clone-only metadata
        // cannot produce a row because clones follow their parent family.
        if SHOW_MANUFACTURER_FILTER && !g.is_clone && !g.manufacturer.is_empty() {
            *mfr_counts.entry(g.manufacturer.clone()).or_default() += 1;
        }
    }

    let mut manufacturers: Vec<_> = mfr_counts.into_iter().collect();
    manufacturers.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    state.sidebar_stats = super::super::state::SidebarStats {
        all: counts.all,
        available: counts.available,
        favorites: counts.favorites,
        missing: counts.missing,
        issues: counts.issues,
        chd_count,
        manufacturers,
        decades: YearDecade::FILTER_CHOICES.to_vec(),
        games_len: app.games.len(),
    };
    state.sidebar_stats_dirty = false;
}

fn collection_counts(app: &MameApp) -> CollectionCounts {
    let all = app.games.len();
    let available = app
        .games
        .iter()
        .filter(|g| RedesignCollection::Available.matches_status(g.status))
        .count();
    let favorites = app.config.favorite_games.len();
    let missing = app
        .games
        .iter()
        .filter(|g| RedesignCollection::Missing.matches_status(g.status))
        .count();
    let issues = app
        .games
        .iter()
        .filter(|g| RedesignCollection::Issues.matches_status(g.status))
        .count();
    CollectionCounts {
        all,
        available,
        favorites,
        missing,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_search_focus_is_applied_to_the_rendered_text_edit() {
        let ctx = egui::Context::default();
        let mut state = RedesignState::default();
        state.request_search_focus();
        let mut focused = false;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let search = ui.add(egui::TextEdit::singleline(&mut state.search_text_buf));
                focus_search_if_requested(&search, &mut state);
                focused = ui.memory(|memory| memory.has_focus(search.id));
            });
        });

        assert!(focused);
        assert!(!state.search_focus_requested);
    }

    #[test]
    fn hidden_manufacturer_is_cleared_while_visible_decade_is_synced() {
        let mut filters = FilterSettings::default();
        let eighties = YearDecade::from_year(1985).expect("valid year");
        let mut state = RedesignState::default();
        state.selected_manufacturer = Some("Capcom".to_string());
        state.manufacturer_open = true;

        enforce_visible_filter_state(&mut state);
        sync_manufacturer_and_year(
            &mut filters,
            state.selected_manufacturer.as_deref(),
            Some(eighties),
        );

        assert_eq!(state.selected_manufacturer, None);
        assert!(!state.manufacturer_open);
        assert!(filters.manufacturer.is_empty());
        assert!(filters.selected_manufacturers.is_empty());
        assert_eq!(filters.year_from, "1980");
        assert_eq!(filters.year_to, "1989");

        sync_manufacturer_and_year(&mut filters, None, None);
        assert!(filters.manufacturer.is_empty());
        assert!(filters.selected_manufacturers.is_empty());
        assert!(filters.year_from.is_empty());
        assert!(filters.year_to.is_empty());
    }

    #[test]
    fn legacy_only_filters_are_cleared_before_redesign_filtering() {
        let mut filters = FilterSettings::default();
        filters.search_mode = SearchMode::Cpu;
        filters.catver_category = Some("Fighter".to_string());
        filters.cpu_filter = "Z80".to_string();
        filters.device_filter = "screen".to_string();
        filters.sound_filter = "YM2151".to_string();
        filters.show_favorites_only = true;
        filters.status_filter = StatusFilter::NotWorkingOnly;
        filters.status_filters.show_working = false;
        filters.other_filters.show_parents_only = true;

        assert!(normalize_hidden_redesign_filters(&mut filters));
        assert_eq!(filters.search_mode, SearchMode::GameTitle);
        assert!(filters.catver_category.is_none());
        assert!(filters.cpu_filter.is_empty());
        assert!(filters.device_filter.is_empty());
        assert!(filters.sound_filter.is_empty());
        assert!(!filters.show_favorites_only);
        assert_eq!(filters.status_filter, StatusFilter::All);
        assert!(filters.status_filters.show_working);
        assert!(filters.status_filters.show_not_working);
        assert!(!filters.other_filters.show_parents_only);
    }

    #[test]
    fn default_filters_need_no_hidden_redesign_normalization() {
        let mut filters = FilterSettings::default();
        assert!(!normalize_hidden_redesign_filters(&mut filters));
    }
}
