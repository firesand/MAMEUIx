use eframe::egui;
use std::process::Command;
use std::collections::HashMap;
use crate::models::{AppConfig, Game, RomStatus, VerificationStatus};
use std::sync::mpsc;
use std::thread;
use std::fs;
use std::sync::{Arc, Mutex};

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
        let mut stats = VerificationStats::default();
        stats.total_verified = results.len();
        
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
                },
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

#[derive(Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Text,
    CSV,
    HTML,
}

impl Default for ExportFormat {
    fn default() -> Self {
        ExportFormat::Text
    }
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
                                let avg_time_per_game = elapsed.as_secs_f32() / self.verified_games as f32;
                                let remaining_games = self.total_games - self.verified_games;
                                let estimated_remaining = avg_time_per_game * remaining_games as f32;
                                
                                if estimated_remaining > 60.0 {
                                    let minutes = (estimated_remaining / 60.0) as u32;
                                    let seconds = (estimated_remaining % 60.0) as u32;
                                    self.estimated_time = Some(format!("{}m {}s", minutes, seconds));
                                } else {
                                    self.estimated_time = Some(format!("{:.0}s", estimated_remaining));
                                }
                            }
                        }
                    }
                    VerifyMessage::Result(result) => {
                        self.verification_results.push(result.clone());
                        // Update verification manager
                        self.verification_manager.update_result(result.game_name.clone(), result);
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
                if ui.button("✕").clicked() {
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
                    
                    if ui.button("✓ Verify Available Only").clicked() {
                        let available_games: Vec<_> = games.iter()
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
            ui.add(egui::TextEdit::singleline(&mut self.filter_text)
                .desired_width(ui.available_width() - 120.0));
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
                    ui.colored_label(egui::Color32::GREEN, format!("✅ Verified: {}", self.stats.passed));
                    ui.colored_label(egui::Color32::RED, format!("❌ Failed: {}", self.stats.failed));
                    ui.colored_label(egui::Color32::YELLOW, format!("⚠️ Warnings: {}", self.stats.warnings));
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
                    let progress_percent = if self.total_games > 0 {
                        (self.verified_games * 100) / self.total_games
                    } else {
                        0
                    };
                    ui.label(format!("📈 Progress: {} / {} ({}%)", 
                        self.verified_games, self.total_games, progress_percent));
                    
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
                ui.add(egui::ProgressBar::new(self.current_progress)
                    .show_percentage()
                    .animate(true)
                    .desired_width(ui.available_width() - 200.0));
                
                if let Some(estimated) = &self.estimated_time {
                    ui.label(format!("ETA: {}", estimated));
                }
            });
            
            // Current status
            ui.horizontal(|ui| {
                ui.label(format!("Current: {}", self.current_game));
                ui.add_space(10.0);
                ui.label(format!("Progress: {} / {}", self.verified_games, self.total_games));
            });
            
            // Live stats during verification
            if !self.verification_results.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(egui::Color32::GREEN, format!("✅ {} passed", self.stats.passed));
                    ui.colored_label(egui::Color32::RED, format!("❌ {} failed", self.stats.failed));
                    ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {} warnings", self.stats.warnings));
                });
            }
            
            // Control buttons
            ui.horizontal_wrapped(|ui| {
                if self.is_paused {
                    if ui.add(egui::Button::new("▶ Resume").fill(egui::Color32::GREEN)).clicked() {
                        if let Some(sender) = &self.pause_sender {
                            let _ = sender.send(false);
                        }
                        self.is_paused = false;
                    }
                } else {
                    if ui.add(egui::Button::new("⏸ Pause").fill(egui::Color32::YELLOW)).clicked() {
                        if let Some(sender) = &self.pause_sender {
                            let _ = sender.send(true);
                        }
                        self.is_paused = true;
                    }
                }
                
                if ui.add(egui::Button::new("⏹ Stop").fill(egui::Color32::RED)).clicked() {
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
        
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            let filtered_results: Vec<_> = self.verification_results.iter()
                .filter(|result| {
                    let matches_filter = self.filter_text.is_empty() || 
                        result.game_name.to_lowercase().contains(&self.filter_text.to_lowercase()) ||
                        result.description.to_lowercase().contains(&self.filter_text.to_lowercase());
                    
                    let matches_show_only = !self.show_only_issues || 
                        matches!(result.status, VerifyStatus::Failed | VerifyStatus::Warning);
                    
                    matches_filter && matches_show_only
                })
                .collect();

            if filtered_results.is_empty() {
                if self.verification_results.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No verification results yet. Click 'Verify All ROMs' to start.");
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
        let (bg_color, border_color) = match result.status {
            VerifyStatus::Passed => (egui::Color32::from_rgba_premultiplied(0, 100, 0, 30), egui::Color32::GREEN),
            VerifyStatus::Failed => (egui::Color32::from_rgba_premultiplied(100, 0, 0, 30), egui::Color32::RED),
            VerifyStatus::Warning => (egui::Color32::from_rgba_premultiplied(100, 100, 0, 30), egui::Color32::YELLOW),
            VerifyStatus::NotFound => (egui::Color32::from_rgba_premultiplied(50, 50, 50, 30), egui::Color32::GRAY),
        };

        // Use a frame instead of group for better control over background
        egui::Frame::none()
            .fill(bg_color)
            .rounding(4.0)
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

    fn start_verification(&mut self, config: &AppConfig, games: &[Game], specific_game: Option<&str>) {
        if let Some(mame) = config.mame_executables.get(config.selected_mame_index) {
            self.is_verifying = true;
            self.should_stop = false;
            self.is_paused = false;
            self.verification_results.clear();
            self.stats = VerificationStats::default();
            self.current_progress = 0.0;
            self.total_games = if specific_game.is_some() { 1 } else { games.len() };
            self.verified_games = 0;
            self.start_time = Some(std::time::Instant::now());

            let (tx, rx) = mpsc::channel();
            let (pause_tx, pause_rx) = mpsc::channel();
            let (stop_tx, stop_rx) = mpsc::channel();
            
            self.receiver = Some(rx);
            self.pause_sender = Some(pause_tx);
            self.stop_sender = Some(stop_tx);

            let mame_path = mame.path.clone();
            let games_to_verify = games.to_vec();
            let specific_game = specific_game.map(|s| s.to_string());
            let verification_manager = self.verification_manager.clone();

            thread::spawn(move || {
                if let Some(game_name) = specific_game {
                    // Verify single game
                    if let Some(game) = games_to_verify.iter().find(|g| g.name == game_name) {
                        let result = Self::verify_single_game(&mame_path, game);
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
                        
                        let progress = (idx + 1) as f32 / games_to_verify.len() as f32;
                        let _ = tx.send(VerifyMessage::Progress(progress, game.name.clone()));
                        
                        let result = Self::verify_single_game(&mame_path, game);
                        verification_manager.update_result(game.name.clone(), result.clone());
                        let _ = tx.send(VerifyMessage::Result(result));
                    }
                }

                let _ = tx.send(VerifyMessage::Complete);
            });
        }
    }

    fn verify_single_game(mame_path: &str, game: &Game) -> VerificationResult {
        let output = Command::new(mame_path)
            .arg("-verifyroms")
            .arg(&game.name)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined_output = format!("{}\n{}", stdout, stderr);

                Self::parse_verification_output(&game.name, &game.description, &combined_output)
            }
            Err(e) => {
                VerificationResult {
                    game_name: game.name.clone(),
                    description: game.description.clone(),
                    status: VerifyStatus::Failed,
                    missing_files: vec![format!("Error running verification: {}", e)],
                    incorrect_files: vec![],
                    extra_files: vec![],
                    chd_status: None,
                }
            }
        }
    }

    fn parse_verification_output(game_name: &str, description: &str, output: &str) -> VerificationResult {
        let mut missing_files = Vec::new();
        let mut incorrect_files = Vec::new();
        let mut extra_files = Vec::new();
        let mut chd_status = None;
        let mut status = VerifyStatus::Passed;

        for line in output.lines() {
            let line = line.trim();
            
            if line.contains("NOT FOUND") {
                if line.contains(".chd") {
                    chd_status = Some("CHD file not found".to_string());
                    status = VerifyStatus::Failed;
                } else if let Some(file) = Self::extract_filename(line) {
                    missing_files.push(file);
                    status = VerifyStatus::Failed;
                }
            } else if line.contains("INCORRECT") || line.contains("BAD") {
                if let Some(file) = Self::extract_filename(line) {
                    incorrect_files.push(file);
                    status = VerifyStatus::Failed;
                }
            } else if line.contains("NO GOOD DUMP") {
                status = VerifyStatus::Warning;
            } else if line.contains("found") && line.contains("extra") {
                if let Some(file) = Self::extract_filename(line) {
                    extra_files.push(file);
                    if status == VerifyStatus::Passed {
                        status = VerifyStatus::Warning;
                    }
                }
            } else if line.contains("is good") || line.contains("OK") {
                // ROM is verified OK
            }
        }

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
        // Try to extract filename from various MAME output formats
        if let Some(start) = line.find('"') {
            if let Some(end) = line[start + 1..].find('"') {
                return Some(line[start + 1..start + 1 + end].to_string());
            }
        }
        
        // Alternative format
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() > 1 {
            Some(parts[0].to_string())
        } else {
            None
        }
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

        content.push_str(&format!("Summary:\n"));
        content.push_str(&format!("Total verified: {}\n", self.stats.total_verified));
        content.push_str(&format!("Passed: {}\n", self.stats.passed));
        content.push_str(&format!("Failed: {}\n", self.stats.failed));
        content.push_str(&format!("Warnings: {}\n", self.stats.warnings));
        content.push_str(&format!("Missing files: {}\n", self.stats.missing));
        content.push_str(&format!("Incorrect files: {}\n", self.stats.incorrect));
        content.push_str(&format!("Missing CHD: {}\n\n", self.stats.missing_chd));

        for result in &self.verification_results {
            content.push_str(&format!("Game: {} ({})\n", result.game_name, result.description));
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

            content.push_str("\n");
        }

        content
    }

    fn generate_csv_report(&self) -> String {
        let mut content = String::new();
        content.push_str("Game Name,Description,Status,Missing Files,Incorrect Files,Extra Files,CHD Status\n");
        
        for result in &self.verification_results {
            let missing_files = result.missing_files.join(";");
            let incorrect_files = result.incorrect_files.join(";");
            let extra_files = result.extra_files.join(";");
            let chd_status = result.chd_status.as_deref().unwrap_or("");
            
            content.push_str(&format!("\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                result.game_name, result.description, result.status,
                missing_files, incorrect_files, extra_files, chd_status));
        }
        
        content
    }

    fn generate_html_report(&self) -> String {
        let mut content = String::new();
        content.push_str(r#"<!DOCTYPE html>
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
        <p>Generated on: "#);
        content.push_str(&chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
        content.push_str("</p></div>");
        
        content.push_str(&format!(r#"
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
            self.stats.passed, self.stats.failed, self.stats.warnings,
            self.stats.missing, self.stats.incorrect, self.stats.missing_chd));
        
        content.push_str("<h2>Detailed Results</h2>");
        
        for result in &self.verification_results {
            let status_class = match result.status {
                VerifyStatus::Passed => "passed",
                VerifyStatus::Failed => "failed",
                VerifyStatus::Warning => "warning",
                VerifyStatus::NotFound => "notfound",
            };
            
            content.push_str(&format!(r#"<div class="result {}">
                <h3>{} - {}</h3>
                <p><strong>Status:</strong> {:?}</p>"#, 
                status_class, result.game_name, result.description, result.status));
            
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
} 