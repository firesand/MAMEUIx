use eframe::egui;
use crate::models::config::{MameExecutable, AppConfig};
use std::path::Path;
use std::process::Command;

pub struct MameFinderDialog;

#[derive(Clone)]
pub struct FoundMame {
    pub path: String,
    pub version: String,
    pub game_count: usize,
}

impl MameFinderDialog {
    /// Search for MAME executables in standard locations
    pub fn find_mame_executables() -> Vec<FoundMame> {
        let mut found_executables = Vec::new();
        
        // Standard search paths for MAME
        let home = std::env::var("HOME").unwrap_or_default();
        let home_local_bin = format!("{}/.local/bin", home);
        let home_bin = format!("{}/bin", home);
        let search_paths = vec![
            "/usr/bin",
            "/usr/local/bin",
            "/opt/mame/bin",
            "/snap/bin",
            "/flatpak/bin",
            &home_local_bin,
            &home_bin,
        ];
        
        // Common MAME executable names
        let mame_patterns = vec![
            "mame",
            "mame64",
            "advmame",
            "sdlmame",
        ];
        
        for dir in search_paths {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            // Check if filename matches any pattern
                            let is_mame = mame_patterns.iter().any(|pattern| {
                                if *pattern == "mame" {
                                    // Handle versioned MAME like mame0279, mame0280
                                    filename == *pattern || 
                                    (filename.starts_with("mame") && 
                                     filename.len() > 4 && 
                                     filename[4..].chars().all(|c| c.is_digit(10)))
                                } else {
                                    filename == *pattern
                                }
                            });
                            
                            if is_mame {
                                // Verify it's actually MAME by running -version
                                if let Ok((version, game_count)) = Self::validate_mame_executable(path.to_str().unwrap()) {
                                    found_executables.push(FoundMame {
                                        path: path.to_string_lossy().to_string(),
                                        version,
                                        game_count,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Remove duplicates based on path
        found_executables.sort_by(|a, b| a.path.cmp(&b.path));
        found_executables.dedup_by(|a, b| a.path == b.path);
        
        found_executables
    }
    
    /// Validate a MAME executable and get its version
    fn validate_mame_executable(path: &str) -> Result<(String, usize), String> {
        if !Path::new(path).exists() {
            return Err("File not found".to_string());
        }
        
        let output = Command::new(path)
            .arg("-version")
            .output()
            .map_err(|e| format!("Cannot run: {}", e))?;
        
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            let version_line = version_output
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("Unknown version");
            
            let version = version_line.trim().to_string();
            
            // Estimate game count based on version
            let game_count = if version.contains("0.27") || version.contains("0.28") {
                48000
            } else if version.contains("0.26") {
                45000
            } else if version.contains("0.25") {
                42000
            } else {
                40000
            };
            
            Ok((version, game_count))
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed: {}", error.lines().next().unwrap_or("Unknown error")))
        }
    }
    
    /// Show dialog for selecting from multiple found MAME executables
    pub fn show_selection_dialog(
        ctx: &egui::Context,
        found_mames: &[FoundMame],
        config: &mut AppConfig,
        open: &mut bool,
    ) -> bool {
        let mut selected_idx = ctx.data_mut(|d| 
            d.get_temp::<Option<usize>>(egui::Id::new("selected_mame_idx"))
                .unwrap_or(Some(0))
        );
        
        let mut made_selection = false;
        let mut should_close = false;
        
        egui::Window::new("MAME Executable Found")
            .default_size([600.0, 400.0])
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("🎮 MAME Executables Detected");
                ui.separator();
                
                if found_mames.len() == 1 {
                    ui.label("Found 1 MAME executable:");
                } else {
                    ui.label(format!("Found {} MAME executables. Please select one:", found_mames.len()));
                }
                
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for (idx, mame) in found_mames.iter().enumerate() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let is_selected = selected_idx == Some(idx);
                                    if ui.radio(is_selected, "").clicked() {
                                        selected_idx = Some(idx);
                                        ctx.data_mut(|d| d.insert_temp(egui::Id::new("selected_mame_idx"), selected_idx));
                                    }
                                    
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&mame.path).strong());
                                        ui.label(&mame.version);
                                        ui.label(format!("Approximately {} games", mame.game_count));
                                    });
                                });
                            });
                            ui.add_space(5.0);
                        }
                    });
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Use Selected").clicked() {
                        if let Some(idx) = selected_idx {
                            if let Some(selected_mame) = found_mames.get(idx) {
                                // Add to config
                                config.mame_executables.push(MameExecutable {
                                    name: format!("MAME {}", if found_mames.len() > 1 { 
                                        format!("#{}", idx + 1) 
                                    } else { 
                                        "".to_string() 
                                    }).trim().to_string(),
                                    path: selected_mame.path.clone(),
                                    version: selected_mame.version.clone(),
                                    total_games: selected_mame.game_count,
                                    working_games: (selected_mame.game_count as f32 * 0.8) as usize,
                                });
                                config.selected_mame_index = 0;
                                made_selection = true;
                                should_close = true;
                            }
                        }
                    }
                    
                    if ui.button("Browse Manually...").clicked() {
                        should_close = true;
                        // This will trigger manual selection
                        made_selection = false;
                    }
                    
                    if found_mames.len() > 1 && ui.button("Add All").clicked() {
                        for (idx, mame) in found_mames.iter().enumerate() {
                            config.mame_executables.push(MameExecutable {
                                name: format!("MAME #{}", idx + 1),
                                path: mame.path.clone(),
                                version: mame.version.clone(),
                                total_games: mame.game_count,
                                working_games: (mame.game_count as f32 * 0.8) as usize,
                            });
                        }
                        config.selected_mame_index = 0;
                        made_selection = true;
                        should_close = true;
                    }
                });
            });
        
        if should_close {
            *open = false;
        }
        
        made_selection
    }
    
    /// Show dialog for manual MAME selection when none found
    pub fn show_manual_selection_dialog(
        ctx: &egui::Context,
        config: &mut AppConfig,
        open: &mut bool,
    ) -> bool {
        let mut made_selection = false;
        let mut path_buffer = ctx.data_mut(|d| 
            d.get_temp::<String>(egui::Id::new("manual_mame_path"))
                .unwrap_or_default()
        );
        let mut should_close = false;
        
        egui::Window::new("MAME Executable Not Found")
            .default_size([500.0, 300.0])
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("🔍 MAME Executable Not Found");
                ui.separator();
                
                ui.label("No MAME executable was found in the standard locations.");
                ui.label("Please manually select your MAME executable:");
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Path:");
                    let path_exists = Path::new(&path_buffer).exists();
                    let is_empty = path_buffer.is_empty();
                    ui.add(
                        egui::TextEdit::singleline(&mut path_buffer)
                            .desired_width(300.0)
                            .text_color(if path_exists {
                                ui.style().visuals.text_color()
                            } else if is_empty {
                                ui.style().visuals.text_color()
                            } else {
                                egui::Color32::RED
                            })
                    );
                    
                    if ui.button("Browse...").clicked() {
                        let mut dialog = rfd::FileDialog::new()
                            .set_title("Select MAME Executable");
                        
                        if cfg!(target_os = "linux") {
                            dialog = dialog.set_directory("/usr/bin");
                        }
                        
                        if let Some(path) = dialog.pick_file() {
                            path_buffer = path.display().to_string();
                        }
                    }
                });
                
                if !path_buffer.is_empty() && Path::new(&path_buffer).exists() {
                    ui.add_space(10.0);
                    match Self::validate_mame_executable(&path_buffer) {
                        Ok((version, game_count)) => {
                            ui.colored_label(egui::Color32::GREEN, format!("✓ Valid MAME: {}", version));
                            ui.label(format!("Games: {} (estimated)", game_count));
                        }
                        Err(err) => {
                            ui.colored_label(egui::Color32::RED, format!("✗ {}", err));
                        }
                    }
                }
                
                ui.add_space(20.0);
                ui.label("Common MAME locations:");
                ui.label("• /usr/bin/mame");
                ui.label("• /usr/local/bin/mame");
                ui.label("• /snap/bin/mame");
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!path_buffer.is_empty() && Path::new(&path_buffer).exists(), |ui| {
                        if ui.button("OK").clicked() {
                            if let Ok((version, game_count)) = Self::validate_mame_executable(&path_buffer) {
                                config.mame_executables.push(MameExecutable {
                                    name: "MAME".to_string(),
                                    path: path_buffer.clone(),
                                    version,
                                    total_games: game_count,
                                    working_games: (game_count as f32 * 0.8) as usize,
                                });
                                config.selected_mame_index = 0;
                                made_selection = true;
                                should_close = true;
                                
                                // Clear the buffer
                                ctx.data_mut(|d| d.insert_temp(egui::Id::new("manual_mame_path"), String::new()));
                            }
                        }
                    });
                    
                    if ui.button("Skip").clicked() {
                        should_close = true;
                    }
                });
            });
        
        if should_close {
            *open = false;
        }
        
        made_selection
    }
} 