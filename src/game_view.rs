//! Game View: renders the game world to a texture and displays it in an egui panel.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::dock::WorkbenchPanel;

/// Marker component for the preview camera that renders to the game view texture.
#[derive(Component)]
pub struct GameViewCamera;

/// Zoom mode for the game view.
#[derive(Debug, Clone, Default)]
pub enum ZoomMode {
    /// Auto-fit to panel size while preserving aspect ratio.
    #[default]
    AutoFit,
    /// Fixed zoom percentage (1.0 = 100%).
    Fixed(f32),
}

/// Resource holding the game view render state.
#[derive(Resource)]
pub struct GameViewState {
    /// Handle to the render target image.
    pub render_target: Handle<Image>,
    /// The egui texture ID (registered on first use).
    pub egui_texture_id: Option<egui::TextureId>,
    /// Current zoom mode.
    pub zoom: ZoomMode,
    /// The preview camera entity.
    pub preview_camera: Option<Entity>,
    /// Resolution of the render target.
    pub resolution: UVec2,
    /// Whether the game view panel has focus (for input forwarding).
    pub has_focus: bool,
    /// Pending gesture events from touch input.
    pub pending_gestures: Vec<GameViewGesture>,
}

/// Gesture events recognized within the Game View panel.
#[derive(Debug, Clone)]
pub enum GameViewGesture {
    /// Pinch-to-zoom gesture.
    PinchZoom {
        /// Scale factor (>1.0 = zoom in, <1.0 = zoom out).
        scale: f32,
    },
    /// Two-finger pan/drag gesture.
    PanDrag {
        /// Pan delta in logical pixels.
        delta: bevy::math::Vec2,
    },
    /// Single tap at a position (mapped to render target coordinates).
    Tap {
        /// Position in render target coordinates.
        position: bevy::math::Vec2,
    },
}

impl Default for GameViewState {
    fn default() -> Self {
        Self {
            render_target: Handle::default(),
            egui_texture_id: None,
            zoom: ZoomMode::AutoFit,
            preview_camera: None,
            resolution: UVec2::new(1280, 720),
            has_focus: false,
            pending_gestures: Vec::new(),
        }
    }
}

/// Plugin that sets up the game view render-to-texture pipeline.
pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewState::default())
            .add_systems(Startup, setup_game_view)
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                register_egui_texture_system,
            );
    }
}

fn setup_game_view(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<GameViewState>,
) {
    let size = Extent3d {
        width: state.resolution.x,
        height: state.resolution.y,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("game_view_render_target"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);
    state.render_target = image_handle.clone();

    // Spawn preview camera rendering to the texture
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

/// System that registers the render target image as an egui texture (once).
fn register_egui_texture_system(
    mut state: ResMut<GameViewState>,
    mut contexts: bevy_egui::EguiContexts,
) {
    if state.egui_texture_id.is_some() {
        return;
    }
    if state.render_target == Handle::default() {
        return;
    }
    let texture_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
        state.render_target.clone(),
    ));
    state.egui_texture_id = Some(texture_id);
}

/// Built-in Game View dock panel that displays the render target texture.
pub struct GameViewPanel;

impl WorkbenchPanel for GameViewPanel {
    fn id(&self) -> &str {
        "workbench_game_view"
    }

    fn title(&self) -> String {
        "Game View".to_string()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Game View\n(Render-to-texture — coming soon)");
        });
    }

    fn closable(&self) -> bool {
        false
    }
}
