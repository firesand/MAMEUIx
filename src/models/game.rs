// src/models/game.rs
use serde::{Deserialize, Serialize};
// Remove unused import: use std::collections::HashMap;

/// Represents a single game/ROM in the MAME system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,           // ROM filename without extension
    pub description: String,    // Human-readable game name
    pub manufacturer: String,   // Company that made the game
    pub year: String,          // Year of release
    pub driver: String,        // MAME driver used
    pub driver_status: String, // Driver status: good, imperfect, preliminary
    pub status: RomStatus,     // Current status of this ROM
    pub parent: Option<String>, // Parent ROM name if this is a clone
    pub category: String,      // Game category/genre
    pub play_count: u32,       // How many times played
    pub is_clone: bool,        // Whether this is a clone ROM
    pub is_device: bool,       // Whether this is a device ROM
    pub is_bios: bool,         // Whether this is a BIOS ROM
    pub controls: String,      // Control scheme description
    pub requires_chd: bool,    // Whether this game requires a CHD file
    pub chd_name: Option<String>, // Name of the required CHD file (if any)
}

/// Represents the status of a ROM file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RomStatus {
    Unknown,        // Status hasn't been determined yet
    Available,      // ROM file is present and correct
    Missing,        // ROM file is not found
    Incorrect,      // ROM file exists but has wrong checksum
    NotWorking,     // ROM is present but game doesn't work
    Preliminary,    // Early driver, not fully working
    ChdRequired,    // ROM is available but CHD is required
    ChdMissing,     // ROM is available but CHD is missing
}

impl RomStatus {
    pub fn to_icon(&self) -> &'static str {
        match self {
            RomStatus::Available => "✅",
            RomStatus::Missing => "❌",
            RomStatus::Incorrect => "⚠️",
            RomStatus::NotWorking => "🔴",
            RomStatus::Preliminary => "🟡",
            RomStatus::ChdRequired => "💿",
            RomStatus::ChdMissing => "💿❌",
            RomStatus::Unknown => "❓",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RomStatus::Available => "Available",
            RomStatus::Missing => "Missing",
            RomStatus::Incorrect => "Incorrect",
            RomStatus::NotWorking => "Not Working",
            RomStatus::Preliminary => "Preliminary",
            RomStatus::ChdRequired => "CHD Required",
            RomStatus::ChdMissing => "CHD Missing",
            RomStatus::Unknown => "Unknown",
        }
    }
}

impl Default for RomStatus {
    fn default() -> Self {
        RomStatus::Unknown  // Default to unknown status
    }
}

impl Game {
    pub fn get_driver_status_display(&self) -> (&'static str, &str) {
        match self.driver_status.as_str() {
            "good" => ("✅", "Good"),
            "imperfect" => ("⚠️", "Imperfect"),
            "preliminary" => ("🔴", "Preliminary"),
            _ => ("❓", &self.driver_status),
        }
    }

    pub fn get_driver_status_color(&self) -> egui::Color32 {
        match self.driver_status.as_str() {
            "good" => egui::Color32::from_rgb(0, 255, 0),
            "imperfect" => egui::Color32::from_rgb(255, 255, 0),
            "preliminary" => egui::Color32::from_rgb(255, 0, 0),
            _ => egui::Color32::GRAY,
        }
    }
}


