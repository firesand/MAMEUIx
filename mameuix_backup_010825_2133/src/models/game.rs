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
    // Verification status tracking
    pub verification_status: Option<VerificationStatus>,
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

/// Represents the verification status from ROM verification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,       // ROM verified successfully
    Failed,         // ROM verification failed
    Warning,        // ROM verification has warnings
    NotFound,       // ROM not found during verification
    NotVerified,    // ROM hasn't been verified yet
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

impl VerificationStatus {
    pub fn to_icon(&self) -> &'static str {
        match self {
            VerificationStatus::Verified => "✅",
            VerificationStatus::Failed => "❌",
            VerificationStatus::Warning => "⚠️",
            VerificationStatus::NotFound => "❓",
            VerificationStatus::NotVerified => "⏳",
        }
    }

    pub fn to_color(&self) -> egui::Color32 {
        match self {
            VerificationStatus::Verified => egui::Color32::GREEN,
            VerificationStatus::Failed => egui::Color32::RED,
            VerificationStatus::Warning => egui::Color32::YELLOW,
            VerificationStatus::NotFound => egui::Color32::GRAY,
            VerificationStatus::NotVerified => egui::Color32::from_rgb(128, 128, 128),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            VerificationStatus::Verified => "Verified",
            VerificationStatus::Failed => "Failed",
            VerificationStatus::Warning => "Warning",
            VerificationStatus::NotFound => "Not Found",
            VerificationStatus::NotVerified => "Not Verified",
        }
    }
}

impl Default for VerificationStatus {
    fn default() -> Self {
        VerificationStatus::NotVerified
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

    /// Get the verification status icon with color
    pub fn get_verification_display(&self) -> (&'static str, egui::Color32) {
        if let Some(verification_status) = &self.verification_status {
            (verification_status.to_icon(), verification_status.to_color())
        } else {
            (VerificationStatus::NotVerified.to_icon(), VerificationStatus::NotVerified.to_color())
        }
    }

    /// Update verification status
    pub fn update_verification_status(&mut self, status: VerificationStatus) {
        self.verification_status = Some(status);
    }
}


