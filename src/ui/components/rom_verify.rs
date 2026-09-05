use crate::models::{AppConfig, Game, RomStatus, VerificationStatus};
use crate::ui::redesign::fonts;
use eframe::egui;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

/// Global verification manager to track verification status across the application
pub struct VerificationManager {
    verification_results: Arc<Mutex<HashMap<String, VerificationResult>>>,
}

impl VerificationManager {
    pub fn new() -> Self {
        Self {
            verification_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Update verification result for a game
    pub fn update_result(&self, game_name: String, result: VerificationResult) {
        if let Ok(mut results) = self.verification_results.lock() {
            results.insert(game_name, result);
        }
    }

    /// Get verification result for a game
    pub fn get_result(&self, game_name: &str) -> Option<VerificationResult> {
        if let Ok(results) = self.verification_results.lock() {
            results.get(game_name).cloned()
        } else {
            None
        }
    }

    /// Get verification status for a game
    pub fn get_verification_status(&self, game_name: &str) -> VerificationStatus {
        if let Some(result) = self.get_result(game_name) {
            match result.status {
                VerifyStatus::Passed => VerificationStatus::Verified,
                VerifyStatus::Failed => VerificationStatus::Failed,
                VerifyStatus::Warning => VerificationStatus::Warning,
                VerifyStatus::NotFound => VerificationStatus::NotFound,
            }
        } else {
            VerificationStatus::NotVerified
        }
    }

    /// Update game verification status
    pub fn update_game_status(&self, game: &mut Game) {
        let status = self.get_verification_status(&game.name);
        game.update_verification_status(status);
    }

    /// Clear all verification results
    pub fn clear_results(&self) {
        if let Ok(mut results) = self.verification_results.lock() {
            results.clear();
        }
    }

    /// Get all verification results
    pub fn get_all_results(&self) -> Vec<VerificationResult> {
        if let Ok(results) = self.verification_results.lock() {
            results.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get verification statistics
    pub fn get_stats(&self) -> VerificationStats {
        let results = self.get_all_results();
        let mut stats = VerificationStats {
            total_verified: results.len(),
            passed: 0,
            failed: 0,
            warnings: 0,
            missing: 0,
            incorrect: 0,
            missing_chd: 0,
        };

        for result in results {
            match result.status {
                VerifyStatus::Passed => stats.passed += 1,
                VerifyStatus::Failed => {
                    stats.failed += 1;
                    stats.missing += result.missing_files.len();
                    stats.incorrect += result.incorrect_files.len();
                    if result.chd_status.is_some() {
                        stats.missing_chd += 1;
                    }
                }
                VerifyStatus::Warning => stats.warnings += 1,
                VerifyStatus::NotFound => stats.failed += 1,
            }
        }

        stats
    }
}

impl Default for VerificationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RomVerifyDialog {
    is_verifying: bool,
    verification_results: Vec<VerificationResult>,
    current_progress: f32,
    current_game: String,
    total_games: usize,
    verified_games: usize,
    receiver: Option<mpsc::Receiver<VerifyMessage>>,
    filter_text: String,
    show_only_issues: bool,
    // New fields for user control
    should_stop: bool,
    is_paused: bool,
    pause_sender: Option<mpsc::Sender<bool>>,
    stop_sender: Option<mpsc::Sender<bool>>,
    show_warning: bool,
    estimated_time: Option<String>,
    start_time: Option<std::time::Instant>,
    // Window state
    window_open: bool,
    // Enhanced stats tracking
    stats: VerificationStats,
    // Export options
    export_format: ExportFormat,
    // Verification manager
    verification_manager: Arc<VerificationManager>,
}

#[derive(Clone, Debug)]
pub struct VerificationResult {
    pub game_name: String,
    pub description: String,
    pub status: VerifyStatus,
    pub missing_files: Vec<String>,
    pub incorrect_files: Vec<String>,
    pub extra_files: Vec<String>,
    pub chd_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerifyStatus {
    Passed,
    Failed,
    Warning,
    NotFound,
}

#[derive(Default)]
pub struct VerificationStats {
    pub total_verified: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub missing: usize,
    pub incorrect: usize,
    pub missing_chd: usize,
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum ExportFormat {
    #[default]
    Text,
    CSV,
    HTML,
}

enum VerifyMessage {
    Progress(f32, String),
    Result(VerificationResult),
    Complete,
    Error(String),
}

impl Default for RomVerifyDialog {
    fn default() -> Self {
        Self {
            is_verifying: false,
            verification_results: Vec::new(),
            current_progress: 0.0,
            current_game: String::new(),
            total_games: 0,
            verified_games: 0,
            receiver: None,
            filter_text: String::new(),
            show_only_issues: true,
            // New fields initialization
            should_stop: false,
            is_paused: false,
            pause_sender: None,
            stop_sender: None,
            show_warning: true,
            estimated_time: None,
            start_time: None,
            // Window state
            window_open: false,
            // Enhanced stats
            stats: VerificationStats::default(),
            // Export format
            export_format: ExportFormat::default(),
            // Verification manager
            verification_manager: Arc::new(VerificationManager::new()),
        }
    }
}

impl RomVerifyDialog {
    // New method to open the window
    pub fn open(&mut self) {
        self.window_open = true;
    }

    // New method to check if window is open
    pub fn is_open(&self) -> bool {
        self.window_open
    }

    /// Get reference to verification manager
    pub fn verification_manager(&self) -> Arc<VerificationManager> {
        self.verification_manager.clone()
    }

    // Modified show method for standalone window
    pub fn show_window(&mut self, ctx: &egui::Context, config: &AppConfig, games: &[Game]) {
        // Check if we should show warning first
        if self.show_warning && !self.is_verifying && games.len() > 100 {
            let mut should_close = false;
            egui::Window::new("⚠️ ROM Verification Warning")
                .open(&mut self.window_open)
                .default_size([500.0, 200.0])
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(egui::Color32::YELLOW, "⚠️ Time Warning");
                        ui.add_space(10.0);
                        ui.label(format!("ROM verification for {} games may take a long time (10-30 minutes for large collections).", games.len()));
                        ui.label("The process can be paused or stopped at any time.");
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.button("I understand, proceed").clicked() {
                                self.show_warning = false;
                            }
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    });
                });
            if should_close {
                self.window_open = false;
            }
            return;
        }

        // Create a separate window that's not modal
        let mut window_open = self.window_open;
        egui::Window::new("🔍 ROM Verification - CLRMamePro Lite")
            .open(&mut window_open)
            .default_size([900.0, 600.0])
            .min_size([700.0, 450.0])
            .max_size([1200.0, 900.0])
            .resizable(true)
            .collapsible(false)
            .drag_to_scroll(false)
            .show(ctx, |ui| {
                self.show_content(ui, config, games);
            });
        self.window_open = window_open;
    }

    // Extract the content into a separate method
    fn show_content(&mut self, ui: &mut egui::Ui, config: &AppConfig, games: &[Game]) {
        // Process any pending messages
        if let Some(rx) = &self.receiver {
            let mut messages = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }

            for msg in messages {
                match msg {
                    VerifyMessage::Progress(progress, game) => {
                        self.current_progress = progress;
                        self.current_game = game;
                        self.verified_games += 1;

                        // Calculate estimated time remaining
                        if let Some(start_time) = self.start_time {
                            let elapsed = start_time.elapsed();
                            if self.verified_games > 0 {
                                let avg_time_per_game =
                                    elapsed.as_secs_f32() / self.verified_games as f32;
                                let remaining_games = self.total_games - self.verified_games;
                                let estimated_remaining =
                                    avg_time_per_game * remaining_games as f32;

                                if estimated_remaining > 60.0 {
                                    let minutes = (estimated_remaining / 60.0) as u32;
                                    let seconds = (estimated_remaining % 60.0) as u32;
                                    self.estimated_time =
                                        Some(format!("{}m {}s", minutes, seconds));
                                } else {
                                    self.estimated_time =
                                        Some(format!("{:.0}s", estimated_remaining));
                                }
                            }
                        }
                    }
                    VerifyMessage::Result(result) => {
                        self.verification_results.push(result.clone());
                        // Update verification manager
                        self.verification_manager
                            .update_result(result.game_name.clone(), result);
                        self.update_stats();
                    }
                    VerifyMessage::Complete => {
                        self.is_verifying = false;
                        self.receiver = None;
                    }
                    VerifyMessage::Error(err) => {
                        eprintln!("Verification error: {}", err);
                        self.is_verifying = false;
                        self.receiver = None;
                    }
                }
            }
        }

        // Header with close button
        ui.horizontal(|ui| {
            ui.heading("🔍 ROM Verification - CLRMamePro Lite");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("×").clicked() {
                    self.window_open = false;
                }
            });
        });

        ui.separator();

        // Enhanced stats display
        self.show_stats_panel(ui);

        ui.separator();

        // Control buttons in a more compact layout
        ui.vertical(|ui| {
            // First row: Main action buttons
            ui.horizontal_wrapped(|ui| {
                if !self.is_verifying {
                    if ui.button("🔍 Verify All ROMs").clicked() {
                        self.start_verification(config, games, None);
                    }

                    if ui.button("Verify Available Only").clicked() {
                        let available_games: Vec<_> = games
                            .iter()
                            .filter(|g| matches!(g.status, RomStatus::Available))
                            .cloned()
                            .collect();
                        self.start_verification(config, &available_games, None);
                    }

                    if ui.button("Clear Results").clicked() {
                        self.verification_results.clear();
                        self.verified_games = 0;
                        self.stats = VerificationStats::default();
                        self.verification_manager.clear_results();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("⏳ Verifying..."));
                }
            });

            // Second row: Options and bulk actions
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_only_issues, "Show only issues");
                ui.add_space(10.0);
                ui.label(format!("Total: {}", games.len()));

                // Bulk actions
                if !self.verification_results.is_empty() {
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    if ui.button("🌐 Find Missing ROMs (No-Intro)").clicked() {
                        self.open_no_intro_search();
                    }

                    ui.add_space(10.0);

                    ui.label("Export:");
                    ui.radio_value(&mut self.export_format, ExportFormat::Text, "Text");
                    ui.radio_value(&mut self.export_format, ExportFormat::CSV, "CSV");
                    ui.radio_value(&mut self.export_format, ExportFormat::HTML, "HTML");

                    if ui.button("📄 Export Report").clicked() {
                        self.export_results();
                    }
                }
            });
        });

        ui.separator();

        // Enhanced progress bar and controls
        if self.is_verifying {
            self.show_progress_panel(ui);
            ui.separator();
        }

        // Filter controls
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(
                egui::TextEdit::singleline(&mut self.filter_text)
                    .desired_width(ui.available_width() - 120.0),
            );
            if ui.button("Clear").clicked() {
                self.filter_text.clear();
            }
        });

        ui.separator();

        // Results display with color coding
        self.show_results_panel(ui);
    }

    fn show_stats_panel(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("📊 Verification Statistics");

            if self.verification_results.is_empty() {
                ui.label("No verification results yet. Click 'Verify All ROMs' to start.");
                return;
            }

            ui.horizontal_wrapped(|ui| {
                // Main stats
                ui.vertical(|ui| {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("✅ Verified: {}", self.stats.passed),
                    );
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("❌ Failed: {}", self.stats.failed),
                    );
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠️ Warnings: {}", self.stats.warnings),
                    );
                });

                ui.add_space(20.0);

                // Detailed stats
                ui.vertical(|ui| {
                    ui.label(format!("📁 Missing Files: {}", self.stats.missing));
                    ui.label(format!("🔧 Incorrect Files: {}", self.stats.incorrect));
                    ui.label(format!("💿 Missing CHD: {}", self.stats.missing_chd));
                });

                ui.add_space(20.0);

                // Progress stats
                ui.vertical(|ui| {
                    let progress_percent =
                        self.verified_games.saturating_mul(100) / self.total_games.max(1);
                    ui.label(format!(
                        "📈 Progress: {} / {} ({}%)",
                        self.verified_games, self.total_games, progress_percent
                    ));

                    if let Some(eta) = &self.estimated_time {
                        ui.label(format!("⏱️ ETA: {}", eta));
                    }
                });
            });
        });
    }

    fn show_progress_panel(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.heading("⏳ Verification Progress");

            // Enhanced progress bar with stats
            ui.horizontal(|ui| {
                ui.add(
                    egui::ProgressBar::new(self.current_progress)
                        .show_percentage()
                        .animate(true)
                        .desired_width(ui.available_width() - 200.0),
                );

                if let Some(estimated) = &self.estimated_time {
                    ui.label(format!("ETA: {}", estimated));
                }
            });

            // Current status
            ui.horizontal(|ui| {
                ui.label(format!("Current: {}", self.current_game));
                ui.add_space(10.0);
                ui.label(format!(
                    "Progress: {} / {}",
                    self.verified_games, self.total_games
                ));
            });

            // Live stats during verification
            if !self.verification_results.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(
                        egui::Color32::GREEN,
                        format!("✅ {} passed", self.stats.passed),
                    );
                    ui.colored_label(
                        egui::Color32::RED,
                        format!("❌ {} failed", self.stats.failed),
                    );
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠️ {} warnings", self.stats.warnings),
                    );
                });
            }

            // Control buttons
            ui.horizontal_wrapped(|ui| {
                if self.is_paused {
                    if ui
                        .add(egui::Button::new("▶ Resume").fill(egui::Color32::GREEN))
                        .clicked()
                    {
                        if let Some(sender) = &self.pause_sender {
                            let _ = sender.send(false);
                        }
                        self.is_paused = false;
                    }
                } else {
                    if ui
                        .add(egui::Button::new("⏸ Pause").fill(egui::Color32::YELLOW))
                        .clicked()
                    {
                        if let Some(sender) = &self.pause_sender {
                            let _ = sender.send(true);
                        }
                        self.is_paused = true;
                    }
                }

                if ui
                    .add(egui::Button::new("⏹ Stop").fill(egui::Color32::RED))
                    .clicked()
                {
                    if let Some(sender) = &self.stop_sender {
                        let _ = sender.send(true);
                    }
                    self.should_stop = true;
                    self.is_verifying = false;
                }

                // Show pause status
                if self.is_paused {
                    ui.colored_label(egui::Color32::YELLOW, "⏸ PAUSED");
                }
            });
        });
    }

    fn show_results_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Verification Results");

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let filtered_results: Vec<_> = self
                    .verification_results
                    .iter()
                    .filter(|result| {
                        let matches_filter = self.filter_text.is_empty()
                            || result
                                .game_name
                                .to_lowercase()
                                .contains(&self.filter_text.to_lowercase())
                            || result
                                .description
                                .to_lowercase()
                                .contains(&self.filter_text.to_lowercase());

                        let matches_show_only = !self.show_only_issues
                            || matches!(
                                result.status,
                                VerifyStatus::Failed | VerifyStatus::Warning
                            );

                        matches_filter && matches_show_only
                    })
                    .collect();

                if filtered_results.is_empty() {
                    if self.verification_results.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(
                                "No verification results yet. Click 'Verify All ROMs' to start.",
                            );
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("No results match the current filter.");
                        });
                    }
                } else {
                    for result in filtered_results {
                        self.render_result_item(ui, result);
                    }
                }
            });
    }

    fn render_result_item(&self, ui: &mut egui::Ui, result: &VerificationResult) {
        let (bg_color, _border_color) = match result.status {
            VerifyStatus::Passed => (
                egui::Color32::from_rgba_premultiplied(0, 100, 0, 30),
                egui::Color32::GREEN,
            ),
            VerifyStatus::Failed => (
                egui::Color32::from_rgba_premultiplied(100, 0, 0, 30),
                egui::Color32::RED,
            ),
            VerifyStatus::Warning => (
                egui::Color32::from_rgba_premultiplied(100, 100, 0, 30),
                egui::Color32::YELLOW,
            ),
            VerifyStatus::NotFound => (
                egui::Color32::from_rgba_premultiplied(50, 50, 50, 30),
                egui::Color32::GRAY,
            ),
        };

        // Use a frame instead of group for better control over background
        egui::Frame::NONE
            .fill(bg_color)
            .corner_radius(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let status_icon = match result.status {
                        VerifyStatus::Passed => "✅",
                        VerifyStatus::Failed => "❌",
                        VerifyStatus::Warning => "⚠️",
                        VerifyStatus::NotFound => "❓",
                    };

                    ui.label(status_icon);
                    ui.label(format!("{} - {}", result.game_name, result.description));
                });

                if !result.missing_files.is_empty() {
                    ui.colored_label(egui::Color32::RED, "Missing files:");
                    for file in &result.missing_files {
                        ui.label(format!("  • {}", file));
                    }
                }

                if !result.incorrect_files.is_empty() {
                    ui.colored_label(egui::Color32::YELLOW, "Incorrect files:");
                    for file in &result.incorrect_files {
                        ui.label(format!("  • {}", file));
                    }
                }

                if !result.extra_files.is_empty() {
                    ui.colored_label(egui::Color32::BLUE, "Extra files:");
                    for file in &result.extra_files {
                        ui.label(format!("  • {}", file));
                    }
                }

                if let Some(chd_status) = &result.chd_status {
                    ui.colored_label(egui::Color32::YELLOW, format!("CHD Status: {}", chd_status));
                }
            });
    }

    fn update_stats(&mut self) {
        self.stats = self.verification_manager.get_stats();
    }

    fn open_no_intro_search(&self) {
        // Open No-Intro website in default browser
        let url = "https://datomatic.no-intro.org/";
        if let Err(e) = webbrowser::open(url) {
            eprintln!("Failed to open browser: {}", e);
        }
    }

    fn start_verification(
        &mut self,
        config: &AppConfig,
        games: &[Game],
        specific_game: Option<&str>,
    ) {
        if let Some(mame) = config.mame_executables.get(config.selected_mame_index) {
            self.is_verifying = true;
            self.should_stop = false;
            self.is_paused = false;
            self.verification_results.clear();
            // A previous worker may still finish an in-flight MAME process after
            // Restart. Give the new run its own result store so stale writes stay
            // isolated from the current statistics.
            self.verification_manager = Arc::new(VerificationManager::new());
            self.stats = VerificationStats::default();
            self.current_progress = 0.0;
            self.total_games = if specific_game.is_some() {
                1
            } else {
                games.len()
            };
            self.verified_games = 0;
            self.start_time = Some(std::time::Instant::now());

            let (tx, rx) = mpsc::channel();
            let (pause_tx, pause_rx) = mpsc::channel();
            let (stop_tx, stop_rx) = mpsc::channel();

            self.receiver = Some(rx);
            self.pause_sender = Some(pause_tx);
            self.stop_sender = Some(stop_tx);

            let mame_path = mame.path.clone();
            let search_paths = crate::mame::VerificationSearchPaths::new(config);
            let games_to_verify = games.to_vec();
            let specific_game = specific_game.map(|s| s.to_string());
            let verification_manager = self.verification_manager.clone();

            thread::spawn(move || {
                if let Some(game_name) = specific_game {
                    // Verify single game
                    if let Some(game) = games_to_verify.iter().find(|g| g.name == game_name) {
                        let result = Self::verify_single_game(
                            &mame_path,
                            &search_paths.for_game(&game.name),
                            game,
                        );
                        verification_manager.update_result(game_name.clone(), result.clone());
                        let _ = tx.send(VerifyMessage::Result(result));
                        let _ = tx.send(VerifyMessage::Progress(1.0, game_name));
                    }
                } else {
                    // Verify all games
                    for (idx, game) in games_to_verify.iter().enumerate() {
                        // Check for stop signal
                        if let Ok(true) = stop_rx.try_recv() {
                            break;
                        }

                        // Check for pause signal
                        while let Ok(paused) = pause_rx.try_recv() {
                            if paused {
                                // Wait for resume signal
                                while let Ok(resume_paused) = pause_rx.recv() {
                                    if !resume_paused {
                                        break;
                                    }
                                }
                            }
                        }

                        // Stop may have been requested while the worker was
                        // blocked waiting for Resume.
                        if let Ok(true) = stop_rx.try_recv() {
                            break;
                        }

                        let progress = (idx + 1) as f32 / games_to_verify.len() as f32;
                        let _ = tx.send(VerifyMessage::Progress(progress, game.name.clone()));

                        let result = Self::verify_single_game(
                            &mame_path,
                            &search_paths.for_game(&game.name),
                            game,
                        );
                        verification_manager.update_result(game.name.clone(), result.clone());
                        let _ = tx.send(VerifyMessage::Result(result));
                    }
                }

                let _ = tx.send(VerifyMessage::Complete);
            });
        }
    }

    fn verify_single_game(
        mame_path: &str,
        search_args: &[OsString],
        game: &Game,
    ) -> VerificationResult {
        let output = Command::new(mame_path)
            .arg("-verifyroms")
            .arg(&game.name)
            // Put overrides last so a malformed trailing path option cannot
            // consume the verification verb as its value and start emulation.
            .args(search_args)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined_output = format!("{}\n{}", stdout, stderr);

                Self::parse_verification_output(
                    &game.name,
                    &game.description,
                    &combined_output,
                    output.status.success(),
                    game.chd_name.as_deref(),
                )
            }
            Err(e) => VerificationResult {
                game_name: game.name.clone(),
                description: game.description.clone(),
                status: VerifyStatus::Failed,
                missing_files: vec![format!("Error running verification: {}", e)],
                incorrect_files: vec![],
                extra_files: vec![],
                chd_status: None,
            },
        }
    }

    fn parse_verification_output(
        game_name: &str,
        description: &str,
        output: &str,
        process_succeeded: bool,
        expected_chd_name: Option<&str>,
    ) -> VerificationResult {
        let mut missing_files = Vec::new();
        let mut incorrect_files = Vec::new();
        let mut extra_files = Vec::new();
        let mut chd_status = None;
        let mut explicit_result = false;
        let mut failed = false;
        let mut warning = false;
        let mut not_found = false;

        for line in output.lines().map(str::trim) {
            // MAME emits a per-set summary, optionally followed by its parent:
            // `romset pacman [puckman] is good`. A total containing "OK" is
            // not evidence that the requested set itself passed verification.
            if let Some(summary) = line.strip_prefix("romset ") {
                let mut fields = summary.split_whitespace();
                if fields.next().map(|name| name.trim_matches('"')) == Some(game_name) {
                    let mut outcome = fields.collect::<Vec<_>>();
                    if outcome.first().is_some_and(|part| part.starts_with('[')) {
                        outcome.remove(0);
                    }
                    match outcome.join(" ").trim_end_matches('!') {
                        "is good" => explicit_result = true,
                        "is best available" => {
                            explicit_result = true;
                            warning = true;
                        }
                        "is bad" => {
                            explicit_result = true;
                            failed = true;
                        }
                        "not found" => {
                            explicit_result = true;
                            not_found = true;
                        }
                        _ => {}
                    }
                }
                continue;
            }

            // Audit records are prefixed by the set name. Restrict parsing to
            // this set, and handle optional/undumped media before NOT FOUND.
            let Some((set_name, record)) = line.split_once(':') else {
                continue;
            };
            if set_name.trim() != game_name {
                continue;
            }
            let record_upper = record.to_ascii_uppercase();
            if record_upper.contains("NO GOOD DUMP")
                || record_upper.contains("NEEDS REDUMP")
                || record_upper.contains("NOT FOUND BUT OPTIONAL")
            {
                warning = true;
            } else if record_upper.contains("NOT FOUND") {
                failed = true;
                if let Some(file) = Self::extract_filename(line) {
                    if file.to_ascii_lowercase().ends_with(".chd")
                        || expected_chd_name.is_some_and(|name| file.eq_ignore_ascii_case(name))
                    {
                        chd_status = Some(format!("CHD file not found: {file}"));
                    } else {
                        missing_files.push(file);
                    }
                }
            } else if record_upper.contains("INCORRECT") || record_upper.contains("BAD") {
                failed = true;
                if let Some(file) = Self::extract_filename(line) {
                    incorrect_files.push(file);
                }
            } else if record_upper.contains("FOUND") && record_upper.contains("EXTRA") {
                warning = true;
                if let Some(file) = Self::extract_filename(line) {
                    extra_files.push(file);
                }
            }
        }

        // Errors must never be downgraded by a later best-available/NO DUMP
        // record. Missing whole sets have their own status, even when MAME
        // reports them using a nonzero exit code.
        let status = if failed {
            VerifyStatus::Failed
        } else if not_found {
            VerifyStatus::NotFound
        } else if !process_succeeded || !explicit_result {
            let reason = if process_succeeded {
                "MAME returned no explicit verification result for this set"
            } else {
                "MAME verification process exited unsuccessfully"
            };
            let details = output.trim();
            missing_files.push(if details.is_empty() {
                reason.to_string()
            } else {
                format!("{reason}: {details}")
            });
            VerifyStatus::Failed
        } else if warning {
            VerifyStatus::Warning
        } else {
            VerifyStatus::Passed
        };

        VerificationResult {
            game_name: game_name.to_string(),
            description: description.to_string(),
            status,
            missing_files,
            incorrect_files,
            extra_files,
            chd_status,
        }
    }

    fn extract_filename(line: &str) -> Option<String> {
        let (_, record) = line.split_once(':')?;
        // MAME's audit format is `set : filename (length bytes) - reason`;
        // disk records omit the length. Preserve spaces within the filename.
        let record = record.trim();
        let name = record.split(" - ").next()?.split(" (").next()?.trim();
        (!name.is_empty()).then(|| name.trim_matches('"').to_string())
    }

    fn export_results(&self) {
        let file_extension = match self.export_format {
            ExportFormat::Text => "txt",
            ExportFormat::CSV => "csv",
            ExportFormat::HTML => "html",
        };

        let file_name = format!("rom_verification_report.{}", file_extension);

        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&file_name)
            .add_filter("Report files", &[file_extension])
            .save_file()
        {
            let content = match self.export_format {
                ExportFormat::Text => self.generate_text_report(),
                ExportFormat::CSV => self.generate_csv_report(),
                ExportFormat::HTML => self.generate_html_report(),
            };

            if let Err(e) = fs::write(&path, content) {
                eprintln!("Failed to save report: {}", e);
            }
        }
    }

    fn generate_text_report(&self) -> String {
        let mut content = String::new();
        content.push_str("ROM Verification Report - CLRMamePro Lite\n");
        content.push_str("==========================================\n\n");

        content.push_str("Summary:\n");
        content.push_str(&format!("Total verified: {}\n", self.stats.total_verified));
        content.push_str(&format!("Passed: {}\n", self.stats.passed));
        content.push_str(&format!("Failed: {}\n", self.stats.failed));
        content.push_str(&format!("Warnings: {}\n", self.stats.warnings));
        content.push_str(&format!("Missing files: {}\n", self.stats.missing));
        content.push_str(&format!("Incorrect files: {}\n", self.stats.incorrect));
        content.push_str(&format!("Missing CHD: {}\n\n", self.stats.missing_chd));

        for result in &self.verification_results {
            content.push_str(&format!(
                "Game: {} ({})\n",
                result.game_name, result.description
            ));
            content.push_str(&format!("Status: {:?}\n", result.status));

            if !result.missing_files.is_empty() {
                content.push_str("Missing files:\n");
                for file in &result.missing_files {
                    content.push_str(&format!("  - {}\n", file));
                }
            }

            if !result.incorrect_files.is_empty() {
                content.push_str("Incorrect files:\n");
                for file in &result.incorrect_files {
                    content.push_str(&format!("  - {}\n", file));
                }
            }

            if let Some(chd) = &result.chd_status {
                content.push_str(&format!("CHD: {}\n", chd));
            }

            content.push('\n');
        }

        content
    }

    fn generate_csv_report(&self) -> String {
        let mut content = String::new();
        content.push_str(
            "Game Name,Description,Status,Missing Files,Incorrect Files,Extra Files,CHD Status\n",
        );

        for result in &self.verification_results {
            let missing_files = result.missing_files.join(";");
            let incorrect_files = result.incorrect_files.join(";");
            let extra_files = result.extra_files.join(";");
            let chd_status = result.chd_status.as_deref().unwrap_or("");

            content.push_str(&format!(
                "\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                result.game_name,
                result.description,
                result.status,
                missing_files,
                incorrect_files,
                extra_files,
                chd_status
            ));
        }

        content
    }

    fn generate_html_report(&self) -> String {
        let mut content = String::new();
        content.push_str(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>ROM Verification Report - CLRMamePro Lite</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .stats { display: flex; gap: 20px; margin: 20px 0; }
        .stat { padding: 10px; border-radius: 5px; color: white; }
        .passed { background-color: #28a745; }
        .failed { background-color: #dc3545; }
        .warning { background-color: #ffc107; color: black; }
        .result { margin: 10px 0; padding: 10px; border-radius: 5px; border-left: 5px solid; }
        .result.passed { background-color: #d4edda; border-color: #28a745; }
        .result.failed { background-color: #f8d7da; border-color: #dc3545; }
        .result.warning { background-color: #fff3cd; border-color: #ffc107; }
        .result.notfound { background-color: #e2e3e5; border-color: #6c757d; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🔍 ROM Verification Report - CLRMamePro Lite</h1>
        <p>Generated on: "#,
        );
        content.push_str(
            &chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        );
        content.push_str("</p></div>");

        content.push_str(&format!(
            r#"
    <div class="stats">
        <div class="stat passed">✅ Passed: {}</div>
        <div class="stat failed">❌ Failed: {}</div>
        <div class="stat warning">⚠️ Warnings: {}</div>
    </div>
    <div class="stats">
        <div class="stat failed">📁 Missing Files: {}</div>
        <div class="stat warning">🔧 Incorrect Files: {}</div>
        <div class="stat warning">💿 Missing CHD: {}</div>
    </div>"#,
            self.stats.passed,
            self.stats.failed,
            self.stats.warnings,
            self.stats.missing,
            self.stats.incorrect,
            self.stats.missing_chd
        ));

        content.push_str("<h2>Detailed Results</h2>");

        for result in &self.verification_results {
            let status_class = match result.status {
                VerifyStatus::Passed => "passed",
                VerifyStatus::Failed => "failed",
                VerifyStatus::Warning => "warning",
                VerifyStatus::NotFound => "notfound",
            };

            content.push_str(&format!(
                r#"<div class="result {}">
                <h3>{} - {}</h3>
                <p><strong>Status:</strong> {:?}</p>"#,
                status_class, result.game_name, result.description, result.status
            ));

            if !result.missing_files.is_empty() {
                content.push_str("<p><strong>Missing files:</strong></p><ul>");
                for file in &result.missing_files {
                    content.push_str(&format!("<li>{}</li>", file));
                }
                content.push_str("</ul>");
            }

            if !result.incorrect_files.is_empty() {
                content.push_str("<p><strong>Incorrect files:</strong></p><ul>");
                for file in &result.incorrect_files {
                    content.push_str(&format!("<li>{}</li>", file));
                }
                content.push_str("</ul>");
            }

            if let Some(chd) = &result.chd_status {
                content.push_str(&format!("<p><strong>CHD Status:</strong> {}</p>", chd));
            }

            content.push_str("</div>");
        }

        content.push_str("</body></html>");
        content
    }

    // --- Public API for the redesign shell (inline page, no window) ---

    pub fn is_verifying(&self) -> bool {
        self.is_verifying
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn start_verification_all(&mut self, config: &AppConfig, games: &[Game]) {
        self.show_warning = false;
        self.start_verification(config, games, None);
    }

    pub fn stop_verification(&mut self) {
        self.should_stop = true;
        if let Some(tx) = &self.stop_sender {
            let _ = tx.send(true);
        }
        if self.is_paused
            && let Some(tx) = &self.pause_sender
        {
            let _ = tx.send(false);
        }
        self.is_verifying = false;
        self.is_paused = false;
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
        if let Some(tx) = &self.pause_sender {
            let _ = tx.send(self.is_paused);
        }
    }

    pub fn show_redesign_panel(&mut self, ui: &mut egui::Ui) {
        self.process_pending_messages();
        use crate::ui::redesign::tokens::RedesignTokens;

        let status_text = if self.is_verifying {
            if self.is_paused {
                "Paused".to_string()
            } else {
                format!("Verifying {} …", self.current_game)
            }
        } else if self.verification_results.is_empty() {
            "Idle — press Start to verify all sets".to_string()
        } else {
            "Complete".to_string()
        };

        let eta_text = format!(
            "{} / {} · ETA {}",
            self.verified_games,
            self.total_games,
            self.estimated_time.as_deref().unwrap_or("—")
        );
        if ui.available_width() < 480.0 {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(&status_text)
                        .size(13.0)
                        .color(RedesignTokens::TEXT_PRIMARY),
                )
                .truncate(),
            );
            ui.label(
                egui::RichText::new(&eta_text)
                    .size(12.0)
                    .color(RedesignTokens::TEXT_MUTED)
                    .family(egui::FontFamily::Monospace),
            );
        } else {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(&status_text)
                            .size(13.0)
                            .color(RedesignTokens::TEXT_PRIMARY),
                    )
                    .truncate(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(&eta_text)
                            .size(12.0)
                            .color(RedesignTokens::TEXT_MUTED)
                            .family(egui::FontFamily::Monospace),
                    );
                });
            });
        }
        ui.add_space(8.0);
        ui.add(
            egui::ProgressBar::new(self.current_progress)
                .fill(RedesignTokens::ACCENT)
                .animate(self.is_verifying),
        );

        ui.add_space(12.0);
        let stat_column_count = if ui.available_width() < 480.0 { 2 } else { 4 };
        ui.columns(stat_column_count, |cols| {
            let stats = [
                (self.stats.passed, "PASSED", RedesignTokens::STATUS_OK),
                (self.stats.warnings, "WARNINGS", RedesignTokens::STATUS_WARN),
                (self.stats.failed, "MISSING", RedesignTokens::STATUS_MISSING),
                (
                    self.total_games.saturating_sub(self.verified_games),
                    "PENDING",
                    RedesignTokens::STATUS_NEUTRAL,
                ),
            ];
            for (i, (n, label, color)) in stats.into_iter().enumerate() {
                let col = i % stat_column_count;
                if i >= stat_column_count {
                    cols[col].add_space(10.0);
                }
                cols[col].vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{n}"))
                            .font(fonts::bold(22.0))
                            .color(color),
                    );
                    ui.label(
                        egui::RichText::new(label)
                            .font(fonts::semibold(11.0))
                            .color(RedesignTokens::TEXT_FAINT),
                    );
                });
            }
        });

        ui.add_space(8.0);
        if ui.available_width() < 520.0 {
            ui.checkbox(&mut self.show_only_issues, "Show only issues");
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Export TXT").clicked() {
                    self.export_format = ExportFormat::Text;
                    self.export_results();
                }
                if ui.button("Export CSV").clicked() {
                    self.export_format = ExportFormat::CSV;
                    self.export_results();
                }
                if ui.button("Export HTML").clicked() {
                    self.export_format = ExportFormat::HTML;
                    self.export_results();
                }
            });
        } else {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_only_issues, "Show only issues");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Export HTML").clicked() {
                        self.export_format = ExportFormat::HTML;
                        self.export_results();
                    }
                    if ui.button("Export CSV").clicked() {
                        self.export_format = ExportFormat::CSV;
                        self.export_results();
                    }
                    if ui.button("Export TXT").clicked() {
                        self.export_format = ExportFormat::Text;
                        self.export_results();
                    }
                });
            });
        }
    }

    pub fn show_redesign_results(&mut self, ui: &mut egui::Ui) {
        self.process_pending_messages();
        use crate::ui::redesign::tokens::RedesignTokens;
        use egui_extras::{Column, TableBuilder};

        let results: Vec<_> = self
            .verification_results
            .iter()
            .rev()
            .filter(|result| !self.show_only_issues || result.status != VerifyStatus::Passed)
            .collect();

        if results.is_empty() {
            ui.label(
                egui::RichText::new(if self.show_only_issues {
                    "No verification issues to show."
                } else {
                    "No verification results yet."
                })
                .size(13.0)
                .color(RedesignTokens::TEXT_FAINT),
            );
            return;
        }

        let presentation = |result: &VerificationResult| {
            let (color, label) = match result.status {
                VerifyStatus::Passed => (RedesignTokens::STATUS_OK, "Passed"),
                VerifyStatus::Warning => (RedesignTokens::STATUS_WARN, "Warning"),
                VerifyStatus::Failed => (RedesignTokens::STATUS_MISSING, "Failed"),
                VerifyStatus::NotFound => (RedesignTokens::STATUS_NEUTRAL, "Not found"),
            };
            let note = match result.status {
                VerifyStatus::Passed => "All CRCs match datfile".to_string(),
                VerifyStatus::Warning => result.chd_status.clone().unwrap_or_else(|| {
                    "MAME reports best-available media or audit warnings".to_string()
                }),
                VerifyStatus::Failed => format!(
                    "{} missing · {} incorrect",
                    result.missing_files.len(),
                    result.incorrect_files.len()
                ),
                VerifyStatus::NotFound => "ROM set was not found".to_string(),
            };
            (color, label, note)
        };

        let available_height = ui.available_height();
        let body_height = if available_height.is_finite() {
            available_height.max(180.0)
        } else {
            // This renderer can live inside the page-level fallback scroll
            // area on short windows. Avoid propagating an infinite height
            // into the nested virtual table in that configuration.
            320.0
        };

        if ui.available_width() < 620.0 {
            TableBuilder::new(ui)
                .id_salt("redesign_verification_results_compact")
                .striped(false)
                .cell_layout(egui::Layout::top_down(egui::Align::LEFT))
                .column(Column::remainder().clip(true))
                .max_scroll_height(body_height)
                .auto_shrink([false, false])
                .body(|body| {
                    body.rows(70.0, results.len(), |mut row| {
                        let Some(result) = results.get(row.index()) else {
                            return;
                        };
                        let (color, label, note) = presentation(result);
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&result.description)
                                        .font(fonts::semibold(13.0))
                                        .color(RedesignTokens::TEXT_PRIMARY),
                                )
                                .truncate(),
                            );
                            ui.horizontal(|ui| {
                                let set_width = (ui.available_width() * 0.55).max(80.0);
                                ui.add_sized(
                                    [set_width, 18.0],
                                    egui::Label::new(
                                        egui::RichText::new(format!("{}.zip", result.game_name))
                                            .monospace()
                                            .size(12.0)
                                            .color(RedesignTokens::TEXT_MUTED),
                                    )
                                    .truncate(),
                                );
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(8.0, 8.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(label)
                                            .font(fonts::semibold(12.0))
                                            .color(color),
                                    )
                                    .truncate(),
                                );
                            });
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(note)
                                        .size(12.0)
                                        .color(RedesignTokens::TEXT_MUTED),
                                )
                                .truncate(),
                            );
                        });
                    });
                });
        } else {
            TableBuilder::new(ui)
                .id_salt("redesign_verification_results")
                .striped(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::remainder().at_least(200.0).clip(true))
                .column(Column::initial(120.0).at_least(90.0).clip(true))
                .column(Column::initial(140.0).at_least(110.0).clip(true))
                .column(Column::remainder().at_least(150.0).clip(true))
                .max_scroll_height(body_height)
                .auto_shrink([false, false])
                .body(|body| {
                    body.rows(38.0, results.len(), |mut row| {
                        let Some(result) = results.get(row.index()) else {
                            return;
                        };
                        let (color, label, note) = presentation(result);
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&result.description)
                                        .font(fonts::semibold(13.0))
                                        .color(RedesignTokens::TEXT_PRIMARY),
                                )
                                .truncate(),
                            );
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("{}.zip", result.game_name))
                                        .monospace()
                                        .size(12.0)
                                        .color(RedesignTokens::TEXT_MUTED),
                                )
                                .truncate(),
                            );
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                let (dot_rect, _) = ui.allocate_exact_size(
                                    egui::vec2(8.0, 8.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(label)
                                            .font(fonts::semibold(12.0))
                                            .color(color),
                                    )
                                    .truncate(),
                                );
                            });
                        });
                        row.col(|ui| {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(note)
                                        .size(12.0)
                                        .color(RedesignTokens::TEXT_MUTED),
                                )
                                .truncate(),
                            );
                        });
                    });
                });
        }
    }

    fn process_pending_messages(&mut self) {
        if let Some(rx) = &self.receiver {
            let mut messages = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
            for msg in messages {
                match msg {
                    VerifyMessage::Progress(progress, game) => {
                        self.current_progress = progress;
                        self.current_game = game;
                        self.verified_games += 1;
                    }
                    VerifyMessage::Result(result) => {
                        self.verification_manager
                            .update_result(result.game_name.clone(), result.clone());
                        self.verification_results.push(result);
                        self.update_stats();
                    }
                    VerifyMessage::Complete => {
                        self.is_verifying = false;
                        self.receiver = None;
                    }
                    VerifyMessage::Error(err) => {
                        eprintln!("Verification error: {err}");
                        self.is_verifying = false;
                        self.receiver = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_fixture(name: &str) -> Game {
        Game {
            name: name.into(),
            description: "Pac-Man".into(),
            manufacturer: "Namco".into(),
            year: "1980".into(),
            driver: "pacman".into(),
            driver_status: "good".into(),
            status: RomStatus::Available,
            parent: Some("puckman".into()),
            category: "Maze".into(),
            play_count: 0,
            is_clone: true,
            is_device: false,
            is_bios: false,
            controls: String::new(),
            requires_roms: true,
            requires_chd: false,
            chd_name: None,
            verification_status: None,
        }
    }

    fn parse(output: &str, process_succeeded: bool) -> VerificationResult {
        RomVerifyDialog::parse_verification_output(
            "pacman",
            "Pac-Man",
            output,
            process_succeeded,
            None,
        )
    }

    #[test]
    fn verification_requires_success_and_an_explicit_result_for_the_requested_set() {
        assert_eq!(
            parse(
                "romset pacman [puckman] is good\n1 romsets found, 1 were OK.\n",
                true
            )
            .status,
            VerifyStatus::Passed
        );
        for (output, success) in [
            ("", false),
            ("", true),
            ("Fatal error: unable to initialize MAME", false),
            ("1 romsets found, 1 were OK.", true),
            ("romset puckman is good", true),
            ("romset pacman is good", false),
        ] {
            let result = parse(output, success);
            assert_eq!(result.status, VerifyStatus::Failed, "{output:?}");
            assert!(!result.missing_files.is_empty(), "{output:?}");
        }
    }

    #[test]
    fn missing_whole_set_is_not_found() {
        assert_eq!(
            parse("romset \"pacman\" not found!\n", false).status,
            VerifyStatus::NotFound
        );
        assert_eq!(
            parse("romset pacman [puckman] not found\n", false).status,
            VerifyStatus::NotFound
        );
    }

    #[test]
    fn warnings_never_downgrade_missing_or_bad_media() {
        let missing = "pacman      : program.bin (4096 bytes) - NOT FOUND\n";
        let warning = "pacman      : undumped.bin (1024 bytes) - NO GOOD DUMP KNOWN\n";
        for output in [
            format!("{missing}{warning}romset pacman [puckman] is bad\n"),
            format!("{warning}{missing}romset pacman [puckman] is bad\n"),
            format!("{missing}{warning}romset pacman is best available\n"),
        ] {
            let result = parse(&output, false);
            assert_eq!(result.status, VerifyStatus::Failed);
            assert_eq!(result.missing_files, vec!["program.bin"]);
        }
        let incorrect = parse(
            "pacman : bad rom.bin (42 bytes) - INCORRECT CHECKSUM:\nEXPECTED: CRC(12345678)\n   FOUND: CRC(87654321)\nromset pacman is bad\n",
            false,
        );
        assert_eq!(incorrect.status, VerifyStatus::Failed);
        assert_eq!(incorrect.incorrect_files, vec!["bad rom.bin"]);
        assert_eq!(
            parse("romset pacman is bad", true).status,
            VerifyStatus::Failed
        );
    }

    #[test]
    fn optional_and_undumped_media_are_best_available_warnings() {
        for diagnostic in [
            "NOT FOUND BUT OPTIONAL",
            "NOT FOUND - NO GOOD DUMP KNOWN",
            "NO GOOD DUMP KNOWN",
            "NEEDS REDUMP",
        ] {
            let result = parse(
                &format!(
                    "pacman : optional.bin (42 bytes) - {diagnostic}\nromset pacman [puckman] is best available\n"
                ),
                true,
            );
            assert_eq!(result.status, VerifyStatus::Warning, "{diagnostic}");
            assert!(result.missing_files.is_empty());
            assert!(result.incorrect_files.is_empty());
        }
    }

    #[test]
    fn missing_chd_matches_xml_disk_name_without_a_file_extension() {
        let result = RomVerifyDialog::parse_verification_output(
            "area51",
            "Area 51",
            "area51      : area51 - NOT FOUND\nromset area51 is bad\n",
            false,
            Some("area51"),
        );
        assert_eq!(result.status, VerifyStatus::Failed);
        assert_eq!(
            result.chd_status.as_deref(),
            Some("CHD file not found: area51")
        );
        assert!(result.missing_files.is_empty());
        let manager = VerificationManager::new();
        manager.update_result("area51".into(), result);
        assert_eq!(manager.get_stats().missing_chd, 1);
    }

    #[test]
    #[ignore = "set MAMEUIX_TEST_MAME to an installed MAME executable"]
    fn native_mame_verification_checks_empty_and_overridden_collections() {
        let executable = std::env::var("MAMEUIX_TEST_MAME").expect("set MAMEUIX_TEST_MAME");
        let temp = tempfile::tempdir().unwrap();
        let empty_roms = temp.path().join("empty");
        fs::create_dir(&empty_roms).unwrap();
        let mut config = AppConfig {
            rom_paths: vec![empty_roms],
            ..Default::default()
        };
        config.default_game_properties.miscellaneous.custom_args = "-noreadconfig".into();
        let snapshot = crate::mame::VerificationSearchPaths::new(&config);
        for (name, expected) in [
            ("pong", VerifyStatus::Warning),
            ("pacman", VerifyStatus::NotFound),
        ] {
            let result = RomVerifyDialog::verify_single_game(
                &executable,
                &snapshot.for_game(name),
                &game_fixture(name),
            );
            assert_eq!(result.status, expected, "{name}: {result:?}");
        }

        // An incomplete set only in the custom root must be audited there:
        // the structured empty root would instead produce NotFound.
        let custom_roms = temp.path().join("custom");
        fs::create_dir_all(custom_roms.join("pacman")).unwrap();
        fs::write(
            custom_roms.join("pacman/pacman.6e"),
            b"deliberately invalid fixture",
        )
        .unwrap();
        for alias in ["-rompath", "-rp", "-biospath", "-bp"] {
            let mut properties = config.default_game_properties.clone();
            properties.miscellaneous.custom_args = format!(
                "-norc {alias} {} -bench 60 -plugin ignored -autoboot_script ignored.lua",
                custom_roms.display()
            );
            config.game_properties.insert("pacman".into(), properties);
            let result = RomVerifyDialog::verify_single_game(
                &executable,
                &crate::mame::VerificationSearchPaths::new(&config).for_game("pacman"),
                &game_fixture("pacman"),
            );
            assert_eq!(result.status, VerifyStatus::Failed, "{alias}: {result:?}");
            assert!(!result.incorrect_files.is_empty(), "{alias}: {result:?}");
        }

        // A missing option value must fail the audit, never consume -verifyroms
        // as a directory and accidentally start the emulator.
        config
            .game_properties
            .get_mut("pacman")
            .unwrap()
            .miscellaneous
            .custom_args = "-norc -rp".into();
        let result = RomVerifyDialog::verify_single_game(
            &executable,
            &crate::mame::VerificationSearchPaths::new(&config).for_game("pacman"),
            &game_fixture("pacman"),
        );
        assert_eq!(result.status, VerifyStatus::Failed);
    }

    #[cfg(unix)]
    #[test]
    fn subprocess_verification_uses_configured_paths_and_checks_exit_status() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("mame-stub");
        let ini = temp.path().join("ini with spaces");
        let home = temp.path().join("home");
        fs::create_dir(&ini).unwrap();
        fs::create_dir(&home).unwrap();
        let mut config = AppConfig {
            rom_paths: vec![temp.path().join("arcade roms")],
            extra_rom_dirs: vec![temp.path().join("extra roms")],
            software_rom_paths: vec![temp.path().join("software roms")],
            ini_path: Some(ini),
            home_path: Some(home),
            ..Default::default()
        };
        let mut per_game = config.default_game_properties.clone();
        per_game.miscellaneous.custom_args = "-rp custom -norc -bench 60 -plugin ignored".into();
        config.game_properties.insert("pacman".into(), per_game);
        let search_args = crate::mame::VerificationSearchPaths::new(&config).for_game("pacman");
        let game = game_fixture("pacman");
        for (body, expected) in [
            (
                "echo 'romset pacman [puckman] is good'\nexit 0\n",
                VerifyStatus::Passed,
            ),
            ("exit 1\n", VerifyStatus::Failed),
            (
                "echo 'romset pacman is good'\nexit 2\n",
                VerifyStatus::Failed,
            ),
            (
                "echo 'romset \"pacman\" not found!' >&2\nexit 2\n",
                VerifyStatus::NotFound,
            ),
        ] {
            fs::write(
                &executable,
                format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/args\"\n{body}"),
            )
            .unwrap();
            fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
            let result = RomVerifyDialog::verify_single_game(
                executable.to_str().unwrap(),
                &search_args,
                &game,
            );
            assert_eq!(result.status, expected, "{body}");
            let mut expected_args = vec!["-verifyroms".to_string(), "pacman".to_string()];
            expected_args.extend(
                search_args
                    .iter()
                    .map(|arg| arg.to_string_lossy().into_owned()),
            );
            let captured = fs::read_to_string(temp.path().join("args")).unwrap();
            assert_eq!(captured.lines().collect::<Vec<_>>(), expected_args);
        }
    }
}
