use eframe::egui;
use std::process::Command;
use std::collections::HashMap;
use crate::models::{AppConfig, Game, RomStatus};
use std::sync::mpsc;
use std::thread;

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
        }
    }
}

impl RomVerifyDialog {
    // New method to open the window
    pub fn open(&mut self) {
        self.window_open = true;
    }
    
    // New method to close the window
    pub fn close(&mut self) {
        self.window_open = false;
    }
    
    // New method to check if window is open
    pub fn is_open(&self) -> bool {
        self.window_open
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
        egui::Window::new("🔍 ROM Verification")
            .open(&mut window_open)
            .default_size([700.0, 500.0])
            .min_size([500.0, 350.0])
            .max_size([1000.0, 800.0])
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
                        self.verification_results.push(result);
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

        // Header with close button - more compact
        ui.horizontal(|ui| {
            ui.heading("🔍 ROM Verification");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✕").clicked() {
                    self.window_open = false;
                }
            });
        });

        ui.separator();

        // Control buttons in a more compact layout
        ui.vertical(|ui| {
            // First row: Main action buttons - wrap if needed
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
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("⏳ Verifying..."));
                }
            });
            
            // Second row: Options - more compact
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_only_issues, "Show only issues");
                ui.add_space(10.0);
                ui.label(format!("Total: {}", games.len()));
            });
        });

        ui.separator();

        // Progress bar and controls - more compact
        if self.is_verifying {
            ui.group(|ui| {
                // Progress bar with ETA
                ui.horizontal(|ui| {
                    ui.add(egui::ProgressBar::new(self.current_progress)
                        .show_percentage()
                        .animate(true)
                        .desired_width(ui.available_width() - 80.0));
                    
                    if let Some(estimated) = &self.estimated_time {
                        ui.label(format!("ETA: {}", estimated));
                    }
                });
                
                // Status info - more compact
                ui.horizontal(|ui| {
                    ui.label(format!("Current: {}", self.current_game));
                    ui.add_space(10.0);
                    ui.label(format!("Progress: {} / {}", self.verified_games, self.total_games));
                });
                
                // Control buttons - more compact
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

        ui.separator();

        // Filter controls - more compact
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.add(egui::TextEdit::singleline(&mut self.filter_text)
                .desired_width(ui.available_width() - 120.0));
            if ui.button("Clear").clicked() {
                self.filter_text.clear();
            }
        });

        ui.separator();

        // Results display - more proportional
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
                    ui.group(|ui| {
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
                            ui.label("Missing files:");
                            for file in &result.missing_files {
                                ui.label(format!("  • {}", file));
                            }
                        }
                        
                        if !result.incorrect_files.is_empty() {
                            ui.label("Incorrect files:");
                            for file in &result.incorrect_files {
                                ui.label(format!("  • {}", file));
                            }
                        }
                        
                        if !result.extra_files.is_empty() {
                            ui.label("Extra files:");
                            for file in &result.extra_files {
                                ui.label(format!("  • {}", file));
                            }
                        }
                        
                        if let Some(chd_status) = &result.chd_status {
                            ui.label(format!("CHD Status: {}", chd_status));
                        }
                    });
                }
            }
        });

        // Summary - more compact
        if !self.verification_results.is_empty() {
            ui.separator();
            let passed = self.verification_results.iter().filter(|r| matches!(r.status, VerifyStatus::Passed)).count();
            let failed = self.verification_results.iter().filter(|r| matches!(r.status, VerifyStatus::Failed)).count();
            let warnings = self.verification_results.iter().filter(|r| matches!(r.status, VerifyStatus::Warning)).count();
            let not_found = self.verification_results.iter().filter(|r| matches!(r.status, VerifyStatus::NotFound)).count();
            
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Summary: ✅{} ❌{} ⚠️{} ❓{}", passed, failed, warnings, not_found));
                ui.add_space(10.0);
                if ui.button("Export").clicked() {
                    self.export_results();
                }
            });
        }
    }

    // Keep the original show method for backward compatibility
    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool, config: &AppConfig, games: &[Game]) {
        self.window_open = *open;
        self.show_window(ctx, config, games);
        *open = self.window_open;
    }

    fn start_verification(&mut self, config: &AppConfig, games: &[Game], specific_game: Option<&str>) {
        if let Some(mame) = config.mame_executables.get(config.selected_mame_index) {
            self.is_verifying = true;
            self.should_stop = false;
            self.is_paused = false;
            self.verification_results.clear();
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

            thread::spawn(move || {
                if let Some(game_name) = specific_game {
                    // Verify single game
                    if let Some(game) = games_to_verify.iter().find(|g| g.name == game_name) {
                        let result = Self::verify_single_game(&mame_path, game);
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
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("rom_verification_report.txt")
            .add_filter("Text files", &["txt"])
            .save_file() 
        {
            let mut content = String::new();
            content.push_str("ROM Verification Report\n");
            content.push_str("======================\n\n");

            let passed = self.verification_results.iter()
                .filter(|r| r.status == VerifyStatus::Passed).count();
            let failed = self.verification_results.iter()
                .filter(|r| r.status == VerifyStatus::Failed).count();
            let warnings = self.verification_results.iter()
                .filter(|r| r.status == VerifyStatus::Warning).count();

            content.push_str(&format!("Summary:\n"));
            content.push_str(&format!("Total verified: {}\n", self.verification_results.len()));
            content.push_str(&format!("Passed: {}\n", passed));
            content.push_str(&format!("Failed: {}\n", failed));
            content.push_str(&format!("Warnings: {}\n\n", warnings));

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

            if let Err(e) = std::fs::write(&path, content) {
                eprintln!("Failed to save report: {}", e);
            }
        }
    }
} 