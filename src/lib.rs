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
pub mod game_view;
pub mod inspector;
pub mod layout;
pub mod menu_bar;
pub mod mode;
pub mod prelude;
pub mod theme;
pub mod undo;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

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

        // Load or create config (project-local)
        let config_path = config::ConfigPath::default();
        let settings = config::WorkbenchSettings::load(&config_path.0);

        app.insert_resource(self.config.clone())
            .insert_resource(settings.clone())
            .insert_resource(config_path)
            .init_state::<mode::EditorMode>()
            .insert_resource(mode::ModeController::default())
            .insert_resource(undo::UndoStack::default())
            .insert_resource(layout::LayoutState::new(self.config.layout))
            .insert_resource(dock::TileLayoutState::default())
            .insert_resource(console::ConsoleState::default())
            .insert_resource(inspector::InspectorSelection::default())
            .insert_resource(theme::ThemeState::default())
            .add_systems(Update, layout::detect_layout_system)
            .add_systems(Update, mode::mode_input_system)
            .add_systems(Update, undo::undo_input_system)
            // UI systems must run in EguiPrimaryContextPass (bevy_egui 0.39 multi-pass mode)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    config::config_apply_system,
                    theme::apply_theme_system,
                    menu_bar::menu_bar_system,
                    dock::tiles_ui_system,
                )
                    .chain(),
            );

        // Register built-in panels
        app.register_panel(game_view::GameViewPanel);
        app.register_panel(inspector::InspectorPanel);
        if self.config.show_console {
            app.register_panel(console::ConsolePanel);
        }
        // Settings panel initialized with loaded scale
        let settings_panel = menu_bar::SettingsPanel {
            edited_scale: settings.ui_scale,
            ..Default::default()
        };
        app.register_panel(settings_panel);
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
