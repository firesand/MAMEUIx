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
    /// Convert the status to a display icon
    /// This provides a visual indicator in the game list
    pub fn to_icon(&self) -> &'static str {
        match self {
            RomStatus::Unknown => "❓",      // Question mark for unknown
            RomStatus::Available => "✅",     // Checkmark for good ROMs
            RomStatus::Missing => "❌",       // X for missing ROMs
            RomStatus::Incorrect => "⚠️",     // Warning for bad ROMs
            RomStatus::NotWorking => "🚫",    // Prohibited sign for non-working
            RomStatus::Preliminary => "🔧",   // Wrench for in-development
            RomStatus::ChdRequired => "💾",   // Diskette for CHD required
            RomStatus::ChdMissing => "❌",    // X for CHD missing
        }
    }

    /// Get a human-readable description of the status
    pub fn description(&self) -> &'static str {
        match self {
            RomStatus::Unknown => "Status unknown",
            RomStatus::Available => "ROM available and verified",
            RomStatus::Missing => "ROM file not found",
            RomStatus::Incorrect => "ROM file has incorrect checksum",
            RomStatus::NotWorking => "Game is not working in MAME",
            RomStatus::Preliminary => "Preliminary driver, may have issues",
            RomStatus::ChdRequired => "ROM requires a CHD file to be playable",
            RomStatus::ChdMissing => "ROM is available but CHD file is missing",
        }
    }

    /// Check if the game is playable with this status
    pub fn is_playable(&self) -> bool {
        matches!(self, RomStatus::Available | RomStatus::Preliminary)
    }
}

impl Default for RomStatus {
    fn default() -> Self {
        RomStatus::Unknown  // Default to unknown status
    }
}

impl Game {
    /// Create a new game with minimal information
    /// This is useful when parsing game lists before full verification
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            manufacturer: String::new(),
            year: String::new(),
            driver: String::new(),
            status: RomStatus::Unknown,
            parent: None,
            category: String::new(),
            play_count: 0,
            is_clone: false,
            is_device: false,
            is_bios: false,
            controls: String::new(),
            requires_chd: false,
            chd_name: None,
        }
    }

    /// Check if this game should be shown based on its properties
    /// Devices and BIOS ROMs are typically hidden from normal game lists
    pub fn is_game(&self) -> bool {
        !self.is_device && !self.is_bios
    }

    /// Get a display name that includes clone information
    pub fn full_name(&self) -> String {
        if let Some(parent) = &self.parent {
            format!("{} (clone of {})", self.description, parent)
        } else {
            self.description.clone()
        }
    }
}
