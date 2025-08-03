// src/ini_utils/mod.rs
use std::collections::HashSet;
use std::path::Path;
use std::fs;

/// Represents a parsed MAME INI file with section headers
#[derive(Debug, Clone)]
pub struct MameIniFile {
    pub sections: HashSet<String>,          // Normalized section headers (without brackets)
}

impl MameIniFile {
    /// Parse an INI file and extract section headers (normalized, no brackets)
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut section_names = HashSet::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            
            // Check if this is a section header (starts and ends with brackets)
            if line.starts_with('[') && line.ends_with(']') {
                // Extract the section name without brackets
                let section_name = &line[1..line.len()-1];
                let normalized = Self::normalize_name(section_name);
                section_names.insert(normalized);
            }
        }
        
        Ok(section_names)
    }

    /// Normalize a name for comparison (lowercase, trim, replace special chars)
    fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .replace(['(', ')', '[', ']', '\'', '"', '.', ','], "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Filter for excluding hardware names from game list based on INI files
#[derive(Debug, Clone)]
pub struct MameIniFilter {
    cpu_sections: HashSet<String>,
    device_sections: HashSet<String>,
    sound_sections: HashSet<String>,
    hide_cpu_names: bool,
    hide_device_names: bool,
    hide_sound_names: bool,
}

impl MameIniFilter {
    /// Normalize a name for comparison (lowercase, trim, replace special chars)
    fn normalize_name(name: &str) -> String {
        name.to_lowercase()
            .replace(['(', ')', '[', ']', '\'', '"', '.', ','], "")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Create a new MameIniFilter with the specified INI files and filter options
    pub fn new(
        cpu_ini_path: Option<&str>,
        device_ini_path: Option<&str>,
        sound_ini_path: Option<&str>,
        hide_cpu_names: bool,
        hide_device_names: bool,
        hide_sound_names: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let cpu_sections = if let Some(path) = cpu_ini_path {
            if hide_cpu_names {
                let set = MameIniFile::parse(path)?;
                println!("[INI Filter] Loaded {} CPU sections from {}", set.len(), path);
                set
            } else { HashSet::new() }
        } else { HashSet::new() };
        
        let device_sections = if let Some(path) = device_ini_path {
            if hide_device_names {
                let set = MameIniFile::parse(path)?;
                println!("[INI Filter] Loaded {} Device sections from {}", set.len(), path);
                set
            } else { HashSet::new() }
        } else { HashSet::new() };
        
        let sound_sections = if let Some(path) = sound_ini_path {
            if hide_sound_names {
                let set = MameIniFile::parse(path)?;
                println!("[INI Filter] Loaded {} Sound sections from {}", set.len(), path);
                set
            } else { HashSet::new() }
        } else { HashSet::new() };

        Ok(Self {
            cpu_sections,
            device_sections,
            sound_sections,
            hide_cpu_names,
            hide_device_names,
            hide_sound_names,
        })
    }

    /// Check if a ROM name should be filtered out based on INI file sections
    pub fn should_filter_out(&self, rom_name: &str) -> bool {
        let normalized_rom = Self::normalize_name(rom_name);
        // Debug output: print the normalized game name and first 5 section headers

        if self.hide_cpu_names {
            let cpu_preview: Vec<_> = self.cpu_sections.iter().take(5).collect();
            println!("  CPU sections (first 5): {:?}", cpu_preview);
        }
        if self.hide_device_names {
            let device_preview: Vec<_> = self.device_sections.iter().take(5).collect();
            println!("  Device sections (first 5): {:?}", device_preview);
        }
        if self.hide_sound_names {
            let sound_preview: Vec<_> = self.sound_sections.iter().take(5).collect();
            println!("  Sound sections (first 5): {:?}", sound_preview);
        }
        // Check for exact match with CPU section
        if self.hide_cpu_names && self.cpu_sections.contains(&normalized_rom) {
            println!("  [MATCH] CPU section");
            return true;
        }
        // Check for exact match with Device section
        if self.hide_device_names && self.device_sections.contains(&normalized_rom) {
            println!("  [MATCH] Device section");
            return true;
        }
        // Check for exact match with Sound section
        if self.hide_sound_names && self.sound_sections.contains(&normalized_rom) {
            println!("  [MATCH] Sound section");
            return true;
        }
        println!("  [NO MATCH]");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(MameIniFilter::normalize_name("3DO DSPP"), "3do dspp");
        assert_eq!(MameIniFilter::normalize_name("ADChips SE3208"), "adchips se3208");
        assert_eq!(MameIniFilter::normalize_name("Fujitsu MB89363B I/O"), "fujitsu mb89363b io");
    }

    #[test]
    fn test_filtering() {
        let filter = MameIniFilter::new(None, None, None, false, false, false).unwrap();
        
        // With no INI files loaded, nothing should be filtered
        assert!(!filter.should_filter_out("pacman"));
        assert!(!filter.should_filter_out("donkeykong"));
    }
}