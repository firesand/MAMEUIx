// src/ui/dialogs/directories.rs
use eframe::egui;
use crate::models::{AppConfig, MameExecutable};
use std::path::PathBuf;
use std::process::Command;

// Smart directory memory categories
const CATEGORY_ROM: &str = "rom_directories";
const CATEGORY_SAMPLE: &str = "sample_directories";
const CATEGORY_ARTWORK: &str = "artwork_extra";
const CATEGORY_SUPPORT_FILES: &str = "support_files";
const CATEGORY_DAT_FILES: &str = "dat_files";
const CATEGORY_INTERNAL_FOLDERS: &str = "internal_folders";

// Structure to collect directory updates for smart memory
#[derive(Default)]
struct DirectoryUpdates {
    updates: Vec<(String, PathBuf)>,
}

impl DirectoryUpdates {
    fn add_update(&mut self, category: &str, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            self.updates.push((category.to_string(), parent.to_path_buf()));
        }
    }
    
    fn apply_to_config(&self, config: &mut AppConfig) {
        for (category, path) in &self.updates {
            config.last_directories.insert(category.clone(), path.clone());
        }
    }
}

/// DirectoriesDialog menangani konfigurasi berbagai file path
/// Versi ini melacak perubahan dan memberitahu main window saat reload diperlukan
pub struct DirectoriesDialog;

impl DirectoriesDialog {
    /// Get last used directory for a category, or default to current directory
    fn get_last_directory(config: &AppConfig, category: &str) -> Option<PathBuf> {
        config.last_directories.get(category).cloned()
    }
    
    /// Save last used directory for a category
    fn save_last_directory(config: &mut AppConfig, category: &str, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            config.last_directories.insert(category.to_string(), parent.to_path_buf());
        }
    }

    // Helper function to create option groups matching HTML style
    fn render_option_group(ui: &mut egui::Ui, title: Option<&str>, content: impl FnOnce(&mut egui::Ui)) {
        ui.group(|ui| {
            ui.set_width(ui.available_width());

            if let Some(title) = title {
                ui.label(egui::RichText::new(title).strong().size(16.0).color(egui::Color32::from_rgb(100, 181, 246)));
                ui.add_space(12.0);
            }

            content(ui);
        });
    }

    // Helper function to render option item matching HTML style
    fn render_option_item(
        ui: &mut egui::Ui,
        name: &str,
        description: &str,
        content: impl FnOnce(&mut egui::Ui)
    ) {
        // Use a fixed height for consistent vertical alignment
        let row_height = 50.0;
        
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Left side - option name and description (fixed width)
                let left_width = ui.available_width() * 0.6; // 60% for labels
                ui.allocate_ui_with_layout(
                    egui::vec2(left_width, ui.available_height()),
                    egui::Layout::top_down_justified(egui::Align::LEFT),
                    |ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(name).monospace().size(15.0));
                        ui.label(egui::RichText::new(description)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(160, 160, 160)));
                    }
                );

                // Add some spacing between left and right
                ui.add_space(20.0);

                // Right side - control (remaining width)
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), ui.available_height()),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        content(ui);
                    }
                );
            }
        );

        ui.add_space(16.0);
    }

    /// Menampilkan dialog konfigurasi directories utama
    /// Mengembalikan true jika ada perubahan yang memerlukan reload
    pub fn show(ctx: &egui::Context, config: &mut AppConfig, open: &mut bool) -> bool {
        let mut close = false;
        let mut changes_made = false;
        let mut directory_updates = DirectoryUpdates::default();
        
        // Take snapshot of last_directories for read operations
        let last_directories_snapshot = config.last_directories.clone();

        // Simpan state awal untuk deteksi perubahan
        let initial_mame_count = config.mame_executables.len();
        let initial_rom_count = config.rom_paths.len();
        let initial_sample_count = config.sample_paths.len();
        let initial_catver_path = config.catver_ini_path.clone();

        egui::Window::new("Directories Selection")
            .default_size([800.0, 700.0])  // Lebih tinggi untuk menampung semua opsi
            .min_size([700.0, 600.0])
            .resizable(true)
            .open(open)
            .show(ctx, |ui| {
                // Tab state
                let mut selected_tab = ui.data_mut(|d| d.get_temp::<usize>(ui.id()).unwrap_or(0));
                
                // Main container dengan horizontal layout untuk sidebar + content
                ui.horizontal(|ui| {
                    // Left sidebar
                    ui.vertical(|ui| {
                        ui.set_width(260.0); // Increased width for 4K displays
                        
                        // Style untuk sidebar background
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
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
                                egui::Vec2::new(240.0, 40.0), // Increased size
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
                            
                            // Draw text with larger font size
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                label,
                                egui::FontId::proportional(15.0), // Increased from 14.0
                                text_color,
                            );
                            
                            if response.clicked() {
                                selected_tab = idx;
                            }
                        }
                    });
                    
                    // Vertical separator
                    ui.separator();
                    
                    // Right content area
                    ui.vertical(|ui| {
                        ui.set_min_width(550.0);
                        
                        // Content berdasarkan selected tab
                        match selected_tab {
                            0 => {
                                // MAME Paths tab
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);  // Consistent left padding
                                    ui.vertical(|ui| {
                                        ui.heading(egui::RichText::new("MAME Paths").size(20.0));
                                        ui.label(egui::RichText::new("Configure MAME executable and ROM directories").size(15.0));
                                        ui.add_space(20.0);
                                        
                                        // MAME Executables section
                                        Self::render_option_group(ui, Some("MAME Executables"), |ui| {
                                            ui.label(egui::RichText::new("These are the MAME emulator programs that will run your games").size(15.0));
                                            ui.add_space(5.0);

                                            if Self::executable_list(ui, &mut config.mame_executables, "mame_exe") {
                                                changes_made = true;
                                            }
                                        });

                                        ui.add_space(20.0);

                                        // ROM Paths section
                                        Self::render_option_group(ui, Some("ROM Directories"), |ui| {
                                            ui.label(egui::RichText::new("Folders containing your game ROM files").size(15.0));
                                            ui.add_space(5.0);

                                            if Self::path_list(ui, &mut config.rom_paths, "roms", &last_directories_snapshot, &mut directory_updates) {
                                                changes_made = true;
                                            }
                                        });
                                    });
                                });
                            }
                            1 => {
                                // MAME Support Files tab
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);  // Consistent left padding
                                    ui.vertical(|ui| {
                                        ui.heading(egui::RichText::new("MAME Support Files").size(20.0));
                                        ui.label(egui::RichText::new("Configure paths for MAME support files:").size(15.0));
                                        ui.add_space(20.0);
                                        
                                        let scroll_height = ui.available_height() - 50.0;
                                        
                                        egui::ScrollArea::vertical()
                                            .max_height(scroll_height)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Artwork path
                                                if Self::optional_path_field(ui, "Artwork", "Game artwork files", &mut config.artwork_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Snap path
                                                if Self::optional_path_field(ui, "Snap", "Game screenshots", &mut config.snap_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Cabinet path
                                                if Self::optional_path_field(ui, "Cabinet", "Cabinet artwork", &mut config.cabinet_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Title path
                                                if Self::optional_path_field(ui, "Title", "Title screens", &mut config.title_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Flyer path
                                                if Self::optional_path_field(ui, "Flyer", "Promotional flyers", &mut config.flyer_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Marquee path
                                                if Self::optional_path_field(ui, "Marquees", "Marquee artwork", &mut config.marquee_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Cheats path
                                                if Self::optional_path_field(ui, "Cheats", "Cheat files", &mut config.cheats_path, &last_directories_snapshot, CATEGORY_SUPPORT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Icons path
                                                if Self::optional_path_field(ui, "Icons", "Game icon files", &mut config.icons_path, &last_directories_snapshot, CATEGORY_ARTWORK, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(30.0);
                                            });
                                    });
                                });
                            }
                            2 => {
                                // History, INI's and DAT's Files tab
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);  // Consistent left padding
                                    ui.vertical(|ui| {
                                        ui.heading(egui::RichText::new("History, INI's & DAT's Files").size(20.0));
                                        ui.label(egui::RichText::new("Configure paths for MAME history, INI and DAT files:").size(15.0));
                                        ui.add_space(20.0);
                                        
                                        let scroll_height = ui.available_height() - 50.0;
                                        
                                        egui::ScrollArea::vertical()
                                            .max_height(scroll_height)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Catver.ini path (for category support)
                                                ui.push_id("catver_section", |ui| {
                                                    ui.label(egui::RichText::new("Category Support").size(16.0).strong());
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(200, 200, 100),
                                                        "The catver.ini file is required to display game categories"
                                                    );
                                                    
                                                    if Self::optional_file_field(ui, "Catver INI", "Game category information (catver.ini)", &mut config.catver_ini_path, Some(&["ini"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                        changes_made = true;
                                                    }
                                                });
                                                
                                                ui.add_space(20.0);
                                                ui.separator();
                                                ui.add_space(10.0);
                                                
                                                // History path
                                                if Self::optional_file_field(ui, "History", "Game history information", &mut config.history_path, Some(&["xml"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // mameinfo.dat path
                                                if Self::optional_file_field(ui, "MAME Info DAT", "Detailed game information", &mut config.mameinfo_dat_path, Some(&["dat"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // hiscore.dat path
                                                if Self::optional_file_field(ui, "High Score DAT", "High score information", &mut config.hiscore_dat_path, Some(&["dat"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // gameinit.dat path
                                                if Self::optional_file_field(ui, "Game Init DAT", "Game initialization data", &mut config.gameinit_dat_path, Some(&["dat"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // command.dat path
                                                if Self::optional_file_field(ui, "Command DAT", "Game command information", &mut config.command_dat_path, Some(&["dat"]), &last_directories_snapshot, CATEGORY_DAT_FILES, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(30.0);
                                            });
                                    });
                                });
                            }
                            3 => {
                                // MAME Internal Folders tab
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);  // Consistent left padding
                                    ui.vertical(|ui| {
                                        ui.heading(egui::RichText::new("MAME Internal Folders").size(20.0));
                                        ui.label(egui::RichText::new("Configure MAME internal folders (these override MAME's default locations):").size(15.0));
                                        ui.colored_label(
                                            egui::Color32::from_rgb(200, 200, 100),
                                            "Note: These folders are used by MAME for saving configuration, high scores, save states, etc."
                                        );
                                        ui.add_space(20.0);
                                        
                                        let scroll_height = ui.available_height() - 50.0;
                                        
                                        egui::ScrollArea::vertical()
                                            .max_height(scroll_height)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                // Configuration files directory
                                                if Self::optional_path_field(ui, "Configuration Files (cfg)", "MAME configuration files directory", &mut config.cfg_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // NVRAM directory
                                                if Self::optional_path_field(ui, "NVRAM", "Non-volatile RAM directory", &mut config.nvram_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Input configuration directory
                                                if Self::optional_path_field(ui, "Input Configuration (input)", "Input configuration files directory", &mut config.input_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Save state directory
                                                if Self::optional_path_field(ui, "Save States (state)", "Save state files directory", &mut config.state_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Hard disk diff directory
                                                if Self::optional_path_field(ui, "Hard Disk Diffs (diff)", "Hard disk diff files directory", &mut config.diff_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(10.0);
                                                
                                                // Comment files directory
                                                if Self::optional_path_field(ui, "Comment Files (comment)", "Comment files directory", &mut config.comment_path, &last_directories_snapshot, CATEGORY_INTERNAL_FOLDERS, &mut directory_updates) {
                                                    changes_made = true;
                                                }
                                                
                                                ui.add_space(30.0);
                                            });
                                    });
                                });
                            }
                            _ => {}
                        }
                    });
                });
                
                // Store selected tab
                ui.data_mut(|d| d.insert_temp(ui.id(), selected_tab));

                ui.separator();

                // Dialog buttons at bottom
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("OK").clicked() {
                        close = true;
                        // Cek jika jumlah berubah atau catver.ini path berubah
                        if config.mame_executables.len() != initial_mame_count ||
                            config.rom_paths.len() != initial_rom_count ||
                            config.sample_paths.len() != initial_sample_count ||
                            config.catver_ini_path != initial_catver_path {
                                changes_made = true;
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close = true;
                        // Jangan simpan perubahan saat cancel
                        changes_made = false;
                    }
                });

                // Tampilkan catatan jika ada perubahan
                if changes_made {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW,
                                     "ℹ Changes detected - games will be reloaded when you click OK");
                }
            });

        if close {
            *open = false;
        }

        // Apply directory updates to config for smart memory
        directory_updates.apply_to_config(config);

        // Return apakah ada perubahan DAN user klik OK
        close && changes_made
    }

    /// Handle MAME executables - return true jika dimodifikasi
    fn executable_list(ui: &mut egui::Ui, executables: &mut Vec<MameExecutable>, _id: &str) -> bool {
        let mut modified = false;

        // Gunakan sebagian besar ruang yang tersedia untuk scroll area
        let scroll_height = ui.available_height() * 0.8; // Gunakan 80% dari ruang yang tersedia
        
        egui::ScrollArea::vertical()
        .max_height(scroll_height)
        .show(ui, |ui| {
            ui.add_space(20.0);
            let mut to_remove = None;

            for (idx, exe) in executables.iter_mut().enumerate() {
                ui.group(|ui| {
                    // Field nama
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Name:").size(15.0));
                        if ui.text_edit_singleline(&mut exe.name).changed() {
                            modified = true;
                        }
                    });

                    // Field path dengan validasi
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Path:").size(15.0));
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

                            // Set starting directory
                            let file_dialog = if !exe.path.is_empty() {
                                if let Some(parent) = std::path::Path::new(&exe.path).parent() {
                                    file_dialog.set_directory(parent)
                                } else {
                                    file_dialog
                                }
                            } else if cfg!(target_os = "linux") {
                                file_dialog.set_directory("/usr/bin")
                            } else {
                                file_dialog
                            };

                            if let Some(path) = file_dialog.pick_file() {
                                exe.path = path.display().to_string();
                                modified = true;
                            }
                        }

                        // Validate button
                        if ui.button("Validate").clicked() {
                            ui.ctx().request_repaint();
                            match Self::validate_mame_executable(&exe.path) {
                                Ok((version, game_count)) => {
                                    exe.version = version;
                                    exe.total_games = game_count;
                                    exe.working_games = game_count; // Perkiraan
                                    modified = true;
                                }
                                Err(err) => {
                                    exe.version = format!("Error: {}", err);
                                    exe.total_games = 0;
                                    exe.working_games = 0;
                                    modified = true;
                                }
                            }
                        }
                    });

                    // Display versi dan jumlah game
                    ui.horizontal(|ui| {
                        if exe.version.starts_with("Error:") {
                            ui.colored_label(egui::Color32::RED, &exe.version);
                        } else {
                            ui.label(egui::RichText::new(format!("Version: {}", exe.version)).size(14.0));
                            if exe.total_games > 0 {
                                ui.label(egui::RichText::new(format!("Games: {} ({} working)",
                                                 exe.total_games, exe.working_games)).size(14.0));
                            }
                        }
                    });

                    // Remove button
                    if ui.button("🗑 Remove").clicked() {
                        to_remove = Some(idx);
                        modified = true;
                    }
                });
            }

            if let Some(idx) = to_remove {
                executables.remove(idx);
            }
            
            ui.add_space(30.0);
        });

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

    /// Validasi MAME executable - versi lebih permisif
    fn validate_mame_executable(path: &str) -> Result<(String, usize), String> {
        // Cek file exists
        if !std::path::Path::new(path).exists() {
            return Err("File not found".to_string());
        }

        // Coba jalankan dengan -version
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

                    // Sangat permisif - terima apapun yang respond ke -version
                    let version = version_line.trim().to_string();

                    // Estimasi jumlah game berdasarkan versi
                    let game_count = if version.contains("0.27") || version.contains("0.28") {
                        40000
                    } else if version.contains("0.26") {
                        39000
                    } else if version.contains("0.25") {
                        38000
                    } else {
                        35000 // Estimasi konservatif
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

    /// Handle path lists - return true jika dimodifikasi
    /// PERBAIKAN: Menghindari move error dengan careful borrowing
    fn path_list(ui: &mut egui::Ui, paths: &mut Vec<PathBuf>, id: &str, last_directories: &std::collections::HashMap<String, PathBuf>, updates: &mut DirectoryUpdates) -> bool {
        let mut modified = false;
        let scroll_id = format!("path_scroll_{}", id);

        // Gunakan sebagian besar ruang yang tersedia untuk scroll area
        let scroll_height = ui.available_height() * 0.8; // Gunakan 80% dari ruang yang tersedia
        
        egui::ScrollArea::vertical()
        .id_salt(scroll_id)
        .max_height(scroll_height)
        .show(ui, |ui| {
            ui.add_space(20.0);
            let mut to_remove = None;

            for (idx, path) in paths.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    let mut path_str = path.display().to_string();
                    let path_exists = path.exists() && path.is_dir();

                    // Simpan apakah path kosong sebelum edit
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
                        // Update path dari string yang diedit
                        *path = PathBuf::from(&path_str); // Gunakan reference untuk hindari move
                        modified = true;
                    }

                    // Tampilkan info tentang directory
                    // PERBAIKAN: Gunakan reference ke path, bukan move
                    if path_exists {
                        // Gunakan &**path untuk mendapat &Path dari &mut PathBuf
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
                            ui.label(egui::RichText::new(format!("({} .zip files)", rom_count)).size(14.0));
                        }
                    } else if !was_empty && !path_str.is_empty() {
                        // Gunakan path_str yang tidak di-move
                        ui.colored_label(egui::Color32::RED, "(not found)");
                    }

                    // Browse button
                    if ui.button("Browse...").clicked() {
                        // Determine category for smart directory memory
                        let category = match id {
                            "roms" => CATEGORY_ROM,
                            "samples" => CATEGORY_SAMPLE,
                            _ => CATEGORY_ROM, // Default fallback
                        };
                        
                        let mut dialog = rfd::FileDialog::new()
                            .set_title(&format!("Select {} Directory",
                                                match id {
                                                    "roms" => "ROM",
                                                    "samples" => "Sample",
                                                    _ => "Directory"
                                                }));
                        
                        // Set starting directory based on smart memory
                        if let Some(last_dir) = last_directories.get(category) {
                            dialog = dialog.set_directory(last_dir);
                        }
                        
                        if let Some(folder) = dialog.pick_folder() {
                            // Save directory for smart memory
                            updates.add_update(category, &folder);
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
            }

            if let Some(idx) = to_remove {
                paths.remove(idx);
            }
            
            ui.add_space(30.0);
        });

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
    
    /// Handle optional single path field - returns true if modified
    fn optional_path_field(ui: &mut egui::Ui, label: &str, description: &str, path: &mut Option<PathBuf>, last_directories: &std::collections::HashMap<String, PathBuf>, category: &str, updates: &mut DirectoryUpdates) -> bool {
        let mut modified = false;
        
        ui.group(|ui| {
            ui.label(egui::RichText::new(label).size(15.0));
            ui.label(egui::RichText::new(description).size(14.0));
            ui.add_space(8.0); // Increased spacing
            
            ui.horizontal(|ui| {
                let mut path_str = path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
                let path_exists = path.as_ref().map(|p| p.exists() && p.is_dir()).unwrap_or(false);
                
                // Determine text color before creating the TextEdit
                let text_color = if path_str.is_empty() || path_exists {
                    ui.style().visuals.text_color()
                } else {
                    egui::Color32::RED
                };
                
                // Field path yang bisa diedit
                let response = ui.add(
                    egui::TextEdit::singleline(&mut path_str)
                        .desired_width(450.0)
                        .text_color(text_color)
                );
                
                if response.changed() {
                    if path_str.is_empty() {
                        *path = None;
                    } else {
                        *path = Some(PathBuf::from(&path_str));
                    }
                    modified = true;
                }
                
                // Browse button
                if ui.button("Browse...").clicked() {
                    let mut dialog = rfd::FileDialog::new()
                        .set_title(&format!("Select {} Directory", label));
                    
                    // Set starting directory based on smart memory
                    if let Some(last_dir) = last_directories.get(category) {
                        dialog = dialog.set_directory(last_dir);
                    }
                    
                    if let Some(folder) = dialog.pick_folder() {
                        // Save directory for smart memory
                        updates.add_update(category, &folder);
                        *path = Some(folder);
                        modified = true;
                    }
                }
                
                // Clear button
                if path.is_some() && ui.button("Clear").clicked() {
                    *path = None;
                    modified = true;
                }
                
                // Show status
                if !path_str.is_empty() {
                    if path_exists {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Not found");
                    }
                }
            });
        });
        
        modified
    }
    
    /// Handle optional single file field - returns true if modified
    fn optional_file_field(ui: &mut egui::Ui, label: &str, description: &str, path: &mut Option<PathBuf>, extensions: Option<&[&str]>, last_directories: &std::collections::HashMap<String, PathBuf>, category: &str, updates: &mut DirectoryUpdates) -> bool {
        let mut modified = false;
        
        ui.group(|ui| {
            ui.label(egui::RichText::new(label).size(15.0));
            ui.label(egui::RichText::new(description).size(14.0));
            ui.add_space(8.0); // Increased spacing
            
            ui.horizontal(|ui| {
                let mut path_str = path.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
                let path_exists = path.as_ref().map(|p| p.exists() && p.is_file()).unwrap_or(false);
                
                // Determine text color before creating the TextEdit
                let text_color = if path_str.is_empty() || path_exists {
                    ui.style().visuals.text_color()
                } else {
                    egui::Color32::RED
                };
                
                // Field path yang bisa diedit
                let response = ui.add(
                    egui::TextEdit::singleline(&mut path_str)
                        .desired_width(450.0)
                        .text_color(text_color)
                );
                
                if response.changed() {
                    if path_str.is_empty() {
                        *path = None;
                    } else {
                        *path = Some(PathBuf::from(&path_str));
                    }
                    modified = true;
                }
                
                // Browse button
                if ui.button("Browse...").clicked() {
                    let mut dialog = rfd::FileDialog::new()
                        .set_title(&format!("Select {} File", label));
                    
                    // Add file filters if extensions are provided
                    if let Some(exts) = extensions {
                        let filter_name = if exts.len() == 1 {
                            format!("{} files", exts[0].to_uppercase())
                        } else {
                            "Supported files".to_string()
                        };
                        dialog = dialog.add_filter(&filter_name, exts);
                        dialog = dialog.add_filter("All files", &["*"]);
                    }
                    
                    // Set starting directory based on smart memory or existing path
                    if let Some(last_dir) = last_directories.get(category) {
                        dialog = dialog.set_directory(last_dir);
                    } else if let Some(existing_path) = path.as_ref() {
                        if let Some(parent) = existing_path.parent() {
                            dialog = dialog.set_directory(parent);
                        }
                    }
                    
                    if let Some(file) = dialog.pick_file() {
                        // Save directory for smart memory
                        updates.add_update(category, &file);
                        *path = Some(file);
                        modified = true;
                    }
                }
                
                // Clear button
                if path.is_some() && ui.button("Clear").clicked() {
                    *path = None;
                    modified = true;
                }
                
                // Show status
                if !path_str.is_empty() {
                    if path_exists {
                        ui.colored_label(egui::Color32::GREEN, "✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "✗ Not found");
                    }
                }
            });
        });
        
        modified
    }
}
