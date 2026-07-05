// src/ui/dialogs/directories.rs
use crate::models::{AppConfig, MameExecutable};
use crate::ui::components::steam_ui::SteamUi;
use eframe::egui;
use std::path::{Path, PathBuf};
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
    fn add_update(&mut self, category: &str, path: &Path) {
        if let Some(parent) = path.parent() {
            self.updates
                .push((category.to_string(), parent.to_path_buf()));
        }
    }

    fn apply_to_config(&self, config: &mut AppConfig) {
        for (category, path) in &self.updates {
            config
                .last_directories
                .insert(category.clone(), path.clone());
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
    fn save_last_directory(config: &mut AppConfig, category: &str, path: &Path) {
        if let Some(parent) = path.parent() {
            config
                .last_directories
                .insert(category.to_string(), parent.to_path_buf());
        }
    }

    // Helper function to create option groups matching HTML style
    fn render_option_group(
        ui: &mut egui::Ui,
        title: Option<&str>,
        content: impl FnOnce(&mut egui::Ui),
    ) {
        SteamUi::panel(ui, |ui| {
            ui.set_width(ui.available_width());

            if let Some(title) = title {
                ui.label(SteamUi::section_title(title));
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
        content: impl FnOnce(&mut egui::Ui),
    ) {
        // Use a comfortable row height for label + description + controls
        let row_height = 72.0;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::top_down(egui::Align::LEFT),
            |ui| {
                ui.label(egui::RichText::new(name).size(14.5).strong());
                ui.label(SteamUi::subtitle(description));
                ui.add_space(8.0);
                content(ui);
            },
        );

        ui.add_space(18.0);
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

        let previous_style = (*ctx.style()).clone();
        SteamUi::apply(ctx);

        egui::Window::new("Directories & Paths")
            .default_size([980.0, 760.0])
            .min_size([860.0, 620.0])
            .frame(SteamUi::window_frame())
            .resizable(true)
            .open(open)
            .show(ctx, |ui| {
                let body_height =
                    (ui.available_height() - SteamUi::FOOTER_HEIGHT).max(460.0);

                // Tab state
                let mut selected_tab = ui.data_mut(|d| d.get_temp::<usize>(ui.id()).unwrap_or(0));

                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), body_height),
                    egui::Layout::left_to_right(egui::Align::TOP),
                    |ui| {
                        SteamUi::sidebar_column(ui, 236.0, body_height, |ui| {
                            ui.label(SteamUi::section_title("Sections"));
                            ui.add_space(10.0);

                            let categories = [
                                ("MAME Paths", 0),
                                ("Support Files", 1),
                                ("INI & DAT Files", 2),
                                ("Internal Folders", 3),
                            ];

                            for (label, idx) in categories {
                                if SteamUi::sidebar_button(ui, label, selected_tab == idx).clicked()
                                {
                                    selected_tab = idx;
                                }
                                ui.add_space(6.0);
                            }
                        });

                        ui.add_space(SteamUi::COLUMN_GAP);

                        SteamUi::content_column(ui, body_height, |ui| {
                            let scroll_height = ui.available_height();
                            match selected_tab {
                                0 => {
                                    SteamUi::page_header(
                                        ui,
                                        "MAME Paths",
                                        "Configure MAME executable and ROM directories",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        Self::render_option_group(
                                            ui,
                                            Some("MAME Executables"),
                                            |ui| {
                                                ui.label(SteamUi::muted(
                                                    "These are the MAME emulator programs that will run your games",
                                                ));
                                                ui.add_space(8.0);
                                                if Self::executable_list(
                                                    ui,
                                                    &mut config.mame_executables,
                                                    "mame_exe",
                                                ) {
                                                    changes_made = true;
                                                }
                                            },
                                        );
                                        ui.add_space(SteamUi::SECTION_GAP);
                                        Self::render_option_group(
                                            ui,
                                            Some("ROM Directories"),
                                            |ui| {
                                                ui.label(SteamUi::muted(
                                                    "Folders containing your game ROM files",
                                                ));
                                                ui.add_space(8.0);
                                                if Self::path_list(
                                                    ui,
                                                    &mut config.rom_paths,
                                                    "roms",
                                                    &last_directories_snapshot,
                                                    &mut directory_updates,
                                                ) {
                                                    changes_made = true;
                                                }
                                            },
                                        );
                                    });
                                }
                                1 => {
                                    SteamUi::page_header(
                                        ui,
                                        "Support Files",
                                        "Artwork, cheats, icons, and other MAME support files",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        if Self::optional_path_field(
                                            ui,
                                            "Artwork",
                                            "Game artwork files",
                                            &mut config.artwork_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Snap",
                                            "Game screenshots",
                                            &mut config.snap_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Cabinet",
                                            "Cabinet artwork",
                                            &mut config.cabinet_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Title",
                                            "Title screens",
                                            &mut config.title_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Flyer",
                                            "Promotional flyers",
                                            &mut config.flyer_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Marquees",
                                            "Marquee artwork",
                                            &mut config.marquee_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Cheats",
                                            "Cheat files",
                                            &mut config.cheats_path,
                                            &last_directories_snapshot,
                                            CATEGORY_SUPPORT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Icons",
                                            "Game icon files",
                                            &mut config.icons_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                    });
                                }
                                2 => {
                                    SteamUi::page_header(
                                        ui,
                                        "INI & DAT Files",
                                        "Category, history, and other MAME data files",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        ui.push_id("catver_section", |ui| {
                                            ui.label(
                                                egui::RichText::new("Category Support").size(16.0).strong(),
                                            );
                                            ui.colored_label(
                                                SteamUi::WARNING,
                                                "The catver.ini file is required to display game categories",
                                            );
                                            if Self::optional_file_field(
                                                ui,
                                                "Catver INI",
                                                "Game category information (catver.ini)",
                                                &mut config.catver_ini_path,
                                                Some(&["ini"]),
                                                &last_directories_snapshot,
                                                CATEGORY_DAT_FILES,
                                                &mut directory_updates,
                                            ) {
                                                changes_made = true;
                                            }
                                        });
                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "History",
                                            "Game history information",
                                            &mut config.history_path,
                                            Some(&["xml"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "MAME Info DAT",
                                            "Detailed game information",
                                            &mut config.mameinfo_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "High Score DAT",
                                            "High score information",
                                            &mut config.hiscore_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "Game Init DAT",
                                            "Game initialization data",
                                            &mut config.gameinit_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "Command DAT",
                                            "Game command information",
                                            &mut config.command_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                    });
                                }
                                3 => {
                                    SteamUi::page_header(
                                        ui,
                                        "Internal Folders",
                                        "MAME folders for config, saves, and other runtime data",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        ui.colored_label(
                                            SteamUi::WARNING,
                                            "These folders override MAME defaults for configuration, high scores, save states, etc.",
                                        );
                                        ui.add_space(12.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Configuration Files (cfg)",
                                            "MAME configuration files directory",
                                            &mut config.cfg_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "NVRAM",
                                            "Non-volatile RAM directory",
                                            &mut config.nvram_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Input Configuration (input)",
                                            "Input configuration files directory",
                                            &mut config.input_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Save States (state)",
                                            "Save state files directory",
                                            &mut config.state_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Hard Disk Diffs (diff)",
                                            "Hard disk diff files directory",
                                            &mut config.diff_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Comment Files (comment)",
                                            "Comment files directory",
                                            &mut config.comment_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            changes_made = true;
                                        }
                                    });
                                }
                                _ => {}
                            }
                        });
                    },
                );

                ui.data_mut(|d| d.insert_temp(ui.id(), selected_tab));

                ui.add_space(12.0);
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
                    ui.colored_label(SteamUi::WARNING,
                                     "Changes detected - games will be reloaded when you click OK");
                }
            });

        ctx.set_style(previous_style);

        if close {
            *open = false;
        }

        // Apply directory updates to config for smart memory
        directory_updates.apply_to_config(config);

        // Return apakah ada perubahan DAN user klik OK
        close && changes_made
    }

    /// Handle MAME executables - return true jika dimodifikasi
    fn executable_list(
        ui: &mut egui::Ui,
        executables: &mut Vec<MameExecutable>,
        _id: &str,
    ) -> bool {
        let mut modified = false;

        // Gunakan sebagian besar ruang yang tersedia untuk scroll area
        let scroll_height = ui.available_height() * 0.8; // Gunakan 80% dari ruang yang tersedia

        egui::ScrollArea::vertical()
            .max_height(scroll_height)
            .show(ui, |ui| {
                ui.add_space(20.0);
                let mut to_remove = None;

                for (idx, exe) in executables.iter_mut().enumerate() {
                    SteamUi::inset_panel(ui, |ui| {
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

                            let response = ui.add(
                                egui::TextEdit::singleline(&mut exe.path)
                                    .desired_width(350.0)
                                    .text_color(if path_exists {
                                        ui.style().visuals.text_color()
                                    } else {
                                        egui::Color32::RED
                                    }),
                            );

                            if response.changed() {
                                modified = true;
                            }

                            // Browse button
                            if ui.button("Browse...").clicked() {
                                let file_dialog =
                                    rfd::FileDialog::new().set_title("Select MAME Executable");

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
                                ui.colored_label(SteamUi::DANGER, &exe.version);
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("Version: {}", exe.version))
                                        .size(14.0),
                                );
                                if exe.total_games > 0 {
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Games: {} ({} working)",
                                            exe.total_games, exe.working_games
                                        ))
                                        .size(14.0),
                                    );
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
        match Command::new(path).arg("-version").output() {
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
                    Err(format!(
                        "Failed: {}",
                        error.lines().next().unwrap_or("Unknown error")
                    ))
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
    fn path_list(
        ui: &mut egui::Ui,
        paths: &mut Vec<PathBuf>,
        id: &str,
        last_directories: &std::collections::HashMap<String, PathBuf>,
        updates: &mut DirectoryUpdates,
    ) -> bool {
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
                                }),
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
                                        e.path()
                                            .extension()
                                            .and_then(|ext| ext.to_str())
                                            .map(|ext| ext.eq_ignore_ascii_case("zip"))
                                            .unwrap_or(false)
                                    })
                                    .count();
                                ui.label(
                                    egui::RichText::new(format!("({} .zip files)", rom_count))
                                        .size(14.0),
                                );
                            }
                        } else if !was_empty && !path_str.is_empty() {
                            // Gunakan path_str yang tidak di-move
                            ui.colored_label(SteamUi::DANGER, "(not found)");
                        }

                        // Browse button
                        if ui.button("Browse...").clicked() {
                            // Determine category for smart directory memory
                            let category = match id {
                                "roms" => CATEGORY_ROM,
                                "samples" => CATEGORY_SAMPLE,
                                _ => CATEGORY_ROM, // Default fallback
                            };

                            let mut dialog = rfd::FileDialog::new().set_title(format!(
                                "Select {} Directory",
                                match id {
                                    "roms" => "ROM",
                                    "samples" => "Sample",
                                    _ => "Directory",
                                }
                            ));

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
        if ui
            .button(format!(
                "➕ Add {}",
                match id {
                    "roms" => "ROM Directory",
                    "samples" => "Sample Directory",
                    _ => "Directory",
                }
            ))
            .clicked()
        {
            paths.push(PathBuf::new());
            modified = true;
        }

        modified
    }

    /// Handle optional single path field - returns true if modified
    fn optional_path_field(
        ui: &mut egui::Ui,
        label: &str,
        description: &str,
        path: &mut Option<PathBuf>,
        last_directories: &std::collections::HashMap<String, PathBuf>,
        category: &str,
        updates: &mut DirectoryUpdates,
    ) -> bool {
        let mut modified = false;

        SteamUi::inset_panel(ui, |ui| {
            ui.label(egui::RichText::new(label).size(15.0).strong());
            ui.label(SteamUi::subtitle(description));
            ui.add_space(8.0); // Increased spacing

            ui.horizontal(|ui| {
                let mut path_str = path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let path_exists = path
                    .as_ref()
                    .map(|p| p.exists() && p.is_dir())
                    .unwrap_or(false);

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
                        .text_color(text_color),
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
                    let mut dialog =
                        rfd::FileDialog::new().set_title(format!("Select {} Directory", label));

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
                        ui.colored_label(SteamUi::SUCCESS, "OK");
                    } else {
                        ui.colored_label(SteamUi::DANGER, "Not found");
                    }
                }
            });
        });

        modified
    }

    /// Handle optional single file field - returns true if modified
    fn optional_file_field(
        ui: &mut egui::Ui,
        label: &str,
        description: &str,
        path: &mut Option<PathBuf>,
        extensions: Option<&[&str]>,
        last_directories: &std::collections::HashMap<String, PathBuf>,
        category: &str,
        updates: &mut DirectoryUpdates,
    ) -> bool {
        let mut modified = false;

        SteamUi::inset_panel(ui, |ui| {
            ui.label(egui::RichText::new(label).size(15.0).strong());
            ui.label(SteamUi::subtitle(description));
            ui.add_space(8.0); // Increased spacing

            ui.horizontal(|ui| {
                let mut path_str = path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                let path_exists = path
                    .as_ref()
                    .map(|p| p.exists() && p.is_file())
                    .unwrap_or(false);

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
                        .text_color(text_color),
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
                    let mut dialog =
                        rfd::FileDialog::new().set_title(format!("Select {} File", label));

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
                    } else if let Some(existing_path) = path.as_ref()
                        && let Some(parent) = existing_path.parent()
                    {
                        dialog = dialog.set_directory(parent);
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
                        ui.colored_label(SteamUi::SUCCESS, "OK");
                    } else {
                        ui.colored_label(SteamUi::DANGER, "Not found");
                    }
                }
            });
        });

        modified
    }
}
