//! Shared game logic for workbench examples.
//!
//! Provides reusable components and systems for spawning animated shapes
//! and controlling entities via WASD/right-click.

use bevy::prelude::*;
use bevy::state::prelude::DespawnOnEnter;
use bevy_workbench::prelude::*;

/// Animation type for demo shapes.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum ShapeAnim {
    BounceY,
    Rotate,
    BounceX,
}

/// Named scene object (visible in control panels).
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SceneObject {
    pub name: String,
}

/// Marker: this entity is currently being controlled by the player.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Controlled;

/// Movement speed component.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MoveSpeed(pub f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(200.0)
    }
}

/// Register shared types with the app.
pub fn register_types(app: &mut App) {
    app.register_type::<ShapeAnim>()
        .register_type::<SceneObject>()
        .register_type::<Controlled>()
        .register_type::<MoveSpeed>();
}

/// Spawn 4 named scene objects (shapes) into the game world.
pub fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Game started — spawning shapes");

    let cleanup = DespawnOnEnter(EditorMode::Edit);

    // Circle — bounces up and down
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(50.0))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 0.5, 0.2))),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ShapeAnim::BounceY,
        SceneObject {
            name: "Circle".into(),
        },
        MoveSpeed(200.0),
        cleanup.clone(),
    ));

    // Rectangle — rotates
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(80.0, 80.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.8, 0.4))),
        Transform::from_xyz(150.0, 80.0, 0.0),
        ShapeAnim::Rotate,
        SceneObject {
            name: "Rectangle".into(),
        },
        MoveSpeed(200.0),
        cleanup.clone(),
    ));

    // Hexagon — moves left-right
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(40.0, 6))),
        MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 1.0))),
        Transform::from_xyz(-120.0, -60.0, 0.0),
        ShapeAnim::BounceX,
        SceneObject {
            name: "Hexagon".into(),
        },
        MoveSpeed(200.0),
        cleanup.clone(),
    ));

    // Triangle
    commands.spawn((
        Mesh2d(meshes.add(RegularPolygon::new(30.0, 3))),
        MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.2))),
        Transform::from_xyz(0.0, -150.0, 1.0),
        SceneObject {
            name: "Triangle".into(),
        },
        MoveSpeed(200.0),
        cleanup,
    ));
}

/// Animate shapes based on their ShapeAnim variant.
pub fn animate_shapes(clock: Res<GameClock>, mut query: Query<(&ShapeAnim, &mut Transform)>) {
    let t = clock.elapsed;
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

/// Move the currently `Controlled` entity with WASD; right-click teleport.
pub fn controlled_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    game_view: Res<GameViewFocus>,
    cameras: Query<(&Camera, &GlobalTransform), With<GameViewCamera>>,
    mut controlled: Query<(&MoveSpeed, &mut Transform), With<Controlled>>,
) {
    if !game_view.hovered {
        return;
    }

    for (speed, mut tr) in &mut controlled {
        let dt = time.delta_secs();

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

        // Right-click teleport
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
}
