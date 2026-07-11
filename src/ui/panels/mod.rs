// src/ui/panels/mod.rs
// Main UI panels and components

pub mod artwork_loader;
pub mod artwork_panel;
pub mod game_index_manager;
pub mod game_list;
pub mod game_list_view;
pub mod history_panel;
pub mod icon_manager;
pub mod icon_performance_monitor;
pub mod performance_manager;
pub mod sidebar;
pub mod software_list_panel;

pub use artwork_panel::ArtworkPanel;
pub use game_index_manager::GameIndexManager;
pub use game_list::GameList;
pub use game_list_view::GameListView;
pub use history_panel::HistoryPanel;
pub use icon_manager::IconManager;
pub use performance_manager::PerformanceManager;
pub use sidebar::Sidebar;
pub use software_list_panel::SoftwareListPanel;
