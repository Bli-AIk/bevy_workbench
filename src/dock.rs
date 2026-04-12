//! Tiling panel system based on egui_tiles.
//!
//! Sets up a desktop editor layout using egui_tiles with the structure:
//! - Root: Vertical split (main area + bottom console)
//! - Main area: Horizontal split (game view + right inspector)

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

mod persistence;
mod ui;

pub use ui::{LayoutPath, tiles_ui_system};

/// Snapshot of the layout for undo/redo (tree + tile mapping).
#[derive(Clone)]
pub(crate) struct LayoutSnapshot {
    pub tree: egui_tiles::Tree<PaneEntry>,
    pub panel_tile_map: HashMap<PanelId, egui_tiles::TileId>,
}

/// Undo action that restores a layout snapshot.
/// Uses Mutex for interior mutability since UndoAction takes &self.
pub(crate) struct LayoutUndoAction {
    before: Mutex<LayoutSnapshot>,
    after: Mutex<LayoutSnapshot>,
    desc: String,
}

impl LayoutUndoAction {
    pub fn new(desc: impl Into<String>, before: LayoutSnapshot, after: LayoutSnapshot) -> Self {
        Self {
            before: Mutex::new(before),
            after: Mutex::new(after),
            desc: desc.into(),
        }
    }
}

impl crate::undo::UndoAction for LayoutUndoAction {
    fn undo(&self, world: &mut World) {
        let snapshot = self.before.lock().unwrap().clone();
        world
            .resource_mut::<TileLayoutState>()
            .restore_snapshot(snapshot);
    }

    fn redo(&self, world: &mut World) {
        let snapshot = self.after.lock().unwrap().clone();
        world
            .resource_mut::<TileLayoutState>()
            .restore_snapshot(snapshot);
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

/// Trait for user-defined editor panels.
pub trait WorkbenchPanel: Send + Sync + std::any::Any + 'static {
    /// Unique panel identifier.
    fn id(&self) -> &str;

    /// Panel title displayed on the tab.
    fn title(&self) -> String;

    /// Draw the panel UI (no ECS access).
    fn ui(&mut self, ui: &mut egui::Ui);

    /// Draw the panel UI with ECS world access (for inspector, etc.).
    /// Default implementation delegates to `ui()`.
    fn ui_world(&mut self, ui: &mut egui::Ui, _world: &mut World) {
        self.ui(ui);
    }

    /// Whether this panel needs World access in `ui_world`.
    fn needs_world(&self) -> bool {
        false
    }

    /// Background color for this panel's tile. Return `None` for transparent.
    fn bg_color(&self) -> Option<egui::Color32> {
        Some(egui::Color32::from_rgb(35, 35, 40))
    }

    /// Whether the panel tab can be closed (default: true).
    fn closable(&self) -> bool {
        true
    }

    /// Whether the panel is visible in the default layout (default: true).
    fn default_visible(&self) -> bool {
        true
    }

    /// Whether this panel appears in the built-in Window menu (default: true).
    /// Return `false` to hide the panel from the Window menu, useful when the panel
    /// is managed via a custom top-level menu instead.
    fn show_in_window_menu(&self) -> bool {
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PaneEntry {
    pub panel_id: PanelId,
}

/// Pending panel registration (before tree is built).
#[allow(dead_code)]
struct PendingPanel {
    panel: Box<dyn WorkbenchPanel>,
    slot: PanelSlot,
    visible: bool,
}

/// Resource holding the tile tree layout and registered panels.
#[derive(Resource, Default)]
pub struct TileLayoutState {
    pub tree: Option<egui_tiles::Tree<PaneEntry>>,
    pub panels: HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    pending: Vec<PendingPanel>,
    next_id: PanelId,
    tree_built: bool,
    /// Maps panel string IDs to PanelIds for lookup.
    pub(crate) panel_id_map: HashMap<String, PanelId>,
    /// Maps PanelIds to TileIds in the tree (for visibility control).
    pub(crate) panel_tile_map: HashMap<PanelId, egui_tiles::TileId>,
    /// Set by menu to request layout reset to default.
    pub(crate) layout_reset_requested: bool,
    /// Path to save layout to (set via file dialog).
    pub(crate) layout_save_path: Option<std::path::PathBuf>,
    /// Path to load layout from (set via file dialog).
    pub(crate) layout_load_path: Option<std::path::PathBuf>,
    /// Panels requested to open (processed in exclusive system with undo recording).
    pub(crate) pending_open_requests: Vec<String>,
    /// Panel IDs hidden from the Window menu at runtime (overrides `show_in_window_menu()`).
    window_menu_hidden: HashSet<String>,
    /// Panel IDs that should be hidden in the default layout
    /// (overrides `default_visible()` when building tree from scratch).
    default_hidden: HashSet<String>,
}

impl TileLayoutState {
    /// Register a panel. Auto-detects slot by panel ID convention.
    pub fn add_panel(&mut self, panel: Box<dyn WorkbenchPanel>) -> PanelId {
        let slot = match panel.id() {
            id if id.contains("inspector") => PanelSlot::Right,
            id if id.contains("console") || id.contains("timeline") => PanelSlot::Bottom,
            id if id.contains("game_view") || id.contains("preview") => PanelSlot::Center,
            _ => PanelSlot::Left,
        };
        let visible = panel.default_visible();
        let id = self.next_id;
        self.next_id += 1;
        self.pending.push(PendingPanel {
            panel,
            slot,
            visible,
        });
        // Store panel ID for later (actual panel moved into pending)
        id
    }

    /// Build the egui_tiles tree from pending panels.
    /// Tries to load from `layout_path` first; falls back to default layout.
    fn build_tree(&mut self, layout_path: Option<&std::path::Path>) {
        if self.tree_built {
            return;
        }

        // Move panels from pending into the panels map (needed before load_layout)
        for pending in self.pending.drain(..) {
            let id = self.panels.len();
            let str_id = pending.panel.id().to_string();
            self.panels.insert(id, pending.panel);
            self.panel_id_map.insert(str_id, id);
        }

        // Try loading saved layout
        if let Some(path) = layout_path
            && self.load_layout(path)
        {
            return;
        }

        // Fall through to default layout
        self.tree_built = true;
        self.build_default_tree();
    }

    /// Build the default layout from panel slots.
    fn build_default_tree(&mut self) {
        let mut tiles = egui_tiles::Tiles::default();

        // Collect panels by slot
        let mut left_panes = Vec::new();
        let mut center_panes = Vec::new();
        let mut right_panes = Vec::new();
        let mut bottom_panes = Vec::new();

        // Collect visible panels with slot info, sorted by ID for deterministic order
        let mut visible_panels: Vec<(&str, PanelId)> = self
            .panel_id_map
            .iter()
            .filter(|(s, pid)| {
                self.panels[pid].default_visible() && !self.default_hidden.contains(s.as_str())
            })
            .map(|(s, pid)| (s.as_str(), *pid))
            .collect();
        visible_panels.sort_by_key(|(s, _)| *s);

        for &(str_id, panel_id) in &visible_panels {
            let slot = match str_id {
                id if id.contains("inspector") => PanelSlot::Right,
                id if id.contains("console") || id.contains("timeline") => PanelSlot::Bottom,
                id if id.contains("game_view") || id.contains("preview") => PanelSlot::Center,
                _ => PanelSlot::Left,
            };
            let tile_id = tiles.insert_pane(PaneEntry { panel_id });
            self.panel_tile_map.insert(panel_id, tile_id);
            match slot {
                PanelSlot::Left => left_panes.push(tile_id),
                PanelSlot::Center => center_panes.push(tile_id),
                PanelSlot::Right => right_panes.push(tile_id),
                PanelSlot::Bottom => bottom_panes.push(tile_id),
            }
        }

        // Sort center panes: non-game_view tabs before game_view (first tab is active by default)
        center_panes.sort_by_key(|tile_id| {
            if let Some(egui_tiles::Tile::Pane(pane)) = tiles.get(*tile_id) {
                visible_panels
                    .iter()
                    .any(|&(s, pid)| pid == pane.panel_id && s.contains("game_view"))
            } else {
                false
            }
        });

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
                ui::set_linear_shares(&mut tiles, row_id, &main_shares);
                row_id
            };

            if let Some(bottom) = bottom_tile {
                // Vertical split: main row on top, bottom panel below
                let root_id = tiles.insert_vertical_tile(vec![main_row, bottom]);
                ui::set_linear_shares(&mut tiles, root_id, &[(main_row, 4.0), (bottom, 1.0)]);
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

    /// Focus an existing panel tab by its string ID, re-inserting if closed.
    pub fn open_or_focus_panel(&mut self, panel_str_id: &str) {
        let Some(&panel_id) = self.panel_id_map.get(panel_str_id) else {
            return;
        };
        let Some(tree) = &mut self.tree else { return };

        if let Some(&tile_id) = self.panel_tile_map.get(&panel_id) {
            // Check if this tile is still in a container
            if tree.tiles.get(tile_id).is_some() {
                // Tile exists, make sure it's visible
                tree.tiles.set_visible(tile_id, true);
            } else {
                // Tile was removed — re-insert the pane entry
                let new_tile_id = tree.tiles.insert_pane(PaneEntry { panel_id });
                self.panel_tile_map.insert(panel_id, new_tile_id);
                ui::insert_pane_into_tree(tree, new_tile_id);
            }
        } else {
            // Panel was never added to tree (default_visible=false) — insert fresh
            let new_tile_id = tree.tiles.insert_pane(PaneEntry { panel_id });
            self.panel_tile_map.insert(panel_id, new_tile_id);
            ui::insert_pane_into_tree(tree, new_tile_id);
        }
    }

    /// Close a panel by removing its tile from the tree entirely.
    pub fn hide_tile(&mut self, tile_id: egui_tiles::TileId) {
        if let Some(tree) = &mut self.tree {
            tree.tiles.remove(tile_id);
        }
    }

    /// Close a panel by its string ID.
    pub fn close_panel(&mut self, panel_str_id: &str) -> bool {
        let Some(&panel_id) = self.panel_id_map.get(panel_str_id) else {
            return false;
        };
        if let Some(&tile_id) = self.panel_tile_map.get(&panel_id) {
            self.hide_tile(tile_id);
            true
        } else {
            false
        }
    }

    /// Request a panel to be opened (with undo recording in the exclusive system).
    pub fn request_open_panel(&mut self, panel_str_id: &str) {
        self.pending_open_requests.push(panel_str_id.to_string());
    }

    /// Build a reverse map from TileId → panel string ID.
    pub(crate) fn tile_to_panel_str_id_map(&self) -> HashMap<egui_tiles::TileId, String> {
        let mut map = HashMap::new();
        for (str_id, &panel_id) in &self.panel_id_map {
            if let Some(&tile_id) = self.panel_tile_map.get(&panel_id) {
                map.insert(tile_id, str_id.clone());
            }
        }
        map
    }

    /// Take a snapshot of the current tree + tile map for undo/redo.
    pub(crate) fn snapshot(&self) -> Option<LayoutSnapshot> {
        self.tree.as_ref().map(|tree| LayoutSnapshot {
            tree: tree.clone(),
            panel_tile_map: self.panel_tile_map.clone(),
        })
    }

    /// Restore a layout snapshot (for undo/redo).
    pub(crate) fn restore_snapshot(&mut self, snapshot: LayoutSnapshot) {
        self.tree = Some(snapshot.tree);
        self.panel_tile_map = snapshot.panel_tile_map;
    }

    /// Returns list of (panel_str_id, title, is_visible) for building the Window menu.
    /// Only includes panels where `show_in_window_menu()` returns `true`
    /// and not hidden via `hide_from_window_menu()`.
    pub fn panel_list(&self) -> Vec<(String, String, bool)> {
        let mut result = Vec::new();
        for (str_id, &panel_id) in &self.panel_id_map {
            let Some(panel) = self.panels.get(&panel_id) else {
                continue;
            };
            if !panel.show_in_window_menu() || self.window_menu_hidden.contains(str_id) {
                continue;
            }
            let title = panel.title();
            let visible = self
                .panel_tile_map
                .get(&panel_id)
                .and_then(|&tid| self.tree.as_ref().map(|t| t.tiles.get(tid).is_some()))
                .unwrap_or(false);
            result.push((str_id.clone(), title, visible));
        }
        result.sort_by(|a, b| a.1.cmp(&b.1));
        result
    }

    /// Hide a panel from the built-in Window menu (it can still be managed
    /// programmatically or via a custom menu).
    pub fn hide_from_window_menu(&mut self, panel_str_id: &str) {
        self.window_menu_hidden.insert(panel_str_id.to_string());
    }

    /// Mark a panel as hidden in the default layout. This overrides
    /// `default_visible()` when building the tree from scratch (no saved layout).
    pub fn set_default_hidden(&mut self, panel_str_id: &str) {
        self.default_hidden.insert(panel_str_id.to_string());
    }

    /// Check whether a panel is currently visible in the layout.
    pub fn is_panel_visible(&self, panel_str_id: &str) -> bool {
        let Some(&panel_id) = self.panel_id_map.get(panel_str_id) else {
            return false;
        };
        self.panel_tile_map
            .get(&panel_id)
            .and_then(|&tid| self.tree.as_ref().map(|t| t.tiles.get(tid).is_some()))
            .unwrap_or(false)
    }

    /// Get a mutable reference to a panel by its string ID, with downcasting.
    /// Also searches pending panels not yet moved into the main map.
    pub fn get_panel_mut<T: WorkbenchPanel + 'static>(
        &mut self,
        panel_str_id: &str,
    ) -> Option<&mut T> {
        // First try the built panels map
        if let Some(&panel_id) = self.panel_id_map.get(panel_str_id)
            && let Some(panel) = self.panels.get_mut(&panel_id)
        {
            return (panel.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>();
        }
        // Fall back to pending panels (not yet moved into the map)
        for pending in &mut self.pending {
            if pending.panel.id() == panel_str_id {
                return (pending.panel.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>();
            }
        }
        None
    }
}
