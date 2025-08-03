// src/ui/components/mod.rs
// Dialog and component files

pub mod dialog_manager;
pub mod preferences;
pub mod directories;
pub mod directories_paths;  // New modern UI implementation
pub mod game_properties;
pub mod rom_verify;
pub mod mame_finder;
pub mod hidden_categories;
pub mod rom_info;
pub mod advanced_mame_settings;

pub use dialog_manager::{DialogManager, DialogType, DialogAction};
pub use advanced_mame_settings::AdvancedMameSettingsDialog;
pub use directories_paths::DirectoriesPathsDialog;