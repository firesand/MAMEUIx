use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterSettings {
    pub search_text: String,
    pub search_mode: SearchMode,  // New field for search mode
    pub category: Option<super::FilterCategory>,
    pub year_from: String,
    pub year_to: String,
    pub manufacturer: String,
    pub show_clones: bool,
    pub hide_non_games: bool,
    pub hide_mahjong: bool,
    pub hide_adult: bool,
    pub hide_casino: bool,
    pub show_favorites_only: bool,
    pub status_filter: StatusFilter,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SearchMode {
    GameTitle,
    Manufacturer,
    RomFileName,
    Year,
    Status,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::GameTitle
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StatusFilter {
    All,
    WorkingOnly,
    ImperfectOnly,
    NotWorkingOnly,
}

impl Default for StatusFilter {
    fn default() -> Self {
        StatusFilter::All
    }
}

impl Default for FilterSettings {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            search_mode: SearchMode::default(),
            category: Some(super::FilterCategory::All),
            year_from: String::new(),
            year_to: String::new(),
            manufacturer: String::new(),
            show_clones: false,
            hide_non_games: false,
            hide_mahjong: false,
            hide_adult: false,
            hide_casino: false,
            show_favorites_only: false,
            status_filter: StatusFilter::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortColumn {
    Title,
    RomName,
    Year,
    Manufacturer,
    Status,
    PlayCount,
    LastPlayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Default for SortColumn {
    fn default() -> Self {
        SortColumn::Title
    }
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}
