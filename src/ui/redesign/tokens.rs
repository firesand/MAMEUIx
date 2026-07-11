//! Design tokens for the MAMEUIx redesign (Steam-inspired shell).
//! Isolated from legacy themes — changing these does not affect the classic UI.

use super::fonts;
use eframe::egui;
use std::collections::BTreeMap;

pub struct RedesignTokens;

impl RedesignTokens {
    pub const BG_ROOT: egui::Color32 = egui::Color32::from_rgb(19, 20, 23);
    pub const BG_PANEL: egui::Color32 = egui::Color32::from_rgb(23, 24, 28);
    pub const BG_SURFACE: egui::Color32 = egui::Color32::from_rgb(28, 30, 35);
    pub const BG_RAISED: egui::Color32 = egui::Color32::from_rgb(35, 38, 45);
    pub const BG_HOVER: egui::Color32 = egui::Color32::from_rgb(32, 35, 42);
    pub const BG_ROW_HOVER: egui::Color32 = egui::Color32::from_rgb(28, 31, 37);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(36, 38, 44);
    pub const BORDER_STRONG: egui::Color32 = egui::Color32::from_rgb(46, 50, 58);
    pub const ROW_DIVIDER: egui::Color32 = egui::Color32::from_rgb(27, 29, 34);
    pub const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(232, 234, 237);
    pub const TEXT_BRIGHT: egui::Color32 = egui::Color32::from_rgb(240, 242, 245);
    pub const TEXT_SECONDARY: egui::Color32 = egui::Color32::from_rgb(154, 160, 171);
    pub const TEXT_MUTED: egui::Color32 = egui::Color32::from_rgb(107, 114, 128);
    pub const TEXT_FAINT: egui::Color32 = egui::Color32::from_rgb(95, 102, 114);
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(122, 165, 216);
    pub const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(139, 181, 230);
    pub const ACCENT_TEXT: egui::Color32 = egui::Color32::from_rgb(21, 23, 27);
    pub const STATUS_OK: egui::Color32 = egui::Color32::from_rgb(111, 191, 138);
    pub const STATUS_WARN: egui::Color32 = egui::Color32::from_rgb(217, 169, 74);
    pub const STATUS_MISSING: egui::Color32 = egui::Color32::from_rgb(222, 110, 99);
    pub const STATUS_NEUTRAL: egui::Color32 = egui::Color32::from_rgb(138, 147, 165);
    pub const STAR_INACTIVE: egui::Color32 = egui::Color32::from_rgb(69, 74, 84);

    pub const TOP_BAR_HEIGHT: f32 = 52.0;
    pub const SIDEBAR_WIDTH: f32 = 236.0;
    pub const ROW_HEIGHT: f32 = 44.0;
    pub const HEADER_ROW_HEIGHT: f32 = 34.0;
    pub const PAGE_PADDING: f32 = 24.0;
    pub const RADIUS_SM: u8 = 4;
    pub const RADIUS_MD: u8 = 6;
    pub const RADIUS_LG: u8 = 8;

    pub fn install_fonts(ctx: &egui::Context) {
        fonts::install(ctx);
    }

    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(0);
        style.spacing.menu_margin = egui::Margin::same(0);

        let mut text_styles = BTreeMap::new();
        text_styles.insert(egui::TextStyle::Heading, fonts::bold(18.0));
        text_styles.insert(egui::TextStyle::Body, fonts::regular(13.0));
        text_styles.insert(egui::TextStyle::Button, fonts::semibold(13.0));
        text_styles.insert(egui::TextStyle::Small, fonts::regular(11.0));
        text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(12.0, egui::FontFamily::Monospace),
        );
        style.text_styles = text_styles;

        let visuals = &mut style.visuals;
        *visuals = egui::Visuals::dark();
        visuals.window_fill = Self::BG_ROOT;
        visuals.panel_fill = Self::BG_ROOT;
        visuals.extreme_bg_color = Self::BG_ROOT;
        visuals.faint_bg_color = Self::BG_PANEL;
        visuals.hyperlink_color = Self::ACCENT;
        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.selection.bg_fill = Self::BG_RAISED;
        visuals.selection.stroke = egui::Stroke::new(1.0, Self::ACCENT);
        visuals.window_corner_radius = egui::CornerRadius::same(Self::RADIUS_LG);
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(Self::RADIUS_MD);
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(Self::RADIUS_MD);
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(Self::RADIUS_MD);
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(Self::RADIUS_MD);

        visuals.widgets.noninteractive.bg_fill = Self::BG_SURFACE;
        visuals.widgets.noninteractive.weak_bg_fill = Self::BG_SURFACE;
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_SECONDARY);
        visuals.widgets.inactive.bg_fill = Self::BG_SURFACE;
        visuals.widgets.inactive.weak_bg_fill = Self::BG_SURFACE;
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Self::BORDER_STRONG);
        visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.weak_bg_fill = Self::BG_HOVER;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, Self::BORDER_STRONG);
        visuals.widgets.active.bg_fill = Self::BG_RAISED;
        visuals.widgets.active.weak_bg_fill = Self::BG_RAISED;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, Self::ACCENT);

        ctx.set_style(style);
    }

    pub fn status_color(status: crate::models::RomStatus) -> egui::Color32 {
        use crate::models::RomStatus;
        match status {
            RomStatus::Available => Self::STATUS_OK,
            RomStatus::ChdMissing
            | RomStatus::ChdRequired
            | RomStatus::Incorrect
            | RomStatus::NotWorking
            | RomStatus::Preliminary => Self::STATUS_WARN,
            RomStatus::Missing => Self::STATUS_MISSING,
            RomStatus::Unknown => Self::STATUS_NEUTRAL,
        }
    }

    pub fn status_label(status: crate::models::RomStatus) -> &'static str {
        use crate::models::RomStatus;
        match status {
            RomStatus::Available => "ROM OK",
            RomStatus::ChdMissing => "CHD missing",
            RomStatus::ChdRequired => "CHD required",
            RomStatus::Missing => "Missing",
            RomStatus::Incorrect => "Incorrect",
            RomStatus::NotWorking => "Not working",
            RomStatus::Preliminary => "Preliminary",
            RomStatus::Unknown => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redesign_style_uses_weighted_fonts_and_hover_fills() {
        let ctx = egui::Context::default();
        RedesignTokens::apply(&ctx);
        let style = ctx.style();

        assert_eq!(
            style.text_styles[&egui::TextStyle::Heading],
            fonts::bold(18.0)
        );
        assert_eq!(
            style.text_styles[&egui::TextStyle::Body],
            fonts::regular(13.0)
        );
        assert_eq!(
            style.text_styles[&egui::TextStyle::Button],
            fonts::semibold(13.0)
        );
        assert_eq!(
            style.visuals.widgets.inactive.weak_bg_fill,
            RedesignTokens::BG_SURFACE
        );
        assert_eq!(
            style.visuals.widgets.hovered.weak_bg_fill,
            RedesignTokens::BG_HOVER
        );
        assert_eq!(
            style.visuals.widgets.active.weak_bg_fill,
            RedesignTokens::BG_RAISED
        );
    }
}
