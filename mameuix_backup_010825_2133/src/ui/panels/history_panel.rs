// src/ui/history_panel.rs
use eframe::egui;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::models::AppConfig;
use std::fs;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Tab selection for history panel
#[derive(Debug, Clone, Copy, PartialEq)]
enum HistoryTab {
    History,    // history.xml
    MameInfo,   // mameinfo.dat
    Other,      // Other DAT files (command, hiscore, gameinit)
}

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
    /// Currently selected tab
    selected_tab: HistoryTab,
    /// Separate display texts for each tab
    history_text: String,
    mameinfo_text: String,
    other_text: String,
}

impl HistoryPanel {
    pub fn new() -> Self {
        Self {
            history_cache: HashMap::new(),
            current_game: None,
            current_display_text: String::new(),
            is_loading: false,
            selected_tab: HistoryTab::History,
            history_text: String::new(),
            mameinfo_text: String::new(),
            other_text: String::new(),
        }
    }
    
    /// Update the selected game and load its history
    pub fn set_selected_game(&mut self, game_name: Option<String>, rom_name: Option<String>, config: &AppConfig) {
        if self.current_game != game_name {
            self.current_game = game_name.clone();
            self.is_loading = true;
            
            // Clear all tab texts
            self.history_text.clear();
            self.mameinfo_text.clear();
            self.other_text.clear();
            self.current_display_text.clear();
            
            // Load history data for the new game using ROM name
            if let Some(rom_name) = rom_name {
                // Strip .zip extension if present
                let rom_name_clean = if rom_name.ends_with(".zip") {
                    rom_name.trim_end_matches(".zip")
                } else {
                    &rom_name
                };
                self.load_game_history(rom_name_clean, config);
            }
            
            self.is_loading = false;
        }
    }
    
    /// Load all history data for a game
    fn load_game_history(&mut self, rom_name: &str, config: &AppConfig) {
        // Load from history.xml if available
        if let Some(history_path) = &config.history_path {
            // Force reload the history XML file to populate cache
            self.load_history_xml(history_path, rom_name);
            
            // Now check if we have the data in cache
            let cache_key = format!("history_xml_{}", rom_name);
            if let Some(text) = self.history_cache.get(&cache_key) {
                self.history_text = text.clone();
            } else {
                // Try case-insensitive search
                for (key, value) in &self.history_cache {
                    if key.starts_with("history_xml_") {
                        let cached_rom = &key[12..]; // Skip "history_xml_" prefix
                        if cached_rom.to_lowercase() == rom_name.to_lowercase() {
                            self.history_text = value.clone();
                            break;
                        }
                    }
                }
            }
        }
        
        // Load from mameinfo.dat if available
        if let Some(mameinfo_path) = &config.mameinfo_dat_path {
            if let Some(text) = self.load_dat_file(mameinfo_path, rom_name, "mameinfo") {
                self.mameinfo_text = text;
            }
        }
        
        // Load other DAT files and combine them
        let mut other_combined = String::new();
        
        // Load from command.dat if available
        if let Some(command_path) = &config.command_dat_path {
            if let Some(text) = self.load_dat_file(command_path, rom_name, "command") {
                if !other_combined.is_empty() {
                    other_combined.push_str("\n\n");
                }
                other_combined.push_str("=== Commands ===\n");
                other_combined.push_str(&text);
            }
        }
        
        // Load from hiscore.dat if available
        if let Some(hiscore_path) = &config.hiscore_dat_path {
            if let Some(text) = self.load_dat_file(hiscore_path, rom_name, "hiscore") {
                if !other_combined.is_empty() {
                    other_combined.push_str("\n\n");
                }
                other_combined.push_str("=== High Score Info ===\n");
                other_combined.push_str(&text);
            }
        }
        
        // Load from gameinit.dat if available
        if let Some(gameinit_path) = &config.gameinit_dat_path {
            if let Some(text) = self.load_dat_file(gameinit_path, rom_name, "gameinit") {
                if !other_combined.is_empty() {
                    other_combined.push_str("\n\n");
                }
                other_combined.push_str("=== Game Init Info ===\n");
                other_combined.push_str(&text);
            }
        }
        
        self.other_text = other_combined;
    }
    
    /// Show the history panel
    pub fn show(&mut self, ui: &mut egui::Ui, _config: &AppConfig) {
        ui.group(|ui| {
            ui.heading("Game Information");
            ui.add_space(8.0); // Add some spacing after heading
            
            // Add tab selector if a game is selected
            if self.current_game.is_some() && !self.is_loading {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add_space(4.0); // Add left padding
                    if ui.selectable_label(matches!(self.selected_tab, HistoryTab::History), "History").clicked() {
                        self.selected_tab = HistoryTab::History;
                    }
                    ui.separator();
                    if ui.selectable_label(matches!(self.selected_tab, HistoryTab::MameInfo), "MAME Info").clicked() {
                        self.selected_tab = HistoryTab::MameInfo;
                    }
                    ui.separator();
                    if ui.selectable_label(matches!(self.selected_tab, HistoryTab::Other), "Other").clicked() {
                        self.selected_tab = HistoryTab::Other;
                    }
                    ui.add_space(4.0); // Add right padding
                });
                ui.separator();
                ui.add_space(4.0); // Add spacing after tabs
            }
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, true]) // Allow vertical shrinking but not horizontal
                .max_height(ui.available_height() - 50.0) // Use most of available height
                .show(ui, |ui| {
                    if self.is_loading {
                        ui.centered_and_justified(|ui| {
                            ui.label("Loading history data...");
                        });
                    } else if self.current_game.is_some() {
                        // Get the content for the selected tab
                        let content = match self.selected_tab {
                            HistoryTab::History => &self.history_text,
                            HistoryTab::MameInfo => &self.mameinfo_text,
                            HistoryTab::Other => &self.other_text,
                        };
                        
                        if content.is_empty() {
                            match self.selected_tab {
                                HistoryTab::History => {
                                    ui.label("No history information available for this game.");
                                    ui.label("Configure history.xml path in Directories settings.");
                                }
                                HistoryTab::MameInfo => {
                                    ui.label("No MAME info available for this game.");
                                    ui.label("Configure mameinfo.dat path in Directories settings.");
                                }
                                HistoryTab::Other => {
                                    ui.label("No additional information available for this game.");
                                    ui.label("Configure DAT file paths in Directories settings.");
                                }
                            }
                        } else {
                            // Display the content for the selected tab
                            // ENHANCED: Better text display with improved formatting
                            ui.add(egui::TextEdit::multiline(&mut content.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(20) // Show more rows by default
                                .font(egui::TextStyle::Monospace)
                                .text_color(ui.style().visuals.text_color()));
                        }
                    } else {
                        ui.label("Select a game to view its history and information.");
                    }
                });
        });
    }
    
    /// Load history from history.xml file
    fn load_history_xml(&mut self, path: &PathBuf, rom_name: &str) {
        // Check cache first
        let cache_key = format!("history_xml_{}", rom_name);
        if self.history_cache.contains_key(&cache_key) {
            return;
        }
        
        if !path.exists() {
            return;
        }
        
        // Try to read and parse the XML file
        match fs::read_to_string(path) {
            Ok(content) => {
                let mut reader = Reader::from_str(&content);
                reader.config_mut().trim_text(true);
                
                let mut buf = Vec::new();
                let mut in_entry = false;
                let mut in_systems = false;
                let mut in_software = false;
                let mut in_text = false;
                let mut found_game = false;
                let mut entry_text = String::new();
                let mut entry_count = 0;
                let mut system_count = 0;
                
                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Start(ref e)) => {
                            match e.name().as_ref() {
                                b"entry" => {
                                    in_entry = true;
                                    found_game = false;
                                    entry_text.clear();
                                    entry_count += 1;
                                }
                                b"systems" if in_entry => {
                                    in_systems = true;
                                }
                                b"software" if in_entry => {
                                    in_software = true;
                                }
                                b"system" if in_systems => {
                                    // Check if this system matches our game
                                    for attr in e.attributes() {
                                        if let Ok(attr) = attr {
                                            if attr.key.as_ref() == b"name" {
                                                if let Ok(value) = std::str::from_utf8(&attr.value) {
                                                    system_count += 1;
                                                    if value == rom_name {
                                                        found_game = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                b"item" if in_software => {
                                    // Check if this software item matches our game
                                    for attr in e.attributes() {
                                        if let Ok(attr) = attr {
                                            if attr.key.as_ref() == b"name" {
                                                if let Ok(value) = String::from_utf8(attr.value.to_vec()) {
                                                    if value == rom_name {
                                                        found_game = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                b"text" if in_entry && found_game => {
                                    in_text = true;
                                    entry_text.clear();
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::Text(e)) => {
                            if in_text {
                                // Decode the text properly
                                match reader.decoder().decode(&e) {
                                    Ok(text) => {
                                        // Check if we've hit the -CONTRIBUTE- marker
                                        if let Some(contribute_pos) = text.find("-CONTRIBUTE-") {
                                            // Only add text up to the marker
                                            entry_text.push_str(&text[..contribute_pos]);
                                            // We're done reading this entry
                                            in_text = false;
                                        } else {
                                            entry_text.push_str(&text);
                                        }
                                    }
                                    Err(_) => {
                                        // Fallback to lossy conversion
                                        let text = String::from_utf8_lossy(&e);
                                        // Check for -CONTRIBUTE- marker in lossy text too
                                        if let Some(contribute_pos) = text.find("-CONTRIBUTE-") {
                                            entry_text.push_str(&text[..contribute_pos]);
                                            in_text = false;
                                        } else {
                                            entry_text.push_str(&text);
                                        }
                                    }
                                }
                            }
                        }
                        Ok(Event::End(ref e)) => {
                            match e.name().as_ref() {
                                b"systems" => {
                                    in_systems = false;
                                }
                                b"software" => {
                                    in_software = false;
                                }
                                b"text" if in_text => {
                                    in_text = false;
                                }
                                b"entry" if in_entry && found_game => {
                                    // We found our game's entry
                                    let result = entry_text.trim().to_string();
                                    if !result.is_empty() {
                                        self.history_cache.insert(cache_key, result);
                                        return;
                                    }
                                }
                                b"entry" => {
                                    in_entry = false;
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::Empty(ref e)) => {
                            // Handle self-closing tags like <system name="..." />
                            match e.name().as_ref() {
                                b"system" if in_systems => {
                                    // Check if this system matches our game
                                    for attr in e.attributes() {
                                        if let Ok(attr) = attr {
                                            if attr.key.as_ref() == b"name" {
                                                if let Ok(value) = std::str::from_utf8(&attr.value) {
                                                    system_count += 1;
                                                    if value == rom_name {
                                                        found_game = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                b"item" if in_software => {
                                    // Check if this software item matches our game
                                    for attr in e.attributes() {
                                        if let Ok(attr) = attr {
                                            if attr.key.as_ref() == b"name" {
                                                if let Ok(value) = String::from_utf8(attr.value.to_vec()) {
                                                    if value == rom_name {
                                                        found_game = true;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Ok(Event::Eof) => break,
                        Err(e) => {
                            eprintln!("Error parsing history.xml at position {}: {:?}", reader.buffer_position(), e);
                            break;
                        }
                        _ => {}
                    }
                    buf.clear();
                }
            }
            Err(e) => {
                eprintln!("Error reading history.xml: {:?}", e);
            }
        }
    }
    
    /// Load information from a DAT file
    fn load_dat_file(&mut self, path: &PathBuf, rom_name: &str, dat_type: &str) -> Option<String> {
        // Check cache first
        let cache_key = format!("{}_{}", dat_type, rom_name);
        if let Some(cached) = self.history_cache.get(&cache_key) {
            return Some(cached.clone());
        }
        
        if !path.exists() {
            return None;
        }
        
        match fs::read_to_string(path) {
            Ok(content) => {
                // Try to find the game entry
                // Format: $info=romname (without .zip)
                let game_marker = format!("$info={}", rom_name);
                
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
                let game_marker_with_comma = format!("$info={},", rom_name);
                
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