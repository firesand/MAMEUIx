use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Loads game categories from a catver.ini file
#[derive(Clone, Debug)]
pub struct CategoryLoader {
    pub categories: HashMap<String, String>,
}

impl CategoryLoader {
    /// Creates a new CategoryLoader and loads categories from the specified file
    pub fn new(catver_path: &Path) -> Result<Self, std::io::Error> {
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
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("catver.ini not found at: {:?}", catver_path)
            ));
        }
        
        Ok(Self { categories })
    }
    
    /// Gets the category for a game by its name (case-insensitive)
    pub fn get_category(&self, game_name: &str) -> Option<&str> {
        self.categories.get(&game_name.to_lowercase()).map(|s| s.as_str())
    }
    
    /// Gets the category for a game, trying both the game name and its parent name
    /// This is useful for clones where the parent ROM has the category defined
    pub fn get_category_with_parent(&self, game_name: &str, parent_name: Option<&str>) -> Option<&str> {
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
    
    /// Returns the number of loaded categories
    pub fn len(&self) -> usize {
        self.categories.len()
    }
    
    /// Returns true if no categories are loaded
    pub fn is_empty(&self) -> bool {
        self.categories.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_load_categories() {
        // Create a temporary catver.ini file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "[FOLDER_SETTINGS]").unwrap();
        writeln!(temp_file, "RootFolderIcon mame").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "[Category]").unwrap();
        writeln!(temp_file, "1942=Shooter / Flying Vertical").unwrap();
        writeln!(temp_file, "pacman=Maze / Collect").unwrap();
        writeln!(temp_file, "sf2=Fighter / Versus").unwrap();
        writeln!(temp_file, "").unwrap();
        writeln!(temp_file, "[ROOT_FOLDER]").unwrap();
        temp_file.flush().unwrap();
        
        // Load categories
        let loader = CategoryLoader::new(temp_file.path()).unwrap();
        
        // Test loaded categories
        assert_eq!(loader.len(), 3);
        assert_eq!(loader.get_category("1942"), Some("Shooter / Flying Vertical"));
        assert_eq!(loader.get_category("pacman"), Some("Maze / Collect"));
        assert_eq!(loader.get_category("sf2"), Some("Fighter / Versus"));
        assert_eq!(loader.get_category("unknown"), None);
    }
    
    #[test]
    fn test_missing_file() {
        let loader = CategoryLoader::new(Path::new("/nonexistent/catver.ini")).unwrap();
        assert!(loader.is_empty());
    }
    
    #[test]
    fn test_real_catver_file() {
        // Test with the actual catver.ini file if it exists
        let catver_path = Path::new("pS_CatVer_277/catver.ini");
        if catver_path.exists() {
            match CategoryLoader::new(catver_path) {
                Ok(loader) => {
                    // Test some known games
                    assert_eq!(loader.get_category("1942"), Some("Shooter / Flying Vertical"));
                    assert_eq!(loader.get_category("pacman"), Some("Maze / Collect"));
                    assert_eq!(loader.get_category("sf2"), Some("Fighter / Versus"));
                    
                    // Test that we have a reasonable number of categories
                    assert!(loader.len() > 1000, "Expected many categories, got {}", loader.len());
                }
                Err(e) => {
                    panic!("Failed to load real catver.ini: {}", e);
                }
            }
        }
    }
}