//! Game View: renders the game world to a texture and displays it in an egui panel.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::dock::{TileLayoutState, WorkbenchPanel};

/// Marker component for the preview camera that renders to the game view texture.
#[derive(Component)]
pub struct GameViewCamera;

/// Resource holding the game view render state.
#[derive(Resource)]
pub struct GameViewState {
    /// Handle to the render target image.
    pub render_target: Handle<Image>,
    /// The egui texture ID (registered on first use).
    pub egui_texture_id: Option<egui::TextureId>,
    /// The preview camera entity.
    pub preview_camera: Option<Entity>,
    /// Resolution of the render target.
    pub resolution: UVec2,
}

impl Default for GameViewState {
    fn default() -> Self {
        Self {
            render_target: Handle::default(),
            egui_texture_id: None,
            preview_camera: None,
            resolution: UVec2::new(1280, 720),
        }
    }
}

/// Plugin that sets up the game view render-to-texture pipeline.
pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewState::default())
            .add_systems(Startup, setup_game_view);
    }
}

fn setup_game_view(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<GameViewState>,
) {
    let image = Image::new_target_texture(
        state.resolution.x,
        state.resolution.y,
        TextureFormat::Bgra8UnormSrgb,
        Some(TextureFormat::Bgra8UnormSrgb),
    );
    let image_handle = images.add(image);
    state.render_target = image_handle.clone();

    let camera_entity = commands
        .spawn((
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            RenderTarget::from(image_handle),
            GameViewCamera,
        ))
        .id();

    state.preview_camera = Some(camera_entity);
}

/// System that registers the render target as an egui texture and syncs to the panel.
pub fn game_view_sync_system(
    mut state: ResMut<GameViewState>,
    mut contexts: bevy_egui::EguiContexts,
    mut tile_state: ResMut<TileLayoutState>,
) {
    // Register texture with egui (once)
    if state.egui_texture_id.is_none() && state.render_target != Handle::default() {
        let texture_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
            state.render_target.clone(),
        ));
        state.egui_texture_id = Some(texture_id);
    }

    // Sync texture ID and resolution to the panel
    if let Some(panel) = tile_state.get_panel_mut::<GameViewPanel>("workbench_game_view") {
        panel.egui_texture_id = state.egui_texture_id;
        panel.resolution = state.resolution;
    }
}

/// Built-in Game View dock panel that displays the render target texture.
#[derive(Default)]
pub struct GameViewPanel {
    /// The egui texture ID for the render target (synced by game_view_sync_system).
    pub egui_texture_id: Option<egui::TextureId>,
    /// Resolution of the render target (for aspect-ratio scaling).
    pub resolution: UVec2,
}

impl WorkbenchPanel for GameViewPanel {
    fn id(&self) -> &str {
        "workbench_game_view"
    }

    fn title(&self) -> String {
        "Game View".to_string()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        if let Some(tex_id) = self.egui_texture_id {
            let available = ui.available_size();
            let res = if self.resolution.x > 0 && self.resolution.y > 0 {
                self.resolution
            } else {
                UVec2::new(1280, 720)
            };
            let aspect = res.x as f32 / res.y as f32;

            // Fit-to-panel while preserving aspect ratio (Unity-style)
            let (w, h) = {
                let w_from_h = available.y * aspect;
                if w_from_h <= available.x {
                    (w_from_h, available.y)
                } else {
                    (available.x, available.x / aspect)
                }
            };

            // Center the image within the panel
            ui.with_layout(
                egui::Layout::centered_and_justified(ui.layout().main_dir()),
                |ui| {
                    ui.image(egui::load::SizedTexture::new(tex_id, [w, h]));
                },
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No render target");
            });
        }
    }

    fn closable(&self) -> bool {
        false
    }
}
