//! Minimal bevy_workbench usage example.
//!
//! Shows a workbench editor with a game view rendering animated colored shapes.
//! Game entities are spawned on Play and auto-despawned on Stop.

use bevy::prelude::*;
use bevy::state::prelude::DespawnOnEnter;
use bevy_workbench::console::console_log_layer;
use bevy_workbench::prelude::*;

fn main() {
    App::new()
        .add_plugins(
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
        .add_plugins(GameViewPlugin)
        // Editor camera — always present
        .add_systems(Startup, setup)
        // Game setup — runs only on fresh Play (not Resume from Pause)
        .add_systems(OnEnter(EditorMode::Play), setup_game.run_if(on_fresh_play))
        // Game logic — runs every frame during Play via GameSchedule
        .add_systems(GameSchedule, animate_shapes)
        .run();
}

fn setup(mut commands: Commands) {
    // Editor window camera (for egui) — always present
    commands.spawn(Camera2d);
}

#[derive(Component)]
enum ShapeAnim {
    BounceY,
    Rotate,
    BounceX,
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Game started — spawning shapes");
    warn!("This is a test warning: shape count is hardcoded");

    // All game entities are marked with DespawnOnEnter(Edit) — auto-cleanup on Stop
    let cleanup = DespawnOnEnter(EditorMode::Edit);

    // Circle — bounces up and down
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ShapeAnim::BounceY,
        cleanup.clone(),
    ));
    // Rectangle — rotates back and forth
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(80.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.8, 0.4))),
        Transform::from_xyz(150.0, 80.0, 0.0),
        ShapeAnim::Rotate,
        cleanup.clone(),
    ));
    // Hexagon — moves left and right
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(40.0, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 1.0))),
        Transform::from_xyz(-120.0, -60.0, 0.0),
        ShapeAnim::BounceX,
        cleanup,
    ));
}

fn animate_shapes(clock: Res<GameClock>, mut query: Query<(&ShapeAnim, &mut Transform)>) {
    let t = clock.elapsed;

    // Periodic test logs
    let frame = (t * 60.0) as u64;
    if frame.is_multiple_of(300) && frame > 0 {
        info!("Animation running: elapsed = {:.1}s", t);
    }
    if frame == 180 {
        error!("Test error: this is a simulated error for console testing");
    }

    for (anim, mut tr) in &mut query {
        match anim {
            ShapeAnim::BounceY => {
                tr.translation.y = (t * 2.0).sin() * 100.0;
            }
            ShapeAnim::Rotate => {
                tr.rotation = Quat::from_rotation_z((t * 1.5).sin() * 0.8);
            }
            ShapeAnim::BounceX => {
                tr.translation.x = -120.0 + (t * 1.2).cos() * 150.0;
            }
        }
    }
}
