// src/hardware_filter.rs
// Module for filtering games by hardware (CPU, Device, Sound)

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::fs;
use std::io::{self, BufRead, BufReader};

/// Manages hardware filtering for games based on CPU, Device, and Sound INI files
pub struct HardwareFilter {
    /// Maps CPU names to sets of game names that use that CPU
    cpu_to_games: HashMap<String, HashSet<String>>,
    /// Maps Device names to sets of game names that use that device
    device_to_games: HashMap<String, HashSet<String>>,
    /// Maps Sound chip names to sets of game names that use that sound chip
    sound_to_games: HashMap<String, HashSet<String>>,
    
    /// Reverse mappings for quick lookup
    game_to_cpus: HashMap<String, Vec<String>>,
    game_to_devices: HashMap<String, Vec<String>>,
    game_to_sounds: HashMap<String, Vec<String>>,
}

impl HardwareFilter {
    /// Create a new empty HardwareFilter
    pub fn new() -> Self {
        Self {
            cpu_to_games: HashMap::new(),
            device_to_games: HashMap::new(),
            sound_to_games: HashMap::new(),
            game_to_cpus: HashMap::new(),
            game_to_devices: HashMap::new(),
            game_to_sounds: HashMap::new(),
        }
    }
    
    /// Load hardware mappings from INI files
    pub fn load_from_ini_files(
        cpu_ini_path: Option<&Path>,
        device_ini_path: Option<&Path>,
        sound_ini_path: Option<&Path>,
    ) -> io::Result<Self> {
        let mut filter = Self::new();
        
        if let Some(path) = cpu_ini_path {
            filter.load_ini_file(path, HardwareType::Cpu)?;
        }
        
        if let Some(path) = device_ini_path {
            filter.load_ini_file(path, HardwareType::Device)?;
        }
        
        if let Some(path) = sound_ini_path {
            filter.load_ini_file(path, HardwareType::Sound)?;
        }
        
        Ok(filter)
    }
    
    /// Load a single INI file
    fn load_ini_file(&mut self, path: &Path, hardware_type: HardwareType) -> io::Result<()> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        
        let mut current_section = None;
        
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                continue;
            }
            
            // Check if this is a section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = Some(trimmed[1..trimmed.len()-1].to_string());
            } else if let Some(section) = &current_section {
                // This is a game name under the current section
                let game_name = trimmed.to_string();
                
                match hardware_type {
                    HardwareType::Cpu => {
                        self.cpu_to_games
                            .entry(section.clone())
                            .or_insert_with(HashSet::new)
                            .insert(game_name.clone());
                        
                        self.game_to_cpus
                            .entry(game_name)
                            .or_insert_with(Vec::new)
                            .push(section.clone());
                    }
                    HardwareType::Device => {
                        self.device_to_games
                            .entry(section.clone())
                            .or_insert_with(HashSet::new)
                            .insert(game_name.clone());
                        
                        self.game_to_devices
                            .entry(game_name)
                            .or_insert_with(Vec::new)
                            .push(section.clone());
                    }
                    HardwareType::Sound => {
                        self.sound_to_games
                            .entry(section.clone())
                            .or_insert_with(HashSet::new)
                            .insert(game_name.clone());
                        
                        self.game_to_sounds
                            .entry(game_name)
                            .or_insert_with(Vec::new)
                            .push(section.clone());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a game uses a specific CPU (case-insensitive partial match)
    pub fn game_uses_cpu(&self, game_name: &str, cpu_search: &str) -> bool {
        let cpu_search_lower = cpu_search.to_lowercase();
        
        if let Some(cpus) = self.game_to_cpus.get(game_name) {
            cpus.iter().any(|cpu| cpu.to_lowercase().contains(&cpu_search_lower))
        } else {
            false
        }
    }
    
    /// Check if a game uses a specific device (case-insensitive partial match)
    pub fn game_uses_device(&self, game_name: &str, device_search: &str) -> bool {
        let device_search_lower = device_search.to_lowercase();
        
        if let Some(devices) = self.game_to_devices.get(game_name) {
            devices.iter().any(|device| device.to_lowercase().contains(&device_search_lower))
        } else {
            false
        }
    }
    
    /// Check if a game uses a specific sound chip (case-insensitive partial match)
    pub fn game_uses_sound(&self, game_name: &str, sound_search: &str) -> bool {
        let sound_search_lower = sound_search.to_lowercase();
        
        if let Some(sounds) = self.game_to_sounds.get(game_name) {
            sounds.iter().any(|sound| sound.to_lowercase().contains(&sound_search_lower))
        } else {
            false
        }
    }
    
    /// Get all unique CPU types
    pub fn get_all_cpus(&self) -> Vec<String> {
        let mut cpus: Vec<String> = self.cpu_to_games.keys().cloned().collect();
        cpus.sort();
        cpus
    }
    
    /// Get all unique device types
    pub fn get_all_devices(&self) -> Vec<String> {
        let mut devices: Vec<String> = self.device_to_games.keys().cloned().collect();
        devices.sort();
        devices
    }
    
    /// Get all unique sound chip types
    pub fn get_all_sounds(&self) -> Vec<String> {
        let mut sounds: Vec<String> = self.sound_to_games.keys().cloned().collect();
        sounds.sort();
        sounds
    }
    
    /// Get hardware info for a specific game
    pub fn get_game_hardware(&self, game_name: &str) -> GameHardware {
        GameHardware {
            cpus: self.game_to_cpus.get(game_name).cloned().unwrap_or_default(),
            devices: self.game_to_devices.get(game_name).cloned().unwrap_or_default(),
            sounds: self.game_to_sounds.get(game_name).cloned().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameHardware {
    pub cpus: Vec<String>,
    pub devices: Vec<String>,
    pub sounds: Vec<String>,
}

enum HardwareType {
    Cpu,
    Device,
    Sound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_cpu_ini() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[Z80]").unwrap();
        writeln!(temp_file, "pacman").unwrap();
        writeln!(temp_file, "galaga").unwrap();
        writeln!(temp_file, "[68000]").unwrap();
        writeln!(temp_file, "sf2").unwrap();
        
        let mut filter = HardwareFilter::new();
        filter.load_ini_file(temp_file.path(), HardwareType::Cpu).unwrap();
        
        assert!(filter.game_uses_cpu("pacman", "Z80"));
        assert!(filter.game_uses_cpu("pacman", "z80")); // Case insensitive
        assert!(filter.game_uses_cpu("sf2", "68000"));
        assert!(!filter.game_uses_cpu("pacman", "68000"));
    }
}