//! Tiling panel system based on egui_tiles.
//!
//! Sets up a desktop editor layout using egui_tiles with the structure:
//! - Root: Vertical split (main area + bottom console)
//! - Main area: Horizontal split (game view + right inspector)

use bevy::prelude::*;
use std::collections::HashMap;

/// Serializable snapshot of the dock layout.
#[derive(serde::Serialize, serde::Deserialize)]
struct LayoutData {
    tree: egui_tiles::Tree<PaneEntry>,
    panel_names: HashMap<PanelId, String>,
}

/// Trait for user-defined editor panels.
pub trait WorkbenchPanel: Send + Sync + std::any::Any + 'static {
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

    /// Whether the panel is visible in the default layout (default: true).
    fn default_visible(&self) -> bool {
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

        for (str_id, &panel_id) in &self.panel_id_map {
            let panel = &self.panels[&panel_id];
            if !panel.default_visible() {
                continue;
            }
            let slot = match str_id.as_str() {
                id if id.contains("inspector") => PanelSlot::Right,
                id if id.contains("console") || id.contains("timeline") => PanelSlot::Bottom,
                id if id.contains("game_view") => PanelSlot::Center,
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

                // Find the root and insert into it
                if let Some(root_id) = tree.root() {
                    tree.move_tile_to_container(new_tile_id, root_id, usize::MAX, false);
                } else {
                    tree.root = Some(new_tile_id);
                }
            }
        } else {
            // Panel was never added to tree (default_visible=false) — insert fresh
            let new_tile_id = tree.tiles.insert_pane(PaneEntry { panel_id });
            self.panel_tile_map.insert(panel_id, new_tile_id);
            if let Some(root_id) = tree.root() {
                tree.move_tile_to_container(new_tile_id, root_id, usize::MAX, false);
            } else {
                tree.root = Some(new_tile_id);
            }
        }
    }

    /// Close a panel by removing its tile from the tree entirely.
    pub fn hide_tile(&mut self, tile_id: egui_tiles::TileId) {
        if let Some(tree) = &mut self.tree {
            tree.tiles.remove(tile_id);
        }
    }

    /// Returns list of (panel_str_id, title, is_visible) for building the Window menu.
    pub fn panel_list(&self) -> Vec<(String, String, bool)> {
        let mut result = Vec::new();
        for (str_id, &panel_id) in &self.panel_id_map {
            let title = self
                .panels
                .get(&panel_id)
                .map(|p| p.title())
                .unwrap_or_default();
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

    /// Get a mutable reference to a panel by its string ID, with downcasting.
    pub fn get_panel_mut<T: WorkbenchPanel + 'static>(
        &mut self,
        panel_str_id: &str,
    ) -> Option<&mut T> {
        let panel_id = self.panel_id_map.get(panel_str_id)?;
        let panel = self.panels.get_mut(panel_id)?;
        // Downcast via Any
        (panel.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>()
    }

    /// Save the current layout to a file (JSON format).
    pub fn save_layout(&self, path: &std::path::Path) {
        let Some(tree) = &self.tree else { return };
        // Build reverse map: panel_id → str_id
        let id_to_str: HashMap<PanelId, String> = self
            .panel_id_map
            .iter()
            .map(|(s, &id)| (id, s.clone()))
            .collect();
        let data = LayoutData {
            tree: tree.clone(),
            panel_names: id_to_str,
        };
        let content = serde_json::to_string_pretty(&data).expect("serialize layout");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, content) {
            warn!("Failed to save layout to {}: {e}", path.display());
        }
    }

    /// Load layout from a JSON file. Returns true if successful.
    /// Must be called after all panels are registered but before the tree is built.
    pub fn load_layout(&mut self, path: &std::path::Path) -> bool {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let data: LayoutData = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to parse layout {}: {e}", path.display());
                return false;
            }
        };

        // Remap panel IDs: saved names → current IDs
        let mut name_to_current_id: HashMap<String, PanelId> = HashMap::new();
        for (str_id, &current_id) in &self.panel_id_map {
            name_to_current_id.insert(str_id.clone(), current_id);
        }

        // Build old_id → new_id mapping
        let mut id_remap: HashMap<PanelId, PanelId> = HashMap::new();
        for (old_id, name) in &data.panel_names {
            if let Some(&new_id) = name_to_current_id.get(name) {
                id_remap.insert(*old_id, new_id);
            }
        }

        // Clone and remap the tree
        let mut tree = data.tree;
        for tile in tree.tiles.tiles_mut() {
            if let egui_tiles::Tile::Pane(pane) = tile
                && let Some(&new_id) = id_remap.get(&pane.panel_id)
            {
                pane.panel_id = new_id;
            }
        }

        // Rebuild panel_tile_map
        self.panel_tile_map.clear();
        for (&tile_id, tile) in tree.tiles.iter() {
            if let egui_tiles::Tile::Pane(pane) = tile {
                self.panel_tile_map.insert(pane.panel_id, tile_id);
            }
        }

        self.tree = Some(tree);
        self.tree_built = true;
        true
    }
}

/// Adapter between egui_tiles::Behavior and our WorkbenchPanel system.
struct WorkbenchBehavior<'a> {
    panels: &'a mut HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    /// Tile IDs to remove from the tree after the UI pass.
    tiles_to_remove: Vec<egui_tiles::TileId>,
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

    fn is_tab_closable(
        &self,
        _tiles: &egui_tiles::Tiles<PaneEntry>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        true
    }

    fn on_tab_close(
        &mut self,
        _tiles: &mut egui_tiles::Tiles<PaneEntry>,
        tile_id: egui_tiles::TileId,
    ) -> bool {
        self.tiles_to_remove.push(tile_id);
        true // return true = remove the tile from its parent
    }

    fn on_tab_button(
        &mut self,
        _tiles: &egui_tiles::Tiles<PaneEntry>,
        tile_id: egui_tiles::TileId,
        button_response: egui::Response,
    ) -> egui::Response {
        // Right-click context menu
        button_response.context_menu(|ui| {
            if ui.button("Close").clicked() {
                self.tiles_to_remove.push(tile_id);
                ui.close();
            }
        });
        button_response
    }

    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }
}

/// Resource holding the layout file path.
#[derive(Resource)]
pub struct LayoutPath(pub std::path::PathBuf);

impl Default for LayoutPath {
    fn default() -> Self {
        Self(std::path::PathBuf::from(".workbench/layout.json"))
    }
}

/// System that renders the tile layout using EguiContexts.
pub fn tiles_ui_system(
    mut contexts: bevy_egui::EguiContexts,
    mut state: ResMut<TileLayoutState>,
    layout_path: Res<LayoutPath>,
) {
    // Build tree on first frame (after all panels registered)
    state.build_tree(Some(&layout_path.0));

    // Handle layout save request (via file dialog path)
    if let Some(path) = state.layout_save_path.take() {
        state.save_layout(&path);
        info!("Layout saved to {}", path.display());
    }

    // Handle layout load request (via file dialog path)
    if let Some(path) = state.layout_load_path.take()
        && state.load_layout(&path)
    {
        info!("Layout loaded from {}", path.display());
    }

    // Handle layout reset request
    if state.layout_reset_requested {
        state.layout_reset_requested = false;
        state.tree = None;
        state.panel_tile_map.clear();
        state.build_default_tree();
        // Delete saved layout file
        let _ = std::fs::remove_file(&layout_path.0);
        info!("Layout reset to default");
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    let state = &mut *state;
    let Some(ref mut tree) = state.tree else {
        return;
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut behavior = WorkbenchBehavior {
            panels: &mut state.panels,
            tiles_to_remove: Vec::new(),
        };
        tree.ui(&mut behavior, ui);

        // Remove tiles that were closed (right-click or X button)
        for tile_id in behavior.tiles_to_remove {
            tree.tiles.remove(tile_id);
        }
    });
}
