// src/ui/history_panel.rs
use eframe::egui;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::models::AppConfig;
use std::fs;
use quick_xml::events::Event;
use quick_xml::Reader;

/// HistoryPanel displays game history and information from various DAT files
pub struct HistoryPanel {
    /// Cache of loaded history data
    history_cache: HashMap<String, String>,
    /// Currently selected game
    current_game: Option<String>,
    /// Cached display text for current game
    current_display_text: String,
    /// Whether we're currently loading data
    is_loading: bool,
}

impl HistoryPanel {
    pub fn new() -> Self {
        Self {
            history_cache: HashMap::new(),
            current_game: None,
            current_display_text: String::new(),
            is_loading: false,
        }
    }
    
    /// Update the selected game and load its history
    pub fn set_selected_game(&mut self, game_name: Option<String>, config: &AppConfig) {
        if self.current_game != game_name {
            self.current_game = game_name.clone();
            self.is_loading = true;
            
            // Load history data for the new game
            if let Some(game_name) = game_name {
                self.load_game_history(&game_name, config);
            } else {
                self.current_display_text.clear();
            }
            
            self.is_loading = false;
        }
    }
    
    /// Load all history data for a game
    fn load_game_history(&mut self, game_name: &str, config: &AppConfig) {
        let mut history_text = String::new();
        
        // Check if we already have the complete text cached
        let complete_cache_key = format!("complete_{}", game_name);
        if let Some(cached_text) = self.history_cache.get(&complete_cache_key) {
            self.current_display_text = cached_text.clone();
            return;
        }
        
        // Load from history.xml if available
        if let Some(history_path) = &config.history_path {
            if let Some(text) = self.load_history_xml(history_path, game_name) {
                history_text.push_str("=== History ===\n");
                history_text.push_str(&text);
                history_text.push_str("\n\n");
            }
        }
        
        // Load from mameinfo.dat if available
        if let Some(mameinfo_path) = &config.mameinfo_dat_path {
            if let Some(text) = self.load_dat_file(mameinfo_path, game_name, "mameinfo") {
                history_text.push_str("=== MAME Info ===\n");
                history_text.push_str(&text);
                history_text.push_str("\n\n");
            }
        }
        
        // Load from command.dat if available
        if let Some(command_path) = &config.command_dat_path {
            if let Some(text) = self.load_dat_file(command_path, game_name, "command") {
                history_text.push_str("=== Commands ===\n");
                history_text.push_str(&text);
                history_text.push_str("\n\n");
            }
        }
        
        // Load from hiscore.dat if available
        if let Some(hiscore_path) = &config.hiscore_dat_path {
            if let Some(text) = self.load_dat_file(hiscore_path, game_name, "hiscore") {
                history_text.push_str("=== High Score Info ===\n");
                history_text.push_str(&text);
                history_text.push_str("\n\n");
            }
        }
        
        // Load from gameinit.dat if available
        if let Some(gameinit_path) = &config.gameinit_dat_path {
            if let Some(text) = self.load_dat_file(gameinit_path, game_name, "gameinit") {
                history_text.push_str("=== Game Init Info ===\n");
                history_text.push_str(&text);
                history_text.push_str("\n\n");
            }
        }
        
        // Cache the complete text
        if !history_text.is_empty() {
            self.history_cache.insert(complete_cache_key, history_text.clone());
        }
        
        self.current_display_text = history_text;
    }
    
    /// Show the history panel
    pub fn show(&mut self, ui: &mut egui::Ui, _config: &AppConfig) {
        ui.group(|ui| {
            ui.heading("Game Information");
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if self.is_loading {
                        ui.centered_and_justified(|ui| {
                            ui.label("Loading history data...");
                        });
                    } else if self.current_game.is_some() {
                        if self.current_display_text.is_empty() {
                            ui.label("No history information available for this game.");
                            ui.label("Configure history and DAT file paths in Directories settings.");
                        } else {
                            // Display the cached history text
                            ui.add(egui::TextEdit::multiline(&mut self.current_display_text.as_str())
                                .desired_width(f32::INFINITY)
                                .font(egui::TextStyle::Monospace));
                        }
                    } else {
                        ui.label("Select a game to view its history and information.");
                    }
                });
        });
    }
    
    /// Load history from history.xml file
    fn load_history_xml(&mut self, path: &PathBuf, game_name: &str) -> Option<String> {
        // Check cache first
        let cache_key = format!("history_xml_{}", game_name);
        if let Some(cached) = self.history_cache.get(&cache_key) {
            return Some(cached.clone());
        }
        
        if !path.exists() {
            return None;
        }
        
        // Try to read and parse the XML file
        match fs::read_to_string(path) {
            Ok(content) => {
                let mut reader = Reader::from_str(&content);
                reader.config_mut().trim_text(true);
                
                let mut buf = Vec::new();
                let mut in_entry = false;
                let mut in_text = false;
                let mut found_game = false;
                let mut history_text = String::new();
                
                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Start(ref e)) => {
                            match e.name().as_ref() {
                                b"entry" => {
                                    in_entry = true;
                                    found_game = false;
                                    history_text.clear();
                                }
                                b"system" | b"item" if in_entry => {
                                    // Check if this entry is for our game
                                    for attr in e.attributes() {
                                        if let Ok(attr) = attr {
                                            if attr.key.as_ref() == b"name" {
                                                if let Ok(value) = String::from_utf8(attr.value.to_vec()) {
                                                    if value == game_name {
                                                        found_game = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                b"text" if in_entry && found_game => {
                                    in_text = true;
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::Text(e)) => {
                            if in_text && found_game {
                                // For quick-xml 0.38, we need to decode the text properly
                                match reader.decoder().decode(&e) {
                                    Ok(text) => history_text.push_str(&text),
                                    Err(_) => {
                                        // Fallback to lossy conversion
                                        let text = String::from_utf8_lossy(&e);
                                        history_text.push_str(&text);
                                    }
                                }
                            }
                        }
                        Ok(Event::End(ref e)) => {
                            match e.name().as_ref() {
                                b"text" if in_text => {
                                    in_text = false;
                                }
                                b"entry" if in_entry && found_game => {
                                    // We found our game, cache and return
                                    let result = history_text.trim().to_string();
                                    if !result.is_empty() {
                                        self.history_cache.insert(cache_key, result.clone());
                                        return Some(result);
                                    }
                                }
                                b"entry" => {
                                    in_entry = false;
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::Eof) => break,
                        Err(e) => {
                            eprintln!("Error parsing history.xml: {:?}", e);
                            break;
                        }
                        _ => {}
                    }
                    buf.clear();
                }
                
                None
            }
            Err(e) => {
                eprintln!("Error reading history.xml: {:?}", e);
                None
            }
        }
    }
    
    /// Load information from a DAT file
    fn load_dat_file(&mut self, path: &PathBuf, game_name: &str, dat_type: &str) -> Option<String> {
        // Check cache first
        let cache_key = format!("{}_{}", dat_type, game_name);
        if let Some(cached) = self.history_cache.get(&cache_key) {
            return Some(cached.clone());
        }
        
        if !path.exists() {
            return None;
        }
        
        match fs::read_to_string(path) {
            Ok(content) => {
                // Try to find the game entry
                // Format: $info=gamename
                let game_marker = format!("$info={}", game_name);
                
                if let Some(start_pos) = content.find(&game_marker) {
                    // Find the next line after $info=gamename
                    let content_after_marker = &content[start_pos..];
                    
                    // Skip to the next line after $info=gamename
                    if let Some(newline_pos) = content_after_marker.find('\n') {
                        let content_after_info = &content_after_marker[newline_pos + 1..];
                        
                        // Find the next $info= marker or end of file
                        let end_pos = content_after_info.find("$info=").unwrap_or(content_after_info.len());
                        let info_text = &content_after_info[..end_pos];
                        
                        // Clean up the text
                        let cleaned_text = info_text
                            .lines()
                            .filter(|line| {
                                // Skip lines that are just markers like $mame, $end, etc.
                                let trimmed = line.trim();
                                !trimmed.starts_with('$') || trimmed.is_empty()
                            })
                            .map(|line| line.trim())
                            .filter(|line| !line.is_empty())
                            .collect::<Vec<_>>()
                            .join("\n");
                        
                        if !cleaned_text.is_empty() {
                            self.history_cache.insert(cache_key, cleaned_text.clone());
                            return Some(cleaned_text);
                        }
                    }
                }
                
                // Alternative format: some DAT files use $bio/$end markers
                let game_marker_with_comma = format!("$info={},", game_name);
                
                if let Some(start_pos) = content.find(&game_marker_with_comma) {
                    let content_after_marker = &content[start_pos..];
                    
                    if let Some(bio_start) = content_after_marker.find("$bio") {
                        let bio_content = &content_after_marker[bio_start + 4..];
                        
                        if let Some(end_pos) = bio_content.find("$end") {
                            let info_text = bio_content[..end_pos].trim();
                            
                            if !info_text.is_empty() {
                                let cleaned_text = info_text
                                    .lines()
                                    .map(|line| line.trim())
                                    .filter(|line| !line.is_empty())
                                    .collect::<Vec<_>>()
                                    .join("\n");
                                
                                self.history_cache.insert(cache_key, cleaned_text.clone());
                                return Some(cleaned_text);
                            }
                        }
                    }
                }
                
                None
            }
            Err(e) => {
                eprintln!("Error reading {}: {:?}", dat_type, e);
                None
            }
        }
    }
    
    /// Clear the cache (useful when files are updated)
    pub fn clear_cache(&mut self) {
        self.history_cache.clear();
    }
}