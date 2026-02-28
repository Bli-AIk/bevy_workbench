//! Minimal bevy_workbench usage example.
//!
//! Shows a workbench editor with a game view rendering colored shapes.

use bevy::prelude::*;
use bevy_workbench::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Workbench — Minimal".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(WorkbenchPlugin::default())
        .add_plugins(GameViewPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Editor window camera (for egui)
    commands.spawn(Camera2d);

    // Demo game entities rendered into the game view texture
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(80.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.8, 0.4))),
        Transform::from_xyz(150.0, 80.0, 0.0),
    ));
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(40.0, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 1.0))),
        Transform::from_xyz(-120.0, -60.0, 0.0),
    ));
}
