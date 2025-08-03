// src/ui/dialog_manager.rs
// Dialog state management and rendering module

use eframe::egui;
use crate::models::*;
use crate::ui::components::mame_finder::{MameFinderDialog, FoundMame};
use crate::ui::components::rom_verify::{RomVerifyDialog, VerificationManager};
use crate::ui::components::game_properties::GamePropertiesDialog;
use crate::ui::components::directories::DirectoriesDialog;
use crate::ui::components::preferences::PreferencesDialog;
use crate::ui::components::hidden_categories::HiddenCategoriesDialog;
use crate::ui::components::rom_info::RomInfoDialog;
use std::collections::HashMap;
use std::sync::Arc;

/// Actions that dialogs can trigger
#[derive(Debug, Clone)]
pub enum DialogAction {
    SaveConfig,
    StartInitialLoad,
    ReloadCategories,
    OnDirectoriesChanged,
}

/// Enum representing all available dialog types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogType {
    Directories,
    Preferences,
    RomInfo,
    About,
    HiddenCategories,
    MameFinder,
    ManualMame,
    GameProperties,
    RomVerify,
}

/// Dialog state management
pub struct DialogManager {
    // Dialog visibility states
    dialog_states: HashMap<DialogType, bool>,
    
    // Dialog-specific data
    found_mame_executables: Vec<FoundMame>,
    rom_verify_dialog: RomVerifyDialog,
    game_properties_dialog: Option<GamePropertiesDialog>,
    
    // Callback for when dialogs need to trigger actions
    on_dialog_closed: Option<Box<dyn Fn(DialogType, bool) + Send + Sync>>,
}

impl DialogManager {
    pub fn new() -> Self {
        let mut dialog_states = HashMap::new();
        
        // Initialize all dialog states to false
        for dialog_type in [
            DialogType::Directories,
            DialogType::Preferences,
            DialogType::RomInfo,
            DialogType::About,
            DialogType::HiddenCategories,
            DialogType::MameFinder,
            DialogType::ManualMame,
            DialogType::GameProperties,
        ] {
            dialog_states.insert(dialog_type, false);
        }
        
        Self {
            dialog_states,
            found_mame_executables: Vec::new(),
            rom_verify_dialog: RomVerifyDialog::default(),
            game_properties_dialog: None,
            on_dialog_closed: None,
        }
    }
    
    /// Set callback for dialog closed events
    pub fn set_dialog_closed_callback<F>(&mut self, callback: F)
    where
        F: Fn(DialogType, bool) + Send + Sync + 'static,
    {
        self.on_dialog_closed = Some(Box::new(callback));
    }
    
    /// Check if any dialog is currently open
    pub fn is_any_dialog_open(&self) -> bool {
        self.dialog_states.values().any(|&state| state) || self.rom_verify_dialog.is_open()
    }
    
    /// Check if a specific dialog is open
    pub fn is_dialog_open(&self, dialog_type: DialogType) -> bool {
        self.dialog_states.get(&dialog_type).copied().unwrap_or(false)
    }
    
    /// Open a dialog
    pub fn open_dialog(&mut self, dialog_type: DialogType) {
        self.dialog_states.insert(dialog_type, true);
    }
    
    /// Close a dialog
    pub fn close_dialog(&mut self, dialog_type: DialogType) {
        if self.dialog_states.insert(dialog_type, false).unwrap_or(false) {
            // Dialog was actually open, trigger callback
            if let Some(ref callback) = self.on_dialog_closed {
                callback(dialog_type, false);
            }
        }
    }
    
    /// Set dialog state
    pub fn set_dialog_state(&mut self, dialog_type: DialogType, state: bool) {
        let was_open = self.dialog_states.insert(dialog_type, state).unwrap_or(false);
        
        // If dialog was closed, trigger callback
        if was_open && !state {
            if let Some(ref callback) = self.on_dialog_closed {
                callback(dialog_type, false);
            }
        }
    }
    
    /// Set MAME finder data
    pub fn set_found_mame_executables(&mut self, executables: Vec<FoundMame>) {
        self.found_mame_executables = executables;
    }
    
    /// Get MAME finder data
    pub fn get_found_mame_executables(&self) -> &[FoundMame] {
        &self.found_mame_executables
    }
    
    /// Set game properties dialog
    pub fn set_game_properties_dialog(&mut self, dialog: Option<GamePropertiesDialog>) {
        self.game_properties_dialog = dialog;
    }
    
    /// Get ROM verify dialog reference
    pub fn rom_verify_dialog(&mut self) -> &mut RomVerifyDialog {
        &mut self.rom_verify_dialog
    }

    /// Get verification manager reference
    pub fn verification_manager(&self) -> Arc<VerificationManager> {
        self.rom_verify_dialog.verification_manager()
    }
    
    /// Render all open dialogs
    pub fn render_dialogs(
        &mut self,
        ctx: &egui::Context,
        config: &mut AppConfig,
        games: &[Game],
        selected_game: Option<usize>,
        category_manager: Option<&filters::CategoryManager>,
        need_reload_after_dialog: &mut bool,
    ) -> Vec<DialogAction> {
        let mut actions = Vec::new();
        
        // Directories Dialog
        if self.is_dialog_open(DialogType::Directories) {
            let changed = DirectoriesDialog::show(
                ctx, 
                config, 
                self.dialog_states.get_mut(&DialogType::Directories).unwrap()
            );
            
            // Check if dialog was closed
            if !self.is_dialog_open(DialogType::Directories) {
                // Always save config when dialog is closed
                actions.push(DialogAction::SaveConfig);
                
                // Check if catver.ini was just configured
                if config.catver_ini_path.is_some() {
                    actions.push(DialogAction::ReloadCategories);
                } else if changed {
                    // For other changes, reload everything
                    *need_reload_after_dialog = true;
                }
            }
            
            if !self.is_dialog_open(DialogType::Directories) && *need_reload_after_dialog {
                actions.push(DialogAction::OnDirectoriesChanged);
                *need_reload_after_dialog = false;
            }
        }
        
        // Preferences Dialog
        if self.is_dialog_open(DialogType::Preferences) {
            PreferencesDialog::show(
                ctx,
                &mut config.preferences,
                &mut config.theme,
                self.dialog_states.get_mut(&DialogType::Preferences).unwrap(),
                config.catver_ini_path.is_some()
            );
            
            // Check if dialog was closed
            if !self.is_dialog_open(DialogType::Preferences) {
                // Save config when preferences dialog is closed
                actions.push(DialogAction::SaveConfig);
            }
        }
        
        // ROM Info Dialog
        if self.is_dialog_open(DialogType::RomInfo) {
            if let Some(idx) = selected_game {
                if let Some(game) = games.get(idx) {
                    RomInfoDialog::show(
                        ctx,
                        game,
                        self.dialog_states.get_mut(&DialogType::RomInfo).unwrap()
                    );
                }
            }
        }
        
        // About Dialog
        if self.is_dialog_open(DialogType::About) {
            self.render_about_dialog(ctx);
        }
        
        // Hidden Categories Dialog
        if self.is_dialog_open(DialogType::HiddenCategories) {
            HiddenCategoriesDialog::show(
                ctx,
                &mut config.hidden_categories,
                category_manager,
                self.dialog_states.get_mut(&DialogType::HiddenCategories).unwrap()
            );
        }
        
        // MAME Finder Dialog
        if self.is_dialog_open(DialogType::MameFinder) {
            if !self.found_mame_executables.is_empty() {
                if MameFinderDialog::show_selection_dialog(
                    ctx,
                    &self.found_mame_executables,
                    config,
                    self.dialog_states.get_mut(&DialogType::MameFinder).unwrap(),
                ) {
                    actions.push(DialogAction::SaveConfig);
                    actions.push(DialogAction::StartInitialLoad);
                } else if !self.is_dialog_open(DialogType::MameFinder) {
                    // User chose to browse manually
                    self.open_dialog(DialogType::ManualMame);
                }
            } else {
                // No MAME found, show manual selection
                self.close_dialog(DialogType::MameFinder);
                self.open_dialog(DialogType::ManualMame);
            }
        }
        
        // Manual MAME Dialog
        if self.is_dialog_open(DialogType::ManualMame) {
            if MameFinderDialog::show_manual_selection_dialog(
                ctx,
                config,
                self.dialog_states.get_mut(&DialogType::ManualMame).unwrap(),
            ) {
                actions.push(DialogAction::SaveConfig);
                actions.push(DialogAction::StartInitialLoad);
            }
        }
        
        // ROM Verification Dialog
        if self.rom_verify_dialog.is_open() {
            self.rom_verify_dialog.show_window(ctx, config, games);
        }
        
        // Game Properties Dialog
        if self.is_dialog_open(DialogType::GameProperties) {
            if let Some(dialog) = &mut self.game_properties_dialog {
                if dialog.show(ctx, self.dialog_states.get_mut(&DialogType::GameProperties).unwrap(), config) {
                    // Properties were applied
                    actions.push(DialogAction::SaveConfig);
                }
            }
        }
        
        actions
    }

    fn render_about_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("About MAMEuix")
            .open(self.dialog_states.get_mut(&DialogType::About).unwrap())
            .default_size([500.0, 450.0])
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("MAMEuix");
                    ui.label("A modern MAME frontend with enhanced features");
                    ui.add_space(10.0);
                    ui.label("Version: 0.1.4");
                    ui.label("Built with Rust and egui");
                    ui.add_space(20.0);
                    
                    ui.label(egui::RichText::new("🎮 Core Features").heading());
                    ui.label("• Fast game scanning and filtering (48,000+ games)");
                    ui.label("• Enhanced search capabilities with real-time results");
                    ui.label("• Customizable UI themes (10 beautiful themes)");
                    ui.label("• Plugin support (hiscore, cheat, autofire)");
                    ui.label("• Virtual scrolling for smooth performance");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("🔍 ROM Verification - CLRMamePro Lite").heading());
                    ui.label("• Real-time verification status with progress tracking");
                    ui.label("• Color-coded game list (Green=Verified, Red=Failed, Yellow=Warning)");
                    ui.label("• Bulk actions: Find missing ROMs (No-Intro integration)");
                    ui.label("• Export reports in Text, CSV, and HTML formats");
                    ui.label("• Pause/Resume/Stop verification controls");
                    ui.label("• Detailed statistics and ETA calculations");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("🎨 Graphics & Performance").heading());
                    ui.label("• BGFX multi-backend support (8 rendering backends)");
                    ui.label("• Embedded GLSL shaders (11 professional effects)");
                    ui.label("• Integer scaling for pixel-perfect display");
                    ui.label("• Core performance options and real-time configuration");
                    ui.label("• Hardware filtering by CPU, device, and sound chip");
                    ui.add_space(10.0);
                    
                    ui.label(egui::RichText::new("⚙️ Advanced Features").heading());
                    ui.label("• Thread pool icon loading with 8x performance improvement");
                    ui.label("• Performance monitoring and adaptive loading");
                    ui.label("• Column width persistence and resizable tables");
                    ui.label("• Category support with catver.ini integration");
                    ui.label("• CHD game support and artwork display");
                });
            });
    }
    
    /// Initialize MAME finder dialog
    pub fn initialize_mame_finder(&mut self, config: &AppConfig) -> bool {
        if config.mame_executables.is_empty() {
            println!("First launch detected - searching for MAME executables...");
            let found_mames = MameFinderDialog::find_mame_executables();
            
            if !found_mames.is_empty() {
                println!("Found {} MAME executable(s)", found_mames.len());
                for mame in &found_mames {
                    println!("  - {} ({})", mame.path, mame.version);
                }
                self.set_found_mame_executables(found_mames);
                self.open_dialog(DialogType::MameFinder);
                return true;
            } else {
                println!("No MAME executables found in standard locations");
                self.open_dialog(DialogType::ManualMame);
                return true;
            }
        }
        false
    }
    
    /// Close all dialogs
    pub fn close_all_dialogs(&mut self) {
        for dialog_type in [
            DialogType::Directories,
            DialogType::Preferences,
            DialogType::RomInfo,
            DialogType::About,
            DialogType::HiddenCategories,
            DialogType::MameFinder,
            DialogType::ManualMame,
            DialogType::GameProperties,
        ] {
            self.close_dialog(dialog_type);
        }
        // self.rom_verify_dialog.close(); // Commented out as close method doesn't exist
    }
} 
