//! Minimal bevy_workbench usage example.
//!
//! Shows a workbench editor with a game view rendering animated colored shapes.
//! Game entities are spawned on Play and auto-despawned on Stop.
//! The first scene object is controllable with WASD/right-click by default.

#[path = "common.rs"]
mod common;

use bevy::prelude::*;
use bevy_workbench::console::console_log_layer;
use bevy_workbench::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Workbench — Minimal".into(),
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

    app.add_systems(Startup, setup)
        .add_systems(
            OnEnter(EditorMode::Play),
            (common::setup_game, auto_control_first)
                .chain()
                .run_if(on_fresh_play),
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

/// Auto-assign control to the first scene object.
fn auto_control_first(mut commands: Commands, objects: Query<Entity, With<common::SceneObject>>) {
    if let Some(entity) = objects.iter().next() {
        commands.entity(entity).insert(common::Controlled);
    }
}
