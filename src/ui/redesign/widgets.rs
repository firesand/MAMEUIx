//! Shared widgets for the redesign shell.

use super::{fonts, tokens::RedesignTokens};
use eframe::egui;

pub fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.label(
        egui::RichText::new(label)
            .font(fonts::bold(11.0))
            .color(RedesignTokens::TEXT_FAINT)
            .extra_letter_spacing(0.8),
    );
}

pub fn card_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(RedesignTokens::BG_PANEL)
        .stroke(egui::Stroke::new(1.0_f32, RedesignTokens::BORDER))
        .corner_radius(egui::CornerRadius::same(RedesignTokens::RADIUS_LG))
        .inner_margin(egui::Margin::symmetric(16, 14))
}

pub fn secondary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(label)
                .font(fonts::semibold(13.0))
                .color(RedesignTokens::TEXT_PRIMARY),
        )
        .corner_radius(egui::CornerRadius::same(RedesignTokens::RADIUS_MD)),
    )
}

pub fn accent_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.scope(|ui| {
        let widgets = &mut ui.style_mut().visuals.widgets;
        widgets.inactive.weak_bg_fill = RedesignTokens::ACCENT;
        widgets.inactive.bg_stroke = egui::Stroke::NONE;
        widgets.hovered.weak_bg_fill = RedesignTokens::ACCENT_HOVER;
        widgets.hovered.bg_stroke = egui::Stroke::NONE;
        widgets.active.weak_bg_fill = RedesignTokens::ACCENT_HOVER;
        widgets.active.bg_stroke = egui::Stroke::NONE;

        ui.add(
            egui::Button::new(
                egui::RichText::new(label)
                    .font(fonts::bold(13.0))
                    .color(RedesignTokens::ACCENT_TEXT)
                    .extra_letter_spacing(0.8),
            )
            .corner_radius(egui::CornerRadius::same(RedesignTokens::RADIUS_MD)),
        )
    })
    .inner
}

/// A compact text-only action with explicit idle and hover colors.
///
/// `egui::Link` always paints with `visuals.hyperlink_color`, ignoring a
/// `RichText` color. The redesign uses neutral text actions that brighten on
/// hover, so allocate the interaction first and paint the galley afterwards.
pub fn text_link(
    ui: &mut egui::Ui,
    label: &str,
    font: egui::FontId,
    idle_color: egui::Color32,
) -> egui::Response {
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font, egui::Color32::PLACEHOLDER);
    let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::click());
    let color = if response.hovered() {
        RedesignTokens::TEXT_BRIGHT
    } else {
        idle_color
    };
    ui.painter().galley(rect.min, galley, color);
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

pub fn collection_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
    let r = egui::Rect::from_center_size(rect.center(), egui::vec2(7.0, 7.0));
    ui.painter()
        .rect_filled(r, egui::CornerRadius::same(2), color);
}

fn checkbox_row_fill(checked: bool, hovered: bool) -> egui::Color32 {
    if checked {
        RedesignTokens::BG_RAISED
    } else if hovered {
        RedesignTokens::BG_HOVER
    } else {
        egui::Color32::TRANSPARENT
    }
}

fn collapsible_header_color(hovered: bool) -> egui::Color32 {
    if hovered {
        RedesignTokens::STATUS_NEUTRAL
    } else {
        RedesignTokens::TEXT_FAINT
    }
}

pub fn checkbox_row(
    ui: &mut egui::Ui,
    checked: &mut bool,
    label: &str,
    count: Option<usize>,
) -> egui::Response {
    let (row_rect, row_response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());
    if row_response.clicked() {
        *checked = !*checked;
    }

    let fill = checkbox_row_fill(*checked, row_response.hovered());
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            row_rect,
            egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
            fill,
        );
    }

    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("checkbox_row_content")
            .max_rect(row_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    content_ui.shrink_clip_rect(row_rect);
    content_ui.spacing_mut().item_spacing.x = 8.0;
    let size = egui::vec2(14.0, 14.0);
    let (rect, _) = content_ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = content_ui.painter();
    painter.rect(
        rect,
        egui::CornerRadius::same(RedesignTokens::RADIUS_SM),
        if *checked {
            RedesignTokens::ACCENT
        } else {
            egui::Color32::TRANSPARENT
        },
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(58, 63, 73)),
        egui::StrokeKind::Inside,
    );
    if *checked {
        let stroke = egui::Stroke::new(1.8_f32, RedesignTokens::ACCENT_TEXT);
        painter.line_segment(
            [
                egui::pos2(rect.left() + 3.0, rect.center().y),
                egui::pos2(rect.left() + 6.0, rect.bottom() - 3.5),
            ],
            stroke,
        );
        painter.line_segment(
            [
                egui::pos2(rect.left() + 6.0, rect.bottom() - 3.5),
                egui::pos2(rect.right() - 2.5, rect.top() + 3.0),
            ],
            stroke,
        );
    }
    content_ui.label(
        egui::RichText::new(label)
            .font(fonts::medium(12.5))
            .color(if *checked {
                RedesignTokens::TEXT_BRIGHT
            } else {
                RedesignTokens::TEXT_SECONDARY
            }),
    );
    if let Some(n) = count {
        content_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{n}"))
                    .size(11.0)
                    .color(RedesignTokens::TEXT_MUTED),
            );
        });
    }
    row_response
}

pub fn sidebar_row(
    ui: &mut egui::Ui,
    selected: bool,
    content: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let width = ui.available_width();
    let background = ui.painter().add(egui::Shape::Noop);
    let frame = egui::Frame::new()
        .fill(egui::Color32::TRANSPARENT)
        .corner_radius(egui::CornerRadius::same(RedesignTokens::RADIUS_MD))
        .inner_margin(egui::Margin::symmetric(10, 7));
    let response = frame
        .show(ui, |ui| {
            // Keep the row inside the sidebar: the frame contributes 10 px on
            // both sides, so the child width must exclude the full 20 px.
            ui.set_width((width - 20.0).max(0.0));
            content(ui);
        })
        .response
        .interact(egui::Sense::click());
    let fill = if selected {
        RedesignTokens::BG_RAISED
    } else if response.hovered() {
        RedesignTokens::BG_HOVER
    } else {
        egui::Color32::TRANSPARENT
    };
    if fill != egui::Color32::TRANSPARENT {
        ui.painter().set(
            background,
            egui::Shape::rect_filled(
                response.rect,
                egui::CornerRadius::same(RedesignTokens::RADIUS_MD),
                fill,
            ),
        );
    }
    response
}

pub fn collapsible_header(
    ui: &mut egui::Ui,
    label: &str,
    open: &mut bool,
    active_hint: Option<&str>,
) -> egui::Response {
    let chevron = if *open { "▾" } else { "▸" };
    let (row_rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), egui::Sense::click());
    let header_color = collapsible_header_color(response.hovered());

    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .id_salt("collapsible_header_content")
            .max_rect(row_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    content_ui.shrink_clip_rect(row_rect);
    content_ui.spacing_mut().item_spacing.x = 6.0;
    content_ui.label(
        egui::RichText::new(label)
            .font(fonts::bold(10.0))
            .color(header_color)
            .extra_letter_spacing(0.8),
    );
    content_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(
            egui::RichText::new(chevron)
                .font(fonts::regular(10.0))
                .color(header_color),
        );
        if let Some(hint) = active_hint.filter(|_| !*open) {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(hint)
                            .font(fonts::semibold(11.0))
                            .color(RedesignTokens::ACCENT),
                    )
                    .truncate()
                    .selectable(false),
                );
            });
        }
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_checkbox_keeps_selected_fill_while_hovered() {
        assert_eq!(checkbox_row_fill(true, false), RedesignTokens::BG_RAISED);
        assert_eq!(checkbox_row_fill(true, true), RedesignTokens::BG_RAISED);
        assert_eq!(checkbox_row_fill(false, true), RedesignTokens::BG_HOVER);
        assert_eq!(checkbox_row_fill(false, false), egui::Color32::TRANSPARENT);
    }

    #[test]
    fn collapsible_header_brightens_foreground_only_on_hover() {
        assert_eq!(collapsible_header_color(false), RedesignTokens::TEXT_FAINT);
        assert_eq!(
            collapsible_header_color(true),
            RedesignTokens::STATUS_NEUTRAL
        );
    }
}
