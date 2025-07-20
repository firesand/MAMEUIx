// src/ui/theme.rs
// Theme management untuk warna-warna UI MAMEUIx

use eframe::egui;

/// Tema warna untuk MAMEUIx UI
#[derive(Debug, Clone)]
pub struct UiTheme {
    // Folder dan direktori
    pub folder_icon_color: egui::Color32,
    pub directory_text_color: egui::Color32,
    
    // Kategori dan filter
    pub hidden_category_color: egui::Color32,
    pub category_text_color: egui::Color32,
    pub filter_active_color: egui::Color32,
    
    // Tombol aksi
    pub play_button_color: egui::Color32,
    pub play_button_hover_color: egui::Color32,
    pub gamepad_icon_color: egui::Color32,
    
    // Ikon properti dan status
    pub properties_icon_color: egui::Color32,
    pub warning_icon_color: egui::Color32,
    pub info_icon_color: egui::Color32,
    
    // Tombol refresh dan update
    pub refresh_icon_color: egui::Color32,
    pub refresh_button_color: egui::Color32,
    
    // Status game
    pub working_status_color: egui::Color32,
    pub not_working_status_color: egui::Color32,
    pub missing_status_color: egui::Color32,
    pub available_status_color: egui::Color32,
    
    // Favorit
    pub favorite_star_color: egui::Color32,
    pub favorite_star_hover_color: egui::Color32,
    
    // Background dan accent
    pub accent_color: egui::Color32,
    pub secondary_accent_color: egui::Color32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            // Folder icons - kuning
            folder_icon_color: egui::Color32::from_rgb(255, 215, 0), // Gold
            directory_text_color: egui::Color32::from_rgb(255, 200, 0), // Darker gold
            
            // Hidden categories - merah
            hidden_category_color: egui::Color32::from_rgb(220, 53, 69), // Red
            category_text_color: egui::Color32::from_rgb(200, 200, 200), // Light gray
            filter_active_color: egui::Color32::from_rgb(0, 123, 255), // Blue
            
            // Play game button - hijau dengan gamepad merah
            play_button_color: egui::Color32::from_rgb(40, 167, 69), // Green
            play_button_hover_color: egui::Color32::from_rgb(30, 157, 59), // Darker green
            gamepad_icon_color: egui::Color32::from_rgb(220, 53, 69), // Red
            
            // Properties dan warning - kuning
            properties_icon_color: egui::Color32::from_rgb(255, 193, 7), // Yellow
            warning_icon_color: egui::Color32::from_rgb(255, 193, 7), // Yellow
            info_icon_color: egui::Color32::from_rgb(23, 162, 184), // Cyan
            
            // Refresh button - biru
            refresh_icon_color: egui::Color32::from_rgb(0, 123, 255), // Blue
            refresh_button_color: egui::Color32::from_rgb(0, 113, 245), // Darker blue
            
            // Status colors
            working_status_color: egui::Color32::from_rgb(40, 167, 69), // Green
            not_working_status_color: egui::Color32::from_rgb(220, 53, 69), // Red
            missing_status_color: egui::Color32::from_rgb(255, 193, 7), // Yellow
            available_status_color: egui::Color32::from_rgb(40, 167, 69), // Green
            
            // Favorite star - kuning
            favorite_star_color: egui::Color32::from_rgb(255, 215, 0), // Gold
            favorite_star_hover_color: egui::Color32::from_rgb(255, 193, 7), // Yellow
            
            // Accent colors
            accent_color: egui::Color32::from_rgb(0, 123, 255), // Blue
            secondary_accent_color: egui::Color32::from_rgb(108, 117, 125), // Gray
        }
    }
}

impl UiTheme {
    /// Membuat tema dengan warna kustom
    pub fn custom() -> Self {
        Self::default()
    }
    
    /// Mendapatkan warna untuk status game
    pub fn get_status_color(&self, status: &crate::models::RomStatus) -> egui::Color32 {
        match status {
            crate::models::RomStatus::Available => self.available_status_color,
            crate::models::RomStatus::NotWorking => self.not_working_status_color,
            crate::models::RomStatus::Missing => self.missing_status_color,
            crate::models::RomStatus::Incorrect => self.missing_status_color,
            crate::models::RomStatus::Preliminary => self.working_status_color,
            crate::models::RomStatus::ChdRequired => self.working_status_color,
            crate::models::RomStatus::ChdMissing => self.missing_status_color,
            crate::models::RomStatus::Unknown => self.secondary_accent_color,
        }
    }
    
    /// Mendapatkan warna untuk kategori
    pub fn get_category_color(&self, is_hidden: bool) -> egui::Color32 {
        if is_hidden {
            self.hidden_category_color
        } else {
            self.category_text_color
        }
    }
    
    /// Mendapatkan warna untuk tombol play
    pub fn get_play_button_color(&self, is_hovered: bool) -> egui::Color32 {
        if is_hovered {
            self.play_button_hover_color
        } else {
            self.play_button_color
        }
    }
    
    /// Mendapatkan warna untuk star favorit
    pub fn get_favorite_color(&self, is_hovered: bool) -> egui::Color32 {
        if is_hovered {
            self.favorite_star_hover_color
        } else {
            self.favorite_star_color
        }
    }
}

/// Helper functions untuk styling UI elements
pub struct ThemeHelper;

impl ThemeHelper {
    /// Styling untuk folder icon dengan warna kuning
    pub fn folder_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.folder_icon_color, "📁")
    }
    
    /// Styling untuk hidden category symbol dengan warna merah
    pub fn hidden_category_symbol(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.hidden_category_color, "🔒")
    }
    
    /// Styling untuk play game button dengan warna hijau
    pub fn play_game_button(ui: &mut egui::Ui, theme: &UiTheme, text: &str) -> egui::Response {
        let button = egui::Button::new(text)
            .fill(theme.play_button_color);
        
        let response = ui.add(button);
        
        // Change color on hover
        if response.hovered() {
            ui.colored_label(theme.play_button_hover_color, text);
        }
        
        response
    }
    
    /// Styling untuk gamepad icon dengan warna merah
    pub fn gamepad_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.gamepad_icon_color, "🎮")
    }
    
    /// Styling untuk properties icon dengan warna kuning
    pub fn properties_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.properties_icon_color, "⚙️")
    }
    
    /// Styling untuk warning icon dengan warna kuning
    pub fn warning_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.warning_icon_color, "⚠️")
    }
    
    /// Styling untuk info icon dengan warna kuning
    pub fn info_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.info_icon_color, "ℹ️")
    }
    
    /// Styling untuk refresh icon dengan warna biru
    pub fn refresh_icon(ui: &mut egui::Ui, theme: &UiTheme) -> egui::Response {
        ui.colored_label(theme.refresh_icon_color, "🔄")
    }
    
    /// Styling untuk favorite star dengan warna kuning
    pub fn favorite_star(ui: &mut egui::Ui, theme: &UiTheme, is_favorite: bool) -> egui::Response {
        let color = if is_favorite {
            theme.favorite_star_color
        } else {
            theme.secondary_accent_color
        };
        
        let response = ui.colored_label(color, "★");
        
        // Change color on hover
        if response.hovered() {
            ui.colored_label(theme.favorite_star_hover_color, "★");
        }
        
        response
    }
    
    /// Styling untuk status text dengan warna yang sesuai
    pub fn status_text(ui: &mut egui::Ui, theme: &UiTheme, status: &crate::models::RomStatus) -> egui::Response {
        let color = theme.get_status_color(status);
        let text = match status {
            crate::models::RomStatus::Available => "Available",
            crate::models::RomStatus::NotWorking => "Not Working",
            crate::models::RomStatus::Missing => "Missing",
            crate::models::RomStatus::Incorrect => "Incorrect",
            crate::models::RomStatus::Preliminary => "Preliminary",
            crate::models::RomStatus::ChdRequired => "CHD Required",
            crate::models::RomStatus::ChdMissing => "CHD Missing",
            crate::models::RomStatus::Unknown => "Unknown",
        };
        
        ui.colored_label(color, text)
    }
    
    /// Styling untuk category text dengan warna yang sesuai
    pub fn category_text(ui: &mut egui::Ui, theme: &UiTheme, text: &str, is_hidden: bool) -> egui::Response {
        let color = theme.get_category_color(is_hidden);
        ui.colored_label(color, text)
    }
    
    /// Styling untuk directory text dengan warna kuning
    pub fn directory_text(ui: &mut egui::Ui, theme: &UiTheme, text: &str) -> egui::Response {
        ui.colored_label(theme.directory_text_color, text)
    }
    
    /// Styling untuk filter active indicator
    pub fn filter_active_indicator(ui: &mut egui::Ui, theme: &UiTheme, text: &str) -> egui::Response {
        ui.colored_label(theme.filter_active_color, text)
    }
} 