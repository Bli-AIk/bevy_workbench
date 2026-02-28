//! Tiling panel system based on egui_tiles.
//!
//! Sets up a desktop editor layout using egui_tiles with the structure:
//! - Root: Vertical split (main area + bottom console)
//! - Main area: Horizontal split (game view + right inspector)

use bevy::prelude::*;
use std::collections::HashMap;

/// Trait for user-defined editor panels.
pub trait WorkbenchPanel: Send + Sync + 'static {
    /// Unique panel identifier.
    fn id(&self) -> &str;

    /// Panel title displayed on the tab.
    fn title(&self) -> String;

    /// Draw the panel UI.
    fn ui(&mut self, ui: &mut egui::Ui);

    /// Whether the panel tab can be closed (default: true).
    fn closable(&self) -> bool {
        true
    }
}

/// Identifies a panel in the tile tree.
pub type PanelId = usize;

/// Where a panel should be placed in the desktop layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelSlot {
    /// Right side (e.g. Inspector).
    Right,
    /// Bottom area (e.g. Console).
    Bottom,
    /// Center area (e.g. Game View).
    Center,
    /// Left side (optional user panels).
    Left,
}

/// A pane entry stored in the egui_tiles tree.
#[derive(Debug, Clone)]
pub struct PaneEntry {
    pub panel_id: PanelId,
}

/// Pending panel registration (before tree is built).
struct PendingPanel {
    panel: Box<dyn WorkbenchPanel>,
    slot: PanelSlot,
}

/// Resource holding the tile tree layout and registered panels.
#[derive(Resource, Default)]
pub struct TileLayoutState {
    pub tree: Option<egui_tiles::Tree<PaneEntry>>,
    pub panels: HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    pending: Vec<PendingPanel>,
    next_id: PanelId,
    tree_built: bool,
}

impl TileLayoutState {
    /// Register a panel. Auto-detects slot by panel ID convention.
    pub fn add_panel(&mut self, panel: Box<dyn WorkbenchPanel>) -> PanelId {
        let slot = match panel.id() {
            id if id.contains("inspector") => PanelSlot::Right,
            id if id.contains("console") || id.contains("timeline") => PanelSlot::Bottom,
            id if id.contains("game_view") => PanelSlot::Center,
            _ => PanelSlot::Left,
        };
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push(PendingPanel { panel, slot });
        // Store panel ID for later (actual panel moved into pending)
        id
    }

    /// Build the egui_tiles tree from pending panels.
    /// Layout: Vertical [ Horizontal [ left? | center | right ], bottom ]
    fn build_tree(&mut self) {
        if self.tree_built {
            return;
        }
        self.tree_built = true;

        let mut tiles = egui_tiles::Tiles::default();

        // Collect panels by slot
        let mut left_panes = Vec::new();
        let mut center_panes = Vec::new();
        let mut right_panes = Vec::new();
        let mut bottom_panes = Vec::new();

        for pending in self.pending.drain(..) {
            let id = self.panels.len();
            self.panels.insert(id, pending.panel);
            let tile_id = tiles.insert_pane(PaneEntry { panel_id: id });
            match pending.slot {
                PanelSlot::Left => left_panes.push(tile_id),
                PanelSlot::Center => center_panes.push(tile_id),
                PanelSlot::Right => right_panes.push(tile_id),
                PanelSlot::Bottom => bottom_panes.push(tile_id),
            }
        }

        // Build tab containers for each slot (always with tab headers for drag support)
        let left_tile = Self::make_tab(&mut tiles, left_panes);
        let center_tile = Self::make_tab(&mut tiles, center_panes);
        let right_tile = Self::make_tab(&mut tiles, right_panes);
        let bottom_tile = Self::make_tab(&mut tiles, bottom_panes);

        // Build main horizontal row: [left? | center | right?]
        let mut main_children = Vec::new();
        let mut main_shares = Vec::new();
        if let Some(left) = left_tile {
            main_children.push(left);
            main_shares.push((left, 1.0));
        }
        if let Some(center) = center_tile {
            main_children.push(center);
            main_shares.push((center, 4.0)); // center takes most space
        }
        if let Some(right) = right_tile {
            main_children.push(right);
            main_shares.push((right, 1.5));
        }

        let root = if main_children.is_empty() && bottom_tile.is_none() {
            // No panels at all
            self.tree = None;
            return;
        } else if main_children.is_empty() {
            // Only bottom panels
            bottom_tile.unwrap()
        } else {
            let main_row = if main_children.len() == 1 {
                main_children[0]
            } else {
                let row_id = tiles.insert_horizontal_tile(main_children);
                // Set shares for horizontal layout
                if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
                    tiles.get_mut(row_id)
                {
                    for (child, share) in &main_shares {
                        linear.shares.set_share(*child, *share);
                    }
                }
                row_id
            };

            if let Some(bottom) = bottom_tile {
                // Vertical split: main row on top, bottom panel below
                let root_id = tiles.insert_vertical_tile(vec![main_row, bottom]);
                if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
                    tiles.get_mut(root_id)
                {
                    linear.shares.set_share(main_row, 4.0);
                    linear.shares.set_share(bottom, 1.0);
                }
                root_id
            } else {
                main_row
            }
        };

        self.tree = Some(egui_tiles::Tree::new("workbench", root, tiles));
    }

    fn make_tab(
        tiles: &mut egui_tiles::Tiles<PaneEntry>,
        panes: Vec<egui_tiles::TileId>,
    ) -> Option<egui_tiles::TileId> {
        if panes.is_empty() {
            None
        } else {
            // Always wrap in a tab container so every slot has a draggable tab header.
            Some(tiles.insert_tab_tile(panes))
        }
    }
}

/// Adapter between egui_tiles::Behavior and our WorkbenchPanel system.
struct WorkbenchBehavior<'a> {
    panels: &'a mut HashMap<PanelId, Box<dyn WorkbenchPanel>>,
}

impl egui_tiles::Behavior<PaneEntry> for WorkbenchBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &PaneEntry) -> egui::WidgetText {
        self.panels
            .get(&pane.panel_id)
            .map(|p| p.title())
            .unwrap_or_else(|| "Unknown".to_string())
            .into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneEntry,
    ) -> egui_tiles::UiResponse {
        if let Some(panel) = self.panels.get_mut(&pane.panel_id) {
            panel.ui(ui);
        }
        egui_tiles::UiResponse::None
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

/// System that renders the tile layout using EguiContexts.
pub fn tiles_ui_system(mut contexts: bevy_egui::EguiContexts, mut state: ResMut<TileLayoutState>) {
    // Build tree on first frame (after all panels registered)
    state.build_tree();

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let state = &mut *state;
    let Some(ref mut tree) = state.tree else {
        return;
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut behavior = WorkbenchBehavior {
            panels: &mut state.panels,
        };
        tree.ui(&mut behavior, ui);
    });
}
