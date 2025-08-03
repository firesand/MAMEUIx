use eframe::egui;
use std::collections::HashSet;
use crate::models::filters::CategoryManager;

pub struct HiddenCategoriesDialog;

impl HiddenCategoriesDialog {
    pub fn show(
        ctx: &egui::Context, 
        hidden_categories: &mut HashSet<String>, 
        category_manager: Option<&CategoryManager>,
        open: &mut bool
    ) {
        let mut close = false;
        
        egui::Window::new("Manage Hidden Categories")
            .default_size([600.0, 500.0])
            .open(open)
            .show(ctx, |ui| {
                ui.label("Select categories to hide from the game list:");
                ui.label("Hidden games will not appear in any filter view.");
                ui.separator();
                
                if let Some(manager) = category_manager {
                    // Search box for categories
                    let search_id = ui.id().with("hidden_cat_search");
                    let mut search_text = ui.data_mut(|d| 
                        d.get_temp::<String>(search_id).unwrap_or_default()
                    );
                    
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        if ui.text_edit_singleline(&mut search_text).changed() {
                            ui.data_mut(|d| d.insert_temp(search_id, search_text.clone()));
                        }
                        if ui.button("Clear").clicked() {
                            search_text.clear();
                            ui.data_mut(|d| d.insert_temp(search_id, search_text.clone()));
                        }
                    });
                    
                    ui.separator();
                    
                    // Quick actions
                    ui.horizontal(|ui| {
                        if ui.button("Hide All Casino/Gambling").clicked() {
                            for (name, category) in &manager.categories {
                                let name_lower = category.display_name.to_lowercase();
                                if name_lower.contains("casino") || 
                                   name_lower.contains("gambling") ||
                                   name_lower.contains("cards") && name_lower.contains("mature") {
                                    hidden_categories.insert(category.display_name.clone());
                                }
                            }
                        }
                        
                        if ui.button("Hide All Mature").clicked() {
                            for (name, category) in &manager.categories {
                                if category.display_name.to_lowercase().contains("mature") {
                                    hidden_categories.insert(category.display_name.clone());
                                }
                            }
                        }
                        
                        if ui.button("Clear All").clicked() {
                            hidden_categories.clear();
                        }
                    });
                    
                    ui.separator();
                    
                    // Statistics
                    let total_hidden_games: usize = manager.categories.values()
                        .filter(|cat| hidden_categories.contains(&cat.display_name))
                        .map(|cat| cat.game_count)
                        .sum();
                    
                    ui.label(format!(
                        "Hidden: {} categories, approximately {} games",
                        hidden_categories.len(),
                        total_hidden_games
                    ));
                    
                    ui.separator();
                    
                    // Category list with checkboxes
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let search_lower = search_text.to_lowercase();
                            
                            // Group by letter
                            for (letter, categories) in manager.get_categories_by_letter() {
                                let has_visible = if search_lower.is_empty() {
                                    true
                                } else {
                                    categories.iter()
                                        .any(|cat| cat.display_name.to_lowercase().contains(&search_lower))
                                };
                                
                                if has_visible {
                                    ui.label(egui::RichText::new(format!("─── {} ───", letter))
                                        .color(egui::Color32::from_rgb(150, 150, 150)));
                                    
                                    for category in categories {
                                        if search_lower.is_empty() || 
                                           category.display_name.to_lowercase().contains(&search_lower) {
                                            let mut is_hidden = hidden_categories.contains(&category.display_name);
                                            
                                            ui.horizontal(|ui| {
                                                if ui.checkbox(&mut is_hidden, &category.display_name).changed() {
                                                    if is_hidden {
                                                        hidden_categories.insert(category.display_name.clone());
                                                    } else {
                                                        hidden_categories.remove(&category.display_name);
                                                    }
                                                }
                                                
                                                ui.label(format!("({} games)", category.game_count));
                                                
                                                // Show warning for common categories
                                                let name_lower = category.display_name.to_lowercase();
                                                if name_lower.contains("casino") || 
                                                   name_lower.contains("gambling") {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(255, 200, 100),
                                                        "🎰"
                                                    );
                                                }
                                            });
                                        }
                                    }
                                    
                                    ui.add_space(5.0);
                                }
                            }
                        });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("No category data available.");
                        ui.label("Please configure catver.ini in Directories settings.");
                    });
                }
                
                ui.separator();
                
                // Dialog buttons
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        close = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        close = true;
                        // In a real implementation, we might want to restore the original state
                    }
                });
            });
        
        if close {
            *open = false;
        }
    }
} 