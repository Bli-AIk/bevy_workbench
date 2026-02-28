//! Control Link example — drag connections to assign control.
//!
//! A custom "Control" panel shows 4 scene objects and a "控制权" node.
//! Drag from "控制权" to an object to assign WASD/right-click control.

#[path = "common.rs"]
mod common;

use bevy::prelude::*;
use bevy_workbench::console::console_log_layer;
use bevy_workbench::dock::WorkbenchPanel;
use bevy_workbench::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Workbench — Control Link".into(),
                    resolution: (1280u32, 720u32).into(),
                    ..default()
                }),
                ..default()
            })
            .set(bevy::log::LogPlugin {
                custom_layer: console_log_layer,
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(WorkbenchPlugin::default())
    .add_plugins(GameViewPlugin);

    common::register_types(&mut app);
    register_panels(&mut app);

    app.init_resource::<ControlLinkState>()
        .add_systems(Startup, setup)
        .add_systems(
            OnEnter(EditorMode::Play),
            common::setup_game.run_if(on_fresh_play),
        )
        .add_systems(
            GameSchedule,
            (common::animate_shapes, common::controlled_movement),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn register_panels(app: &mut App) {
    app.register_panel(ControlLinkPanel);
}

/// Tracks the drag state and current control link.
#[derive(Resource, Default)]
struct ControlLinkState {
    /// Entity currently linked (controlled).
    linked_entity: Option<Entity>,
    /// True while dragging from the control node.
    dragging: bool,
    /// Screen position of the drag start (control node center).
    drag_start: egui::Pos2,
}

/// Custom panel: shows scene objects + "控制权" node with drag-to-link.
struct ControlLinkPanel;

impl WorkbenchPanel for ControlLinkPanel {
    fn id(&self) -> &str {
        "control_link"
    }

    fn title(&self) -> String {
        "Control".into()
    }

    fn ui(&mut self, _ui: &mut egui::Ui) {
        // Unused — we use ui_world
    }

    fn default_visible(&self) -> bool {
        true
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let node_size = egui::vec2(100.0, 36.0);
        let control_color = egui::Color32::from_rgb(255, 180, 50);
        let object_color = egui::Color32::from_rgb(80, 160, 255);
        let linked_color = egui::Color32::from_rgb(100, 255, 100);

        // Collect scene objects
        let mut objects: Vec<(Entity, String)> = Vec::new();
        let mut scene_q = world.query::<(Entity, &common::SceneObject)>();
        for (entity, obj) in scene_q.iter(world) {
            objects.push((entity, obj.name.clone()));
        }

        let mut state = world
            .remove_resource::<ControlLinkState>()
            .unwrap_or_default();

        // Layout: control node on the left, objects on the right
        let panel_rect = ui.available_rect_before_wrap();
        let control_pos = egui::pos2(
            panel_rect.left() + 30.0,
            panel_rect.center().y - node_size.y / 2.0,
        );
        let control_rect = egui::Rect::from_min_size(control_pos, node_size);

        // Draw control node "控制权"
        let control_response = ui.allocate_rect(control_rect, egui::Sense::drag());

        // Object node positions (stacked vertically on the right)
        let obj_x = panel_rect.right() - node_size.x - 30.0;
        let obj_start_y = panel_rect.top() + 20.0;
        let obj_spacing = node_size.y + 16.0;

        let mut object_rects: Vec<(Entity, egui::Rect)> = Vec::new();
        let mut clicked_linked: Option<Entity> = None;
        for (i, (entity, _name)) in objects.iter().enumerate() {
            let y = obj_start_y + i as f32 * obj_spacing;
            let rect = egui::Rect::from_min_size(egui::pos2(obj_x, y), node_size);
            object_rects.push((*entity, rect));

            let obj_response = ui.allocate_rect(rect, egui::Sense::click());
            let is_linked = state.linked_entity == Some(*entity);
            if is_linked && obj_response.clicked() {
                clicked_linked = Some(*entity);
            }
        }

        // Handle click-to-unlink
        if let Some(old) = clicked_linked {
            if state.linked_entity == Some(old) {
                state.linked_entity = None;
                if let Ok(mut e) = world.get_entity_mut(old) {
                    e.remove::<common::Controlled>();
                }
            }
        }

        // Handle drag from control node
        if control_response.drag_started() {
            state.dragging = true;
            state.drag_start = control_rect.right_center();
        }

        if state.dragging && ui.input(|i| i.pointer.any_released()) {
            state.dragging = false;
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                for &(entity, rect) in &object_rects {
                    if rect.contains(pointer_pos) {
                        // Unlink previous
                        if let Some(old) = state.linked_entity.take() {
                            if let Ok(mut e) = world.get_entity_mut(old) {
                                e.remove::<common::Controlled>();
                            }
                        }
                        // Link new
                        if let Ok(mut e) = world.get_entity_mut(entity) {
                            e.insert(common::Controlled);
                        }
                        state.linked_entity = Some(entity);
                        break;
                    }
                }
            }
        }

        // --- All painting after allocations ---
        let painter = ui.painter();

        // Control node
        painter.rect_filled(control_rect, 6.0, control_color);
        painter.text(
            control_rect.center(),
            egui::Align2::CENTER_CENTER,
            "控制权",
            egui::FontId::proportional(14.0),
            egui::Color32::BLACK,
        );

        // Object nodes
        for (i, (entity, name)) in objects.iter().enumerate() {
            let (_, rect) = object_rects[i];
            let is_linked = state.linked_entity == Some(*entity);
            let fill = if is_linked {
                linked_color
            } else {
                object_color
            };
            painter.rect_filled(rect, 6.0, fill);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                name,
                egui::FontId::proportional(13.0),
                egui::Color32::WHITE,
            );
        }

        // Drag line
        if state.dragging {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                painter.line_segment(
                    [state.drag_start, pointer_pos],
                    egui::Stroke::new(2.5, control_color),
                );
            }
        }

        // Permanent link line
        if let Some(linked) = state.linked_entity {
            for &(entity, rect) in &object_rects {
                if entity == linked {
                    painter.line_segment(
                        [control_rect.right_center(), rect.left_center()],
                        egui::Stroke::new(2.0, linked_color),
                    );
                    break;
                }
            }
        }

        world.insert_resource(state);
    }
}
