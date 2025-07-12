// src/rom_utils/mod.rs
use std::path::PathBuf;  // PERBAIKAN: Hapus 'Path' karena tidak digunakan
use std::collections::HashMap;
use crate::models::{Game, RomStatus};
use walkdir::WalkDir;

pub struct RomLoader {
    rom_dirs: Vec<PathBuf>,
}

impl RomLoader {
    pub fn new(rom_dirs: Vec<PathBuf>) -> Self {
        Self { rom_dirs }
    }

    /// Load ROM dengan scanning directories dan mencocokkan dengan metadata MAME
    /// Versi ini dioptimasi untuk menangani koleksi besar tanpa freezing
    pub fn load_roms(&self, metadata: HashMap<String, Game>) -> Vec<Game> {
        println!("Starting ROM scan with {} games in metadata", metadata.len());

        // Pertama, temukan semua file ROM di directories
        let rom_files = self.scan_rom_files();
        println!("Found {} ROM files in directories", rom_files.len());

        // Buat map untuk lookup lebih cepat (case-insensitive)
        let mut available_roms: HashMap<String, PathBuf> = HashMap::new();

        for (rom_path, rom_name) in rom_files {
            // Simpan dengan key lowercase untuk matching case-insensitive
            available_roms.insert(rom_name.to_lowercase(), rom_path);
        }

        // Cocokkan file ROM dengan game metadata
        let mut games = Vec::new();
        let mut found_count = 0;
        let mut missing_count = 0;

        // Process games dalam chunks untuk menghindari memory pressure
        for (game_name, mut game) in metadata {
            // Cek apakah kita punya ROM ini (case-insensitive)
            let rom_key = game_name.to_lowercase();

            if available_roms.contains_key(&rom_key) {
                game.status = RomStatus::Available;
                found_count += 1;
            } else {
                game.status = RomStatus::Missing;
                missing_count += 1;
            }

            games.push(game);
        }

        println!("ROM scan complete:");
        println!("  - Available: {} games", found_count);
        println!("  - Missing: {} games", missing_count);
        println!("  - Total: {} games", games.len());

        // Sort games berdasarkan description untuk display konsisten
        games.sort_by(|a, b| a.description.cmp(&b.description));

        games
    }

    /// Load ROM dengan progress callback
    /// Versi ini memungkinkan UI menampilkan progress updates
    pub fn load_roms_with_progress<F>(&self, metadata: HashMap<String, Game>, mut progress_callback: F) -> Vec<Game>
    where
    F: FnMut(usize, usize) + Send,
    {
        println!("Starting ROM scan with progress reporting");

        // Pertama, hitung total files untuk progress reporting
        let total_estimate = self.estimate_total_files();
        progress_callback(0, total_estimate);

        // Scan ROM files dengan progress updates
        let rom_files = self.scan_rom_files_with_progress(&mut progress_callback);
        println!("Found {} ROM files", rom_files.len());

        // Buat lookup map
        let mut available_roms: HashMap<String, PathBuf> = HashMap::new();
        for (rom_path, rom_name) in rom_files {
            available_roms.insert(rom_name.to_lowercase(), rom_path);
        }

        // Cocokkan dengan metadata
        let mut games = Vec::new();
        let mut found_count = 0;
        let mut missing_count = 0;

        let total_games = metadata.len();
        let mut processed = 0;

        for (game_name, mut game) in metadata {
            let rom_key = game_name.to_lowercase();

            if available_roms.contains_key(&rom_key) {
                game.status = RomStatus::Available;
                found_count += 1;
            } else {
                game.status = RomStatus::Missing;
                missing_count += 1;
            }

            games.push(game);

            // Update progress secara periodik
            processed += 1;
            if processed % 1000 == 0 {
                progress_callback(processed, total_games);
            }
        }

        // Final progress update
        progress_callback(total_games, total_games);

        println!("ROM matching complete: {} available, {} missing", found_count, missing_count);

        // Sort untuk display
        games.sort_by(|a, b| a.description.cmp(&b.description));

        games
    }

    /// Scan ROM directories dan return list dari (path, name) tuples
    fn scan_rom_files(&self) -> Vec<(PathBuf, String)> {
        let mut rom_files = Vec::new();

        // Common ROM file extensions
        let rom_extensions = ["zip", "7z", "chd"];

        for rom_dir in &self.rom_dirs {
            if !rom_dir.exists() {
                eprintln!("ROM directory does not exist: {}", rom_dir.display());
                continue;
            }

            if !rom_dir.is_dir() {
                eprintln!("ROM path is not a directory: {}", rom_dir.display());
                continue;
            }

            println!("Scanning ROM directory: {}", rom_dir.display());
            let mut count_in_dir = 0;

            // Gunakan WalkDir untuk scan directories secara rekursif
            // Set max_depth untuk mencegah terlalu dalam ke subdirectories
            for entry in WalkDir::new(rom_dir)
                .max_depth(3)  // Jangan terlalu dalam
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                {
                    let path = entry.path();

                    // Skip jika bukan file
                    if !path.is_file() {
                        continue;
                    }

                    // Cek apakah punya extension ROM
                    if let Some(extension) = path.extension() {
                        if let Some(ext_str) = extension.to_str() {
                            if rom_extensions.contains(&ext_str.to_lowercase().as_str()) {
                                // Extract nama ROM (filename tanpa extension)
                                if let Some(file_stem) = path.file_stem() {
                                    if let Some(name) = file_stem.to_str() {
                                        rom_files.push((path.to_path_buf(), name.to_string()));
                                        count_in_dir += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                println!("  Found {} ROM files in this directory", count_in_dir);
        }

        rom_files
    }

    /// Scan ROM files dengan progress reporting
    fn scan_rom_files_with_progress<F>(&self, progress_callback: &mut F) -> Vec<(PathBuf, String)>
    where
    F: FnMut(usize, usize) + Send,
    {
        let mut rom_files = Vec::new();
        let rom_extensions = ["zip", "7z", "chd"];
        let mut files_scanned = 0;

        for rom_dir in &self.rom_dirs {
            if !rom_dir.exists() || !rom_dir.is_dir() {
                continue;
            }

            for entry in WalkDir::new(rom_dir)
                .max_depth(3)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                {
                    let path = entry.path();

                    if path.is_file() {
                        files_scanned += 1;

                        // Report progress setiap 100 files
                        if files_scanned % 100 == 0 {
                            progress_callback(files_scanned, 0); // 0 berarti total tidak diketahui
                        }

                        if let Some(extension) = path.extension() {
                            if let Some(ext_str) = extension.to_str() {
                                if rom_extensions.contains(&ext_str.to_lowercase().as_str()) {
                                    if let Some(file_stem) = path.file_stem() {
                                        if let Some(name) = file_stem.to_str() {
                                            rom_files.push((path.to_path_buf(), name.to_string()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
        }

        rom_files
    }

    /// Estimasi total files untuk progress reporting
    fn estimate_total_files(&self) -> usize {
        let mut estimate = 0;

        for rom_dir in &self.rom_dirs {
            if rom_dir.exists() && rom_dir.is_dir() {
                // Quick estimation berdasarkan directory size
                if let Ok(entries) = std::fs::read_dir(rom_dir) {
                    estimate += entries.count() * 2; // Rough estimate termasuk subdirs
                }
            }
        }

        estimate.max(1000) // Minimum estimate untuk progress bar
    }
}

/// Debug function untuk analisa ROM directories
pub fn debug_rom_directories(rom_dirs: &[PathBuf]) {
    println!("\n=== ROM Directory Analysis ===");

    for (idx, dir) in rom_dirs.iter().enumerate() {
        println!("\nDirectory {}: {}", idx + 1, dir.display());

        if !dir.exists() {
            println!("  ERROR: Directory does not exist!");
            continue;
        }

        if !dir.is_dir() {
            println!("  ERROR: Path is not a directory!");
            continue;
        }

        // Hitung files berdasarkan extension
        let mut extension_counts: HashMap<String, usize> = HashMap::new();
        let mut total_files = 0;
        let mut total_size = 0u64;

        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    total_files += 1;

                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }

                    if let Some(ext) = entry.path().extension() {
                        if let Some(ext_str) = ext.to_str() {
                            *extension_counts.entry(ext_str.to_lowercase()).or_insert(0) += 1;
                        }
                    } else {
                        *extension_counts.entry("(no extension)".to_string()).or_insert(0) += 1;
                    }
                }
            }

            println!("  Total files: {}", total_files);
            println!("  Total size: {:.2} GB", total_size as f64 / 1_073_741_824.0);
            println!("  Files by extension:");

            // Sort berdasarkan count descending
            let mut ext_vec: Vec<_> = extension_counts.iter().collect();
            ext_vec.sort_by(|a, b| b.1.cmp(a.1));

            for (ext, count) in ext_vec.iter().take(10) {
                println!("    .{}: {} files", ext, count);
            }

            // Cek untuk common ROM archive formats
            let rom_count = extension_counts.get("zip").unwrap_or(&0) +
            extension_counts.get("7z").unwrap_or(&0) +
            extension_counts.get("chd").unwrap_or(&0);

            println!("  Likely ROM files: {}", rom_count);

            if rom_count == 0 {
                println!("  WARNING: No common ROM file types found!");
                println!("  Expected: .zip, .7z, or .chd files");
            }
    }

    println!("\n=== End of ROM Directory Analysis ===\n");
}
