use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub description: String,
    pub manufacturer: String,
    pub year: String,
    pub driver: String,
    pub status: RomStatus,
    pub parent: Option<String>,
    pub category: String,
    pub play_count: u32,
    pub is_clone: bool,
    pub is_device: bool,
    pub is_bios: bool,
    pub controls: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RomStatus {
    Available,
    Missing,
    Incorrect,
    Working,
    Imperfect,
    NotWorking,
}

impl RomStatus {
    pub fn to_icon(&self) -> &'static str {
        match self {
            RomStatus::Available | RomStatus::Working => "✅",
            RomStatus::Imperfect => "⚠️",
            RomStatus::Missing | RomStatus::NotWorking => "❌",
            RomStatus::Incorrect => "⚠️",
        }
    }

    pub fn to_color(&self) -> egui::Color32 {
        match self {
            RomStatus::Available | RomStatus::Working => egui::Color32::from_rgb(0, 255, 0),
            RomStatus::Imperfect => egui::Color32::from_rgb(255, 200, 0),
            RomStatus::Missing | RomStatus::NotWorking => egui::Color32::from_rgb(255, 0, 0),
            RomStatus::Incorrect => egui::Color32::from_rgb(255, 150, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub play_count: u32,
    pub last_played: Option<String>,
    pub total_play_time: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            play_count: 0,
            last_played: None,
            total_play_time: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IconInfo {
    pub loaded: bool,
    pub last_accessed: std::time::Instant,
}
