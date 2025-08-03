// src/ui/components/directories_paths.rs
// New Directories & Paths dialog with modern UI design
// This is a safe redundant implementation that doesn't replace the existing Directories dialog

use eframe::egui;
use crate::models::{AppConfig, MameExecutable};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TabCategory {
    MamePaths,
    SupportFiles,
    HistoryFiles,
    InternalFolders,
}

/// DirectoriesPathsDialog - Modern UI implementation based on HTML mockup
/// This is a safe alternative to the existing DirectoriesDialog
pub struct DirectoriesPathsDialog {
    selected_tab: TabCategory,
    temp_config: AppConfig,
    changes_made: bool,
}

impl DirectoriesPathsDialog {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            selected_tab: TabCategory::MamePaths,
            temp_config: config.clone(),
            changes_made: false,
        }
    }

    /// Show the dialog and return true if changes were made and confirmed
    pub fn show(&mut self, ctx: &egui::Context, config: &mut AppConfig, open: &mut bool) -> bool {
        let mut close = false;
        let mut apply_changes = false;

        // Modern dark theme colors matching the mockup
        let bg_primary = egui::Color32::from_rgb(10, 10, 10);
        let bg_secondary = egui::Color32::from_rgb(22, 22, 22);
        let bg_tertiary = egui::Color32::from_rgb(30, 30, 30);
        let bg_hover = egui::Color32::from_rgb(37, 37, 37);
        let border_color = egui::Color32::from_rgb(51, 51, 51);
        let text_primary = egui::Color32::from_rgb(240, 240, 250);
        let text_secondary = egui::Color32::from_rgb(160, 160, 160);
        let accent_primary = egui::Color32::from_rgb(76, 139, 245);
        let accent_hover = egui::Color32::from_rgb(61, 122, 229);
        let success_color = egui::Color32::from_rgb(76, 175, 80);
        let error_color = egui::Color32::from_rgb(244, 67, 54);

        egui::Window::new("Directories Selection")
            .default_size([800.0, 600.0])
            .min_size([700.0, 500.0])
            .resizable(true)
            .open(open)
            .show(ctx, |ui| {
                // Tab state
                let mut selected_tab = ui.data_mut(|d| d.get_temp::<usize>(ui.id()).unwrap_or(0));
                
                // Calculate available height untuk konten
                let total_height = ui.available_height();
                let footer_height = 60.0; // Space untuk buttons dan separator
                let content_height = total_height - footer_height;
                
                // Main container dengan fixed height
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), content_height),
                    egui::Layout::left_to_right(egui::Align::TOP),
                    |ui| {
                        // Left sidebar
                        ui.allocate_ui_with_layout(
                            egui::vec2(200.0, content_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                // Style untuk sidebar background
                                let rect = ui.available_rect_before_wrap();
                                ui.painter().rect_filled(
                                    rect,
                                    0.0,
                                    egui::Color32::from_gray(25), // Darker background untuk sidebar
                                );
                                
                                ui.add_space(10.0);
                                
                                // Sidebar items
                                let categories = [
                                    ("MAME Paths", 0),
                                    ("MAME Support Files", 1),
                                    ("History, INI's & DAT's Files", 2),
                                    ("MAME Internal Folders", 3),
                                ];
                                
                                for (label, idx) in categories {
                                    let is_selected = selected_tab == idx;
                                    
                                    // Custom styling untuk sidebar items
                                    let response = ui.allocate_response(
                                        egui::Vec2::new(180.0, 40.0),
                                        egui::Sense::click()
                                    );
                                    
                                    let rect = response.rect;
                                    let text_color = if is_selected {
                                        egui::Color32::WHITE
                                    } else if response.hovered() {
                                        egui::Color32::from_gray(220)
                                    } else {
                                        egui::Color32::from_gray(160)
                                    };
                                    
                                    // Background untuk selected/hover
                                    if is_selected {
                                        ui.painter().rect_filled(
                                            rect,
                                            6.0,
                                            egui::Color32::from_rgb(76, 139, 245), // Accent color
                                        );
                                    } else if response.hovered() {
                                        ui.painter().rect_filled(
                                            rect,
                                            6.0,
                                            egui::Color32::from_gray(40),
                                        );
                                    }
                                    
                                    // Draw text
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        label,
                                        egui::FontId::proportional(14.0),
                                        text_color,
                                    );
                                    
                                    if response.clicked() {
                                        selected_tab = idx;
                                    }
                                }
                            }
                        );
                        
                        // Vertical separator
                        ui.separator();
                        
                        // Right content area - gunakan seluruh ruang yang tersedia
                        ui.allocate_ui_with_layout(
                            egui::vec2(ui.available_width(), content_height),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| {
                                // Content berdasarkan selected tab
                                match selected_tab {
                                    0 => {
                                        // MAME Paths tab
                                        ui.heading("MAME Paths");
                                        ui.add_space(10.0);
                                        
                                        // ScrollArea untuk seluruh konten
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // MAME Executables section
                                                ui.group(|ui| {
                                                    ui.label(egui::RichText::new("MAME Executables").size(16.0).strong());
                                                    ui.label("These are the MAME emulator programs that will run your games");
                                                    ui.add_space(5.0);

                                                    if Self::executable_list_inline(ui, &mut self.temp_config.mame_executables) {
                                                        self.changes_made = true;
                                                    }
                                                });

                                                ui.add_space(10.0);

                                                // ROM Paths section
                                                ui.group(|ui| {
                                                    ui.label(egui::RichText::new("ROM Directories").size(16.0).strong());
                                                    ui.label("Folders containing your game ROM files");
                                                    ui.add_space(5.0);

                                                    if Self::path_list_inline(ui, &mut self.temp_config.rom_paths, "roms", &std::collections::HashMap::new(), &mut ()) {
                                                        self.changes_made = true;
                                                    }
                                                });
                                            });
                                    }
                                    1 => {
                                        // MAME Support Files tab
                                        ui.heading("MAME Support Files");
                                        ui.label("Configure paths for MAME support files:");
                                        ui.add_space(10.0);
                                        
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Artwork path
                                                if Self::render_optional_path_field_static(ui, "Artwork:", &mut self.temp_config.artwork_path, "/path/to/artwork") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Cabinets path
                                                if Self::render_optional_path_field_static(ui, "Cabinets:", &mut self.temp_config.cabinet_path, "/path/to/cabinets") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Flyers path
                                                if Self::render_optional_path_field_static(ui, "Flyers:", &mut self.temp_config.flyer_path, "/path/to/flyers") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Marquees path
                                                if Self::render_optional_path_field_static(ui, "Marquees:", &mut self.temp_config.marquee_path, "/path/to/marquees") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Sample paths
                                                ui.separator();
                                                ui.add_space(8.0);
                                                ui.label("Sample Directories:");
                                                ui.add_space(4.0);
                                                
                                                let mut sample_paths = self.temp_config.sample_paths.clone();
                                                let mut to_remove = None;
                                                
                                                for (idx, path) in sample_paths.iter_mut().enumerate() {
                                                    ui.horizontal(|ui| {
                                                        let mut path_str = path.display().to_string();
                                                        if ui.add(egui::TextEdit::singleline(&mut path_str).desired_width(350.0)).changed() {
                                                            *path = PathBuf::from(&path_str);
                                                            self.changes_made = true;
                                                        }
                                                        
                                                        if ui.button("Browse...").clicked() {
                                                            if let Some(folder) = rfd::FileDialog::new()
                                                                .set_title("Select Sample Directory")
                                                                .pick_folder() {
                                                                *path = folder;
                                                                self.changes_made = true;
                                                            }
                                                        }
                                                        
                                                        if ui.button("🗑").clicked() {
                                                            to_remove = Some(idx);
                                                            self.changes_made = true;
                                                        }
                                                    });
                                                }
                                                
                                                self.temp_config.sample_paths = sample_paths;
                                                
                                                if let Some(idx) = to_remove {
                                                    self.temp_config.sample_paths.remove(idx);
                                                }
                                                
                                                if ui.button("➕ Add Sample Directory").clicked() {
                                                    self.temp_config.sample_paths.push(PathBuf::new());
                                                    self.changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Snapshots path
                                                if Self::render_optional_path_field_static(ui, "Snapshots:", &mut self.temp_config.snap_path, "/path/to/snap") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Titles path
                                                if Self::render_optional_path_field_static(ui, "Titles:", &mut self.temp_config.title_path, "/path/to/titles") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Cheats path
                                                if Self::render_optional_path_field_static(ui, "Cheats:", &mut self.temp_config.cheats_path, "/path/to/cheats") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Icons path
                                                if Self::render_optional_path_field_static(ui, "Icons:", &mut self.temp_config.icons_path, "/path/to/icons") {
                                                    self.changes_made = true;
                                                }
                                            });
                                    }
                                    2 => {
                                        // History, INI's and DAT's Files tab
                                        ui.heading("History, INI's & DAT's Files");
                                        ui.label("Configure paths for MAME history, INI and DAT files:");
                                        ui.add_space(10.0);
                                        
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Catver.ini path (for category support)
                                                ui.push_id("catver_section", |ui| {
                                                    ui.label(egui::RichText::new("Category Support").size(16.0).strong());
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(200, 200, 100),
                                                        "The catver.ini file is required to display game categories"
                                                    );
                                                    
                                                    if Self::render_optional_file_field_static(ui, "Catver INI:", &mut self.temp_config.catver_ini_path, "/path/to/catver.ini") {
                                                        self.changes_made = true;
                                                    }
                                                });
                                                
                                                ui.add_space(20.0);
                                                ui.separator();
                                                ui.add_space(10.0);
                                                
                                                // History path
                                                if Self::render_optional_file_field_static(ui, "History:", &mut self.temp_config.history_path, "/path/to/history.xml") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // mameinfo.dat path
                                                if Self::render_optional_file_field_static(ui, "MAME Info DAT:", &mut self.temp_config.mameinfo_dat_path, "/path/to/mameinfo.dat") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // hiscore.dat path
                                                if Self::render_optional_file_field_static(ui, "High Score DAT:", &mut self.temp_config.hiscore_dat_path, "/path/to/hiscore.dat") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // gameinit.dat path
                                                if Self::render_optional_file_field_static(ui, "Game Init DAT:", &mut self.temp_config.gameinit_dat_path, "/path/to/gameinit.dat") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // command.dat path
                                                if Self::render_optional_file_field_static(ui, "Command DAT:", &mut self.temp_config.command_dat_path, "/path/to/command.dat") {
                                                    self.changes_made = true;
                                                }
                                            });
                                    }
                                    3 => {
                                        // MAME Internal Folders tab
                                        ui.heading("MAME Internal Folders");
                                        ui.label("Configure MAME internal folders (these override MAME's default locations):");
                                        ui.colored_label(
                                            egui::Color32::from_rgb(200, 200, 100),
                                            "Note: These folders are used by MAME for saving configuration, high scores, save states, etc."
                                        );
                                        ui.add_space(10.0);
                                        
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Configuration files directory
                                                if Self::render_optional_path_field_static(ui, "Configuration Files (cfg):", &mut self.temp_config.cfg_path, "/path/to/cfg") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // NVRAM directory
                                                if Self::render_optional_path_field_static(ui, "NVRAM:", &mut self.temp_config.nvram_path, "/path/to/nvram") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Input configuration directory
                                                if Self::render_optional_path_field_static(ui, "Input Configuration (input):", &mut self.temp_config.input_path, "/path/to/input") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Save state directory
                                                if Self::render_optional_path_field_static(ui, "Save States (state):", &mut self.temp_config.state_path, "/path/to/state") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Hard disk diff directory
                                                if Self::render_optional_path_field_static(ui, "Hard Disk Diffs (diff):", &mut self.temp_config.diff_path, "/path/to/diff") {
                                                    self.changes_made = true;
                                                }
                                                ui.add_space(10.0);
                                                
                                                // Comment files directory
                                                if Self::render_optional_path_field_static(ui, "Comment Files (comment):", &mut self.temp_config.comment_path, "/path/to/comment") {
                                                    self.changes_made = true;
                                                }
                                            });
                                    }
                                    _ => {}
                                }
                            }
                        );
                    }
                );
                
                // Store selected tab
                ui.data_mut(|d| d.insert_temp(ui.id(), selected_tab));

                // Footer area dengan fixed position
                ui.separator();
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // OK button
                    if ui.add(
                        egui::Button::new("OK")
                            .fill(accent_primary)
                            .min_size(egui::vec2(80.0, 32.0))
                    ).clicked() {
                        close = true;
                        apply_changes = true;
                    }
                    
                    ui.add_space(8.0);
                    
                    // Cancel button
                    if ui.add(
                        egui::Button::new("Cancel")
                            .fill(bg_hover)
                            .min_size(egui::vec2(80.0, 32.0))
                    ).clicked() {
                        close = true;
                    }
                });
            });

        if close {
            *open = false;
        }

        // Apply changes if OK was clicked
        if apply_changes && self.changes_made {
            *config = self.temp_config.clone();
            return true;
        }

        false
    }

    fn render_category_item(&mut self, ui: &mut egui::Ui, label: &str, category: TabCategory, accent_color: &egui::Color32, hover_color: &egui::Color32, text_secondary: &egui::Color32, text_primary: &egui::Color32) {
        let is_selected = self.selected_tab == category;
        
        let response = ui.allocate_response(
            egui::vec2(ui.available_width() - 32.0, 48.0),
            egui::Sense::click()
        );
        
        if response.clicked() {
            self.selected_tab = category;
        }
        
        let rect = response.rect;
        let rounding = egui::CornerRadius::same(6);
        
        // Background
        let bg_color = if is_selected {
            *accent_color
        } else if response.hovered() {
            *hover_color
        } else {
            egui::Color32::TRANSPARENT
        };
        
        ui.painter().rect_filled(rect, rounding, bg_color);
        
        // Text
        let text_color = if is_selected {
            egui::Color32::WHITE
        } else if response.hovered() {
            *text_primary
        } else {
            *text_secondary
        };
        
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            text_color,
        );
    }

    fn render_mame_paths_content(&mut self, ui: &mut egui::Ui, accent_color: &egui::Color32, success_color: &egui::Color32, error_color: &egui::Color32, bg_color: &egui::Color32, border_color: &egui::Color32, text_secondary: &egui::Color32) {
        // MAME Executables section
        ui.heading("MAME Executables");
        ui.colored_label(*text_secondary, "These are the MAME emulator programs that will run your games");
        ui.add_space(24.0);
        
        // Clone to avoid borrow issues
        let mut executables = self.temp_config.mame_executables.clone();
        let mut to_remove = None;
        
        for (idx, exe) in executables.iter_mut().enumerate() {
            // Executable box
            egui::Frame::none()
                .fill(*bg_color)
                .stroke(egui::Stroke::new(1.0, *border_color))
                .rounding(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(16))
                .show(ui, |ui| {
                    // Name field
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.add_space(8.0);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut exe.name)
                                .desired_width(200.0)  // Set explicit width for name field
                        );
                        if response.changed() {
                            self.changes_made = true;
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Path field
                    ui.horizontal(|ui| {
                        ui.label("Path:");
                        ui.add_space(8.0);
                        
                        let path_exists = std::path::Path::new(&exe.path).exists();
                        let text_color = if path_exists {
                            ui.style().visuals.text_color()
                        } else {
                            *error_color
                        };
                        
                        ui.visuals_mut().override_text_color = Some(text_color);
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut exe.path)
                                .desired_width(350.0)  // More compact width
                        );
                        if response.changed() {
                            self.changes_made = true;
                        }
                        ui.visuals_mut().override_text_color = None;
                        
                        if ui.button("Browse...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select MAME Executable")
                                .pick_file() {
                                exe.path = path.display().to_string();
                                self.changes_made = true;
                            }
                        }
                        
                        if ui.button("Validate").clicked() {
                            match self.validate_mame_executable(&exe.path) {
                                Ok((version, game_count)) => {
                                    exe.version = version;
                                    exe.total_games = game_count;
                                    exe.working_games = game_count;
                                    self.changes_made = true;
                                }
                                Err(err) => {
                                    exe.version = format!("Error: {}", err);
                                    exe.total_games = 0;
                                    exe.working_games = 0;
                                }
                            }
                        }
                    });
                    
                    ui.add_space(8.0);
                    
                    // Version info
                    if exe.version.starts_with("Error:") {
                        ui.colored_label(*error_color, &exe.version);
                    } else {
                        ui.colored_label(*success_color, format!("Version: {} • Games: {} ({} working)", 
                            exe.version, exe.total_games, exe.working_games));
                    }
                    
                    ui.add_space(8.0);
                    
                    // Remove button
                    if ui.small_button("🗑 Remove").clicked() {
                        to_remove = Some(idx);
                        self.changes_made = true;
                    }
                });
            
            ui.add_space(8.0);
        }
        
        // Apply changes back
        self.temp_config.mame_executables = executables;
        
        if let Some(idx) = to_remove {
            self.temp_config.mame_executables.remove(idx);
        }
        
        // Add button
        ui.add_space(8.0);
        if ui.button("➕ Add MAME Executable").clicked() {
            self.temp_config.mame_executables.push(MameExecutable {
                name: "MAME".to_string(),
                path: String::new(),
                version: "Not validated".to_string(),
                total_games: 0,
                working_games: 0,
            });
            self.changes_made = true;
        }
        
        ui.add_space(32.0);
        
        // ROM Directories section
        ui.heading("ROM Directories");
        ui.colored_label(*text_secondary, "Folders containing your game ROM files");
        ui.add_space(24.0);
        
        let mut to_remove = None;
        let mut browse_clicked_idx = None;
        
        for (idx, path) in self.temp_config.rom_paths.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let mut path_str = path.display().to_string();
                let path_exists = path.exists() && path.is_dir();
                
                let text_color = if path_exists {
                    ui.style().visuals.text_color()
                } else {
                    *error_color
                };
                
                ui.visuals_mut().override_text_color = Some(text_color);
                if ui.add(egui::TextEdit::singleline(&mut path_str).desired_width(350.0)).changed() {  // More compact width
                    *path = PathBuf::from(&path_str);
                    self.changes_made = true;
                }
                ui.visuals_mut().override_text_color = None;
                
                // Show file count
                if path_exists {
                    if let Ok(entries) = std::fs::read_dir(path) {
                        let zip_count = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path().extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext.eq_ignore_ascii_case("zip"))
                                    .unwrap_or(false)
                            })
                            .count();
                        ui.colored_label(*text_secondary, format!("({} .zip files)", zip_count));
                    }
                }
                
                if ui.button("Browse...").clicked() {
                    browse_clicked_idx = Some(idx);
                }
                
                if ui.button("🗑").clicked() {
                    to_remove = Some(idx);
                    self.changes_made = true;
                }
            });
            
            ui.add_space(8.0);
        }
        
        // Handle browse button click outside the loop
        if let Some(idx) = browse_clicked_idx {
            if let Some(folder) = rfd::FileDialog::new()
                .set_title("Select ROM Directory")
                .pick_folder() {
                self.temp_config.rom_paths[idx] = folder;
                self.changes_made = true;
            }
        }
        
        if let Some(idx) = to_remove {
            self.temp_config.rom_paths.remove(idx);
        }
        
        // Add ROM Directory button
        if ui.button("➕ Add ROM Directory").clicked() {
            self.temp_config.rom_paths.push(PathBuf::new());
            self.changes_made = true;
        }
        
        // Add extra spacing to fill the content area better
        ui.add_space(40.0);
    }

    fn render_support_files_content(&mut self, ui: &mut egui::Ui, bg_color: &egui::Color32, border_color: &egui::Color32, text_secondary: &egui::Color32) {
        ui.heading("MAME Support Files");
        ui.colored_label(*text_secondary, "Configure directories for artwork, samples, and other support files");
        ui.add_space(12.0);
        
        // Support file fields - using static helper function
        if Self::render_optional_path_field_static(ui, "Artwork:", &mut self.temp_config.artwork_path, "/path/to/artwork") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Cabinets:", &mut self.temp_config.cabinet_path, "/path/to/cabinets") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Flyers:", &mut self.temp_config.flyer_path, "/path/to/flyers") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Marquees:", &mut self.temp_config.marquee_path, "/path/to/marquees") {
            self.changes_made = true;
        }
        ui.add_space(4.0);  // Reduced spacing
        
        // Sample paths
        ui.separator();
        ui.add_space(4.0);
        ui.label("Sample Directories:");
        ui.add_space(4.0);
        
        let mut sample_paths = self.temp_config.sample_paths.clone();
        let mut to_remove = None;
        
        for (idx, path) in sample_paths.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let mut path_str = path.display().to_string();
                if ui.add(egui::TextEdit::singleline(&mut path_str).desired_width(350.0)).changed() {  // More compact width
                    *path = PathBuf::from(&path_str);
                    self.changes_made = true;
                }
                
                if ui.button("Browse...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title("Select Sample Directory")
                        .pick_folder() {
                        *path = folder;
                        self.changes_made = true;
                    }
                }
                
                if ui.button("🗑").clicked() {
                    to_remove = Some(idx);
                    self.changes_made = true;
                }
            });
        }
        
        self.temp_config.sample_paths = sample_paths;
        
        if let Some(idx) = to_remove {
            self.temp_config.sample_paths.remove(idx);
        }
        
        if ui.button("➕ Add Sample Directory").clicked() {
            self.temp_config.sample_paths.push(PathBuf::new());
            self.changes_made = true;
        }
        
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Snapshots:", &mut self.temp_config.snap_path, "/path/to/snap") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Titles:", &mut self.temp_config.title_path, "/path/to/titles") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Cheats:", &mut self.temp_config.cheats_path, "/path/to/cheats") {
            self.changes_made = true;
        }
        ui.add_space(8.0);  // Reduced spacing
        
        if Self::render_optional_path_field_static(ui, "Icons:", &mut self.temp_config.icons_path, "/path/to/icons") {
            self.changes_made = true;
        }
        
        // Add extra spacing to fill the content area better
        ui.add_space(40.0);
    }

    fn render_history_files_content(&mut self, ui: &mut egui::Ui, bg_color: &egui::Color32, border_color: &egui::Color32, text_secondary: &egui::Color32) {
        ui.heading("History & Documentation Files");
        ui.colored_label(*text_secondary, "Configure paths for game history and documentation files");
        ui.add_space(16.0);
        
        if Self::render_optional_file_field_static(ui, "History.xml:", &mut self.temp_config.history_path, "/path/to/history.xml") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_file_field_static(ui, "MAMEinfo.dat:", &mut self.temp_config.mameinfo_dat_path, "/path/to/mameinfo.dat") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_file_field_static(ui, "Category.ini:", &mut self.temp_config.catver_ini_path, "/path/to/category.ini") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_file_field_static(ui, "Command.dat:", &mut self.temp_config.command_dat_path, "/path/to/command.dat") {
            self.changes_made = true;
        }
    }

    fn render_internal_folders_content(&mut self, ui: &mut egui::Ui, bg_color: &egui::Color32, border_color: &egui::Color32, text_secondary: &egui::Color32) {
        ui.heading("MAME Internal Folders");
        ui.colored_label(*text_secondary, "Configure MAME's internal working directories");
        ui.add_space(16.0);
        
        if Self::render_optional_path_field_static(ui, "Config:", &mut self.temp_config.cfg_path, "/home/user/.mame/cfg") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_path_field_static(ui, "NVRAM:", &mut self.temp_config.nvram_path, "/home/user/.mame/nvram") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_path_field_static(ui, "Input:", &mut self.temp_config.input_path, "/home/user/.mame/inp") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_path_field_static(ui, "States:", &mut self.temp_config.state_path, "/home/user/.mame/sta") {
            self.changes_made = true;
        }
        ui.add_space(16.0);  // Increased spacing
        
        if Self::render_optional_path_field_static(ui, "Diff:", &mut self.temp_config.diff_path, "/home/user/.mame/diff") {
            self.changes_made = true;
        }
    }

    // Static helper functions to avoid borrow checker issues
    fn render_optional_path_field_static(ui: &mut egui::Ui, label: &str, path: &mut Option<PathBuf>, placeholder: &str) -> bool {
        let mut changed = false;
        
        // Use a group for better visual organization
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Fixed width label for alignment
                ui.add_sized([100.0, 20.0], egui::Label::new(label));  // Compact label width
                ui.add_space(8.0);
            
            let mut path_str = path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut path_str)
                    .desired_width(350.0)  // Compact width
                    .hint_text(placeholder)
            );
            
            if response.changed() {
                if path_str.is_empty() {
                    *path = None;
                } else {
                    *path = Some(PathBuf::from(&path_str));
                }
                changed = true;
            }
            
            if ui.button("Browse...").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title(&format!("Select {} Directory", label.trim_end_matches(':')))
                    .pick_folder() {
                    *path = Some(folder);
                    changed = true;
                }
            }
            });
        });
        
        ui.add_space(4.0);  // Small space after each field group
        
        changed
    }

    fn render_optional_file_field_static(ui: &mut egui::Ui, label: &str, path: &mut Option<PathBuf>, placeholder: &str) -> bool {
        let mut changed = false;
        
        // Use a group for better visual organization
        ui.group(|ui| {
            ui.horizontal(|ui| {
                // Fixed width label for alignment
                ui.add_sized([100.0, 20.0], egui::Label::new(label));  // Compact label width
                ui.add_space(8.0);
            
            let mut path_str = path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
            
            let response = ui.add(
                egui::TextEdit::singleline(&mut path_str)
                    .desired_width(350.0)  // Compact width
                    .hint_text(placeholder)
            );
            
            if response.changed() {
                if path_str.is_empty() {
                    *path = None;
                } else {
                    *path = Some(PathBuf::from(&path_str));
                }
                changed = true;
            }
            
            if ui.button("Browse...").clicked() {
                let extension = if label.contains(".xml") {
                    Some("xml")
                } else if label.contains(".dat") {
                    Some("dat")
                } else if label.contains(".ini") {
                    Some("ini")
                } else {
                    None
                };
                
                let mut dialog = rfd::FileDialog::new()
                    .set_title(&format!("Select {} File", label.trim_end_matches(':')));
                
                if let Some(ext) = extension {
                    dialog = dialog.add_filter(&format!("{} files", ext.to_uppercase()), &[ext]);
                }
                
                if let Some(file) = dialog.pick_file() {
                    *path = Some(file);
                    changed = true;
                }
            }
            });
        });
        
        ui.add_space(4.0);  // Small space after each field group
        
        changed
    }

    fn validate_mame_executable(&self, path: &str) -> Result<(String, usize), String> {
        if !std::path::Path::new(path).exists() {
            return Err("File not found".to_string());
        }

        match Command::new(path)
            .arg("-version")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    let version_line = version_output
                        .lines()
                        .find(|line| !line.trim().is_empty())
                        .unwrap_or("Unknown version");

                    let version = version_line.trim().to_string();

                    let game_count = if version.contains("0.27") || version.contains("0.28") {
                        40000
                    } else if version.contains("0.26") {
                        39000
                    } else if version.contains("0.25") {
                        38000
                    } else {
                        35000
                    };

                    Ok((version, game_count))
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed: {}", error.lines().next().unwrap_or("Unknown error")))
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err("Permission denied - check file permissions".to_string())
                } else {
                    Err(format!("Cannot run: {}", e))
                }
            }
        }
    }

    // Inline version dari executable_list tanpa ScrollArea internal
    fn executable_list_inline(ui: &mut egui::Ui, executables: &mut Vec<MameExecutable>) -> bool {
        let mut modified = false;
        let mut to_remove = None;

        for (idx, exe) in executables.iter_mut().enumerate() {
            ui.group(|ui| {
                // Field nama
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(&mut exe.name).changed() {
                        modified = true;
                    }
                });

                // Field path dengan validasi
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    let path_exists = std::path::Path::new(&exe.path).exists();

                    let response = ui.add(egui::TextEdit::singleline(&mut exe.path)
                        .desired_width(350.0)
                        .text_color(if path_exists {
                            ui.style().visuals.text_color()
                        } else {
                            egui::Color32::RED
                        }));

                    if response.changed() {
                        modified = true;
                    }

                    // Browse button
                    if ui.button("Browse...").clicked() {
                        let file_dialog = rfd::FileDialog::new()
                            .set_title("Select MAME Executable");

                        // Konfigurasi berdasarkan OS
                        let file_dialog = if cfg!(target_os = "windows") {
                            file_dialog
                                .add_filter("Executable files", &["exe", "EXE"])
                                .add_filter("All files", &["*"])
                        } else {
                            file_dialog.add_filter("All files", &["*"])
                        };

                        if let Some(path) = file_dialog.pick_file() {
                            exe.path = path.display().to_string();
                            modified = true;
                        }
                    }

                    // Validate button
                    if ui.button("Validate").clicked() {
                        // Simulasi validasi
                        exe.version = "0.250".to_string();
                        exe.total_games = 1000;
                        exe.working_games = 950;
                        modified = true;
                    }
                });

                // Display versi dan jumlah game
                ui.horizontal(|ui| {
                    if exe.version.starts_with("Error:") {
                        ui.colored_label(egui::Color32::RED, &exe.version);
                    } else {
                        ui.label(format!("Version: {}", exe.version));
                        if exe.total_games > 0 {
                            ui.label(format!("Games: {} ({} working)",
                                             exe.total_games, exe.working_games));
                        }
                    }
                });

                // Remove button
                if ui.button("🗑 Remove").clicked() {
                    to_remove = Some(idx);
                    modified = true;
                }
            });
            
            ui.add_space(8.0);
        }

        if let Some(idx) = to_remove {
            executables.remove(idx);
        }

        // Add button
        if ui.button("➕ Add MAME Executable").clicked() {
            executables.push(MameExecutable {
                name: "New MAME".to_string(),
                path: String::new(),
                version: "Not validated".to_string(),
                total_games: 0,
                working_games: 0,
            });
            modified = true;
        }

        modified
    }

    // Inline version dari path_list tanpa ScrollArea internal
    fn path_list_inline(ui: &mut egui::Ui, paths: &mut Vec<PathBuf>, id: &str, last_directories: &std::collections::HashMap<String, PathBuf>, _updates: &mut ()) -> bool {
        let mut modified = false;
        let mut to_remove = None;

        for (idx, path) in paths.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let mut path_str = path.display().to_string();
                let path_exists = path.exists() && path.is_dir();

                let was_empty = path_str.is_empty();

                // Field path yang bisa diedit
                let response = ui.add(
                    egui::TextEdit::singleline(&mut path_str)
                        .desired_width(400.0)
                        .text_color(if path_exists {
                            ui.style().visuals.text_color()
                        } else {
                            egui::Color32::RED
                        })
                );

                if response.changed() {
                    *path = PathBuf::from(&path_str);
                    modified = true;
                }

                // Tampilkan info tentang directory
                if path_exists {
                    if let Ok(entries) = std::fs::read_dir(&**path) {
                        let rom_count = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| {
                                e.path().extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext.eq_ignore_ascii_case("zip"))
                                    .unwrap_or(false)
                            })
                            .count();
                        ui.label(format!("({} .zip files)", rom_count));
                    }
                } else if !was_empty && !path_str.is_empty() {
                    ui.colored_label(egui::Color32::RED, "(not found)");
                }

                // Browse button
                if ui.button("Browse...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new()
                        .set_title(&format!("Select {} Directory",
                                            match id {
                                                "roms" => "ROM",
                                                "samples" => "Sample",
                                                _ => "Directory"
                                            }))
                        .pick_folder() {
                        *path = folder;
                        modified = true;
                    }
                }

                // Remove button
                if ui.button("🗑").clicked() {
                    to_remove = Some(idx);
                    modified = true;
                }
            });
            
            ui.add_space(8.0);
        }

        if let Some(idx) = to_remove {
            paths.remove(idx);
        }

        // Add button
        if ui.button(format!("➕ Add {}",
            match id {
                "roms" => "ROM Directory",
                "samples" => "Sample Directory",
                _ => "Directory"
            }
        )).clicked() {
            paths.push(PathBuf::new());
            modified = true;
        }

        modified
    }
}
