//! Tiling panel system based on egui_tiles.
//!
//! Sets up a desktop editor layout using egui_tiles with the structure:
//! - Root: Vertical split (main area + bottom console)
//! - Main area: Horizontal split (game view + right inspector)

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::PrimaryEguiContext;
use std::collections::HashMap;
use std::sync::Mutex;

/// Serializable snapshot of the dock layout.
#[derive(serde::Serialize, serde::Deserialize)]
struct LayoutData {
    tree: egui_tiles::Tree<PaneEntry>,
    panel_names: HashMap<PanelId, String>,
}

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
    /// Panels requested to open (processed in exclusive system with undo recording).
    pub(crate) pending_open_requests: Vec<String>,
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

    /// Close a panel by its string ID.
    pub fn close_panel(&mut self, panel_str_id: &str) {
        let Some(&panel_id) = self.panel_id_map.get(panel_str_id) else {
            return;
        };
        if let Some(&tile_id) = self.panel_tile_map.get(&panel_id) {
            self.hide_tile(tile_id);
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
    /// World access for panels that need it (inspector, etc.).
    world: Option<&'a mut World>,
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
            if panel.needs_world() {
                if let Some(world) = self.world.as_deref_mut() {
                    panel.ui_world(ui, world);
                } else {
                    panel.ui(ui);
                }
            } else {
                panel.ui(ui);
            }
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

/// Exclusive system that renders the tile layout with World access for panels.
pub fn tiles_ui_system(world: &mut World) {
    // Phase 1: Build tree & handle save/load/reset
    world.resource_scope(|world, mut state: Mut<TileLayoutState>| {
        let layout_path = world.resource::<LayoutPath>();
        state.build_tree(Some(&layout_path.0));

        if let Some(path) = state.layout_save_path.take() {
            state.save_layout(&path);
            info!("Layout saved to {}", path.display());
        }
        if let Some(path) = state.layout_load_path.take()
            && state.load_layout(&path)
        {
            info!("Layout loaded from {}", path.display());
        }
        if state.layout_reset_requested {
            state.layout_reset_requested = false;
            let before = state.snapshot();
            state.tree = None;
            state.panel_tile_map.clear();
            state.build_default_tree();
            let after = state.snapshot();
            let _ = std::fs::remove_file(&layout_path.0);
            info!("Layout reset to default");

            // Record undo for layout reset
            if let (Some(before), Some(after)) = (before, after)
                && let Some(mut undo_stack) = world.get_resource_mut::<crate::undo::UndoStack>()
            {
                undo_stack.push(LayoutUndoAction::new("Reset layout", before, after));
            }
        }
    });

    // Phase 2: Get egui context (clone is cheap — Arc internally)
    let ctx = {
        let mut sys =
            SystemState::<Query<&mut bevy_egui::EguiContext, With<PrimaryEguiContext>>>::new(world);
        let mut query = sys.get_mut(world);
        let Ok(mut egui_ctx) = query.single_mut() else {
            return;
        };
        let ctx = egui_ctx.get_mut().clone();
        sys.apply(world);
        ctx
    };

    // Phase 3: Snapshot layout BEFORE extracting tree (for close undo)
    let before_snapshot = {
        let state = world.resource::<TileLayoutState>();
        state.snapshot()
    };

    // Build reverse map before world is borrowed by behavior
    let tile_to_str_id = {
        let state = world.resource::<TileLayoutState>();
        state.tile_to_panel_str_id_map()
    };

    // Temporarily take tree+panels out of resource so we can pass &mut World
    let (mut tree, mut panels) = {
        let mut state = world.resource_mut::<TileLayoutState>();
        (state.tree.take(), std::mem::take(&mut state.panels))
    };

    let mut closed_panel_ids: Vec<String> = Vec::new();

    if let Some(ref mut tree) = tree {
        egui::CentralPanel::default().show(&ctx, |ui| {
            let mut behavior = WorkbenchBehavior {
                panels: &mut panels,
                world: Some(world),
                tiles_to_remove: Vec::new(),
            };
            tree.ui(&mut behavior, ui);

            for tile_id in behavior.tiles_to_remove {
                if let Some(str_id) = tile_to_str_id.get(&tile_id) {
                    closed_panel_ids.push(str_id.clone());
                }
                tree.tiles.remove(tile_id);
            }
        });
    }

    // Phase 4: Put tree+panels back
    let mut state = world.resource_mut::<TileLayoutState>();
    state.tree = tree;
    state.panels = panels;

    // Record undo action with layout snapshots for closed panels
    if !closed_panel_ids.is_empty() {
        let after_snapshot = state.snapshot();
        if let (Some(before), Some(after)) = (before_snapshot.clone(), after_snapshot) {
            let desc = format!("Close {}", closed_panel_ids.join(", "));
            // Release state borrow before accessing undo_stack
            let _ = state;
            if let Some(mut undo_stack) = world.get_resource_mut::<crate::undo::UndoStack>() {
                undo_stack.push(LayoutUndoAction::new(desc, before, after));
            }
        }
    }

    // Phase 5: Process pending open requests with undo recording
    let pending_opens = {
        let mut state = world.resource_mut::<TileLayoutState>();
        std::mem::take(&mut state.pending_open_requests)
    };
    if !pending_opens.is_empty() {
        let open_before = {
            let state = world.resource::<TileLayoutState>();
            state.snapshot()
        };
        {
            let mut state = world.resource_mut::<TileLayoutState>();
            for str_id in &pending_opens {
                state.open_or_focus_panel(str_id);
            }
        }

        let open_after = {
            let state = world.resource::<TileLayoutState>();
            state.snapshot()
        };

        if let (Some(before), Some(after)) = (open_before, open_after) {
            let desc = format!("Open {}", pending_opens.join(", "));
            if let Some(mut undo_stack) = world.get_resource_mut::<crate::undo::UndoStack>() {
                undo_stack.push(LayoutUndoAction::new(desc, before, after));
            }
        }
    }
}
