//! Minimal bevy_workbench usage example.
//!
//! Shows a workbench editor with a game view rendering animated colored shapes.
//! Game entities are spawned on Play and auto-despawned on Stop.
//! Includes a player triangle controllable with WASD/right-click.

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
        .register_type::<ShapeAnim>()
        .register_type::<Player>()
        .register_type::<MoveSpeed>()
        // Editor camera — always present
        .add_systems(Startup, setup)
        // Game setup — runs only on fresh Play (not Resume from Pause)
        .add_systems(OnEnter(EditorMode::Play), setup_game.run_if(on_fresh_play))
        // Game logic — runs every frame during Play via GameSchedule
        .add_systems(GameSchedule, (animate_shapes, player_movement))
        .run();
}

fn setup(mut commands: Commands) {
    // Editor window camera (for egui) — always present
    commands.spawn(Camera2d);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
enum ShapeAnim {
    BounceY,
    Rotate,
    BounceX,
}

/// Marker for the player entity.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct Player;

/// Movement speed of the player.
#[derive(Component, Reflect)]
#[reflect(Component)]
struct MoveSpeed(f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(200.0)
    }
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
        cleanup.clone(),
    ));

    // Player — triangle, controllable with WASD/right-click
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(30.0, 3))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.2))),
        Transform::from_xyz(0.0, -150.0, 1.0),
        Player,
        MoveSpeed(200.0),
        cleanup,
    ));

    info!("Player spawned — use WASD to move, right-click to teleport");
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

fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    game_view: Res<GameViewFocus>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameViewCamera>>,
    mut player: Query<(&MoveSpeed, &mut Transform), With<Player>>,
) {
    let Ok((speed, mut tr)) = player.single_mut() else {
        return;
    };

    // Only process input when game view is hovered
    if !game_view.hovered {
        return;
    }

    let dt = time.delta_secs();

    // WASD movement
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if dir != Vec2::ZERO {
        let delta = dir.normalize() * speed.0 * dt;
        tr.translation.x += delta.x;
        tr.translation.y += delta.y;
    }

    // Right-click teleport using game view viewport coordinates
    if mouse_buttons.just_pressed(MouseButton::Right)
        && let Some(viewport_pos) = game_view.cursor_viewport_pos
    {
        for (camera, camera_transform) in &cameras {
            if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, viewport_pos) {
                tr.translation.x = world_pos.x;
                tr.translation.y = world_pos.y;
                break;
            }
        }
    }
}
