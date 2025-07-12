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
