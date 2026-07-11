use eframe::egui;
use std::collections::BTreeMap;

pub struct SteamUi;

impl SteamUi {
    pub const BG: egui::Color32 = egui::Color32::from_rgb(22, 29, 38);
    pub const PANEL: egui::Color32 = egui::Color32::from_rgb(31, 39, 51);
    pub const PANEL_ALT: egui::Color32 = egui::Color32::from_rgb(37, 47, 61);
    pub const HOVER: egui::Color32 = egui::Color32::from_rgb(46, 59, 76);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(64, 80, 99);
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(102, 192, 244);
    pub const ACCENT_DARK: egui::Color32 = egui::Color32::from_rgb(27, 116, 178);
    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(232, 239, 247);
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(158, 174, 194);
    pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(112, 130, 151);
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(94, 206, 118);
    pub const WARNING: egui::Color32 = egui::Color32::from_rgb(222, 181, 86);
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(224, 92, 92);

    pub const FOOTER_HEIGHT: f32 = 58.0;
    pub const COLUMN_GAP: f32 = 18.0;
    pub const SECTION_GAP: f32 = 18.0;

    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16);
        style.spacing.menu_margin = egui::Margin::same(8);
        style.spacing.indent = 18.0;

        let mut text_styles = BTreeMap::new();
        text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(18.0, egui::FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(12.0, egui::FontFamily::Proportional),
        );
        text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(13.0, egui::FontFamily::Monospace),
        );
        style.text_styles = text_styles;

        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.window_fill = Self::BG;
        visuals.panel_fill = Self::BG;
        visuals.extreme_bg_color = egui::Color32::from_rgb(14, 20, 28);
        visuals.faint_bg_color = Self::PANEL;
        visuals.hyperlink_color = Self::ACCENT;
        visuals.override_text_color = Some(Self::TEXT);
        visuals.selection.bg_fill = Self::ACCENT_DARK;
        visuals.selection.stroke = egui::Stroke::new(1.0_f32, Self::ACCENT);
        visuals.window_corner_radius = egui::CornerRadius::same(8);
        visuals.menu_corner_radius = egui::CornerRadius::same(6);
        visuals.window_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);

        visuals.widgets.noninteractive.bg_fill = Self::PANEL;
        visuals.widgets.noninteractive.weak_bg_fill = Self::PANEL;
        visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT_SECONDARY);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.inactive.bg_fill = Self::PANEL_ALT;
        visuals.widgets.inactive.weak_bg_fill = Self::PANEL;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, Self::BORDER);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.hovered.bg_fill = Self::HOVER;
        visuals.widgets.hovered.weak_bg_fill = Self::HOVER;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, Self::ACCENT);
        visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.active.bg_fill = Self::ACCENT_DARK;
        visuals.widgets.active.weak_bg_fill = Self::ACCENT_DARK;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, Self::ACCENT);
        visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::WHITE);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.open.bg_fill = Self::PANEL_ALT;
        visuals.widgets.open.weak_bg_fill = Self::PANEL_ALT;
        visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, Self::ACCENT);
        visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, Self::TEXT);
        visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

        ctx.set_style(style);
    }

    pub fn window_frame() -> egui::Frame {
        egui::Frame::window(&egui::Style::default())
            .fill(Self::BG)
            .stroke(egui::Stroke::new(1.0_f32, Self::BORDER))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(18, 16))
    }

    pub fn panel<R>(
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        egui::Frame::new()
            .fill(Self::PANEL)
            .stroke(egui::Stroke::new(1.0_f32, Self::BORDER))
            .corner_radius(egui::CornerRadius::same(7))
            .inner_margin(egui::Margin::symmetric(16, 14))
            .show(ui, content)
    }

    pub fn inset_panel<R>(
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        egui::Frame::new()
            .fill(Self::PANEL_ALT)
            .stroke(egui::Stroke::new(1.0_f32, Self::BORDER))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, content)
    }

    /// Fixed-width sidebar column with full body height.
    pub fn sidebar_column<R>(
        ui: &mut egui::Ui,
        width: f32,
        height: f32,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| Self::panel(ui, content).inner,
        )
        .inner
    }

    /// Remaining-width content column with full body height.
    pub fn content_column<R>(
        ui: &mut egui::Ui,
        height: f32,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let width = ui.available_width().max(320.0);
        ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::LEFT),
            content,
        )
        .inner
    }

    pub fn page_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.label(Self::title(title));
        ui.add_space(4.0);
        ui.label(Self::subtitle(subtitle));
        ui.add_space(16.0);
    }

    pub fn scroll_content<R>(
        ui: &mut egui::Ui,
        height: f32,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        egui::ScrollArea::vertical()
            .id_salt(ui.id().with("steam_scroll"))
            .auto_shrink([false, false])
            .max_height(height.max(120.0))
            .show(ui, content)
            .inner
    }

    pub fn title(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .size(18.0)
            .strong()
            .color(Self::TEXT)
    }

    pub fn subtitle(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .size(13.0)
            .color(Self::TEXT_SECONDARY)
    }

    pub fn section_title(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .size(14.5)
            .strong()
            .color(Self::ACCENT)
    }

    pub fn muted(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .size(13.0)
            .color(Self::TEXT_MUTED)
    }

    pub fn command(text: impl Into<String>) -> egui::RichText {
        egui::RichText::new(text.into())
            .monospace()
            .size(13.0)
            .color(Self::SUCCESS)
    }

    pub fn sidebar_button(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
        let width = ui.available_width().max(120.0);
        let fill = if selected {
            Self::ACCENT_DARK
        } else {
            egui::Color32::TRANSPARENT
        };
        let text_color = if selected {
            egui::Color32::WHITE
        } else {
            Self::TEXT_SECONDARY
        };
        ui.add_sized(
            [width, 36.0],
            egui::Button::new(
                egui::RichText::new(label)
                    .size(13.5)
                    .strong()
                    .color(text_color),
            )
            .fill(fill)
            .stroke(if selected {
                egui::Stroke::new(1.0_f32, Self::ACCENT)
            } else {
                egui::Stroke::NONE
            }),
        )
    }
}
