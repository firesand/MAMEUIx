use crate::app::MameApp;
use eframe::egui;
use egui_dock::{DockState, NodeIndex, Style, TabViewer};

/// Dockable panel identifiers for the main MAMEUIx layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockTab {
    Sidebar,
    GameList,
    SoftwareLists,
    Artwork,
    History,
}

impl DockTab {
    pub fn title(self) -> &'static str {
        match self {
            DockTab::Sidebar => "Filters",
            DockTab::GameList => "Games",
            DockTab::SoftwareLists => "Software Lists",
            DockTab::Artwork => "Artwork",
            DockTab::History => "History",
        }
    }
}

pub fn create_default_layout() -> DockState<DockTab> {
    let mut dock_state = DockState::new(vec![DockTab::GameList, DockTab::SoftwareLists]);
    let surface = dock_state.main_surface_mut();
    surface.split_left(NodeIndex::root(), 0.22, vec![DockTab::Sidebar]);
    surface.split_right(
        NodeIndex::root(),
        0.72,
        vec![DockTab::Artwork, DockTab::History],
    );
    dock_state
}

pub struct MameTabViewer<'a> {
    pub ctx: &'a egui::Context,
    pub app: &'a mut MameApp,
}

impl TabViewer for MameTabViewer<'_> {
    type Tab = DockTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            DockTab::Sidebar => self.app.render_sidebar_panel(ui),
            DockTab::GameList => self.app.render_game_list_panel(ui, self.ctx),
            DockTab::SoftwareLists => self.app.render_software_list_panel(ui),
            DockTab::Artwork => self.app.render_artwork_panel(ui),
            DockTab::History => self.app.render_history_panel(ui),
        }
    }
}

pub fn dock_style(ui: &egui::Ui) -> Style {
    Style::from_egui(ui.style().as_ref())
}
