// src/mame/scanner.rs
use crate::models::Game;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use anyhow::{Result, Context};

pub struct GameScanner {
    mame_path: String,
}

impl GameScanner {
    pub fn new(mame_path: &str) -> Self {
        Self {
            mame_path: mame_path.to_string(),
        }
    }

    /// Scan games dari MAME menggunakan -listxml
    /// Versi ini dioptimasi untuk eksekusi background thread
    pub fn scan_games(&self) -> Result<Vec<Game>> {
        println!("GameScanner: Starting scan with MAME at: {}", self.mame_path);

        // Pertama, verifikasi MAME executable valid
        if !std::path::Path::new(&self.mame_path).exists() {
            return Err(anyhow::anyhow!("MAME executable not found at: {}", self.mame_path));
        }

        // Jalankan mame -listxml dan capture output
        println!("GameScanner: Running {} -listxml", self.mame_path);
        let start_time = std::time::Instant::now();

        let output = Command::new(&self.mame_path)
        .arg("-listxml")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to execute MAME")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("MAME -listxml failed: {}", stderr));
        }

        let elapsed = start_time.elapsed();
        println!("GameScanner: MAME -listxml completed in {:.2}s", elapsed.as_secs_f32());

        // Parse XML output
        let xml_str = String::from_utf8_lossy(&output.stdout);
        println!("GameScanner: Parsing XML data ({:.2} MB)", xml_str.len() as f32 / 1_048_576.0);

        let games = self.parse_xml(&xml_str)?;

        println!("GameScanner: Successfully parsed {} games", games.len());
        Ok(games)
    }

    /// Parse output XML MAME untuk extract informasi game
    /// Ini adalah parser sederhana - di production, gunakan XML library yang proper
    fn parse_xml(&self, xml_str: &str) -> Result<Vec<Game>> {
        let mut games = Vec::new();
        let mut current_pos = 0;
        // PERBAIKAN: Hapus variabel yang tidak digunakan
        // let xml_bytes = xml_str.as_bytes(); // TIDAK DIGUNAKAN

        // Process XML dalam chunks untuk menghindari memory pressure
        while let Some(machine_start) = xml_str[current_pos..].find("<machine ") {
            let machine_start = current_pos + machine_start;

            // Temukan akhir dari machine entry ini
            let machine_end = if let Some(end) = xml_str[machine_start..].find("</machine>") {
                machine_start + end + 10 // Include closing tag
            } else {
                break; // Tidak ada entry lengkap lagi
            };

            // Extract machine entry ini
            let entry = &xml_str[machine_start..machine_end];

            // Parse machine entry
            if let Some(game) = self.parse_machine_entry(entry) {
                games.push(game);
            }

            current_pos = machine_end;

            // Yield secara periodik untuk menghindari blocking terlalu lama
            if games.len() % 1000 == 0 {
                println!("GameScanner: Parsed {} games so far...", games.len());
            }
        }

        Ok(games)
    }

    /// Parse single machine entry dari XML
    fn parse_machine_entry(&self, entry: &str) -> Option<Game> {
        // Extract required attributes
        let name = Self::extract_attribute(entry, "name")?;

        // Skip devices dan BIOS entries secara default
        let is_device = entry.contains("isdevice=\"yes\"");
        let is_bios = entry.contains("isbios=\"yes\"");

        // Extract informasi dasar
        let description = Self::extract_tag(entry, "description")
        .unwrap_or_else(|| name.clone());
        let year = Self::extract_tag(entry, "year")
        .unwrap_or_else(|| "????".to_string());
        let manufacturer = Self::extract_tag(entry, "manufacturer")
        .unwrap_or_else(|| "Unknown".to_string());

        // Extract parent information untuk clones
        let parent = Self::extract_attribute(entry, "cloneof");

        // Extract driver information
        let driver_name = Self::extract_driver_name(entry);
        let _driver_status = Self::extract_driver_status(entry);
        
        // Extract source file (which often indicates the driver/system)
        let source_file = Self::extract_attribute(entry, "sourcefile")
            .unwrap_or_else(|| "unknown".to_string());

        // Detect CHD requirements from XML data
        let (requires_chd, chd_name) = Self::extract_chd_info(entry, &name, &description, &source_file);

        Some(Game {
            name: name.to_string(),
             description,
             manufacturer,
             year,
             driver: driver_name.unwrap_or_else(|| {
                 // Use source file name without extension as driver name
                 source_file.trim_end_matches(".cpp")
                     .trim_end_matches(".c")
                     .to_string()
             }),
             status: crate::models::RomStatus::Unknown, // Akan ditentukan oleh ROM scan
             parent: parent.clone(),
             category: Self::extract_category(entry).unwrap_or_else(|| "Misc.".to_string()),
             play_count: 0,
             is_clone: parent.is_some(),
             is_device,
             is_bios,
             controls: String::new(), // Bisa extract dari input tags
             requires_chd,
             chd_name,
        })
    }

    /// Extract nilai attribute dari XML text
    fn extract_attribute(text: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        let start = text.find(&pattern)?;
        let start = start + pattern.len();
        let end = text[start..].find('"')?;
        Some(text[start..start + end].to_string())
    }

    /// Extract content antara XML tags
    fn extract_tag(text: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        let start = text.find(&start_tag)?;
        let start = start + start_tag.len();
        let end = text[start..].find(&end_tag)?;

        Some(text[start..start + end].to_string())
    }

    /// Extract informasi driver status
    fn extract_driver_status(entry: &str) -> String {
        if let Some(driver_start) = entry.find("<driver ") {
            if let Some(driver_end) = entry[driver_start..].find("/>") {
                let driver_tag = &entry[driver_start..driver_start + driver_end];

                // Cek status attribute
                if let Some(status) = Self::extract_attribute(driver_tag, "status") {
                    return status;
                }
            }
        }

        "unknown".to_string()
    }

    /// Extract driver name from entry
    fn extract_driver_name(entry: &str) -> Option<String> {
        // Try to get driver name from driver tag
        if let Some(driver_start) = entry.find("<driver ") {
            if let Some(driver_end) = entry[driver_start..].find("/>") {
                let driver_tag = &entry[driver_start..driver_start + driver_end];
                
                // Some MAME versions include name attribute in driver tag
                if let Some(name) = Self::extract_attribute(driver_tag, "name") {
                    return Some(name);
                }
            }
        }
        
        None
    }

    /// Extract category from entry (if available in XML)
    fn extract_category(entry: &str) -> Option<String> {
        // Try to extract category from XML if present
        // Note: Standard MAME XML doesn't include category, this would come from catver.ini
        // But some custom MAME builds might include it
        if let Some(category) = Self::extract_tag(entry, "category") {
            return Some(category);
        }
        
        // Try to guess category from description or other fields
        let description = Self::extract_tag(entry, "description").unwrap_or_default().to_lowercase();
        
        // Simple categorization based on common patterns
        if description.contains("poker") || description.contains("slot") {
            Some("Casino".to_string())
        } else if description.contains("mahjong") {
            Some("Mahjong".to_string())
        } else if description.contains("quiz") {
            Some("Quiz".to_string())
        } else if description.contains("puzzle") {
            Some("Puzzle".to_string())
        } else if description.contains("fighter") || description.contains("boxing") {
            Some("Fighter".to_string())
        } else if description.contains("shoot") || description.contains("gun") {
            Some("Shooter".to_string())
        } else if description.contains("drive") || description.contains("racing") {
            Some("Driving".to_string())
        } else {
            None
        }
    }

    /// Extract CHD information from XML entry
    fn extract_chd_info(entry: &str, name: &str, description: &str, source_file: &str) -> (bool, Option<String>) {
        // PRIMARY METHOD: Check for CHD disk images in the XML - this is the most reliable method
        if entry.contains("<disk") {
            // Look for disk entries that indicate CHD files
            let mut chd_name = None;
            let mut current_pos = 0;
            
            while let Some(disk_start) = entry[current_pos..].find("<disk ") {
                let disk_start = current_pos + disk_start;
                
                // Find the end of this disk entry
                let disk_end = if let Some(end) = entry[disk_start..].find("/>") {
                    disk_start + end + 2
                } else if let Some(end) = entry[disk_start..].find("</disk>") {
                    disk_start + end + 7
                } else {
                    break;
                };
                
                let disk_entry = &entry[disk_start..disk_end];
                
                // Extract disk name attribute
                if let Some(name_start) = disk_entry.find("name=\"") {
                    let name_start = name_start + 6;
                    if let Some(name_end) = disk_entry[name_start..].find("\"") {
                        let disk_name = &disk_entry[name_start..name_start + name_end];
                        chd_name = Some(disk_name.to_string());
                        break; // Found the CHD disk name
                    }
                }
                
                current_pos = disk_end;
            }
            
            // If we found a disk entry, this is definitely a CHD game
            if chd_name.is_some() {
                return (true, chd_name);
            }
        }
        
        // SECONDARY METHOD: Check for harddisk device with CHD extension
        if entry.contains("<device type=\"harddisk\"") && entry.contains("<extension name=\"chd\"/>") {
            // This is a harddisk device that supports CHD files
            // Look for the disk name in the device section
            if let Some(disk_name) = Self::extract_disk_name_from_device(entry) {
                return (true, Some(disk_name));
            }
        }
        
        // TERTIARY METHOD: Very specific known CHD games - ONLY games that definitely require CHD
        // This list should be very small and only include games that are confirmed to use CHD files
        let known_chd_games = [
            // Killer Instinct series (confirmed CHD games)
            "kinst", "kinst2", 
            
            // Gauntlet series (confirmed CHD games)
            "gauntdl", "gauntleg", 
            
            // NFL Blitz series (confirmed CHD games)
            "blitz99", "blitz", "blitz2k", "blitz2k1", "blitz2k2", "blitz2k3", "blitz2k4",
            
            // Fisherman's Bait series (confirmed CHD games)
            "fbaitbc", "fbait2bc", 
            
            // Mortal Kombat 4/5 (confirmed CHD games)
            "mk4", "mk5", 
            
            // Mace: The Dark Age (confirmed CHD game)
            "mace", 
            
            // Hydro Thunder (confirmed CHD game)
            "hydro", 
            
            // Cruisin' series (confirmed CHD games)
            "crusnusa", "crusnwld", "crusnexo",
            
            // California Speed/Rush series (confirmed CHD games)
            "calspeed", "calrushi", "calrush2",
            
            // San Francisco Rush series (confirmed CHD games)
            "sfrush", "sf2049se", "rush2049",
            
            // Area 51 series (confirmed CHD games)
            "area51", "area51t", "area51mx",
            
            // Maximum Force (confirmed CHD game)
            "maxforce", "maxforc2",
            
            // War: Final Assault (confirmed CHD game)
            "warfa",
            
            // Vapor TRX (confirmed CHD game)
            "vaportrx",
            
            // Carnevil (confirmed CHD game)
            "carnevil",
            
            // The Grid (confirmed CHD game)
            "thegrid",
            
            // NBA Showtime (confirmed CHD game)
            "nbashowt",
            
            // NFL Blitz 2000 Gold (confirmed CHD game)
            "blitz2kg",
        ];
        
        if known_chd_games.contains(&name) {
            return (true, Some(name.to_string()));
        }
        
        // FOURTH METHOD: Very specific game patterns that indicate CHD usage
        // Only patterns that are 100% confirmed to indicate CHD usage
        let chd_game_patterns = [
            // Only very specific patterns that indicate CHD usage
            "gauntlet legends", "gauntlet dark legacy", 
            "blitz 99", "blitz 2000", "nfl blitz", "nba showtime", 
            "mortal kombat 4", "mortal kombat 5", "mace the dark age",
            "fisherman's bait", "bass challenge", "hydro thunder",
            "crusin' usa", "crusin' world", "crusin' exotica",
            "california speed", "california rush",
            "san francisco rush", "rush 2049", "rush 2",
            "area 51", "maximum force", "war final assault",
            "vapor trx", "carnevil", "the grid",
        ];
        
        let description_lower = description.to_lowercase();
        for pattern in &chd_game_patterns {
            if description_lower.contains(pattern) {
                return (true, Some(name.to_string()));
            }
        }
        
        // FIFTH METHOD: System-based detection (most restrictive)
        // Only systems that definitely use CHD files AND have specific characteristics
        let chd_systems = [
            // Only systems that definitely use CHD files
            "naomi", "atomiswave", "cps3", "tgm2", "konamigx", "konamigv",
            "segasaturn", "psx", "n64", "dreamcast", "gd-rom",
            "jaguar", "midtunit",
        ];
        
        let source_lower = source_file.to_lowercase();
        for system in &chd_systems {
            if source_lower.contains(system) {
                // Additional check to avoid false positives
                // Only mark as CHD if it's a known CHD system AND has specific characteristics
                if Self::is_likely_chd_system(source_file, description) {
                    return (true, Some(name.to_string()));
                }
            }
        }
        
        (false, None)
    }
    
    /// Extract disk name from device section
    fn extract_disk_name_from_device(entry: &str) -> Option<String> {
        // Look for disk name in the device section
        if let Some(disk_start) = entry.find("<disk ") {
            if let Some(name_start) = entry[disk_start..].find("name=\"") {
                let name_start = disk_start + name_start + 6;
                if let Some(name_end) = entry[name_start..].find("\"") {
                    return Some(entry[name_start..name_start + name_end].to_string());
                }
            }
        }
        None
    }
    
    /// Check if a system is likely to use CHD files
    fn is_likely_chd_system(source_file: &str, description: &str) -> bool {
        // Additional validation to avoid false positives
        let source_lower = source_file.to_lowercase();
        let desc_lower = description.to_lowercase();
        
        // Specific checks for known CHD systems
        if source_lower.contains("jaguar") {
            // Jaguar games that use CHD are typically light gun games
            return desc_lower.contains("area 51") || 
                   desc_lower.contains("maximum force") || 
                   desc_lower.contains("war final assault") ||
                   desc_lower.contains("vapor trx");
        }
        
        if source_lower.contains("midtunit") {
            // Midway T-Unit games that use CHD are typically sports/fighting games
            return desc_lower.contains("blitz") || 
                   desc_lower.contains("nba showtime") || 
                   desc_lower.contains("gauntlet") ||
                   desc_lower.contains("mace") ||
                   desc_lower.contains("carnevil") ||
                   desc_lower.contains("the grid");
        }
        
        // For other systems, be very restrictive
        if source_lower.contains("naomi") || source_lower.contains("atomiswave") {
            // Only specific Naomi/Atomiswave games use CHD
            return desc_lower.contains("naomi") || desc_lower.contains("atomiswave");
        }
        
        false
    }
}

/// Scanner alternatif yang menggunakan streaming untuk instalasi MAME sangat besar
pub struct StreamingGameScanner {
    mame_path: String,
}

impl StreamingGameScanner {
    pub fn new(mame_path: &str) -> Self {
        Self {
            mame_path: mame_path.to_string(),
        }
    }

    /// Scan games menggunakan pendekatan streaming untuk minimize memory usage
    /// Ini lebih baik untuk sistem dengan RAM terbatas atau MAME set sangat besar
    pub fn scan_games_streaming(&self) -> Result<Vec<Game>> {
        let mut games = Vec::new();

        // Start MAME process dengan piped output
        let mut child = Command::new(&self.mame_path)
        .arg("-listxml")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start MAME")?;

        // Baca output line by line
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut current_machine = String::new();
            let mut in_machine = false;

            for line in reader.lines() {
                let line = line.context("Failed to read MAME output")?;

                if line.contains("<machine ") {
                    in_machine = true;
                    current_machine.clear();
                }

                if in_machine {
                    current_machine.push_str(&line);
                    current_machine.push('\n');
                }

                if line.contains("</machine>") && in_machine {
                    in_machine = false;

                    // Parse complete machine entry
                    let scanner = GameScanner::new(&self.mame_path);
                    if let Some(game) = scanner.parse_machine_entry(&current_machine) {
                        games.push(game);
                    }

                    // Report progress
                    if games.len() % 1000 == 0 {
                        println!("Streaming scanner: {} games processed", games.len());
                    }
                }
            }
        }

        // Wait for process to complete
        let status = child.wait().context("Failed to wait for MAME")?;
        if !status.success() {
            return Err(anyhow::anyhow!("MAME exited with error"));
        }

        Ok(games)
    }
}
