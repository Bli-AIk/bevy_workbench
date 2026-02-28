//! Game View: renders the game world to a texture and displays it in an egui panel.

use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::state::prelude::DespawnOnEnter;

use crate::dock::{TileLayoutState, WorkbenchPanel};
use crate::mode::EditorMode;

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
    /// Resolution of the render target.
    pub resolution: UVec2,
}

impl Default for GameViewState {
    fn default() -> Self {
        Self {
            render_target: Handle::default(),
            egui_texture_id: None,
            resolution: UVec2::new(1280, 720),
        }
    }
}

/// Plugin that sets up the game view render-to-texture pipeline.
pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameViewState::default())
            .insert_resource(GameViewFocus::default())
            .add_systems(Startup, setup_render_target)
            .add_systems(OnEnter(EditorMode::Play), spawn_game_view_camera);
    }
}

/// Creates the render target texture (persistent, survives Play/Stop cycles).
fn setup_render_target(mut images: ResMut<Assets<Image>>, mut state: ResMut<GameViewState>) {
    let image = Image::new_target_texture(
        state.resolution.x,
        state.resolution.y,
        TextureFormat::Bgra8UnormSrgb,
        Some(TextureFormat::Bgra8UnormSrgb),
    );
    state.render_target = images.add(image);
}

/// Spawns the preview camera on Play if one doesn't already exist (e.g., after Resume).
/// `DespawnOnEnter(Edit)` ensures cleanup on Stop.
fn spawn_game_view_camera(
    mut commands: Commands,
    state: Res<GameViewState>,
    existing: Query<(), With<GameViewCamera>>,
) {
    if !existing.is_empty() {
        return;
    }
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::from(state.render_target.clone()),
        GameViewCamera,
        DespawnOnEnter(EditorMode::Edit),
    ));
}

/// System that registers the render target as an egui texture and syncs to the panel.
pub fn game_view_sync_system(
    mut state: ResMut<GameViewState>,
    mut contexts: bevy_egui::EguiContexts,
    mut tile_state: ResMut<TileLayoutState>,
    mode: Res<State<EditorMode>>,
    i18n: Res<crate::i18n::I18n>,
) {
    // Register texture with egui (once)
    if state.egui_texture_id.is_none() && state.render_target != Handle::default() {
        let texture_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(
            state.render_target.clone(),
        ));
        state.egui_texture_id = Some(texture_id);
    }

    let is_playing = matches!(mode.get(), EditorMode::Play | EditorMode::Pause);

    // Sync state to the panel
    if let Some(panel) = tile_state.get_panel_mut::<GameViewPanel>("workbench_game_view") {
        panel.egui_texture_id = state.egui_texture_id;
        panel.resolution = state.resolution;
        panel.is_playing = is_playing;
        panel.press_play_text = i18n.t("game-view-press-play");
    }
}

/// Resource tracking game view focus and rect (for input routing).
#[derive(Resource, Default)]
pub struct GameViewFocus {
    /// Whether the game view panel is hovered.
    pub hovered: bool,
    /// The screen-space rect of the rendered game image.
    pub image_rect: Option<egui::Rect>,
    /// Resolution of the render target.
    pub resolution: UVec2,
    /// Cursor position in render target coordinates (if pointer is over the game view).
    pub cursor_viewport_pos: Option<Vec2>,
}

/// Built-in Game View dock panel that displays the render target texture.
#[derive(Default)]
pub struct GameViewPanel {
    /// The egui texture ID for the render target (synced by game_view_sync_system).
    pub egui_texture_id: Option<egui::TextureId>,
    /// Resolution of the render target (for aspect-ratio scaling).
    pub resolution: UVec2,
    /// Whether the game is currently playing (has a camera rendering).
    pub is_playing: bool,
    /// Localized "press play" text.
    pub press_play_text: String,
}

impl WorkbenchPanel for GameViewPanel {
    fn id(&self) -> &str {
        "workbench_game_view"
    }

    fn title(&self) -> String {
        "Game View".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui) {}

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if !self.is_playing {
            // Reset focus when not playing
            if let Some(mut focus) = world.get_resource_mut::<GameViewFocus>() {
                focus.hovered = false;
                focus.image_rect = None;
            }
            ui.centered_and_justified(|ui| {
                ui.label(&self.press_play_text);
            });
            return;
        }

        if let Some(tex_id) = self.egui_texture_id {
            let available = ui.available_size();
            let res = if self.resolution.x > 0 && self.resolution.y > 0 {
                self.resolution
            } else {
                UVec2::new(1280, 720)
            };
            let aspect = res.x as f32 / res.y as f32;

            // Fit-to-panel while preserving aspect ratio
            let (w, h) = {
                let w_from_h = available.y * aspect;
                if w_from_h <= available.x {
                    (w_from_h, available.y)
                } else {
                    (available.x, available.x / aspect)
                }
            };

            // Center the image within the panel
            let response = ui
                .with_layout(
                    egui::Layout::centered_and_justified(ui.layout().main_dir()),
                    |ui| ui.image(egui::load::SizedTexture::new(tex_id, [w, h])),
                )
                .inner;

            // Update focus resource
            let hovered = response.hovered();
            let image_rect = response.rect;

            // Compute cursor position in render target coordinates
            let cursor_viewport_pos = if hovered {
                ui.ctx().pointer_latest_pos().and_then(|pointer_pos| {
                    if image_rect.contains(pointer_pos) {
                        let uv_x = (pointer_pos.x - image_rect.left()) / image_rect.width();
                        let uv_y = (pointer_pos.y - image_rect.top()) / image_rect.height();
                        Some(Vec2::new(uv_x * res.x as f32, uv_y * res.y as f32))
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            // Show focus border when hovered
            if hovered {
                let painter = ui.painter();
                painter.rect_stroke(
                    image_rect,
                    0.0,
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 180, 255)),
                    egui::StrokeKind::Outside,
                );
            }

            if let Some(mut focus) = world.get_resource_mut::<GameViewFocus>() {
                focus.hovered = hovered;
                focus.image_rect = Some(image_rect);
                focus.resolution = res;
                focus.cursor_viewport_pos = cursor_viewport_pos;
            }
        } else {
            if let Some(mut focus) = world.get_resource_mut::<GameViewFocus>() {
                focus.hovered = false;
                focus.image_rect = None;
            }
            ui.centered_and_justified(|ui| {
                ui.label("No render target");
            });
        }
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        false
    }
}
