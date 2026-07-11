//! Session state for the redesign shell — kept separate from legacy UI state.

use std::collections::HashSet;

use crate::models::RomStatus;
use crate::ui::panels::artwork_loader::ArtworkLoader;
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedesignScreen {
    #[default]
    Library,
    Detail,
    Verification,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedesignNavTab {
    Library,
    Verification,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedesignCollection {
    #[default]
    AllGames,
    Available,
    Favorites,
    Missing,
    Issues,
}

impl RedesignCollection {
    /// Match the ROM states represented by this collection.
    ///
    /// Favorites are selected by the shared filter engine rather than ROM state.
    /// Missing sets are deliberately excluded from Issues so the two collections
    /// remain useful, disjoint views.
    pub fn matches_status(self, status: RomStatus) -> bool {
        match self {
            Self::AllGames | Self::Favorites => true,
            Self::Available => matches!(status, RomStatus::Available),
            Self::Missing => matches!(status, RomStatus::Missing),
            Self::Issues => matches!(
                status,
                RomStatus::ChdMissing
                    | RomStatus::ChdRequired
                    | RomStatus::Incorrect
                    | RomStatus::NotWorking
                    | RomStatus::Preliminary
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    Directories,
    Appearance,
    Performance,
    Shaders,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct YearDecade {
    start: u16,
}

impl YearDecade {
    pub const FILTER_CHOICES: [Self; 7] = [
        Self { start: 1960 },
        Self { start: 1970 },
        Self { start: 1980 },
        Self { start: 1990 },
        Self { start: 2000 },
        Self { start: 2010 },
        Self { start: 2020 },
    ];

    pub fn label(self) -> String {
        format!("{}s", self.start)
    }

    pub fn range(self) -> (u16, u16) {
        (self.start, self.start.saturating_add(9))
    }

    pub fn from_year(year: u16) -> Option<Self> {
        Self::FILTER_CHOICES.iter().copied().find(|decade| {
            let (start, end) = decade.range();
            (start..=end).contains(&year)
        })
    }
}

pub struct RedesignState {
    pub screen: RedesignScreen,
    pub detail_game_index: Option<usize>,
    pub collection: RedesignCollection,
    pub selected_manufacturer: Option<String>,
    pub year_decade: Option<YearDecade>,
    pub chd_only: bool,
    pub manufacturer_open: bool,
    pub year_open: bool,
    /// The library sidebar becomes an on-demand drawer on narrow windows.
    pub narrow_sidebar_open: bool,
    pub expanded_parents: HashSet<String>,
    pub settings_section: SettingsSection,
    /// One-shot request consumed by whichever library search field is visible.
    pub search_focus_requested: bool,
    pub style_applied: bool,
    /// Legacy style snapshot restored when leaving the redesign shell.
    pub previous_style: Option<egui::Style>,
    /// Cached flattened row indices for the game table (rebuilt when filters change).
    pub table_rows: Vec<TableRow>,
    pub table_rows_dirty: bool,
    /// Cached sidebar stats (rebuilt when game list changes).
    pub sidebar_stats: SidebarStats,
    pub sidebar_stats_dirty: bool,
    /// Local search buffer — debounced before applying to filter_settings.
    pub search_text_buf: String,
    search_buf_initialized: bool,
    pub search_debounce_deadline: Option<f64>,
    /// Avoid re-syncing filter_settings when nothing changed.
    filter_fingerprint: FilterFingerprint,
    /// Texture cache for artwork rendered by the redesign shell.
    pub artwork_loader: ArtworkLoader,
}

#[derive(Default, Clone, PartialEq, Eq)]
struct FilterFingerprint {
    collection: RedesignCollection,
    chd_only: bool,
    manufacturer: Option<String>,
    year_decade: Option<YearDecade>,
    search_text: String,
}

#[derive(Default, Clone)]
pub struct SidebarStats {
    pub all: usize,
    pub available: usize,
    pub favorites: usize,
    pub missing: usize,
    pub issues: usize,
    pub chd_count: usize,
    pub manufacturers: Vec<(String, usize)>,
    pub decades: Vec<YearDecade>,
    pub games_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableRow {
    Parent { index: usize, clone_count: usize },
    Clone { index: usize },
}

impl Default for RedesignState {
    fn default() -> Self {
        Self {
            screen: RedesignScreen::default(),
            detail_game_index: None,
            collection: RedesignCollection::default(),
            selected_manufacturer: None,
            year_decade: None,
            chd_only: false,
            manufacturer_open: false,
            year_open: false,
            narrow_sidebar_open: false,
            expanded_parents: HashSet::new(),
            settings_section: SettingsSection::default(),
            search_focus_requested: false,
            style_applied: false,
            previous_style: None,
            table_rows: Vec::new(),
            table_rows_dirty: true,
            sidebar_stats: SidebarStats::default(),
            sidebar_stats_dirty: true,
            search_text_buf: String::new(),
            search_buf_initialized: false,
            search_debounce_deadline: None,
            filter_fingerprint: FilterFingerprint::default(),
            artwork_loader: ArtworkLoader::new(),
        }
    }
}

impl RedesignState {
    pub fn active_nav_tab(&self) -> RedesignNavTab {
        match self.screen {
            RedesignScreen::Library | RedesignScreen::Detail => RedesignNavTab::Library,
            RedesignScreen::Verification => RedesignNavTab::Verification,
            RedesignScreen::Settings => RedesignNavTab::Settings,
        }
    }

    pub fn open_detail(&mut self, game_index: usize) {
        self.detail_game_index = Some(game_index);
        self.screen = RedesignScreen::Detail;
    }

    pub fn back_to_library(&mut self) {
        self.screen = RedesignScreen::Library;
    }

    pub fn navigate_to(&mut self, tab: RedesignNavTab) {
        self.screen = match tab {
            RedesignNavTab::Library => RedesignScreen::Library,
            RedesignNavTab::Verification => RedesignScreen::Verification,
            RedesignNavTab::Settings => RedesignScreen::Settings,
        };
    }

    pub fn mark_table_dirty(&mut self) {
        self.table_rows_dirty = true;
    }

    pub fn mark_sidebar_stats_dirty(&mut self) {
        self.sidebar_stats_dirty = true;
    }

    pub fn ensure_search_buf(&mut self, current: &str) {
        if !self.search_buf_initialized {
            self.search_text_buf = current.to_string();
            self.search_buf_initialized = true;
        }
    }

    pub fn request_search_focus(&mut self) {
        self.search_focus_requested = true;
    }

    pub fn take_search_focus_request(&mut self) -> bool {
        std::mem::take(&mut self.search_focus_requested)
    }

    pub fn invalidate_on_games_loaded(&mut self, games_len: usize) {
        if self.sidebar_stats.games_len != games_len {
            self.mark_table_dirty();
            self.mark_sidebar_stats_dirty();
            self.filter_fingerprint = FilterFingerprint::default();
        }
    }

    pub fn mark_filters_synced(
        &mut self,
        collection: RedesignCollection,
        chd_only: bool,
        manufacturer: &Option<String>,
        year_decade: Option<YearDecade>,
        search_text: &str,
    ) {
        self.filter_fingerprint = FilterFingerprint {
            collection,
            chd_only,
            manufacturer: manufacturer.clone(),
            year_decade,
            search_text: search_text.to_string(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{RedesignCollection, RedesignState, YearDecade};
    use crate::models::RomStatus;

    #[test]
    fn filter_decades_are_fixed_to_1960s_through_2020s() {
        let sixties = YearDecade::from_year(1965).expect("valid four-digit year");
        let twenties = YearDecade::from_year(2029).expect("valid filter year");

        assert_eq!(sixties.label(), "1960s");
        assert_eq!(sixties.range(), (1960, 1969));
        assert_eq!(twenties.label(), "2020s");
        assert_eq!(twenties.range(), (2020, 2029));
        assert_eq!(
            YearDecade::FILTER_CHOICES.map(|decade| decade.label()),
            [
                "1960s", "1970s", "1980s", "1990s", "2000s", "2010s", "2020s"
            ]
        );
        assert_eq!(YearDecade::from_year(1959), None);
        assert_eq!(YearDecade::from_year(2030), None);
    }

    #[test]
    fn missing_and_issues_are_disjoint_status_collections() {
        assert!(RedesignCollection::Missing.matches_status(RomStatus::Missing));
        assert!(!RedesignCollection::Issues.matches_status(RomStatus::Missing));

        for status in [
            RomStatus::ChdMissing,
            RomStatus::ChdRequired,
            RomStatus::Incorrect,
            RomStatus::NotWorking,
            RomStatus::Preliminary,
        ] {
            assert!(RedesignCollection::Issues.matches_status(status));
            assert!(!RedesignCollection::Missing.matches_status(status));
        }

        assert!(!RedesignCollection::Issues.matches_status(RomStatus::Available));
        assert!(!RedesignCollection::Issues.matches_status(RomStatus::Unknown));
    }

    #[test]
    fn search_focus_request_is_consumed_once() {
        let mut state = RedesignState::default();

        assert!(!state.take_search_focus_request());
        state.request_search_focus();
        assert!(state.take_search_focus_request());
        assert!(!state.take_search_focus_request());
    }
}
