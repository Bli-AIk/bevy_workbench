//! Game View: renders the game world to a texture and displays it in an egui panel.

use bevy::camera::RenderTarget;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::state::prelude::DespawnOnEnter;

use crate::dock::{TileLayoutState, WorkbenchPanel};
use crate::mode::EditorMode;
use crate::theme::gray;

/// Marker component for the preview camera that renders to the game view texture.
#[derive(Component)]
pub struct GameViewCamera;

/// Optional resource: when present, GameViewPlugin will hijack the specified camera
/// entity instead of spawning its own. In Play mode, the camera's RenderTarget is
/// redirected to the game view texture and is_active set to true. On Edit, reversed.
///
/// Insert this resource after the external camera entity has been created.
#[derive(Resource)]
pub struct ExternalGameCamera(pub Entity);

/// Zoom mode for the game view panel.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewZoom {
    /// Fit the image within the available panel space, preserving aspect ratio.
    #[default]
    Auto,
    /// Display at a fixed scale factor (1.0 = 100%).
    Fixed(f32),
}

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
            .add_systems(OnEnter(EditorMode::Play), activate_game_view_camera)
            .add_systems(OnEnter(EditorMode::Edit), deactivate_external_camera);
    }
}

/// Creates the render target texture (persistent, survives Play/Stop cycles).
fn setup_render_target(mut images: ResMut<Assets<Image>>, mut state: ResMut<GameViewState>) {
    let mut image = Image::new_target_texture(
        state.resolution.x,
        state.resolution.y,
        TextureFormat::Bgra8UnormSrgb,
        Some(TextureFormat::Bgra8UnormSrgb),
    );
    image.sampler = ImageSampler::nearest();
    state.render_target = images.add(image);
}

/// Activates the game view camera on Play.
///
/// If `ExternalGameCamera` resource exists, hijacks that camera by redirecting its
/// render target to the game view texture. Otherwise, spawns a new internal camera.
fn activate_game_view_camera(
    mut commands: Commands,
    state: Res<GameViewState>,
    existing: Query<(), With<GameViewCamera>>,
    external: Option<Res<ExternalGameCamera>>,
    mut cameras: Query<(&mut Camera, &mut Projection)>,
) {
    if !existing.is_empty() {
        return;
    }

    if let Some(ext) = external {
        if let Ok((mut camera, mut projection)) = cameras.get_mut(ext.0) {
            camera.is_active = true;
            camera.order = -1;
            // Force projection change detection so camera_system re-computes
            // the projection using the render target texture dimensions.
            projection.set_changed();
        }
        commands.entity(ext.0).insert((
            GameViewCamera,
            RenderTarget::from(state.render_target.clone()),
        ));
        return;
    }

    // Fallback: spawn internal camera
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

/// Deactivates external camera on Edit mode. Internal cameras are despawned
/// automatically by `DespawnOnEnter(Edit)`.
fn deactivate_external_camera(
    mut commands: Commands,
    external: Option<Res<ExternalGameCamera>>,
    mut cameras: Query<&mut Camera, With<GameViewCamera>>,
) {
    let Some(ext) = external else { return };
    let Ok(mut camera) = cameras.get_mut(ext.0) else {
        return;
    };
    camera.is_active = false;
    commands
        .entity(ext.0)
        .remove::<GameViewCamera>()
        .remove::<RenderTarget>();
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

/// A toggle button that external code can register on the game view toolbar.
pub struct ToolbarToggle {
    /// Unique identifier for this toggle.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Current enabled state.
    pub enabled: bool,
}

/// Resource for registering custom toolbar items on the game view.
/// External code (e.g., editor plugins) inserts this and reads toggle states.
#[derive(Resource, Default)]
pub struct GameViewToolbar {
    pub toggles: Vec<ToolbarToggle>,
}

impl GameViewToolbar {
    /// Returns the enabled state of a toggle by ID, or None if not found.
    pub fn is_enabled(&self, id: &str) -> Option<bool> {
        self.toggles.iter().find(|t| t.id == id).map(|t| t.enabled)
    }
}

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
    /// Current zoom mode.
    pub zoom: ViewZoom,
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
        game_view_panel_ui(self, ui, world);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        false
    }
}

/// Renders the game view toolbar (zoom, toggles, resolution).
fn game_view_toolbar_ui(ui: &mut egui::Ui, zoom: &mut ViewZoom, world: &mut World, res: UVec2) {
    let zoom_label = match *zoom {
        ViewZoom::Auto => "Auto".to_string(),
        ViewZoom::Fixed(z) => format!("{:.0}%", z * 100.0),
    };
    egui::ComboBox::from_id_salt("game_view_zoom")
        .selected_text(&zoom_label)
        .show_ui(ui, |ui| {
            ui.selectable_value(zoom, ViewZoom::Auto, "Auto");
            ui.selectable_value(zoom, ViewZoom::Fixed(0.5), "50%");
            ui.selectable_value(zoom, ViewZoom::Fixed(0.75), "75%");
            ui.selectable_value(zoom, ViewZoom::Fixed(1.0), "100%");
            ui.selectable_value(zoom, ViewZoom::Fixed(1.25), "125%");
            ui.selectable_value(zoom, ViewZoom::Fixed(1.5), "150%");
            ui.selectable_value(zoom, ViewZoom::Fixed(2.0), "200%");
        });

    if let Some(mut toolbar) = world.get_resource_mut::<GameViewToolbar>() {
        for toggle in &mut toolbar.toggles {
            let icon = if toggle.enabled { "✅" } else { "⬜" };
            if ui
                .small_button(format!("{icon} {}", toggle.label))
                .clicked()
            {
                toggle.enabled = !toggle.enabled;
            }
        }
    }

    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.colored_label(gray::S550, format!("{}×{}", res.x, res.y));
    });
}

/// Computes display size based on zoom mode, resolution, and available space.
fn compute_display_size(zoom: ViewZoom, res: UVec2, available: egui::Vec2) -> egui::Vec2 {
    let aspect = res.x as f32 / res.y as f32;
    match zoom {
        ViewZoom::Auto => {
            let w = available.x;
            let h = w / aspect;
            if h > available.y {
                egui::vec2(available.y * aspect, available.y)
            } else {
                egui::vec2(w, h)
            }
        }
        ViewZoom::Fixed(z) => egui::vec2(res.x as f32 * z, res.y as f32 * z),
    }
}

/// Resets focus state when the game view is not active.
fn reset_game_view_focus(world: &mut World) {
    if let Some(mut focus) = world.get_resource_mut::<GameViewFocus>() {
        focus.hovered = false;
        focus.image_rect = None;
    }
}

/// Main game view panel rendering, extracted to reduce nesting.
fn game_view_panel_ui(panel: &mut GameViewPanel, ui: &mut egui::Ui, world: &mut World) {
    if !panel.is_playing {
        reset_game_view_focus(world);
        ui.centered_and_justified(|ui| {
            ui.label(&panel.press_play_text);
        });
        return;
    }

    let Some(tex_id) = panel.egui_texture_id else {
        reset_game_view_focus(world);
        ui.centered_and_justified(|ui| {
            ui.label("No render target");
        });
        return;
    };

    let res = if panel.resolution.x > 0 && panel.resolution.y > 0 {
        panel.resolution
    } else {
        UVec2::new(1280, 720)
    };

    ui.horizontal(|ui| {
        game_view_toolbar_ui(ui, &mut panel.zoom, world, res);
    });

    ui.separator();

    let available = ui.available_size();
    let display_size = compute_display_size(panel.zoom, res, available);
    let padding = (available - display_size).max(egui::Vec2::ZERO) * 0.5;

    let response = if matches!(panel.zoom, ViewZoom::Fixed(_))
        && (display_size.x > available.x || display_size.y > available.y)
    {
        egui::ScrollArea::both()
            .show(ui, |ui| {
                ui.image(egui::load::SizedTexture::new(tex_id, display_size))
            })
            .inner
    } else {
        ui.add_space(padding.y);
        ui.with_layout(
            egui::Layout::centered_and_justified(ui.layout().main_dir()),
            |ui| ui.image(egui::load::SizedTexture::new(tex_id, display_size)),
        )
        .inner
    };

    let hovered = response.hovered();
    let image_rect = response.rect;

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
}
