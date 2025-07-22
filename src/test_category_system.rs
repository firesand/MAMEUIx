use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// Simple version of CategoryLoader for testing
struct CategoryLoader {
    categories: HashMap<String, String>,
}

impl CategoryLoader {
    fn new(catver_path: &Path) -> Result<Self, std::io::Error> {
        let mut categories = HashMap::new();
        
        if catver_path.exists() {
            let file = File::open(catver_path)?;
            let reader = BufReader::new(file);
            let mut in_category_section = false;
            
            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                
                // Skip empty lines and comments
                if trimmed.is_empty() || trimmed.starts_with(';') {
                    continue;
                }
                
                // Check for section headers
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_category_section = trimmed == "[Category]";
                    continue;
                }
                
                // Parse category entries
                if in_category_section {
                    if let Some(equals_pos) = trimmed.find('=') {
                        let game_name = trimmed[..equals_pos].trim();
                        let category = trimmed[equals_pos + 1..].trim().to_string();
                        // Store with lowercase key for case-insensitive lookup
                        categories.insert(game_name.to_lowercase(), category);
                    }
                }
            }
        }
        
        Ok(Self { categories })
    }
    
    fn get_category(&self, game_name: &str) -> Option<&String> {
        self.categories.get(&game_name.to_lowercase())
    }
    
    fn get_category_with_parent(&self, game_name: &str, parent_name: Option<&str>) -> Option<&String> {
        // First try the game name
        if let Some(category) = self.get_category(game_name) {
            return Some(category);
        }
        
        // If not found and we have a parent, try the parent name
        if let Some(parent) = parent_name {
            if let Some(category) = self.get_category(parent) {
                return Some(category);
            }
        }
        
        None
    }
}

fn main() {
    println!("Testing Category System...");
    
    // Test with the actual catver.ini file if it exists
    let catver_path = Path::new("pS_CatVer_277/catver.ini");
    let loader_opt = if catver_path.exists() {
        CategoryLoader::new(catver_path).ok()
    } else {
        println!("catver.ini not found at {:?}", catver_path);
        None
    };
    
    if let Some(loader) = loader_opt {
        println!("✓ Successfully loaded {} categories", loader.categories.len());
        
        // Test some known games
        let test_games = ["1944", "1944d", "1944j", "1944u", "pacman", "sf2"];
        
        for game_name in &test_games {
            if let Some(category) = loader.get_category(game_name) {
                println!("✓ {} -> {}", game_name, category);
            } else {
                println!("✗ {} -> No category found", game_name);
            }
        }
        
        // Test clone category inheritance
        println!("\nTesting clone category inheritance:");
        let clone_tests = [
            ("1944d", Some("1944")),
            ("1944j", Some("1944")),
            ("1944u", Some("1944")),
        ];
        
        for (clone_name, parent_name) in &clone_tests {
            if let Some(category) = loader.get_category_with_parent(clone_name, *parent_name) {
                println!("✓ {} (parent: {}) -> {}", clone_name, parent_name.unwrap_or("None"), category);
            } else {
                println!("✗ {} (parent: {}) -> No category found", clone_name, parent_name.unwrap_or("None"));
            }
        }
    } else {
        println!("✗ Failed to load categories");
    }
}