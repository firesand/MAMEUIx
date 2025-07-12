use crate::models::Game;
use std::path::PathBuf;
use std::collections::HashMap;
use walkdir::WalkDir;

pub struct RomLoader {
    rom_dirs: Vec<PathBuf>,
}

impl RomLoader {
    pub fn new(rom_dirs: Vec<PathBuf>) -> Self {
        Self { rom_dirs }
    }

    pub fn load_roms(&self, metadata: HashMap<String, Game>) -> Vec<Game> {
        let mut games = Vec::new();

        for dir in &self.rom_dirs {
            if dir.exists() {
                for entry in WalkDir::new(dir).max_depth(1) {
                    if let Ok(entry) = entry {
                        if entry.path().extension().and_then(|s| s.to_str()) == Some("zip") {
                            if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                                if let Some(game) = metadata.get(name) {
                                    games.push(game.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        games
    }
}
