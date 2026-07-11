use super::fonts;
use super::state::{RedesignNavTab, RedesignState};
use super::tokens::RedesignTokens;
use crate::models::UiShellMode;
use eframe::egui;
use std::sync::Arc;

const NAV_TABS: [(RedesignNavTab, &str, &str); 3] = [
    (RedesignNavTab::Library, "LIBRARY", "LIB"),
    (RedesignNavTab::Verification, "VERIFICATION", "VERIFY"),
    (RedesignNavTab::Settings, "SETTINGS", "SET"),
];

const LOGO_SIZE: f32 = 22.0;
const LOGO_TEXT_GAP: f32 = 10.0;
const WORDMARK_NAV_GAP: f32 = 24.0;
const TAB_GAP: f32 = 2.0;
const TAB_PADDING_X: f32 = 14.0;
const STATUS_GAP: f32 = 16.0;
const WORDMARK_LETTER_SPACING: f32 = 1.96;
const NAV_LETTER_SPACING: f32 = 1.2;

pub fn show_top_bar(
    ui: &mut egui::Ui,
    state: &mut RedesignState,
    mame_version: &str,
    total_sets: usize,
    available_sets: usize,
) {
    let bar_h = RedesignTokens::TOP_BAR_HEIGHT;
    let rect = ui.max_rect();
    let painter = ui.painter().clone();
    let center_y = rect.center().y;

    ui.set_min_height(bar_h);
    ui.set_max_height(bar_h);

    let logo_x = rect.left();
    let logo_rect = egui::Rect::from_center_size(
        egui::pos2(logo_x + LOGO_SIZE * 0.5, center_y),
        egui::vec2(LOGO_SIZE, LOGO_SIZE),
    );
    painter.rect_filled(
        logo_rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
        RedesignTokens::BG_RAISED,
    );
    painter.rect_stroke(
        logo_rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
        egui::Stroke::new(1.0, RedesignTokens::BORDER_STRONG),
        egui::StrokeKind::Inside,
    );
    painter.text(
        logo_rect.center(),
        egui::Align2::CENTER_CENTER,
        "M",
        fonts::bold(11.0),
        RedesignTokens::TEXT_BRIGHT,
    );

    let wordmark = "MAMEUIX";
    let wordmark_galley = spaced_galley(
        &painter,
        wordmark,
        fonts::bold(14.0),
        WORDMARK_LETTER_SPACING,
    );
    let wordmark_x = logo_rect.right() + LOGO_TEXT_GAP;

    let full_widths = nav_widths(&painter, false);
    let compact_widths = nav_widths(&painter, true);
    let branded_tab_x = wordmark_x + wordmark_galley.size().x + WORDMARK_NAV_GAP;
    let unbranded_tab_x = logo_rect.right() + LOGO_TEXT_GAP;

    // Preserve the handoff layout whenever it fits. On narrower windows, branding is
    // progressively reduced before navigation is allowed to leave the visible bar.
    let (show_wordmark, compact_tabs, mut tab_x, tab_widths) =
        if tabs_end(branded_tab_x, &full_widths) <= rect.right() {
            (true, false, branded_tab_x, full_widths)
        } else if tabs_end(unbranded_tab_x, &full_widths) <= rect.right() {
            (false, false, unbranded_tab_x, full_widths)
        } else {
            (false, true, unbranded_tab_x, compact_widths)
        };

    if show_wordmark {
        paint_galley(
            &painter,
            egui::pos2(wordmark_x, center_y),
            egui::Align2::LEFT_CENTER,
            wordmark_galley,
            RedesignTokens::TEXT_BRIGHT,
        );
    }

    let mut tabs_right = tab_x;
    for (index, (tab, full_label, compact_label)) in NAV_TABS.into_iter().enumerate() {
        let label = if compact_tabs {
            compact_label
        } else {
            full_label
        };
        let tab_rect = nav_tab(
            ui,
            state,
            tab,
            label,
            tab_x,
            tab_widths[index],
            center_y,
            bar_h,
        );
        tabs_right = tab_rect.right();
        tab_x = tabs_right + TAB_GAP;
    }

    let status_text =
        format!("MAME {mame_version} · {total_sets} sets · {available_sets} available");
    let status_font = fonts::regular(12.0);
    let status_galley = painter.layout_no_wrap(
        status_text.clone(),
        status_font.clone(),
        RedesignTokens::TEXT_MUTED,
    );
    let status_left = rect.right() - status_galley.size().x;
    if status_left >= tabs_right + STATUS_GAP {
        painter.text(
            egui::pos2(rect.right(), center_y),
            egui::Align2::RIGHT_CENTER,
            status_text,
            status_font,
            RedesignTokens::TEXT_MUTED,
        );
    }

    ui.advance_cursor_after_rect(rect);
}

fn nav_widths(painter: &egui::Painter, compact: bool) -> [f32; NAV_TABS.len()] {
    NAV_TABS.map(|(_, full_label, compact_label)| {
        let label = if compact { compact_label } else { full_label };
        let galley = spaced_galley(painter, label, fonts::semibold(12.0), NAV_LETTER_SPACING);
        galley.size().x + TAB_PADDING_X * 2.0
    })
}

fn tabs_end(start_x: f32, widths: &[f32; NAV_TABS.len()]) -> f32 {
    start_x + widths.iter().sum::<f32>() + TAB_GAP * (widths.len() - 1) as f32
}

fn nav_tab(
    ui: &mut egui::Ui,
    state: &mut RedesignState,
    tab: RedesignNavTab,
    label: &str,
    x: f32,
    width: f32,
    center_y: f32,
    bar_h: f32,
) -> egui::Rect {
    let active = state.active_nav_tab() == tab;
    let text_color = if active {
        RedesignTokens::TEXT_BRIGHT
    } else {
        RedesignTokens::STATUS_NEUTRAL
    };

    let rect = egui::Rect::from_center_size(
        egui::pos2(x + width * 0.5, center_y),
        egui::vec2(width, bar_h),
    );
    let hit_rect = rect.intersect(ui.max_rect()).intersect(ui.clip_rect());
    let response = hit_rect
        .is_positive()
        .then(|| ui.interact(hit_rect, nav_tab_id(ui, tab), egui::Sense::click()));
    let hovered = response.as_ref().is_some_and(egui::Response::hovered);

    if hovered && !active {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::ZERO, RedesignTokens::BG_HOVER);
    }

    let label_galley = spaced_galley(
        ui.painter(),
        label,
        fonts::semibold(12.0),
        NAV_LETTER_SPACING,
    );
    paint_galley(
        ui.painter(),
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label_galley,
        if hovered && !active {
            RedesignTokens::TEXT_BRIGHT
        } else {
            text_color
        },
    );

    if active {
        let underline = egui::Rect::from_min_size(
            egui::pos2(rect.left(), rect.bottom() - 2.0),
            egui::vec2(rect.width(), 2.0),
        );
        ui.painter()
            .rect_filled(underline, egui::CornerRadius::ZERO, RedesignTokens::ACCENT);
    }

    if response.is_some_and(|response| response.clicked()) {
        state.navigate_to(tab);
    }

    rect
}

fn spaced_galley(
    painter: &egui::Painter,
    text: &str,
    font_id: egui::FontId,
    letter_spacing: f32,
) -> Arc<egui::Galley> {
    let mut format = egui::TextFormat::simple(font_id, egui::Color32::PLACEHOLDER);
    format.extra_letter_spacing = letter_spacing;
    let mut job = egui::text::LayoutJob::single_section(text.to_string(), format);
    job.break_on_newline = false;
    painter.layout_job(job)
}

fn paint_galley(
    painter: &egui::Painter,
    position: egui::Pos2,
    anchor: egui::Align2,
    galley: Arc<egui::Galley>,
    color: egui::Color32,
) {
    let rect = anchor.anchor_size(position, galley.size());
    painter.galley(rect.min, galley, color);
}

fn nav_tab_id(ui: &egui::Ui, tab: RedesignNavTab) -> egui::Id {
    let name = match tab {
        RedesignNavTab::Library => "library",
        RedesignNavTab::Verification => "verification",
        RedesignNavTab::Settings => "settings",
    };
    ui.id().with(("redesign_topbar_tab", name))
}

/// Quick escape hatch back to legacy UI from the redesign settings page.
pub fn legacy_shell_switcher(ui: &mut egui::Ui, ui_shell: &mut UiShellMode, on_change: &mut bool) {
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("UI shell")
            .font(fonts::bold(11.0))
            .color(RedesignTokens::TEXT_FAINT),
    );
    for mode in [
        UiShellMode::LegacyDock,
        UiShellMode::LegacyClassic,
        UiShellMode::RedesignPreview,
    ] {
        if ui.radio(*ui_shell == mode, mode.display_name()).clicked() {
            *ui_shell = mode;
            *on_change = true;
        }
        ui.label(
            egui::RichText::new(mode.description())
                .size(11.0)
                .color(RedesignTokens::TEXT_MUTED),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_measurement_includes_configured_letter_spacing() {
        let ctx = egui::Context::default();
        fonts::install(&ctx);
        let mut plain_width = 0.0;
        let mut spaced_width = 0.0;

        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let font = fonts::semibold(12.0);
                plain_width = ui
                    .painter()
                    .layout_no_wrap("LIBRARY".to_string(), font.clone(), egui::Color32::WHITE)
                    .size()
                    .x;
                spaced_width = spaced_galley(ui.painter(), "LIBRARY", font, NAV_LETTER_SPACING)
                    .size()
                    .x;
            });
        });

        let glyph_count = "LIBRARY".chars().count() as f32;
        let minimum_extra = (glyph_count - 1.0) * NAV_LETTER_SPACING - 1.1;
        let maximum_extra = glyph_count * NAV_LETTER_SPACING + 1.0;
        let actual_extra = spaced_width - plain_width;
        assert!(spaced_width > plain_width);
        assert!(
            (minimum_extra..=maximum_extra).contains(&actual_extra),
            "letter spacing added {actual_extra}, expected {minimum_extra}..={maximum_extra}"
        );
    }
}
