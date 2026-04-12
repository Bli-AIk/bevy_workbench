//! # ui.rs
//!
//! # ui.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Renders and mutates the workbench dock UI. It bridges `egui_tiles` with Bevy `World`
//! access, handles tab closing and layout undo snapshots, and turns pending panel-open requests
//! into concrete changes in the tile tree.
//!
//! 负责渲染并更新 workbench 的停靠式 UI。它把 `egui_tiles` 与 Bevy `World`
//! 访问连接起来，处理标签页关闭和布局撤销快照，并把待打开的面板请求落实为 tile 树上的具体变更。

use super::{LayoutUndoAction, PaneEntry, PanelId, TileLayoutState, WorkbenchPanel};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy_egui::PrimaryEguiContext;
use std::collections::HashMap;

pub(super) fn set_linear_shares(
    tiles: &mut egui_tiles::Tiles<PaneEntry>,
    tile_id: egui_tiles::TileId,
    shares: &[(egui_tiles::TileId, f32)],
) {
    if let Some(egui_tiles::Tile::Container(egui_tiles::Container::Linear(linear))) =
        tiles.get_mut(tile_id)
    {
        for &(child, share) in shares {
            linear.shares.set_share(child, share);
        }
    }
}

pub(super) fn insert_pane_into_tree(
    tree: &mut egui_tiles::Tree<PaneEntry>,
    tile_id: egui_tiles::TileId,
) {
    if let Some(root_id) = tree.root() {
        tree.move_tile_to_container(tile_id, root_id, usize::MAX, false);
    } else {
        tree.root = Some(tile_id);
    }
}

struct WorkbenchBehavior<'a> {
    panels: &'a mut HashMap<PanelId, Box<dyn WorkbenchPanel>>,
    world: Option<&'a mut World>,
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
            // Draw tile background from panel's bg_color (None = transparent for 3D scene)
            if let Some(color) = panel.bg_color() {
                ui.painter().rect_filled(ui.max_rect(), 0.0, color);
            }
            if panel.needs_world()
                && let Some(world) = self.world.as_deref_mut()
            {
                panel.ui_world(ui, world);
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
        true
    }

    fn on_tab_button(
        &mut self,
        _tiles: &egui_tiles::Tiles<PaneEntry>,
        tile_id: egui_tiles::TileId,
        button_response: egui::Response,
    ) -> egui::Response {
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

            if let (Some(before), Some(after)) = (before, after)
                && let Some(mut undo_stack) = world.get_resource_mut::<crate::undo::UndoStack>()
            {
                undo_stack.push(LayoutUndoAction::new("Reset layout", before, after));
            }
        }
    });

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

    let before_snapshot = {
        let state = world.resource::<TileLayoutState>();
        state.snapshot()
    };

    let tile_to_str_id = {
        let state = world.resource::<TileLayoutState>();
        state.tile_to_panel_str_id_map()
    };

    let (mut tree, mut panels) = {
        let mut state = world.resource_mut::<TileLayoutState>();
        (state.tree.take(), std::mem::take(&mut state.panels))
    };

    let mut closed_panel_ids: Vec<String> = Vec::new();

    if let Some(ref mut tree) = tree {
        // Transparent CentralPanel — lets 3D scene show through behind tiles
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(&ctx, |ui| {
            let mut behavior = WorkbenchBehavior {
                panels: &mut panels,
                world: Some(world),
                tiles_to_remove: Vec::new(),
            };
            tree.ui(&mut behavior, ui);

            for tile_id in behavior.tiles_to_remove {
                closed_panel_ids.extend(tile_to_str_id.get(&tile_id).cloned());
                tree.tiles.remove(tile_id);
            }
        });
    }

    let mut state = world.resource_mut::<TileLayoutState>();
    state.tree = tree;
    state.panels = panels;

    if !closed_panel_ids.is_empty() {
        let after_snapshot = state.snapshot();
        if let (Some(before), Some(after)) = (before_snapshot.clone(), after_snapshot) {
            let desc = format!("Close {}", closed_panel_ids.join(", "));
            let _ = state;
            if let Some(mut undo_stack) = world.get_resource_mut::<crate::undo::UndoStack>() {
                undo_stack.push(LayoutUndoAction::new(desc, before, after));
            }
        }
    }

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
