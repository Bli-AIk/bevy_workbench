//! Dockable panel system based on egui_dock.

use bevy::prelude::*;
use egui_dock::{DockArea, Style, TabViewer};
use std::collections::HashMap;

/// Trait for user-defined editor panels.
pub trait WorkbenchPanel: Send + Sync + 'static {
    /// Unique panel identifier.
    fn id(&self) -> &str;

    /// Panel title displayed on the tab.
    fn title(&self) -> String;

    /// Draw the panel UI.
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World);

    /// Whether the panel tab can be closed (default: true).
    fn closable(&self) -> bool {
        true
    }
}

/// Identifies a panel in the dock tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PanelId(pub usize);

/// Resource holding the dock tree layout and registered panels.
#[derive(Resource)]
pub struct DockLayoutState {
    pub tree: egui_dock::DockState<PanelId>,
    panels: HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    next_id: usize,
}

impl Default for DockLayoutState {
    fn default() -> Self {
        Self {
            tree: egui_dock::DockState::new(vec![]),
            panels: HashMap::new(),
            next_id: 0,
        }
    }
}

impl DockLayoutState {
    /// Register a new panel and add it to the dock tree.
    pub fn add_panel(&mut self, panel: Box<dyn WorkbenchPanel>) -> PanelId {
        let id = PanelId(self.next_id);
        self.next_id += 1;
        self.panels.insert(id, panel);
        self.tree.push_to_focused_leaf(id);
        id
    }
}

/// Adapter between egui_dock::TabViewer and our WorkbenchPanel system.
struct WorkbenchTabViewer<'a> {
    panels: &'a mut HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    world: &'a mut World,
}

impl TabViewer for WorkbenchTabViewer<'_> {
    type Tab = PanelId;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.panels
            .get(tab)
            .map(|p| p.title())
            .unwrap_or_else(|| "Unknown".to_string())
            .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if let Some(panel) = self.panels.get_mut(tab) {
            panel.ui(ui, self.world);
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        self.panels.get(tab).map(|p| p.closable()).unwrap_or(true)
    }
}

/// System that renders the dock area. Uses exclusive world access.
pub fn dock_ui_system(world: &mut World) {
    let Some(mut dock) = world.remove_resource::<DockLayoutState>() else {
        return;
    };

    if dock.panels.is_empty() {
        world.insert_resource(dock);
        return;
    }

    // Get the egui context from the primary camera
    let mut ctx_query =
        world.query_filtered::<&mut bevy_egui::EguiContext, With<bevy_egui::PrimaryEguiContext>>();
    let ctx = ctx_query
        .iter_mut(world)
        .next()
        .map(|mut c| c.get_mut().clone());

    if let Some(ctx) = ctx {
        egui::CentralPanel::default().show(&ctx, |ui| {
            let mut viewer = WorkbenchTabViewer {
                panels: &mut dock.panels,
                world,
            };
            DockArea::new(&mut dock.tree)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut viewer);
        });
    } else {
        bevy::log::warn_once!("dock_ui_system: no PrimaryEguiContext found ({} panels registered)", dock.panels.len());
    }

    world.insert_resource(dock);
}
