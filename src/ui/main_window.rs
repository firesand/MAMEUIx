use eframe::egui;
use crate::models::*;
use crate::mame::GameScanner;
use crate::rom_utils::RomLoader;
use super::{game_list::GameList, sidebar::Sidebar, dialogs::*, artwork_panel::ArtworkPanel};
use std::collections::{HashMap, VecDeque};
use std::sync::mpsc;
use std::time::Instant;

pub struct MameApp {
    pub config: AppConfig,
    pub games: Vec<Game>,
    pub game_metadata: HashMap<String, Game>,
    pub selected_filter: FilterCategory,
    pub selected_game: Option<usize>,
    pub show_directories_dialog: bool,
    pub show_preferences_dialog: bool,
    pub show_rom_info_dialog: bool,
    pub show_video_settings: bool,
    pub game_list: GameList,
    pub sidebar: Sidebar,
    pub artwork_panel: ArtworkPanel,
    pub all_manufacturers: Vec<String>,
    pub running_games: HashMap<String, (std::process::Child, Instant)>,
    pub rom_icons: HashMap<String, egui::TextureHandle>,
    pub default_icon_texture: Option<egui::TextureHandle>,
    pub icon_load_queue: VecDeque<String>,
    pub icon_info: HashMap<String, IconInfo>,
    pub last_icon_cleanup: Instant,
    pub roms_loading: bool,
    pub roms_tx: Option<mpsc::Receiver<Vec<Game>>>,
    pub expanded_parents: HashMap<String, bool>,
}

impl MameApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = crate::config::load_config().unwrap_or_default();

        let mut app = Self {
            games: vec![],
            game_metadata: HashMap::new(),
            selected_filter: FilterCategory::All,
            selected_game: None,
            config,
            show_directories_dialog: false,
            show_preferences_dialog: false,
            show_rom_info_dialog: false,
            show_video_settings: false,
            game_list: GameList::new(),
            sidebar: Sidebar::new(),
            artwork_panel: ArtworkPanel::new(),
            all_manufacturers: Vec::new(),
            running_games: HashMap::new(),
            rom_icons: HashMap::new(),
            default_icon_texture: None,
                icon_load_queue: VecDeque::new(),
                icon_info: HashMap::new(),
                last_icon_cleanup: Instant::now(),
                roms_loading: false,
                roms_tx: None,
                expanded_parents: HashMap::new(),
        };

        if !app.config.mame_executables.is_empty() && app.config.selected_mame_index < app.config.mame_executables.len() {
            app.load_mame_data();
            if !app.config.rom_dirs.is_empty() {
                app.reload_roms();
            }
        }

        app
    }

    pub fn load_mame_data(&mut self) {
        if let Some(mame) = self.config.mame_executables.get(self.config.selected_mame_index) {
            let scanner = GameScanner::new(&mame.path);
            if let Ok(games) = scanner.scan_games() {
                self.game_metadata = games.iter()
                .map(|g| (g.name.clone(), g.clone()))
                .collect();

                let mut manufacturers: Vec<String> = games.iter()
                .map(|g| g.manufacturer.clone())
                .filter(|m| !m.is_empty())
                .collect();
                manufacturers.sort();
                manufacturers.dedup();
                self.all_manufacturers = manufacturers;
            }
        }
    }

    pub fn reload_roms(&mut self) {
        let rom_dirs = self.config.rom_dirs.clone();
        let metadata = self.game_metadata.clone();
        let (tx, rx) = mpsc::channel();

        self.roms_tx = Some(rx);
        self.roms_loading = true;

        std::thread::spawn(move || {
            let loader = RomLoader::new(rom_dirs);
            let games = loader.load_roms(metadata);
            let _ = tx.send(games);
        });
    }

    pub fn save_config(&self) {
        let _ = crate::config::save_config(&self.config);
    }

    pub fn toggle_favorite(&mut self, rom_name: &str) {
        if self.config.favorite_games.contains(rom_name) {
            self.config.favorite_games.remove(rom_name);
        } else {
            self.config.favorite_games.insert(rom_name.to_string());
        }
        self.save_config();
    }

    pub fn update_game_stats(&mut self, rom_name: &str, play_time: u32) {
        let stats = self.config.game_stats.entry(rom_name.to_string())
        .or_insert_with(GameStats::default);

        stats.play_count += 1;
        stats.last_played = Some(chrono::Local::now().to_rfc3339());
        stats.total_play_time += play_time;

        self.save_config();
    }

    pub fn check_running_games(&mut self) {
        let mut finished_games = Vec::new();
        let mut still_running = HashMap::new();

        let running_games = std::mem::take(&mut self.running_games);

        for (rom_name, (mut child, start_time)) in running_games {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let play_time = start_time.elapsed().as_secs() as u32;
                    finished_games.push((rom_name, play_time));
                }
                Ok(None) => {
                    still_running.insert(rom_name, (child, start_time));
                }
                Err(_) => {}
            }
        }

        self.running_games = still_running;

        for (rom_name, play_time) in finished_games {
            self.update_game_stats(&rom_name, play_time);
        }
    }

    pub fn init_default_icon(&mut self, ctx: &egui::Context) {
        let size = self.config.icon_size as usize;
        let pixels = vec![80u8; size * size * 4];

        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [size, size],
            &pixels,
        );

        self.default_icon_texture = Some(ctx.load_texture(
            "default_icon",
            color_image,
            egui::TextureOptions::default(),
        ));
    }

    pub fn queue_icon_load(&mut self, rom_name: String) {
        if !self.rom_icons.contains_key(&rom_name)
            && !self.icon_load_queue.contains(&rom_name)
            && !self.icon_info.contains_key(&rom_name) {
                self.icon_load_queue.push_back(rom_name);
            }
    }

    pub fn process_icon_queue(&mut self, ctx: &egui::Context) {
        if !self.config.show_rom_icons {
            return;
        }

        for _ in 0..5 {
            if let Some(rom_name) = self.icon_load_queue.pop_front() {
                // Simplified icon loading - just use default for now
                if let Some(default_icon) = &self.default_icon_texture {
                    self.rom_icons.insert(rom_name.clone(), default_icon.clone());
                    self.icon_info.insert(rom_name, IconInfo {
                        loaded: true,
                        last_accessed: Instant::now(),
                    });
                }
            } else {
                break;
            }
        }
    }

    pub fn get_rom_icon(&mut self, rom_name: &str) -> Option<egui::TextureHandle> {
        if let Some(info) = self.icon_info.get_mut(rom_name) {
            info.last_accessed = Instant::now();
        }

        self.rom_icons.get(rom_name).cloned()
        .or_else(|| self.default_icon_texture.clone())
    }
}

impl eframe::App for MameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.config.theme.apply(ctx);

        if self.default_icon_texture.is_none() && self.config.show_rom_icons {
            self.init_default_icon(ctx);
        }

        self.check_running_games();
        self.process_icon_queue(ctx);

        if let Some(rx) = &self.roms_tx {
            if let Ok(games) = rx.try_recv() {
                self.games = games;
                self.roms_loading = false;
                self.roms_tx = None;
            }
        }

        self.show_toolbar(ctx);

        egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(200.0)
        .show(ctx, |ui| {
            self.sidebar.show(ui, &mut self.selected_filter);
        });

        egui::SidePanel::right("artwork")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            self.artwork_panel.show(ui, &self.selected_game, &self.games, &self.config.extra_asset_dirs);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let available_width = ui.available_width();

                ui.allocate_ui_with_layout(
                    egui::vec2(available_width, ui.available_height()),
                                           egui::Layout::top_down(egui::Align::LEFT),
                                           |ui| {
                                               self.game_list.show(
                                                   ui,
                                                   &self.games,
                                                   &self.config.filter_settings,
                                                   &mut self.selected_game,
                                                   &mut self.expanded_parents,
                                                   &self.config.favorite_games,
                                                   &mut self.rom_icons,
                                                   self.config.show_rom_icons,
                                                   self.config.icon_size,
                                               );
                                           },
                );
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.label(format!("{} games", self.games.len()));
        });

        if self.show_directories_dialog {
            DirectoriesDialog::show(ctx, &mut self.config, &mut self.show_directories_dialog);
        }

        if self.show_preferences_dialog {
            PreferencesDialog::show(ctx, &mut self.config, &mut self.show_preferences_dialog);
        }

        if self.show_rom_info_dialog {
            if let Some(idx) = self.selected_game {
                if let Some(game) = self.games.get(idx) {
                    RomInfoDialog::show(ctx, game, &mut self.show_rom_info_dialog);
                }
            }
        }

        if self.show_video_settings {
            VideoSettingsDialog::show(ctx, &mut self.config.video_settings, &mut self.show_video_settings);
        }
    }
}

impl MameApp {
    fn show_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("MAME Manager").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Options", |ui| {
                    if ui.button("Directories").clicked() {
                        self.show_directories_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Preferences").clicked() {
                        self.show_preferences_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Video Settings").clicked() {
                        self.show_video_settings = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                    }
                });
            });

            ui.horizontal(|ui| {
                if ui.button("🎮 Play Game").clicked() {
                    if let Some(idx) = self.selected_game {
                        if let Some(game) = self.games.get(idx) {
                            if let Ok(child) = crate::mame::launch_game(&game.name, &self.config) {
                                self.running_games.insert(game.name.clone(), (child, Instant::now()));
                            }
                        }
                    }
                }
                if ui.button("ℹ Properties").clicked() {
                    self.show_rom_info_dialog = true;
                }
                if ui.button("🔄 Refresh").clicked() {
                    self.reload_roms();
                }
            });
        });
    }
}
