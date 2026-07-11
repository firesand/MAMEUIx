//! Virtual-scrolled game table for the redesign library view.

use super::super::fonts;
use super::super::state::{RedesignCollection, RedesignState, TableRow};
use super::super::tokens::RedesignTokens;
use super::super::widgets::status_dot;
use super::library::LibraryAction;
use crate::app::MameApp;
use crate::models::Game;
use crate::ui::panels::artwork_loader::ArtworkType;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::collections::{HashMap, HashSet};

const EXPAND_WIDTH: f32 = 18.0;
const STAR_WIDTH: f32 = 30.0;
const ICON_WIDTH: f32 = 40.0;
const YEAR_WIDTH: f32 = 44.0;
const TABLE_COLUMN_GAP: f32 = 10.0;
const TABLE_ROW_HORIZONTAL_PADDING: f32 = 16.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TableMode {
    Full,
    Compact,
    NarrowWithIcon,
    Narrow,
}

#[derive(Clone, Copy)]
enum TableColumn {
    Expand,
    Star,
    Icon,
    Title,
    Manufacturer,
    Year,
    Set,
    Status,
}

const FULL_COLUMNS: &[TableColumn] = &[
    TableColumn::Expand,
    TableColumn::Star,
    TableColumn::Icon,
    TableColumn::Title,
    TableColumn::Manufacturer,
    TableColumn::Year,
    TableColumn::Set,
    TableColumn::Status,
];
const COMPACT_COLUMNS: &[TableColumn] = &[
    TableColumn::Expand,
    TableColumn::Star,
    TableColumn::Icon,
    TableColumn::Title,
    TableColumn::Year,
    TableColumn::Status,
];
const NARROW_ICON_COLUMNS: &[TableColumn] = &[
    TableColumn::Expand,
    TableColumn::Star,
    TableColumn::Icon,
    TableColumn::Title,
    TableColumn::Status,
];
const NARROW_COLUMNS: &[TableColumn] = &[
    TableColumn::Expand,
    TableColumn::Star,
    TableColumn::Title,
    TableColumn::Status,
];

impl TableMode {
    fn columns(self) -> &'static [TableColumn] {
        match self {
            Self::Full => FULL_COLUMNS,
            Self::Compact => COMPACT_COLUMNS,
            Self::NarrowWithIcon => NARROW_ICON_COLUMNS,
            Self::Narrow => NARROW_COLUMNS,
        }
    }

    fn preferred_content_width(self) -> f32 {
        match self {
            // Fixed columns + the handoff's minimum widths for flexible columns.
            Self::Full => {
                EXPAND_WIDTH + STAR_WIDTH + ICON_WIDTH + 130.0 + 80.0 + YEAR_WIDTH + 64.0 + 96.0
            }
            Self::Compact => EXPAND_WIDTH + STAR_WIDTH + ICON_WIDTH + 130.0 + YEAR_WIDTH + 96.0,
            Self::NarrowWithIcon => EXPAND_WIDTH + STAR_WIDTH + ICON_WIDTH + 110.0 + 80.0,
            Self::Narrow => EXPAND_WIDTH + STAR_WIDTH + 88.0 + 64.0,
        }
    }

    fn minimum_width(self, column: TableColumn) -> f32 {
        match (self, column) {
            (Self::Full | Self::Compact, TableColumn::Title) => 130.0,
            (Self::NarrowWithIcon, TableColumn::Title) => 110.0,
            (Self::Narrow, TableColumn::Title) => 88.0,
            (Self::Full | Self::Compact, TableColumn::Status) => 96.0,
            (Self::NarrowWithIcon, TableColumn::Status) => 80.0,
            (Self::Narrow, TableColumn::Status) => 64.0,
            (_, TableColumn::Manufacturer) => 80.0,
            (_, TableColumn::Set) => 64.0,
            _ => 0.0,
        }
    }
}

struct TableWidths {
    expand: f32,
    star: f32,
    icon: f32,
    title: f32,
    manufacturer: f32,
    year: f32,
    set: f32,
    status: f32,
}

impl TableWidths {
    fn get(&self, column: TableColumn) -> f32 {
        match column {
            TableColumn::Expand => self.expand,
            TableColumn::Star => self.star,
            TableColumn::Icon => self.icon,
            TableColumn::Title => self.title,
            TableColumn::Manufacturer => self.manufacturer,
            TableColumn::Year => self.year,
            TableColumn::Set => self.set,
            TableColumn::Status => self.status,
        }
    }
}

fn mode_fits(mode: TableMode, available_width: f32, column_gap: f32, scrollbar: f32) -> bool {
    let gap_count = mode.columns().len().saturating_sub(1) as f32;
    mode.preferred_content_width() + gap_count * column_gap + scrollbar <= available_width
}

fn choose_table_mode(available_width: f32, column_gap: f32, scrollbar: f32) -> TableMode {
    if mode_fits(TableMode::Full, available_width, column_gap, scrollbar) {
        TableMode::Full
    } else if mode_fits(TableMode::Compact, available_width, column_gap, scrollbar) {
        TableMode::Compact
    } else if mode_fits(
        TableMode::NarrowWithIcon,
        available_width,
        column_gap,
        scrollbar,
    ) {
        TableMode::NarrowWithIcon
    } else {
        TableMode::Narrow
    }
}

fn allocate_weighted<const N: usize>(
    available: f32,
    minimums: [f32; N],
    weights: [f32; N],
) -> [f32; N] {
    let available = available.max(0.0);
    let minimum_total: f32 = minimums.iter().sum();
    if available <= minimum_total {
        let scale = if minimum_total > 0.0 {
            available / minimum_total
        } else {
            0.0
        };
        return minimums.map(|minimum| minimum * scale);
    }

    let mut widths = [0.0; N];
    let mut active = [true; N];
    let mut remaining = available;

    loop {
        let active_weight: f32 = (0..N)
            .filter(|&index| active[index])
            .map(|index| weights[index])
            .sum();
        if active_weight <= 0.0 {
            break;
        }

        let mut clamped_any = false;
        for index in 0..N {
            if active[index] && remaining * weights[index] / active_weight < minimums[index] {
                widths[index] = minimums[index];
                remaining -= minimums[index];
                active[index] = false;
                clamped_any = true;
            }
        }

        if !clamped_any {
            for index in 0..N {
                if active[index] {
                    widths[index] = remaining * weights[index] / active_weight;
                }
            }
            break;
        }
    }

    widths
}

fn compute_table_widths(content_width: f32, mode: TableMode) -> TableWidths {
    // Fixed columns keep their handoff sizes while there is enough room. On extremely
    // narrow windows they scale with the flexible minimums so the table never over-allocates.
    let fixed_scale = (content_width / mode.preferred_content_width()).min(1.0);
    let mut widths = TableWidths {
        expand: EXPAND_WIDTH * fixed_scale,
        star: STAR_WIDTH * fixed_scale,
        icon: if matches!(mode, TableMode::Narrow) {
            0.0
        } else {
            ICON_WIDTH * fixed_scale
        },
        title: 0.0,
        manufacturer: 0.0,
        year: if matches!(mode, TableMode::Full | TableMode::Compact) {
            YEAR_WIDTH * fixed_scale
        } else {
            0.0
        },
        set: 0.0,
        status: 0.0,
    };
    let fixed_width = widths.expand + widths.star + widths.icon + widths.year;
    let flexible_width = (content_width - fixed_width).max(0.0);

    match mode {
        TableMode::Full => {
            let [title, manufacturer, set, status] = allocate_weighted(
                flexible_width,
                [130.0, 80.0, 64.0, 96.0],
                [1.5, 1.0, 0.7, 0.8],
            );
            widths.title = title;
            widths.manufacturer = manufacturer;
            widths.set = set;
            widths.status = status;
        }
        TableMode::Compact => {
            let [title, status] = allocate_weighted(flexible_width, [130.0, 96.0], [1.5, 0.8]);
            widths.title = title;
            widths.status = status;
        }
        TableMode::NarrowWithIcon => {
            let [title, status] = allocate_weighted(flexible_width, [110.0, 80.0], [1.5, 0.8]);
            widths.title = title;
            widths.status = status;
        }
        TableMode::Narrow => {
            let [title, status] = allocate_weighted(flexible_width, [88.0, 64.0], [1.5, 0.8]);
            widths.title = title;
            widths.status = status;
        }
    }

    widths
}

fn table_column(column: TableColumn, mode: TableMode, widths: &TableWidths) -> Column {
    let width = widths.get(column);
    match column {
        TableColumn::Expand | TableColumn::Star | TableColumn::Icon | TableColumn::Year => {
            Column::exact(width)
        }
        // STATUS is always the final visible column. Let it absorb rounding and
        // small viewport changes so the right edge cannot disappear again.
        TableColumn::Status => Column::remainder()
            .at_least(mode.minimum_width(column).min(width))
            .clip(true),
        TableColumn::Title | TableColumn::Manufacturer | TableColumn::Set => Column::initial(width)
            .at_least(mode.minimum_width(column).min(width))
            .clip(true)
            .resizable(true),
    }
}

fn table_content_width(
    available_width: f32,
    column_count: usize,
    column_gap: f32,
    scrollbar: f32,
) -> f32 {
    let gap_count = column_count.saturating_sub(1) as f32;
    (available_width - gap_count * column_gap - scrollbar).max(0.0)
}

fn header_text(column: TableColumn) -> &'static str {
    match column {
        TableColumn::Expand | TableColumn::Icon => "",
        TableColumn::Star => "★",
        TableColumn::Title => "TITLE",
        TableColumn::Manufacturer => "MANUFACTURER",
        TableColumn::Year => "YEAR",
        TableColumn::Set => "SET",
        TableColumn::Status => "STATUS",
    }
}

pub fn show_game_table(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    state: &mut RedesignState,
    action: &mut LibraryAction,
) {
    use crate::models::LoadingStage;

    match app.loading_stage {
        LoadingStage::LoadingMame => {
            loading_message(ui, "Loading MAME database…", true);
            return;
        }
        LoadingStage::ScanningRoms => {
            ui.centered_and_justified(|ui| {
                ui.spinner();
                ui.add_space(8.0);
                let (current, total) = app.loading_progress;
                if total > 0 {
                    ui.add(
                        egui::ProgressBar::new(current as f32 / total as f32)
                            .text(format!("Scanning ROMs {current}/{total}"))
                            .desired_width(320.0),
                    );
                } else {
                    ui.label(msg("Scanning ROM files…"));
                }
            });
            return;
        }
        LoadingStage::Error => {
            loading_message(
                ui,
                "Failed to load games. Check MAME path in Settings → Directories.",
                false,
            );
            return;
        }
        LoadingStage::Idle if app.games.is_empty() => {
            loading_message(ui, "Waiting for MAME configuration…", true);
            return;
        }
        LoadingStage::Complete | LoadingStage::Idle => {}
    }

    if state.table_rows_dirty {
        rebuild_table_rows(app, state);
    }

    if state.table_rows.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(msg("No games match this search or filter."));
        });
        return;
    }

    let scroll_h = ui.available_height().max(120.0);
    let body_h = (scroll_h - RedesignTokens::HEADER_ROW_HEIGHT).max(80.0);

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(16, 0))
        .show(ui, |ui| {
            let previous_scroll = ui.spacing().scroll;
            let previous_item_spacing = ui.spacing().item_spacing;
            ui.spacing_mut().scroll = egui::style::ScrollStyle {
                floating: false,
                bar_width: 10.0,
                ..previous_scroll
            };
            // TableBody adds item_spacing.y to every virtual row. Keep it at
            // zero so ROW_HEIGHT remains the actual 44 px row height; the 1 px
            // divider is painted explicitly below.
            ui.spacing_mut().item_spacing = egui::vec2(TABLE_COLUMN_GAP, 0.0);

            // Body painters are clipped inside the scroll area's content box.
            // Keep the outer painter so row hover can include the frame's 16 px
            // horizontal padding, while the body clip below still constrains Y.
            let table_painter = ui.painter().clone();
            let row_x_range = ui
                .max_rect()
                .expand2(egui::vec2(TABLE_ROW_HORIZONTAL_PADDING, 0.0))
                .x_range();

            let scrollbar = ui.spacing().scroll.allocated_width();
            let available_width = ui.available_width();
            let mode = choose_table_mode(available_width, TABLE_COLUMN_GAP, scrollbar);
            let columns = mode.columns();
            let content_width =
                table_content_width(available_width, columns.len(), TABLE_COLUMN_GAP, scrollbar);
            let widths = compute_table_widths(content_width, mode);
            // TableBuilder owns its resize state. Key it by the current logical
            // width so a window resize recalculates safe proportions, while a
            // user-adjusted layout remains stable for that viewport width.
            let viewport_width_key = available_width.round() as i32;

            let mut table = TableBuilder::new(ui)
                .id_salt(("redesign_game_table", mode, viewport_width_key))
                .striped(false)
                .resizable(true)
                .sense(egui::Sense::click())
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .min_scrolled_height(0.0)
                .max_scroll_height(body_h)
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .vscroll(true)
                .drag_to_scroll(true);

            for &column in columns {
                table = table.column(table_column(column, mode, &widths));
            }

            table
                .header(RedesignTokens::HEADER_ROW_HEIGHT, |mut header| {
                    for &column in columns {
                        header.col(|ui| {
                            ui.label(header_label(header_text(column)));
                        });
                    }
                })
                .body(|mut body| {
                    let row_count = state.table_rows.len();
                    let body_clip = body.ui_mut().clip_rect();
                    let row_painter = table_painter.with_clip_rect(egui::Rect::from_x_y_ranges(
                        table_painter.clip_rect().x_range(),
                        body_clip.y_range(),
                    ));
                    body.rows(RedesignTokens::ROW_HEIGHT, row_count, |mut row| {
                        let hover_background = row_painter.add(egui::Shape::Noop);
                        // egui_extras remembers row hover from the previous frame.
                        // Disable that delayed paint and use the current response below.
                        row.set_hovered(false);
                        let Some(table_row) = state.table_rows.get(row.index()).cloned() else {
                            return;
                        };
                        render_table_row(
                            &mut row,
                            app,
                            state,
                            action,
                            &table_row,
                            columns,
                            &row_painter,
                            hover_background,
                            row_x_range,
                        );
                    });
                });

            ui.spacing_mut().scroll = previous_scroll;
            ui.spacing_mut().item_spacing = previous_item_spacing;
        });
}

fn loading_message(ui: &mut egui::Ui, text: &str, spinner: bool) {
    ui.centered_and_justified(|ui| {
        if spinner {
            ui.spinner();
            ui.add_space(8.0);
        }
        ui.label(msg(text));
    });
}

fn msg(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .size(13.0)
        .color(RedesignTokens::TEXT_FAINT)
}

fn header_label(text: &str) -> egui::RichText {
    egui::RichText::new(text)
        .font(fonts::bold(11.0))
        .color(RedesignTokens::TEXT_FAINT)
        .extra_letter_spacing(0.8)
}

fn clipped_label(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.add(egui::Label::new(text).truncate().selectable(false));
}

fn clickable_cell_text(
    ui: &mut egui::Ui,
    text: &str,
    font: egui::FontId,
    idle_color: egui::Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::click());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        font,
        if response.hovered() {
            RedesignTokens::TEXT_BRIGHT
        } else {
            idle_color
        },
    );
    response
}

fn table_row_background_rect(row_rect: egui::Rect, x_range: egui::Rangef) -> egui::Rect {
    egui::Rect::from_x_y_ranges(x_range, row_rect.y_range())
}

pub fn rebuild_table_rows(app: &MameApp, state: &mut RedesignState) {
    let filtered = app.game_index_manager.get_filtered_games();
    state.table_rows = build_table_rows(
        &app.games,
        filtered,
        &state.expanded_parents,
        state.collection,
    );
    state.table_rows_dirty = false;
}

fn build_table_rows(
    games: &[Game],
    filtered: &[usize],
    expanded_parents: &HashSet<String>,
    collection: RedesignCollection,
) -> Vec<TableRow> {
    let mut clone_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, game) in games.iter().enumerate() {
        if game.is_clone
            && let Some(parent) = &game.parent
        {
            clone_map.entry(parent.clone()).or_default().push(idx);
        }
    }

    let mut rows = Vec::new();
    for &idx in filtered {
        let Some(game) = games.get(idx) else {
            continue;
        };
        if game.is_clone || !collection.matches_status(game.status) {
            continue;
        }
        let clone_count = clone_map.get(&game.name).map(|v| v.len()).unwrap_or(0);
        rows.push(TableRow::Parent {
            index: idx,
            clone_count,
        });
        if expanded_parents.contains(&game.name)
            && let Some(clones) = clone_map.get(&game.name)
        {
            for &cidx in clones {
                rows.push(TableRow::Clone { index: cidx });
            }
        }
    }
    rows
}

fn render_table_row(
    row: &mut egui_extras::TableRow<'_, '_>,
    app: &mut MameApp,
    state: &mut RedesignState,
    action: &mut LibraryAction,
    table_row: &TableRow,
    columns: &[TableColumn],
    hover_painter: &egui::Painter,
    hover_background: egui::layers::ShapeIdx,
    row_x_range: egui::Rangef,
) {
    let (index, is_clone, clone_count) = match table_row {
        TableRow::Parent { index, clone_count } => (*index, false, *clone_count),
        TableRow::Clone { index } => (*index, true, 0),
    };

    let Some(game) = app.games.get(index).cloned() else {
        return;
    };

    let game_name = game.name.clone();
    let mut suppress_row_open = false;
    let mut interactive_child_hovered = false;

    for &column in columns {
        match column {
            TableColumn::Expand => row.col(|ui| {
                if is_clone || clone_count == 0 {
                    return;
                }
                let expanded = state.expanded_parents.contains(&game_name);
                let chevron = if expanded { "▾" } else { "▸" };
                let response = clickable_cell_text(
                    ui,
                    chevron,
                    fonts::regular(10.0),
                    RedesignTokens::TEXT_FAINT,
                );
                interactive_child_hovered |= response.hovered();
                if response.clicked() {
                    suppress_row_open = true;
                    if expanded {
                        state.expanded_parents.remove(&game_name);
                    } else {
                        state.expanded_parents.insert(game_name.clone());
                    }
                    state.mark_table_dirty();
                }
            }),
            TableColumn::Star => row.col(|ui| {
                if is_clone {
                    return;
                }
                let is_fav = app.config.favorite_games.contains(&game_name);
                let star = if is_fav { "★" } else { "☆" };
                let color = if is_fav {
                    RedesignTokens::STATUS_WARN
                } else {
                    RedesignTokens::STAR_INACTIVE
                };
                let response = clickable_cell_text(ui, star, fonts::regular(14.0), color);
                interactive_child_hovered |= response.hovered();
                if response.clicked() {
                    suppress_row_open = true;
                    action.toggle_favorite = Some(game_name.clone());
                }
            }),
            TableColumn::Icon => row.col(|ui| {
                let texture = table_icon_texture(ui, app, state, &game);
                paint_table_icon(ui, texture, is_clone);
            }),
            TableColumn::Title => row.col(|ui| {
                let title = if is_clone {
                    format!("↳ {}", game.description)
                } else if clone_count > 0 {
                    format!("{}  ·  {clone_count} clones", game.description)
                } else {
                    game.description.clone()
                };
                clipped_label(
                    ui,
                    egui::RichText::new(title)
                        .font(fonts::semibold(13.0))
                        .color(if is_clone {
                            RedesignTokens::TEXT_SECONDARY
                        } else {
                            RedesignTokens::TEXT_PRIMARY
                        }),
                );
            }),
            TableColumn::Manufacturer => row.col(|ui| {
                clipped_label(
                    ui,
                    egui::RichText::new(&game.manufacturer)
                        .font(fonts::regular(12.0))
                        .color(RedesignTokens::TEXT_SECONDARY),
                );
            }),
            TableColumn::Year => row.col(|ui| {
                clipped_label(
                    ui,
                    egui::RichText::new(&game.year)
                        .font(fonts::regular(12.0))
                        .color(RedesignTokens::TEXT_SECONDARY),
                );
            }),
            TableColumn::Set => row.col(|ui| {
                clipped_label(
                    ui,
                    egui::RichText::new(format!("{}.zip", game.name))
                        .monospace()
                        .size(12.0)
                        .color(RedesignTokens::TEXT_MUTED),
                );
            }),
            TableColumn::Status => row.col(|ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    ui.set_max_width(ui.available_width());
                    status_dot(ui, RedesignTokens::status_color(game.status));
                    clipped_label(
                        ui,
                        egui::RichText::new(RedesignTokens::status_label(game.status))
                            .font(fonts::regular(12.0))
                            .color(RedesignTokens::TEXT_SECONDARY),
                    );
                });
            }),
        };
    }

    let row_response = row.response();
    if row_response.hovered() || interactive_child_hovered {
        hover_painter.set(
            hover_background,
            egui::Shape::rect_filled(
                table_row_background_rect(row_response.rect, row_x_range),
                egui::CornerRadius::ZERO,
                RedesignTokens::BG_ROW_HOVER,
            ),
        );
    }
    hover_painter.hline(
        row_x_range,
        row_response.rect.bottom() - 0.5,
        egui::Stroke::new(1.0, RedesignTokens::ROW_DIVIDER),
    );
    if row_response.clicked() && !suppress_row_open {
        action.open_detail = Some(index);
    }
}

fn table_icon_texture(
    ui: &mut egui::Ui,
    app: &mut MameApp,
    state: &mut RedesignState,
    game: &Game,
) -> Option<egui::TextureHandle> {
    if app.config.show_rom_icons && app.config.icons_path.is_some() {
        if let Some(texture) = app.icon_manager.rom_icons.get(&game.name) {
            return Some(texture.clone());
        }

        app.queue_icon_load(game.name.clone());
        ui.ctx().request_repaint();
    }

    load_snapshot_thumbnail(ui.ctx(), state, game, app)
}

fn load_snapshot_thumbnail(
    ctx: &egui::Context,
    state: &mut RedesignState,
    game: &Game,
    app: &MameApp,
) -> Option<egui::TextureHandle> {
    for rom_name in table_artwork_candidates(game) {
        if let Some(texture) =
            state
                .artwork_loader
                .load_artwork(ctx, rom_name, ArtworkType::Screenshot, &app.config)
        {
            return Some(texture);
        }
    }
    None
}

fn table_artwork_candidates(game: &Game) -> Vec<&str> {
    let mut candidates = vec![game.name.as_str()];
    if let Some(parent) = game.parent.as_deref()
        && parent != game.name
    {
        candidates.push(parent);
    }
    candidates
}

fn paint_table_icon(ui: &mut egui::Ui, texture: Option<egui::TextureHandle>, is_clone: bool) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
    let alpha = if is_clone { 0.45 } else { 1.0 };
    ui.painter().rect_filled(
        rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_SM),
        RedesignTokens::BG_RAISED.linear_multiply(alpha),
    );

    if let Some(texture) = texture {
        paint_contained_icon(ui, rect.shrink(2.0), &texture, alpha);
    }
}

fn paint_contained_icon(
    ui: &egui::Ui,
    rect: egui::Rect,
    texture: &egui::TextureHandle,
    alpha: f32,
) {
    let texture_size = texture.size_vec2();
    if texture_size.x <= 0.0 || texture_size.y <= 0.0 || rect.width() <= 0.0 || rect.height() <= 0.0
    {
        return;
    }

    let scale = (rect.width() / texture_size.x).min(rect.height() / texture_size.y);
    let image_size = texture_size * scale;
    let image_rect = egui::Rect::from_center_size(rect.center(), image_size);
    let uv = egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0));
    ui.painter().image(
        texture.id(),
        image_rect,
        uv,
        egui::Color32::WHITE.linear_multiply(alpha),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{FilterCategory, FilterSettings, GameIndex, RomStatus};
    use crate::ui::panels::GameIndexManager;

    const SCROLLBAR: f32 = 16.0;

    #[test]
    fn mode_thresholds_keep_required_columns_visible() {
        assert_eq!(
            choose_table_mode(588.0, TABLE_COLUMN_GAP, SCROLLBAR),
            TableMode::Full
        );
        assert_eq!(
            choose_table_mode(587.0, TABLE_COLUMN_GAP, SCROLLBAR),
            TableMode::Compact
        );
        assert_eq!(
            choose_table_mode(423.0, TABLE_COLUMN_GAP, SCROLLBAR),
            TableMode::NarrowWithIcon
        );
        assert_eq!(
            choose_table_mode(333.0, TABLE_COLUMN_GAP, SCROLLBAR),
            TableMode::Narrow
        );
    }

    #[test]
    fn computed_columns_never_exceed_content_budget() {
        for (available, mode) in [
            (900.0, TableMode::Full),
            (500.0, TableMode::Compact),
            (360.0, TableMode::NarrowWithIcon),
            (220.0, TableMode::Narrow),
        ] {
            let columns = mode.columns();
            let content =
                table_content_width(available, columns.len(), TABLE_COLUMN_GAP, SCROLLBAR);
            let widths = compute_table_widths(content, mode);
            let used: f32 = columns.iter().map(|&column| widths.get(column)).sum();
            assert!(
                (used - content).abs() < 0.01,
                "{mode:?} uses {used} from {content}"
            );
        }
    }

    #[test]
    fn hover_background_uses_outer_width_without_changing_row_height() {
        let row_rect = egui::Rect::from_min_size(egui::pos2(24.0, 50.0), egui::vec2(200.0, 44.0));
        let background = table_row_background_rect(row_rect, egui::Rangef::new(8.0, 248.0));

        assert_eq!(background.left(), 8.0);
        assert_eq!(background.right(), 248.0);
        assert_eq!(background.top(), row_rect.top());
        assert_eq!(background.bottom(), row_rect.bottom());
        assert_eq!(background.height(), RedesignTokens::ROW_HEIGHT);
    }

    fn hierarchy_game(name: &str, manufacturer: &str, year: &str, parent: Option<&str>) -> Game {
        Game {
            name: name.to_string(),
            description: name.to_string(),
            manufacturer: manufacturer.to_string(),
            year: year.to_string(),
            driver: "test".to_string(),
            driver_status: "good".to_string(),
            status: RomStatus::Available,
            parent: parent.map(str::to_string),
            category: "Test".to_string(),
            play_count: 0,
            is_clone: parent.is_some(),
            is_device: false,
            is_bios: false,
            controls: String::new(),
            requires_chd: false,
            chd_name: None,
            verification_status: None,
        }
    }

    #[test]
    fn matching_parent_keeps_all_clones_and_clone_match_is_not_promoted() {
        let games = vec![
            hierarchy_game("parent", "Capcom", "1989", None),
            hierarchy_game("clone", "Bootleg", "1991", Some("parent")),
        ];
        let expanded = HashSet::from(["parent".to_string()]);

        assert_eq!(
            build_table_rows(&games, &[0], &expanded, RedesignCollection::AllGames,),
            vec![
                TableRow::Parent {
                    index: 0,
                    clone_count: 1,
                },
                TableRow::Clone { index: 1 },
            ]
        );
        assert!(
            build_table_rows(&games, &[1], &expanded, RedesignCollection::AllGames,).is_empty()
        );
    }

    #[test]
    fn manufacturer_and_year_filter_parent_families_end_to_end() {
        let games = vec![
            hierarchy_game("capcom_parent", "Capcom", "1989", None),
            hierarchy_game("bootleg_clone", "Bootleg", "1991", Some("capcom_parent")),
            hierarchy_game("sega_parent", "Sega", "1991", None),
            hierarchy_game("capcom_clone", "Capcom", "1989", Some("sega_parent")),
        ];
        let mut manager = GameIndexManager::new();
        manager.enhanced_search = None;
        manager.game_index = Some(GameIndex::build(games.clone(), HashSet::new()));

        let mut filters = FilterSettings {
            year_from: "1980".to_string(),
            year_to: "1989".to_string(),
            ..FilterSettings::default()
        };
        filters.selected_manufacturers.insert("Capcom".to_string());
        manager.update_filtered_games_cache(
            &games,
            FilterCategory::All,
            &filters,
            &HashSet::new(),
            None,
        );

        // The core predicate also matches capcom_clone, but a clone-only match
        // must not promote its non-matching Sega parent into the redesign table.
        assert_eq!(manager.get_filtered_games(), &[0, 3]);
        let expanded = HashSet::from(["capcom_parent".to_string()]);
        assert_eq!(
            build_table_rows(
                &games,
                manager.get_filtered_games(),
                &expanded,
                RedesignCollection::AllGames,
            ),
            vec![
                TableRow::Parent {
                    index: 0,
                    clone_count: 1,
                },
                TableRow::Clone { index: 1 },
            ]
        );
    }

    #[test]
    fn missing_and_issue_rows_follow_the_sidebar_status_predicates() {
        let mut games = vec![
            hierarchy_game("missing", "Test", "1980", None),
            hierarchy_game("chd_missing", "Test", "1981", None),
            hierarchy_game("chd_required", "Test", "1982", None),
            hierarchy_game("available", "Test", "1983", None),
        ];
        games[0].status = RomStatus::Missing;
        games[1].status = RomStatus::ChdMissing;
        games[2].status = RomStatus::ChdRequired;
        let filtered = [0, 1, 2, 3];

        assert_eq!(
            build_table_rows(
                &games,
                &filtered,
                &HashSet::new(),
                RedesignCollection::Missing,
            ),
            vec![TableRow::Parent {
                index: 0,
                clone_count: 0,
            }]
        );
        assert_eq!(
            build_table_rows(
                &games,
                &filtered,
                &HashSet::new(),
                RedesignCollection::Issues,
            ),
            vec![
                TableRow::Parent {
                    index: 1,
                    clone_count: 0,
                },
                TableRow::Parent {
                    index: 2,
                    clone_count: 0,
                },
            ]
        );
    }
}
