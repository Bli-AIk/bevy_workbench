//! # persistence.rs
//!
//! # persistence.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Serializes and restores the dock layout used by `bevy_workbench`. It translates the
//! in-memory tile tree into a stable on-disk snapshot and remaps saved panel ids back onto the
//! current session's panel registry when a layout is loaded.
//!
//! 负责序列化和恢复 `bevy_workbench` 使用的停靠布局。它会把内存中的 tile 树转换成
//! 稳定的磁盘快照，并在加载布局时把保存下来的 panel id 重新映射回当前会话的 panel 注册表。

use super::{PaneEntry, PanelId, TileLayoutState};
use bevy::prelude::*;
use std::collections::HashMap;

/// Serializable snapshot of the dock layout.
#[derive(serde::Serialize, serde::Deserialize)]
struct LayoutData {
    tree: egui_tiles::Tree<PaneEntry>,
    panel_names: HashMap<PanelId, String>,
}

impl TileLayoutState {
    /// Save the current layout to a file (JSON format).
    pub fn save_layout(&self, path: &std::path::Path) {
        let Some(tree) = &self.tree else { return };
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

        let mut name_to_current_id: HashMap<String, PanelId> = HashMap::new();
        for (str_id, &current_id) in &self.panel_id_map {
            name_to_current_id.insert(str_id.clone(), current_id);
        }

        let mut id_remap: HashMap<PanelId, PanelId> = HashMap::new();
        for (old_id, name) in &data.panel_names {
            if let Some(&new_id) = name_to_current_id.get(name) {
                id_remap.insert(*old_id, new_id);
            }
        }

        let mut tree = data.tree;
        for tile in tree.tiles.tiles_mut() {
            if let egui_tiles::Tile::Pane(pane) = tile
                && let Some(&new_id) = id_remap.get(&pane.panel_id)
            {
                pane.panel_id = new_id;
            }
        }

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
