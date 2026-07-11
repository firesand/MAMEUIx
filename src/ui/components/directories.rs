// src/ui/dialogs/directories.rs
use crate::models::{AppConfig, MameExecutable};
use crate::ui::components::mame_finder::MameFinderDialog;
use crate::ui::components::steam_ui::SteamUi;
use eframe::egui;
use std::path::{Path, PathBuf};

// Smart directory memory categories
const CATEGORY_ROM: &str = "rom_directories";
const CATEGORY_SAMPLE: &str = "sample_directories";
const CATEGORY_ARTWORK: &str = "artwork_extra";
const CATEGORY_SUPPORT_FILES: &str = "support_files";
const CATEGORY_DAT_FILES: &str = "dat_files";
const CATEGORY_INTERNAL_FOLDERS: &str = "internal_folders";
const CATEGORY_SOFTWARE_LISTS: &str = "software_lists";

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
#[derive(Default)]
pub struct DirectoriesDialog {
    original: Option<AppConfig>,
    draft: Option<AppConfig>,
    dirty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloseAction {
    Commit,
    Discard,
}

impl DirectoriesDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discard any in-progress edit session.
    pub fn reset(&mut self) {
        self.original = None;
        self.draft = None;
        self.dirty = false;
    }

    fn begin_session(&mut self, config: &AppConfig) {
        self.original = Some(config.clone());
        self.draft = Some(config.clone());
        self.dirty = false;
    }

    fn executable_paths_equal(left: &[MameExecutable], right: &[MameExecutable]) -> bool {
        left.len() == right.len()
            && left
                .iter()
                .zip(right)
                .all(|(left, right)| left.path == right.path)
    }

    fn executables_equal(left: &[MameExecutable], right: &[MameExecutable]) -> bool {
        left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                left.name == right.name
                    && left.path == right.path
                    && left.version == right.version
                    && left.total_games == right.total_games
                    && left.working_games == right.working_games
            })
    }

    /// Compare only values edited by this dialog. Smart-directory memory is
    /// persisted on OK, but does not require a game/category reload by itself.
    fn relevant_settings_changed(original: &AppConfig, draft: &AppConfig) -> bool {
        !Self::executable_paths_equal(&original.mame_executables, &draft.mame_executables)
            || original.rom_paths != draft.rom_paths
            || original.software_rom_paths != draft.software_rom_paths
            || original.hash_path != draft.hash_path
            || original.sw_path != draft.sw_path
            || original.artwork_path != draft.artwork_path
            || original.snap_path != draft.snap_path
            || original.cabinet_path != draft.cabinet_path
            || original.title_path != draft.title_path
            || original.flyer_path != draft.flyer_path
            || original.marquee_path != draft.marquee_path
            || original.cheats_path != draft.cheats_path
            || original.icons_path != draft.icons_path
            || original.catver_ini_path != draft.catver_ini_path
            || original.history_path != draft.history_path
            || original.mameinfo_dat_path != draft.mameinfo_dat_path
            || original.hiscore_dat_path != draft.hiscore_dat_path
            || original.gameinit_dat_path != draft.gameinit_dat_path
            || original.command_dat_path != draft.command_dat_path
            || original.cfg_path != draft.cfg_path
            || original.nvram_path != draft.nvram_path
            || original.input_path != draft.input_path
            || original.state_path != draft.state_path
            || original.diff_path != draft.diff_path
            || original.comment_path != draft.comment_path
    }

    fn owned_settings_changed(original: &AppConfig, draft: &AppConfig) -> bool {
        Self::relevant_settings_changed(original, draft)
            || !Self::executables_equal(&original.mame_executables, &draft.mame_executables)
            || original.last_directories != draft.last_directories
    }

    /// Commit only settings owned by this dialog so unrelated configuration
    /// changes made while it is open cannot be overwritten by an old snapshot.
    fn commit_draft(config: &mut AppConfig, draft: AppConfig) {
        config.mame_executables = draft.mame_executables;
        config.rom_paths = draft.rom_paths;
        config.software_rom_paths = draft.software_rom_paths;
        config.hash_path = draft.hash_path;
        config.sw_path = draft.sw_path;
        config.artwork_path = draft.artwork_path;
        config.snap_path = draft.snap_path;
        config.cabinet_path = draft.cabinet_path;
        config.title_path = draft.title_path;
        config.flyer_path = draft.flyer_path;
        config.marquee_path = draft.marquee_path;
        config.cheats_path = draft.cheats_path;
        config.icons_path = draft.icons_path;
        config.catver_ini_path = draft.catver_ini_path;
        config.history_path = draft.history_path;
        config.mameinfo_dat_path = draft.mameinfo_dat_path;
        config.hiscore_dat_path = draft.hiscore_dat_path;
        config.gameinit_dat_path = draft.gameinit_dat_path;
        config.command_dat_path = draft.command_dat_path;
        config.cfg_path = draft.cfg_path;
        config.nvram_path = draft.nvram_path;
        config.input_path = draft.input_path;
        config.state_path = draft.state_path;
        config.diff_path = draft.diff_path;
        config.comment_path = draft.comment_path;
        config.last_directories = draft.last_directories;
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
    pub fn show(&mut self, ctx: &egui::Context, config: &mut AppConfig, open: &mut bool) -> bool {
        if self.draft.is_none() || self.original.is_none() {
            self.begin_session(config);
        }

        let mut close_action = None;
        let mut edited_this_frame = false;
        let mut directory_updates = DirectoryUpdates::default();
        let original = self.original.as_ref().expect("session was initialized");
        let draft = self.draft.as_mut().expect("session was initialized");

        // Take snapshot of last_directories for read operations
        let last_directories_snapshot = draft.last_directories.clone();

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
                                ("Software Lists", 1),
                                ("Support Files", 2),
                                ("INI & DAT Files", 3),
                                ("Internal Folders", 4),
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
                                                    &mut draft.mame_executables,
                                                    "mame_exe",
                                                ) {
                                                    edited_this_frame = true;
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
                                                    &mut draft.rom_paths,
                                                    "roms",
                                                    &last_directories_snapshot,
                                                    &mut directory_updates,
                                                ) {
                                                    edited_this_frame = true;
                                                }
                                            },
                                        );
                                    });
                                }
                                1 => {
                                    SteamUi::page_header(
                                        ui,
                                        "Software Lists",
                                        "Configure MAME software-list definitions and media paths",
                                    );
                                    SteamUi::scroll_content(ui, scroll_height, |ui| {
                                        Self::render_option_group(
                                            ui,
                                            Some("Software-list Database"),
                                            |ui| {
                                                ui.label(SteamUi::muted(
                                                    "Folder containing MAME hash XML files such as a2600.xml, nes.xml, and msx1_cart.xml",
                                                ));
                                                ui.add_space(8.0);
                                                if Self::optional_path_field(
                                                    ui,
                                                    "Hash XML Directory",
                                                    "Software-list definition files used to build the Software Lists table",
                                                    &mut draft.hash_path,
                                                    &last_directories_snapshot,
                                                    CATEGORY_SOFTWARE_LISTS,
                                                    &mut directory_updates,
                                                ) {
                                                    edited_this_frame = true;
                                                }
                                            },
                                        );

                                        ui.add_space(SteamUi::SECTION_GAP);

                                        Self::render_option_group(
                                            ui,
                                            Some("Software-list ROMs"),
                                            |ui| {
                                                ui.label(SteamUi::muted(
                                                    "Root folders for split software-list sets, usually containing subfolders named after software lists such as a2600, nes, or apple2_flop_orig",
                                                ));
                                                ui.add_space(8.0);
                                                if Self::path_list(
                                                    ui,
                                                    &mut draft.software_rom_paths,
                                                    "software_roms",
                                                    &last_directories_snapshot,
                                                    &mut directory_updates,
                                                ) {
                                                    edited_this_frame = true;
                                                }
                                            },
                                        );

                                        ui.add_space(SteamUi::SECTION_GAP);

                                        Self::render_option_group(
                                            ui,
                                            Some("Loose Software"),
                                            |ui| {
                                                ui.label(SteamUi::muted(
                                                    "Folder containing loose software media used by MAME through -swpath",
                                                ));
                                                ui.add_space(8.0);
                                                if Self::optional_path_field(
                                                    ui,
                                                    "Software Media Directory",
                                                    "Loose cartridge, disk, cassette, or other software media files",
                                                    &mut draft.sw_path,
                                                    &last_directories_snapshot,
                                                    CATEGORY_SOFTWARE_LISTS,
                                                    &mut directory_updates,
                                                ) {
                                                    edited_this_frame = true;
                                                }
                                            },
                                        );
                                    });
                                }
                                2 => {
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
                                            &mut draft.artwork_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Snap",
                                            "Game screenshots",
                                            &mut draft.snap_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Cabinet",
                                            "Cabinet artwork",
                                            &mut draft.cabinet_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Title",
                                            "Title screens",
                                            &mut draft.title_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Flyer",
                                            "Promotional flyers",
                                            &mut draft.flyer_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Marquees",
                                            "Marquee artwork",
                                            &mut draft.marquee_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Cheats",
                                            "Cheat files",
                                            &mut draft.cheats_path,
                                            &last_directories_snapshot,
                                            CATEGORY_SUPPORT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Icons",
                                            "Game icon files",
                                            &mut draft.icons_path,
                                            &last_directories_snapshot,
                                            CATEGORY_ARTWORK,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                    });
                                }
                                3 => {
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
                                                &mut draft.catver_ini_path,
                                                Some(&["ini"]),
                                                &last_directories_snapshot,
                                                CATEGORY_DAT_FILES,
                                                &mut directory_updates,
                                            ) {
                                                edited_this_frame = true;
                                            }
                                        });
                                        ui.add_space(20.0);
                                        ui.separator();
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "History",
                                            "Game history information",
                                            &mut draft.history_path,
                                            Some(&["xml"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "MAME Info DAT",
                                            "Detailed game information",
                                            &mut draft.mameinfo_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "High Score DAT",
                                            "High score information",
                                            &mut draft.hiscore_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "Game Init DAT",
                                            "Game initialization data",
                                            &mut draft.gameinit_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_file_field(
                                            ui,
                                            "Command DAT",
                                            "Game command information",
                                            &mut draft.command_dat_path,
                                            Some(&["dat"]),
                                            &last_directories_snapshot,
                                            CATEGORY_DAT_FILES,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                    });
                                }
                                4 => {
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
                                            &mut draft.cfg_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "NVRAM",
                                            "Non-volatile RAM directory",
                                            &mut draft.nvram_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Input Configuration (input)",
                                            "Input configuration files directory",
                                            &mut draft.input_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Save States (state)",
                                            "Save state files directory",
                                            &mut draft.state_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Hard Disk Diffs (diff)",
                                            "Hard disk diff files directory",
                                            &mut draft.diff_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
                                        }
                                        ui.add_space(10.0);
                                        if Self::optional_path_field(
                                            ui,
                                            "Comment Files (comment)",
                                            "Comment files directory",
                                            &mut draft.comment_path,
                                            &last_directories_snapshot,
                                            CATEGORY_INTERNAL_FOLDERS,
                                            &mut directory_updates,
                                        ) {
                                            edited_this_frame = true;
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
                        close_action = Some(CloseAction::Commit);
                    }

                    if ui.button("Cancel").clicked() {
                        close_action = Some(CloseAction::Discard);
                    }
                });

                // Tampilkan catatan jika ada perubahan
                if Self::relevant_settings_changed(original, draft) {
                    ui.separator();
                    ui.colored_label(SteamUi::WARNING,
                                     "Changes detected - games will be reloaded when you click OK");
                } else if Self::owned_settings_changed(original, draft) || edited_this_frame {
                    ui.separator();
                    ui.colored_label(
                        SteamUi::WARNING,
                        "Changes detected - settings will be saved when you click OK",
                    );
                }
            });

        ctx.set_style(previous_style);

        directory_updates.apply_to_config(draft);
        let reload_required = Self::relevant_settings_changed(original, draft);
        let dirty = Self::owned_settings_changed(original, draft);
        self.dirty = dirty;

        if close_action.is_some() {
            *open = false;
        }

        match close_action {
            Some(CloseAction::Commit) => {
                let had_changes = self.dirty;
                let draft = self.draft.take().expect("session was initialized");
                if had_changes {
                    Self::commit_draft(config, draft);
                }
                self.original = None;
                self.dirty = false;
                reload_required
            }
            Some(CloseAction::Discard) => {
                self.reset();
                false
            }
            None if !*open => {
                // Closing through the window title-bar is equivalent to Cancel.
                self.reset();
                false
            }
            None => false,
        }
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
                                } else if cfg!(any(
                                    target_os = "linux",
                                    target_os = "freebsd",
                                    target_os = "dragonfly",
                                    target_os = "netbsd",
                                    target_os = "openbsd"
                                )) {
                                    file_dialog
                                        .set_directory(MameFinderDialog::unix_browse_directory())
                                } else {
                                    file_dialog
                                };

                                if let Some(path) = file_dialog.pick_file() {
                                    exe.path = MameFinderDialog::resolve_executable_path(
                                        &path.display().to_string(),
                                    );
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
        MameFinderDialog::validate_mame_executable(path)
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
                                "software_roms" => CATEGORY_SOFTWARE_LISTS,
                                _ => CATEGORY_ROM, // Default fallback
                            };

                            let mut dialog = rfd::FileDialog::new().set_title(format!(
                                "Select {} Directory",
                                match id {
                                    "roms" => "ROM",
                                    "samples" => "Sample",
                                    "software_roms" => "Software-list ROM",
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
                    "software_roms" => "Software-list ROM Directory",
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

#[cfg(test)]
mod tests {
    use super::*;

    fn executable(path: &str) -> MameExecutable {
        MameExecutable {
            name: "MAME".to_string(),
            path: path.to_string(),
            version: "0.280".to_string(),
            total_games: 1,
            working_games: 1,
        }
    }

    #[test]
    fn relevant_change_detection_checks_values_not_only_list_lengths() {
        let mut original = AppConfig::default();
        original.rom_paths.push(PathBuf::from("/roms/old"));
        original.mame_executables.push(executable("/usr/bin/mame"));

        let mut changed_path = original.clone();
        changed_path.rom_paths[0] = PathBuf::from("/roms/new");
        assert!(DirectoriesDialog::relevant_settings_changed(
            &original,
            &changed_path
        ));

        let mut changed_executable = original.clone();
        changed_executable.mame_executables[0].path = "/usr/local/bin/mame".to_string();
        assert!(DirectoriesDialog::relevant_settings_changed(
            &original,
            &changed_executable
        ));

        let mut display_metadata_only = original.clone();
        display_metadata_only.mame_executables[0].name = "Arcade MAME".to_string();
        display_metadata_only.mame_executables[0].version = "0.281".to_string();
        display_metadata_only.mame_executables[0].total_games = 2;
        assert!(!DirectoriesDialog::relevant_settings_changed(
            &original,
            &display_metadata_only
        ));
        assert!(DirectoriesDialog::owned_settings_changed(
            &original,
            &display_metadata_only
        ));
    }

    #[test]
    fn smart_directory_memory_alone_does_not_request_reload() {
        let original = AppConfig::default();
        let mut draft = original.clone();
        draft
            .last_directories
            .insert(CATEGORY_ROM.to_string(), PathBuf::from("/home/user/roms"));

        assert!(!DirectoriesDialog::relevant_settings_changed(
            &original, &draft
        ));
        assert!(DirectoriesDialog::owned_settings_changed(&original, &draft));
    }

    #[test]
    fn commit_updates_owned_paths_without_overwriting_unrelated_config() {
        let original = AppConfig::default();
        let mut draft = original.clone();
        draft.rom_paths.push(PathBuf::from("/games/roms"));
        draft
            .last_directories
            .insert(CATEGORY_ROM.to_string(), PathBuf::from("/games"));

        let mut live = original;
        live.selected_mame_index = 7;
        DirectoriesDialog::commit_draft(&mut live, draft);

        assert_eq!(live.rom_paths, vec![PathBuf::from("/games/roms")]);
        assert_eq!(
            live.last_directories.get(CATEGORY_ROM),
            Some(&PathBuf::from("/games"))
        );
        assert_eq!(live.selected_mame_index, 7);
    }

    #[test]
    fn reset_discards_the_persistent_draft() {
        let config = AppConfig::default();
        let mut dialog = DirectoriesDialog::new();
        dialog.begin_session(&config);
        dialog
            .draft
            .as_mut()
            .expect("draft should exist")
            .rom_paths
            .push(PathBuf::from("/discarded"));
        dialog.dirty = true;

        dialog.reset();

        assert!(dialog.original.is_none());
        assert!(dialog.draft.is_none());
        assert!(!dialog.dirty);
        assert!(config.rom_paths.is_empty());
    }
}
