//! # bevy_workbench
//!
//! A mid-level editor scaffold for Bevy, positioned between bevy-inspector-egui
//! (too basic) and Unity/Godot editors (too complex).
//!
//! ## We Don't Make Scenes
//!
//! Many games don't have a "scene" concept. Bevy Workbench follows this philosophy:
//! - No default scene management
//! - No scene-based asset loading
//! - No scene hierarchy by default

pub mod bench_ui;
pub mod config;
pub mod console;
pub mod dock;
pub mod font;
pub mod game_view;
pub mod i18n;
pub mod inspector;
pub mod keybind;
pub mod layout;
pub mod menu_bar;
pub mod mode;
pub mod prelude;
pub mod theme;
pub mod undo;

use bevy::prelude::*;
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass};

/// Main configuration for the workbench editor.
#[derive(Resource, Clone)]
pub struct WorkbenchConfig {
    pub layout: layout::LayoutMode,
    pub show_menu_bar: bool,
    pub show_console: bool,
}

impl Default for WorkbenchConfig {
    fn default() -> Self {
        Self {
            layout: layout::LayoutMode::Auto,
            show_menu_bar: true,
            show_console: true,
        }
    }
}

/// The main workbench plugin. Add this to your Bevy app to enable the editor.
#[derive(Default)]
pub struct WorkbenchPlugin {
    pub config: WorkbenchConfig,
}

impl Plugin for WorkbenchPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        // Disable auto PrimaryEguiContext — we assign it explicitly to the
        // window camera so the GameViewCamera doesn't accidentally steal it.
        if let Some(mut settings) = app.world_mut().get_resource_mut::<EguiGlobalSettings>() {
            settings.auto_create_primary_context = false;
        }

        // Load or create config (project-local)
        let config_path = config::ConfigPath::default();
        let settings = config::WorkbenchSettings::load(&config_path.0);

        app.insert_resource(self.config.clone())
            .insert_resource(settings.clone())
            .insert_resource(config_path)
            .insert_resource(dock::LayoutPath::default())
            .init_state::<mode::EditorMode>()
            .insert_resource(mode::ModeController::default())
            .insert_resource(mode::GameClock::default())
            .init_schedule(mode::GameSchedule)
            .insert_resource(undo::UndoStack::default())
            .init_resource::<keybind::KeyBindings>()
            .insert_resource(layout::LayoutState::new(self.config.layout))
            .insert_resource(dock::TileLayoutState::default())
            .init_resource::<console::ConsoleState>()
            .insert_resource(inspector::InspectorSelection::default())
            .insert_resource(theme::ThemeState {
                config: settings.theme.clone(),
                ..Default::default()
            })
            .insert_resource(i18n::I18n::new(settings.locale))
            .insert_resource(font::FontState::default())
            .add_systems(Update, layout::detect_layout_system)
            .add_systems(Update, mode::mode_input_system)
            .add_systems(Update, mode::run_game_schedule_system)
            .add_systems(OnEnter(mode::EditorMode::Play), mode::on_enter_play)
            .add_systems(
                OnEnter(mode::EditorMode::Play),
                console::console_auto_clear_system,
            )
            .add_systems(OnEnter(mode::EditorMode::Pause), mode::on_enter_pause)
            .add_systems(OnEnter(mode::EditorMode::Edit), mode::on_enter_edit)
            .add_systems(Update, undo::undo_input_system)
            .add_systems(PreUpdate, assign_primary_egui_context_system)
            .add_systems(PreUpdate, console::console_drain_system)
            .add_systems(PreUpdate, inspector::mark_internal_entities_system)
            // UI systems must run in EguiPrimaryContextPass (bevy_egui 0.39 multi-pass mode)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    (
                        config::config_apply_system,
                        font::install_fonts_system,
                        theme::apply_theme_system,
                    )
                        .chain(),
                    game_view::game_view_sync_system,
                    menu_bar::menu_bar_system,
                    dock::tiles_ui_system,
                )
                    .chain(),
            );

        // Register built-in panels
        app.register_panel(game_view::GameViewPanel::default());
        app.register_panel(inspector::InspectorPanel);
        if self.config.show_console {
            app.register_panel(console::ConsolePanel);
        }
        // Settings panel initialized with loaded values
        let settings_panel = menu_bar::SettingsPanel {
            edited_scale: settings.ui_scale,
            edited_edit_theme: settings.theme.edit_theme,
            edited_play_theme: settings.theme.play_theme,
            edited_edit_brightness: settings.theme.edit_brightness,
            edited_play_brightness: settings.theme.play_brightness,
            edited_locale: settings.locale,
            edited_font_path: settings.font.custom_font_path.clone(),
            ..Default::default()
        };
        app.register_panel(settings_panel);
        app.register_panel(menu_bar::KeybindingsPanel);
        app.register_panel(undo::UndoHistoryPanel);
    }
}

/// Extension trait for registering custom panels with the app.
pub trait WorkbenchApp {
    /// Register a custom panel. The panel will be added to the dock layout.
    fn register_panel(&mut self, panel: impl dock::WorkbenchPanel) -> &mut Self;
}

impl WorkbenchApp for App {
    fn register_panel(&mut self, panel: impl dock::WorkbenchPanel) -> &mut Self {
        let mut tile_state = self
            .world_mut()
            .get_resource_mut::<dock::TileLayoutState>()
            .expect("WorkbenchPlugin must be added before registering panels");
        tile_state.add_panel(Box::new(panel));
        self
    }
}

/// Assigns PrimaryEguiContext to the first window-targeting camera that doesn't
/// have one yet. Runs every frame until assigned. This replaces bevy_egui's
/// auto_create_primary_context so the GameViewCamera never steals it.
#[allow(clippy::type_complexity)]
fn assign_primary_egui_context_system(
    mut commands: Commands,
    cameras: Query<
        (Entity, Option<&bevy::camera::RenderTarget>),
        (
            With<bevy::camera::Camera>,
            Without<bevy_egui::PrimaryEguiContext>,
            Without<game_view::GameViewCamera>,
        ),
    >,
    existing: Query<(), With<bevy_egui::PrimaryEguiContext>>,
) {
    if !existing.is_empty() {
        return;
    }
    // Find the first camera NOT used for render-to-texture
    for (entity, target) in &cameras {
        let is_image_target = matches!(
            target,
            Some(bevy::camera::RenderTarget::Image(_))
                | Some(bevy::camera::RenderTarget::TextureView(_))
        );
        if !is_image_target {
            commands.entity(entity).insert((
                bevy_egui::EguiContext::default(),
                bevy_egui::PrimaryEguiContext,
                Name::new("workbench_ui_camera"),
                inspector::WorkbenchInternal,
            ));
            return;
        }
    }
}
