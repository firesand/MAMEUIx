// This file contains fixes for category display and filtering issues

// Fix 1: Ensure category column is visible by default
// In src/models/filters.rs, update VisibleColumns::default()

// Fix 2: Make category filtering case-insensitive
// In src/ui/main_window.rs, update the category filter logic

// Fix 3: Add debugging to category loading
// In src/mame/category_loader.rs, add more detailed logging

// Fix 4: Ensure categories are properly displayed
// In src/ui/game_list.rs, show "Misc." for empty categories

use std::collections::HashMap;

pub fn debug_category_loading(catver_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Category Loading Debug ===");
    println!("catver.ini path: {:?}", catver_path);
    println!("File exists: {}", catver_path.exists());
    
    if catver_path.exists() {
        let content = std::fs::read_to_string(catver_path)?;
        let lines: Vec<&str> = content.lines().collect();
        println!("Total lines in file: {}", lines.len());
        
        // Find [Category] section
        let mut in_category_section = false;
        let mut category_count = 0;
        let mut sample_categories = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if trimmed == "[Category]" {
                in_category_section = true;
                println!("Found [Category] section at line {}", i + 1);
                continue;
            }
            
            if in_category_section && trimmed.starts_with('[') && trimmed.ends_with(']') {
                println!("Category section ends at line {}", i + 1);
                break;
            }
            
            if in_category_section && trimmed.contains('=') {
                category_count += 1;
                if sample_categories.len() < 10 {
                    sample_categories.push(trimmed.to_string());
                }
            }
        }
        
        println!("\nTotal categories found: {}", category_count);
        println!("\nSample categories:");
        for cat in sample_categories {
            println!("  {}", cat);
        }
    }
    
    Ok(())
}

pub fn test_category_matching() {
    println!("\n=== Category Matching Test ===");
    
    // Test exact match
    let category1 = "Shooter / Flying Vertical";
    let category2 = "Shooter / Flying Vertical";
    println!("Exact match test: {} == {} -> {}", category1, category2, category1 == category2);
    
    // Test case sensitivity
    let category3 = "shooter / flying vertical";
    println!("Case test: {} == {} -> {}", category1, category3, category1.to_lowercase() == category3.to_lowercase());
    
    // Test trimming
    let category4 = " Shooter / Flying Vertical ";
    println!("Trim test: {} == {} -> {}", category1, category4.trim(), category1 == category4.trim());
}