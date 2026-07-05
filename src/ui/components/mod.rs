// src/ui/components/mod.rs
// Dialog and component files

pub mod advanced_mame_settings;
pub mod dialog_manager;
pub mod directories;
pub mod directories_paths; // New modern UI implementation
pub mod game_properties;
pub mod hidden_categories;
pub mod mame_finder;
pub mod preferences;
pub mod rom_info;
pub mod rom_verify;
pub mod steam_ui;

pub use advanced_mame_settings::AdvancedMameSettingsDialog;
pub use dialog_manager::{DialogAction, DialogManager, DialogType};
