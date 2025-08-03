// src/models/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
// GraphicsConfig is used in the struct definition below
use crate::utils::graphics::GraphicsConfig;
use super::{GameStats, Preferences, FilterSettings, SortColumn, SortDirection};

// View mode for game list display
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Table,  // Traditional table view
    List,   // Modern list view with cards
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Table
    }
}

// Window size and position settings for dialogs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSettings {
    pub width: f32,
    pub height: f32,
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            width: 600.0,
            height: 550.0,
            pos_x: None,
            pos_y: None,
        }
    }
}

// MameExecutable represents a MAME emulator executable
// Unlike simple paths, this contains metadata about each MAME version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MameExecutable {
    pub name: String,        // User-friendly name for this MAME version
    pub path: String,        // Full path to the executable
    pub version: String,     // MAME version number
    pub total_games: usize,  // Total number of games this version supports
    pub working_games: usize, // Number of games that work properly
}

// VideoSettings controls how MAME displays games

// Theme controls the visual appearance of the frontend
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    DarkBlue,        // Professional dark theme with blue accents
    DarkGrey,        // Neutral dark theme
    ArcadePurple,    // Retro arcade-inspired theme
    LightClassic,    // Classic light theme
    NeonGreen,       // Cyberpunk green theme
    SunsetOrange,    // Warm orange theme
    OceanBlue,       // Deep ocean blue theme
    MidnightBlack,   // Pure black theme
    ForestGreen,     // Nature-inspired green theme
    RetroAmber,      // Vintage amber theme
    ModernSpacious,  // Modern spacious theme with better spacing
}



impl Theme {
    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::DarkBlue => "Dark Blue",
            Theme::DarkGrey => "Dark Grey",
            Theme::ArcadePurple => "Arcade Purple",
            Theme::LightClassic => "Light Classic",
            Theme::NeonGreen => "Neon Green",
            Theme::SunsetOrange => "Sunset Orange",
            Theme::OceanBlue => "Ocean Blue",
            Theme::MidnightBlack => "Midnight Black",
            Theme::ForestGreen => "Forest Green",
            Theme::RetroAmber => "Retro Amber",
            Theme::ModernSpacious => "Modern Spacious",
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = match self {
            Theme::DarkBlue => {
                let mut v = egui::Visuals::dark();
                // Blue accent colors
                v.hyperlink_color = egui::Color32::from_rgb(0, 123, 255);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0, 123, 255);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 100, 200);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 30, 50);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 60);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(0, 123, 255);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(40, 60, 100);
                v.selection.stroke.color = egui::Color32::from_rgb(0, 123, 255);
                v.selection.stroke.width = 2.0;
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(25, 25, 35);
                v.window_fill = egui::Color32::from_rgb(20, 20, 30);
                v.faint_bg_color = egui::Color32::from_rgb(35, 35, 45);
                v
            }
            Theme::DarkGrey => {
                let mut v = egui::Visuals::dark();
                // Grey accent colors
                v.hyperlink_color = egui::Color32::from_rgb(108, 117, 125);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(108, 117, 125);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 100, 110);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 40);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 50, 50);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(108, 117, 125);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(70, 80, 90);
                v.selection.stroke.color = egui::Color32::from_rgb(108, 117, 125);
                v.selection.stroke.width = 2.0;
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(35, 35, 35);
                v.window_fill = egui::Color32::from_rgb(30, 30, 30);
                v.faint_bg_color = egui::Color32::from_rgb(45, 45, 45);
                v
            }
            Theme::ArcadePurple => {
                let mut v = egui::Visuals::dark();
                // Purple accent colors
                v.hyperlink_color = egui::Color32::from_rgb(111, 66, 193);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(111, 66, 193);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(90, 50, 160);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 30, 60);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(50, 35, 70);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(111, 66, 193);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(70, 50, 100);
                v.selection.stroke.color = egui::Color32::from_rgb(111, 66, 193);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(35, 25, 55);
                v.window_fill = egui::Color32::from_rgb(30, 20, 45);
                v.faint_bg_color = egui::Color32::from_rgb(45, 30, 65);
                v
            }
            Theme::LightClassic => {
                let mut v = egui::Visuals::light();
                // Blue accent colors for light theme
                v.hyperlink_color = egui::Color32::from_rgb(0, 123, 255);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0, 123, 255);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 100, 200);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 240, 250);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(230, 230, 240);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(0, 123, 255);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(200, 220, 255);
                v.selection.stroke.color = egui::Color32::from_rgb(0, 123, 255);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(50, 50, 50));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(245, 245, 255);
                v.window_fill = egui::Color32::from_rgb(250, 250, 255);
                v.faint_bg_color = egui::Color32::from_rgb(235, 235, 245);
                v
            }
            Theme::NeonGreen => {
                let mut v = egui::Visuals::dark();
                // Neon green accent colors
                v.hyperlink_color = egui::Color32::from_rgb(0, 255, 127);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0, 255, 127);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 200, 100);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 40, 20);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 50, 30);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(0, 255, 127);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(40, 80, 40);
                v.selection.stroke.color = egui::Color32::from_rgb(0, 255, 127);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(25, 45, 25);
                v.window_fill = egui::Color32::from_rgb(20, 35, 20);
                v.faint_bg_color = egui::Color32::from_rgb(35, 55, 35);
                v
            }
            Theme::SunsetOrange => {
                let mut v = egui::Visuals::dark();
                // Orange accent colors
                v.hyperlink_color = egui::Color32::from_rgb(255, 69, 0);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(255, 69, 0);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 60, 0);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 30, 20);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 35, 25);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(255, 69, 0);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(80, 50, 30);
                v.selection.stroke.color = egui::Color32::from_rgb(255, 69, 0);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(45, 25, 15);
                v.window_fill = egui::Color32::from_rgb(40, 20, 10);
                v.faint_bg_color = egui::Color32::from_rgb(55, 30, 20);
                v
            }
            Theme::OceanBlue => {
                let mut v = egui::Visuals::dark();
                // Ocean blue accent colors
                v.hyperlink_color = egui::Color32::from_rgb(0, 191, 255);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(0, 191, 255);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(0, 160, 220);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 30, 50);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 35, 55);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(0, 191, 255);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(40, 60, 80);
                v.selection.stroke.color = egui::Color32::from_rgb(0, 191, 255);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(25, 35, 55);
                v.window_fill = egui::Color32::from_rgb(20, 30, 45);
                v.faint_bg_color = egui::Color32::from_rgb(35, 45, 65);
                v
            }
            Theme::MidnightBlack => {
                let mut v = egui::Visuals::dark();
                // Pure black and white
                v.hyperlink_color = egui::Color32::from_rgb(255, 255, 255);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(255, 255, 255);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(200, 200, 200);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(10, 10, 10);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(15, 15, 15);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(255, 255, 255);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(50, 50, 50);
                v.selection.stroke.color = egui::Color32::from_rgb(255, 255, 255);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(5, 5, 5);
                v.window_fill = egui::Color32::from_rgb(0, 0, 0);
                v.faint_bg_color = egui::Color32::from_rgb(15, 15, 15);
                v
            }
            Theme::ForestGreen => {
                let mut v = egui::Visuals::dark();
                // Forest green accent colors
                v.hyperlink_color = egui::Color32::from_rgb(34, 139, 34);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(34, 139, 34);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(28, 120, 28);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 40, 20);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 45, 25);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(34, 139, 34);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(40, 70, 40);
                v.selection.stroke.color = egui::Color32::from_rgb(34, 139, 34);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(25, 45, 25);
                v.window_fill = egui::Color32::from_rgb(20, 35, 20);
                v.faint_bg_color = egui::Color32::from_rgb(35, 55, 35);
                v
            }
            Theme::RetroAmber => {
                let mut v = egui::Visuals::dark();
                // Retro amber accent colors
                v.hyperlink_color = egui::Color32::from_rgb(255, 191, 0);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(255, 191, 0);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 165, 0);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(50, 40, 20);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 45, 25);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(255, 191, 0);
                // Selection colors - more subtle for better text visibility
                v.selection.bg_fill = egui::Color32::from_rgb(80, 60, 30);
                v.selection.stroke.color = egui::Color32::from_rgb(255, 191, 0);
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(255, 255, 255));
                // Window and panel backgrounds
                v.panel_fill = egui::Color32::from_rgb(45, 35, 15);
                v.window_fill = egui::Color32::from_rgb(40, 30, 10);
                v.faint_bg_color = egui::Color32::from_rgb(55, 45, 25);
                v
            }
            Theme::ModernSpacious => {
                let mut v = egui::Visuals::dark();
                // Modern blue accent colors with better contrast
                v.hyperlink_color = egui::Color32::from_rgb(64, 156, 255);
                v.widgets.active.bg_fill = egui::Color32::from_rgb(64, 156, 255);
                v.widgets.hovered.bg_fill = egui::Color32::from_rgb(52, 126, 205);
                v.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(45, 45, 55);
                v.widgets.inactive.bg_fill = egui::Color32::from_rgb(55, 55, 65);
                v.widgets.open.bg_fill = egui::Color32::from_rgb(64, 156, 255);
                // Selection colors - more visible and modern
                v.selection.bg_fill = egui::Color32::from_rgb(70, 90, 120);
                v.selection.stroke.color = egui::Color32::from_rgb(64, 156, 255);
                v.selection.stroke.width = 2.0;
                // Text colors for better visibility
                v.override_text_color = Some(egui::Color32::from_rgb(240, 240, 250));
                // Window and panel backgrounds - more spacious feeling
                v.panel_fill = egui::Color32::from_rgb(40, 40, 50);
                v.window_fill = egui::Color32::from_rgb(35, 35, 45);
                v.faint_bg_color = egui::Color32::from_rgb(50, 50, 60);
                // Improved spacing and borders - using available fields
                v.window_shadow.blur = 8;
                v.window_shadow.offset = [2, 2];
                v
            }
        };
        
        // Apply the visuals
        ctx.set_visuals(visuals);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::ModernSpacious
    }
}

/// Color scheme for the game list with enhanced visual design
#[derive(Debug, Clone)]
pub struct GameListColors {
    pub row_bg_even: egui::Color32,
    pub row_bg_odd: egui::Color32,
    pub row_bg_selected: egui::Color32,
    pub row_bg_hover: egui::Color32,
    pub row_separator: egui::Color32,
    pub favorite_active: egui::Color32,
    pub favorite_inactive: egui::Color32,
    pub status_available: egui::Color32,
    pub status_missing: egui::Color32,
    pub status_unknown: egui::Color32,
    pub clone_text: egui::Color32,
    pub header_bg: egui::Color32,
    pub header_text: egui::Color32,
}

impl Default for GameListColors {
    fn default() -> Self {
        Self {
            row_bg_even: egui::Color32::from_rgb(28, 28, 32),
            row_bg_odd: egui::Color32::from_rgb(32, 32, 36),
            row_bg_selected: egui::Color32::from_rgb(45, 65, 95),
            row_bg_hover: egui::Color32::from_rgba_premultiplied(255, 255, 255, 8),
            row_separator: egui::Color32::from_rgba_premultiplied(255, 255, 255, 15),
            favorite_active: egui::Color32::from_rgb(255, 200, 50),
            favorite_inactive: egui::Color32::from_rgb(100, 100, 110),
            status_available: egui::Color32::from_rgb(50, 200, 100),
            status_missing: egui::Color32::from_rgb(200, 50, 50),
            status_unknown: egui::Color32::from_rgb(150, 150, 150),
            clone_text: egui::Color32::from_rgb(180, 180, 200),
            header_bg: egui::Color32::from_rgb(42, 42, 48),
            header_text: egui::Color32::from_rgb(180, 180, 200),
        }
    }
}

impl GameListColors {
    /// Get colors for a specific theme
    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::DarkBlue => Self {
                row_bg_even: egui::Color32::from_rgb(25, 25, 35),
                row_bg_odd: egui::Color32::from_rgb(30, 30, 40),
                row_bg_selected: egui::Color32::from_rgb(40, 60, 100),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(0, 123, 255, 15),
                row_separator: egui::Color32::from_rgba_premultiplied(0, 123, 255, 20),
                favorite_active: egui::Color32::from_rgb(255, 200, 50),
                favorite_inactive: egui::Color32::from_rgb(80, 80, 100),
                status_available: egui::Color32::from_rgb(50, 200, 100),
                status_missing: egui::Color32::from_rgb(200, 50, 50),
                status_unknown: egui::Color32::from_rgb(150, 150, 150),
                clone_text: egui::Color32::from_rgb(180, 180, 220),
                header_bg: egui::Color32::from_rgb(35, 35, 45),
                header_text: egui::Color32::from_rgb(200, 200, 255),
            },
            Theme::DarkGrey => Self {
                row_bg_even: egui::Color32::from_rgb(35, 35, 35),
                row_bg_odd: egui::Color32::from_rgb(40, 40, 40),
                row_bg_selected: egui::Color32::from_rgb(70, 80, 90),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(255, 255, 255, 10),
                row_separator: egui::Color32::from_rgba_premultiplied(255, 255, 255, 15),
                favorite_active: egui::Color32::from_rgb(255, 200, 50),
                favorite_inactive: egui::Color32::from_rgb(100, 100, 100),
                status_available: egui::Color32::from_rgb(50, 200, 100),
                status_missing: egui::Color32::from_rgb(200, 50, 50),
                status_unknown: egui::Color32::from_rgb(150, 150, 150),
                clone_text: egui::Color32::from_rgb(180, 180, 180),
                header_bg: egui::Color32::from_rgb(45, 45, 45),
                header_text: egui::Color32::from_rgb(220, 220, 220),
            },
            Theme::ArcadePurple => Self {
                row_bg_even: egui::Color32::from_rgb(35, 25, 55),
                row_bg_odd: egui::Color32::from_rgb(40, 30, 60),
                row_bg_selected: egui::Color32::from_rgb(70, 50, 100),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(111, 66, 193, 20),
                row_separator: egui::Color32::from_rgba_premultiplied(111, 66, 193, 25),
                favorite_active: egui::Color32::from_rgb(255, 200, 80),
                favorite_inactive: egui::Color32::from_rgb(120, 100, 140),
                status_available: egui::Color32::from_rgb(80, 220, 120),
                status_missing: egui::Color32::from_rgb(220, 80, 80),
                status_unknown: egui::Color32::from_rgb(160, 140, 180),
                clone_text: egui::Color32::from_rgb(200, 180, 220),
                header_bg: egui::Color32::from_rgb(45, 30, 65),
                header_text: egui::Color32::from_rgb(220, 200, 255),
            },
            Theme::LightClassic => Self {
                row_bg_even: egui::Color32::from_rgb(245, 245, 255),
                row_bg_odd: egui::Color32::from_rgb(250, 250, 255),
                row_bg_selected: egui::Color32::from_rgb(200, 220, 255),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(0, 123, 255, 15),
                row_separator: egui::Color32::from_rgba_premultiplied(0, 0, 0, 20),
                favorite_active: egui::Color32::from_rgb(255, 180, 0),
                favorite_inactive: egui::Color32::from_rgb(180, 180, 180),
                status_available: egui::Color32::from_rgb(0, 150, 50),
                status_missing: egui::Color32::from_rgb(200, 50, 50),
                status_unknown: egui::Color32::from_rgb(120, 120, 120),
                clone_text: egui::Color32::from_rgb(80, 80, 100),
                header_bg: egui::Color32::from_rgb(235, 235, 245),
                header_text: egui::Color32::from_rgb(50, 50, 50),
            },
            Theme::NeonGreen => Self {
                row_bg_even: egui::Color32::from_rgb(25, 45, 25),
                row_bg_odd: egui::Color32::from_rgb(30, 50, 30),
                row_bg_selected: egui::Color32::from_rgb(40, 80, 40),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(0, 255, 127, 15),
                row_separator: egui::Color32::from_rgba_premultiplied(0, 255, 127, 20),
                favorite_active: egui::Color32::from_rgb(255, 255, 0),
                favorite_inactive: egui::Color32::from_rgb(80, 120, 80),
                status_available: egui::Color32::from_rgb(0, 255, 127),
                status_missing: egui::Color32::from_rgb(255, 80, 80),
                status_unknown: egui::Color32::from_rgb(150, 200, 150),
                clone_text: egui::Color32::from_rgb(180, 255, 180),
                header_bg: egui::Color32::from_rgb(35, 55, 35),
                header_text: egui::Color32::from_rgb(200, 255, 200),
            },
            Theme::SunsetOrange => Self {
                row_bg_even: egui::Color32::from_rgb(45, 25, 15),
                row_bg_odd: egui::Color32::from_rgb(50, 30, 20),
                row_bg_selected: egui::Color32::from_rgb(80, 50, 30),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(255, 69, 0, 20),
                row_separator: egui::Color32::from_rgba_premultiplied(255, 69, 0, 25),
                favorite_active: egui::Color32::from_rgb(255, 200, 0),
                favorite_inactive: egui::Color32::from_rgb(140, 100, 80),
                status_available: egui::Color32::from_rgb(100, 220, 100),
                status_missing: egui::Color32::from_rgb(220, 80, 80),
                status_unknown: egui::Color32::from_rgb(180, 140, 120),
                clone_text: egui::Color32::from_rgb(255, 200, 180),
                header_bg: egui::Color32::from_rgb(55, 30, 20),
                header_text: egui::Color32::from_rgb(255, 220, 200),
            },
            Theme::OceanBlue => Self {
                row_bg_even: egui::Color32::from_rgb(25, 35, 55),
                row_bg_odd: egui::Color32::from_rgb(30, 40, 60),
                row_bg_selected: egui::Color32::from_rgb(40, 60, 80),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(0, 191, 255, 15),
                row_separator: egui::Color32::from_rgba_premultiplied(0, 191, 255, 20),
                favorite_active: egui::Color32::from_rgb(255, 200, 80),
                favorite_inactive: egui::Color32::from_rgb(100, 120, 140),
                status_available: egui::Color32::from_rgb(80, 220, 180),
                status_missing: egui::Color32::from_rgb(220, 80, 80),
                status_unknown: egui::Color32::from_rgb(150, 170, 190),
                clone_text: egui::Color32::from_rgb(180, 220, 255),
                header_bg: egui::Color32::from_rgb(35, 45, 65),
                header_text: egui::Color32::from_rgb(200, 230, 255),
            },
            Theme::MidnightBlack => Self {
                row_bg_even: egui::Color32::from_rgb(5, 5, 5),
                row_bg_odd: egui::Color32::from_rgb(10, 10, 10),
                row_bg_selected: egui::Color32::from_rgb(50, 50, 50),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(255, 255, 255, 20),
                row_separator: egui::Color32::from_rgba_premultiplied(255, 255, 255, 40),
                favorite_active: egui::Color32::from_rgb(255, 255, 0),
                favorite_inactive: egui::Color32::from_rgb(128, 128, 128),
                status_available: egui::Color32::from_rgb(0, 255, 0),
                status_missing: egui::Color32::from_rgb(255, 0, 0),
                status_unknown: egui::Color32::from_rgb(192, 192, 192),
                clone_text: egui::Color32::from_rgb(255, 255, 255),
                header_bg: egui::Color32::from_rgb(15, 15, 15),
                header_text: egui::Color32::from_rgb(255, 255, 255),
            },
            Theme::ForestGreen => Self {
                row_bg_even: egui::Color32::from_rgb(25, 45, 25),
                row_bg_odd: egui::Color32::from_rgb(30, 50, 30),
                row_bg_selected: egui::Color32::from_rgb(40, 70, 40),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(34, 139, 34, 20),
                row_separator: egui::Color32::from_rgba_premultiplied(34, 139, 34, 25),
                favorite_active: egui::Color32::from_rgb(255, 220, 0),
                favorite_inactive: egui::Color32::from_rgb(100, 140, 100),
                status_available: egui::Color32::from_rgb(34, 200, 34),
                status_missing: egui::Color32::from_rgb(200, 80, 80),
                status_unknown: egui::Color32::from_rgb(150, 180, 150),
                clone_text: egui::Color32::from_rgb(180, 220, 180),
                header_bg: egui::Color32::from_rgb(35, 55, 35),
                header_text: egui::Color32::from_rgb(200, 240, 200),
            },
            Theme::RetroAmber => Self {
                row_bg_even: egui::Color32::from_rgb(45, 35, 15),
                row_bg_odd: egui::Color32::from_rgb(50, 40, 20),
                row_bg_selected: egui::Color32::from_rgb(80, 60, 30),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(255, 191, 0, 20),
                row_separator: egui::Color32::from_rgba_premultiplied(255, 191, 0, 25),
                favorite_active: egui::Color32::from_rgb(255, 191, 0),
                favorite_inactive: egui::Color32::from_rgb(140, 120, 80),
                status_available: egui::Color32::from_rgb(100, 220, 100),
                status_missing: egui::Color32::from_rgb(220, 80, 80),
                status_unknown: egui::Color32::from_rgb(180, 160, 120),
                clone_text: egui::Color32::from_rgb(255, 220, 180),
                header_bg: egui::Color32::from_rgb(55, 45, 25),
                header_text: egui::Color32::from_rgb(255, 230, 180),
            },
            Theme::ModernSpacious => Self {
                row_bg_even: egui::Color32::from_rgb(40, 40, 50),
                row_bg_odd: egui::Color32::from_rgb(45, 45, 55),
                row_bg_selected: egui::Color32::from_rgb(70, 90, 120),
                row_bg_hover: egui::Color32::from_rgba_premultiplied(64, 156, 255, 15),
                row_separator: egui::Color32::from_rgba_premultiplied(64, 156, 255, 20),
                favorite_active: egui::Color32::from_rgb(255, 200, 80),
                favorite_inactive: egui::Color32::from_rgb(120, 120, 140),
                status_available: egui::Color32::from_rgb(80, 220, 120),
                status_missing: egui::Color32::from_rgb(220, 80, 80),
                status_unknown: egui::Color32::from_rgb(160, 160, 180),
                clone_text: egui::Color32::from_rgb(200, 200, 220),
                header_bg: egui::Color32::from_rgb(50, 50, 60),
                header_text: egui::Color32::from_rgb(240, 240, 250),
            },
        }
    }
}

// Column width settings for the game list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnWidths {
    pub expand: f32,      // Expand/collapse arrow column
    pub favorite: f32,    // Favorite star column
    pub icon: f32,        // Game icon column
    pub status: f32,      // Status column
    pub game: f32,
    pub play_count: f32,
    pub manufacturer: f32,
    pub year: f32,
    pub driver: f32,
    pub driver_status: f32,
    pub category: f32,
    pub rom: f32,
    pub chd: f32,
}

impl ColumnWidths {
    pub fn reset_to_defaults(&mut self) {
        self.expand = 30.0;
        self.favorite = 35.0;
        self.icon = 50.0;
        self.status = 40.0;
        self.game = 350.0;
        self.play_count = 80.0;
        self.manufacturer = 250.0;
        self.year = 80.0;
        self.driver = 120.0;
        self.driver_status = 150.0;
        self.category = 150.0;
        self.rom = 100.0;
        self.chd = 80.0;
    }
}

impl Default for ColumnWidths {
    fn default() -> Self {
        Self {
            expand: 30.0,       // Increased default width for expand column
            favorite: 35.0,     // Increased default width for favorite column
            icon: 50.0,         // Increased default width for icon column
            status: 40.0,       // Increased default width for status column
            game: 350.0,        // Increased default width for game column
            play_count: 80.0,   // Increased default width for play count column
            manufacturer: 250.0, // Increased default width for manufacturer column
            year: 80.0,         // Increased default width for year column
            driver: 120.0,      // Increased default width for driver column
            driver_status: 150.0, // Increased default width for driver status column
            category: 150.0,    // Increased default width for category column
            rom: 100.0,         // Increased default width for ROM column
            chd: 80.0,          // Increased default width for CHD column
        }
    }
}



// Helper struct for TOML serialization that skips None values
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfigToml {
    // MAME executable management
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub mame_executables: Vec<MameExecutable>,
    pub selected_mame_index: usize,

    // ROM directory configuration
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rom_dirs: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub rom_paths: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sample_paths: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra_rom_dirs: Vec<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra_asset_dirs: Vec<PathBuf>,
    
    // MAME Support Files paths - only serialize if Some
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artwork_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snap_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cabinet_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flyer_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marquee_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cheats_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons_path: Option<PathBuf>,
    
    // Additional MAME Search Paths
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctrlr_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crosshair_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sw_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ini_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_path: Option<PathBuf>,
    
    // History and DAT files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mameinfo_dat_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hiscore_dat_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gameinit_dat_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_dat_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catver_ini_path: Option<PathBuf>,

    // MAME Internal Folders Configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_path: Option<PathBuf>,

    // Filter and display settings
    pub filter_settings: FilterSettings,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,

    // Game-specific settings
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub game_preferred_mame: HashMap<String, usize>,
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub favorite_games: HashSet<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub game_stats: HashMap<String, GameStats>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub game_properties: HashMap<String, super::game_properties::GameProperties>,
    pub default_game_properties: super::game_properties::GameProperties,
    
    // Hidden categories
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub hidden_categories: HashSet<String>,

    // UI preferences
    pub show_filters: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_rom: Option<String>,
    pub theme: Theme,
    pub show_rom_icons: bool,
    pub icon_size: u32,
    pub max_cached_icons: usize,
    pub column_widths: ColumnWidths,
    pub view_mode: ViewMode,

    // MAME audit settings
    pub use_mame_audit: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub mame_audit_times: HashMap<String, String>,
    pub assume_merged_sets: bool,

    // Graphics and video
    pub graphics_config: GraphicsConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bgfx_path: Option<PathBuf>,

    // Smart directory memory - remembers last used directory per category
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub last_directories: HashMap<String, PathBuf>,

    // Window settings for dialogs
    pub game_properties_window: WindowSettings,

    pub preferences: Preferences,
}

// AppConfig is the main configuration struct that stores all settings
#[derive(Clone, Debug)]
pub struct AppConfig {
    // MAME executable management
    pub mame_executables: Vec<MameExecutable>,  // List of available MAME versions
    pub selected_mame_index: usize,              // Currently selected MAME

    // ROM directory configuration
    // Note: We have both rom_dirs and rom_paths for compatibility
    // Eventually these should be consolidated
    pub rom_dirs: Vec<PathBuf>,      // Primary ROM directories
    pub rom_paths: Vec<PathBuf>,     // Additional ROM paths (for UI compatibility)
    pub sample_paths: Vec<PathBuf>,  // Sound sample directories
    pub extra_rom_dirs: Vec<PathBuf>, // Extra ROM search paths
    pub extra_asset_dirs: Vec<PathBuf>, // Artwork, icons, etc.
    
    // MAME Support Files paths
    pub artwork_path: Option<PathBuf>,   // Artwork directory
    pub snap_path: Option<PathBuf>,      // Screenshots directory
    pub cabinet_path: Option<PathBuf>,   // Cabinet artwork directory
    pub title_path: Option<PathBuf>,     // Title screens directory
    pub flyer_path: Option<PathBuf>,     // Promotional flyers directory
    pub marquee_path: Option<PathBuf>,   // Marquee artwork directory
    pub cheats_path: Option<PathBuf>,    // Cheat files directory
    pub icons_path: Option<PathBuf>,     // Icons directory
    
    // Additional MAME Search Paths
    pub ctrlr_path: Option<PathBuf>,     // Controller definitions directory
    pub crosshair_path: Option<PathBuf>, // Crosshair files directory
    pub font_path: Option<PathBuf>,      // Font files directory
    pub plugins_path: Option<PathBuf>,   // Plugin files directory
    pub language_path: Option<PathBuf>,  // UI translation files directory
    pub sw_path: Option<PathBuf>,        // Loose software directory
    pub hash_path: Option<PathBuf>,      // Software definition files directory
    pub ini_path: Option<PathBuf>,       // INI files directory
    pub home_path: Option<PathBuf>,      // Base folder for plugin data (read/write)
    
    // History and DAT files
    pub history_path: Option<PathBuf>,    // History XML file path
    pub mameinfo_dat_path: Option<PathBuf>,  // mameinfo.dat file path
    pub hiscore_dat_path: Option<PathBuf>,   // hiscore.dat file path
    pub gameinit_dat_path: Option<PathBuf>,  // gameinit.dat file path
    pub command_dat_path: Option<PathBuf>,   // command.dat file path
    pub catver_ini_path: Option<PathBuf>,    // catver.ini file path for category support

    // MAME Internal Folders Configuration
    pub cfg_path: Option<PathBuf>,        // Configuration files directory
    pub nvram_path: Option<PathBuf>,      // Non-volatile RAM directory
    pub input_path: Option<PathBuf>,      // Input configuration directory
    pub state_path: Option<PathBuf>,      // Save state directory
    pub diff_path: Option<PathBuf>,       // Hard disk diff directory
    pub comment_path: Option<PathBuf>,    // Comment files directory

    // Filter and display settings
    pub filter_settings: FilterSettings,  // Game filtering options
    pub sort_column: SortColumn,         // Current sort column
    pub sort_direction: SortDirection,   // Ascending or descending

    // Game-specific settings
    pub game_preferred_mame: HashMap<String, usize>, // Preferred MAME for each game
    pub favorite_games: HashSet<String>,            // User's favorite games
    pub game_stats: HashMap<String, GameStats>,     // Play statistics per game
    pub game_properties: HashMap<String, super::game_properties::GameProperties>, // Per-game properties
    pub default_game_properties: super::game_properties::GameProperties, // Default properties for all games
    
    // Hidden categories - categories that should not be shown in the game list
    pub hidden_categories: HashSet<String>,

    // UI preferences
    pub show_filters: bool,          // Show filter panel
    pub selected_rom: Option<String>, // Currently selected ROM
    pub theme: Theme,                // Visual theme
    pub show_rom_icons: bool,        // Display game icons
    pub icon_size: u32,              // Icon size in pixels
    pub max_cached_icons: usize,     // Maximum icons to keep in memory
    pub column_widths: ColumnWidths, // Column width settings
    pub view_mode: ViewMode,         // View mode (Table or List)

    // MAME audit settings
    pub use_mame_audit: bool,                    // Use MAME's built-in audit
    pub mame_audit_times: HashMap<String, String>, // Last audit time per directory
    pub assume_merged_sets: bool,                // Assume ROMs are merged sets

    // Graphics and video
    pub graphics_config: GraphicsConfig, // Graphics backend configuration
    pub bgfx_path: Option<PathBuf>,      // BGFX shader path

    // Smart directory memory - remembers last used directory per category
    pub last_directories: HashMap<String, PathBuf>, // Category -> Last directory

    // Window settings for dialogs
    pub game_properties_window: WindowSettings, // Game properties dialog window settings

    pub preferences: Preferences,
}

impl AppConfig {
    /// Convert to TOML-serializable format
    fn to_toml(&self) -> AppConfigToml {
        AppConfigToml {
            mame_executables: self.mame_executables.clone(),
            selected_mame_index: self.selected_mame_index,
            rom_dirs: self.rom_dirs.clone(),
            rom_paths: self.rom_paths.clone(),
            sample_paths: self.sample_paths.clone(),
            extra_rom_dirs: self.extra_rom_dirs.clone(),
            extra_asset_dirs: self.extra_asset_dirs.clone(),
            artwork_path: self.artwork_path.clone(),
            snap_path: self.snap_path.clone(),
            cabinet_path: self.cabinet_path.clone(),
            title_path: self.title_path.clone(),
            flyer_path: self.flyer_path.clone(),
            marquee_path: self.marquee_path.clone(),
            cheats_path: self.cheats_path.clone(),
            icons_path: self.icons_path.clone(),
            ctrlr_path: self.ctrlr_path.clone(),
            crosshair_path: self.crosshair_path.clone(),
            font_path: self.font_path.clone(),
            plugins_path: self.plugins_path.clone(),
            language_path: self.language_path.clone(),
            sw_path: self.sw_path.clone(),
            hash_path: self.hash_path.clone(),
            ini_path: self.ini_path.clone(),
            home_path: self.home_path.clone(),
            history_path: self.history_path.clone(),
            mameinfo_dat_path: self.mameinfo_dat_path.clone(),
            hiscore_dat_path: self.hiscore_dat_path.clone(),
            gameinit_dat_path: self.gameinit_dat_path.clone(),
            command_dat_path: self.command_dat_path.clone(),
            catver_ini_path: self.catver_ini_path.clone(),
            cfg_path: self.cfg_path.clone(),
            nvram_path: self.nvram_path.clone(),
            input_path: self.input_path.clone(),
            state_path: self.state_path.clone(),
            diff_path: self.diff_path.clone(),
            comment_path: self.comment_path.clone(),
            filter_settings: self.filter_settings.clone(),
            sort_column: self.sort_column.clone(),
            sort_direction: self.sort_direction.clone(),
            game_preferred_mame: self.game_preferred_mame.clone(),
            favorite_games: self.favorite_games.clone(),
            game_stats: self.game_stats.clone(),
            game_properties: self.game_properties.clone(),
            default_game_properties: self.default_game_properties.clone(),
            hidden_categories: self.hidden_categories.clone(),
            show_filters: self.show_filters,
            selected_rom: self.selected_rom.clone(),
            theme: self.theme.clone(),
            show_rom_icons: self.show_rom_icons,
            icon_size: self.icon_size,
            max_cached_icons: self.max_cached_icons,
            column_widths: self.column_widths.clone(),
            view_mode: self.view_mode,
            use_mame_audit: self.use_mame_audit,
            mame_audit_times: self.mame_audit_times.clone(),
            assume_merged_sets: self.assume_merged_sets,
            graphics_config: self.graphics_config.clone(),
            bgfx_path: self.bgfx_path.clone(),
            last_directories: self.last_directories.clone(),
            game_properties_window: self.game_properties_window.clone(),
            preferences: self.preferences.clone(),
        }
    }

    /// Convert from TOML-serializable format
    fn from_toml(toml: AppConfigToml) -> Self {
        Self {
            mame_executables: toml.mame_executables,
            selected_mame_index: toml.selected_mame_index,
            rom_dirs: toml.rom_dirs,
            rom_paths: toml.rom_paths,
            sample_paths: toml.sample_paths,
            extra_rom_dirs: toml.extra_rom_dirs,
            extra_asset_dirs: toml.extra_asset_dirs,
            artwork_path: toml.artwork_path,
            snap_path: toml.snap_path,
            cabinet_path: toml.cabinet_path,
            title_path: toml.title_path,
            flyer_path: toml.flyer_path,
            marquee_path: toml.marquee_path,
            cheats_path: toml.cheats_path,
            icons_path: toml.icons_path,
            ctrlr_path: toml.ctrlr_path,
            crosshair_path: toml.crosshair_path,
            font_path: toml.font_path,
            plugins_path: toml.plugins_path,
            language_path: toml.language_path,
            sw_path: toml.sw_path,
            hash_path: toml.hash_path,
            ini_path: toml.ini_path,
            home_path: toml.home_path,
            history_path: toml.history_path,
            mameinfo_dat_path: toml.mameinfo_dat_path,
            hiscore_dat_path: toml.hiscore_dat_path,
            gameinit_dat_path: toml.gameinit_dat_path,
            command_dat_path: toml.command_dat_path,
            catver_ini_path: toml.catver_ini_path,
            cfg_path: toml.cfg_path,
            nvram_path: toml.nvram_path,
            input_path: toml.input_path,
            state_path: toml.state_path,
            diff_path: toml.diff_path,
            comment_path: toml.comment_path,
            filter_settings: toml.filter_settings,
            sort_column: toml.sort_column,
            sort_direction: toml.sort_direction,
            game_preferred_mame: toml.game_preferred_mame,
            favorite_games: toml.favorite_games,
            game_stats: toml.game_stats,
            game_properties: toml.game_properties,
            default_game_properties: toml.default_game_properties,
            hidden_categories: toml.hidden_categories,
            show_filters: toml.show_filters,
            selected_rom: toml.selected_rom,
            theme: toml.theme,
            show_rom_icons: toml.show_rom_icons,
            icon_size: toml.icon_size,
            max_cached_icons: toml.max_cached_icons,
            column_widths: toml.column_widths,
            view_mode: toml.view_mode,
            use_mame_audit: toml.use_mame_audit,
            mame_audit_times: toml.mame_audit_times,
            assume_merged_sets: toml.assume_merged_sets,
            graphics_config: toml.graphics_config,
            bgfx_path: toml.bgfx_path,
            last_directories: toml.last_directories,
            game_properties_window: toml.game_properties_window,
            preferences: toml.preferences,
        }
    }
}

impl serde::Serialize for AppConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_toml().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for AppConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        AppConfigToml::deserialize(deserializer).map(AppConfig::from_toml)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // Initialize with empty MAME list
            mame_executables: vec![],
            selected_mame_index: 0,

            // Initialize all path lists as empty
            rom_dirs: vec![],
            rom_paths: vec![],      // Important: This was missing in original
            sample_paths: vec![],   // Important: This was missing in original
            extra_rom_dirs: vec![],
            extra_asset_dirs: vec![],
            
            // MAME Support Files paths
            artwork_path: None,
            snap_path: None,
            cabinet_path: None,
            title_path: None,
            flyer_path: None,
            marquee_path: None,
            cheats_path: None,
            icons_path: None,
            
            // Additional MAME Search Paths
            ctrlr_path: None,
            crosshair_path: None,
            font_path: None,
            plugins_path: None,
            language_path: None,
            sw_path: None,
            hash_path: None,
            ini_path: None,
            home_path: None,
            
            // History and DAT files
            history_path: None,
            mameinfo_dat_path: None,
            hiscore_dat_path: None,
            gameinit_dat_path: None,
            command_dat_path: None,
            catver_ini_path: None,

            // MAME Internal Folders Configuration
            cfg_path: None,
            nvram_path: None,
            input_path: None,
            state_path: None,
            diff_path: None,
            comment_path: None,

            // Use default filter settings
            filter_settings: FilterSettings::default(),
            sort_column: SortColumn::default(),
            sort_direction: SortDirection::default(),

            // Empty game-specific maps
            game_preferred_mame: HashMap::new(),
            favorite_games: HashSet::new(),
            game_stats: HashMap::new(),
            game_properties: HashMap::new(),
            default_game_properties: super::game_properties::GameProperties::default(),
            
            // Initialize empty hidden categories
            hidden_categories: HashSet::new(),

            // Default UI settings
            show_filters: false,
            selected_rom: None,
            theme: Theme::default(),
            show_rom_icons: true,
            icon_size: 32,
            max_cached_icons: 500,
            column_widths: ColumnWidths::default(),
            view_mode: ViewMode::default(),

            // Audit disabled by default
            use_mame_audit: false,
            mame_audit_times: HashMap::new(),
            assume_merged_sets: false,

            // Default graphics settings
            graphics_config: GraphicsConfig::default(),
            bgfx_path: None,

            // Initialize empty smart directory memory
            last_directories: HashMap::new(),
            
            // Default window settings
            game_properties_window: WindowSettings::default(),
    
            preferences: Preferences::default(),  // Add this line
        }
    }
}
