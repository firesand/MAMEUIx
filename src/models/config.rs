use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use crate::graphics::GraphicsConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MameExecutable {
    pub name: String,
    pub path: String,
    pub version: String,
    pub total_games: usize,
    pub working_games: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub video_backend: String,
    pub window_mode: bool,
    pub maximize: bool,
    pub wait_vsync: bool,
    pub sync_refresh: bool,
    pub prescale: u8,
    pub keep_aspect: bool,
    pub filter: bool,
    pub num_screens: u8,
    pub custom_args: String,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            video_backend: "auto".to_string(),
            window_mode: true,
            maximize: false,
            wait_vsync: false,
            sync_refresh: false,
            prescale: 0,
            keep_aspect: true,
            filter: true,
            num_screens: 1,
            custom_args: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    DarkBlue,
    DarkGrey,
    ArcadePurple,
}

impl Theme {
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();

        match self {
            Theme::DarkBlue => {
                visuals.panel_fill = egui::Color32::from_rgb(20, 25, 40);
                visuals.window_fill = egui::Color32::from_rgb(25, 30, 45);
            }
            Theme::DarkGrey => {
                visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 35, 40);
            }
            Theme::ArcadePurple => {
                visuals.panel_fill = egui::Color32::from_rgb(25, 20, 35);
                visuals.window_fill = egui::Color32::from_rgb(35, 25, 45);
                visuals.selection.bg_fill = egui::Color32::from_rgb(100, 50, 150);
            }
        }

        ctx.set_visuals(visuals);
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::DarkBlue
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub mame_executables: Vec<MameExecutable>,
    pub selected_mame_index: usize,
    pub rom_dirs: Vec<PathBuf>,
    pub extra_rom_dirs: Vec<PathBuf>,
    pub extra_asset_dirs: Vec<PathBuf>,
    pub filter_settings: super::FilterSettings,
    pub sort_column: super::SortColumn,
    pub sort_direction: super::SortDirection,
    pub game_preferred_mame: HashMap<String, usize>,
    pub show_filters: bool,
    pub selected_rom: Option<String>,
    pub use_mame_audit: bool,
    pub mame_audit_times: HashMap<String, String>,
    pub assume_merged_sets: bool,
    pub favorite_games: HashSet<String>,
    pub game_stats: HashMap<String, GameStats>,
    pub theme: Theme,
    pub show_rom_icons: bool,
    pub icon_size: u32,
    pub max_cached_icons: usize,
    pub graphics_config: GraphicsConfig,
    pub video_settings: VideoSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mame_executables: vec![],
            selected_mame_index: 0,
            rom_dirs: vec![],
            extra_rom_dirs: vec![],
            extra_asset_dirs: vec![],
            filter_settings: super::FilterSettings::default(),
            sort_column: super::SortColumn::default(),
            sort_direction: super::SortDirection::default(),
            game_preferred_mame: HashMap::new(),
            show_filters: false,
            selected_rom: None,
            use_mame_audit: false,
            mame_audit_times: HashMap::new(),
            assume_merged_sets: false,
            favorite_games: HashSet::new(),
            game_stats: HashMap::new(),
            theme: Theme::default(),
            show_rom_icons: true,
            icon_size: 32,
            max_cached_icons: 500,
            graphics_config: GraphicsConfig::default(),
            video_settings: VideoSettings::default(),
        }
    }
}
