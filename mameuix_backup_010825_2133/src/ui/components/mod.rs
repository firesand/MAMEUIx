// src/ui/components/mod.rs
// Dialog and component files

pub mod dialog_manager;
pub mod preferences;
pub mod directories;
pub mod game_properties;
pub mod rom_verify;
pub mod mame_finder;
pub mod hidden_categories;
pub mod rom_info;

pub use dialog_manager::{DialogManager, DialogType, DialogAction}; 