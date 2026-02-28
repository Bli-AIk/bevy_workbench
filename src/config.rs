//! TOML-based editor configuration.

use bevy::prelude::*;
use std::path::PathBuf;

/// Persistent editor settings, stored as TOML.
#[derive(Resource, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkbenchSettings {
    /// UI scale factor (1.0 = default).
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
}

fn default_ui_scale() -> f32 {
    1.0
}

impl Default for WorkbenchSettings {
    fn default() -> Self {
        Self { ui_scale: 1.0 }
    }
}

impl WorkbenchSettings {
    /// Load from a TOML file, or return defaults if not found.
    pub fn load(path: &std::path::Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
                warn!("Failed to parse {}: {e}", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    /// Save to a TOML file.
    pub fn save(&self, path: &std::path::Path) {
        let content = toml::to_string_pretty(self).expect("serialize WorkbenchSettings");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(path, content) {
            warn!("Failed to save config to {}: {e}", path.display());
        }
    }
}

/// Resource holding the config file path (project-local).
#[derive(Resource)]
pub struct ConfigPath(pub PathBuf);

impl Default for ConfigPath {
    /// Default: `.workbench/settings.toml` in the current working directory.
    fn default() -> Self {
        Self(PathBuf::from(".workbench/settings.toml"))
    }
}

/// System that applies settings and handles save requests from SettingsPanel.
pub fn config_apply_system(
    mut settings: ResMut<WorkbenchSettings>,
    config_path: Res<ConfigPath>,
    mut egui_contexts: Query<&mut bevy_egui::EguiContextSettings>,
    mut tile_state: ResMut<crate::dock::TileLayoutState>,
) {
    // Check if SettingsPanel has a pending save
    if let Some(panel) = tile_state.get_panel_mut::<crate::menu_bar::SettingsPanel>("settings")
        && panel.save_requested
    {
        panel.save_requested = false;
        settings.ui_scale = panel.edited_scale;
        settings.save(&config_path.0);
    }

    // Apply scale via EguiContextSettings (bevy_egui handles viewport sync)
    for mut ctx_settings in &mut egui_contexts {
        if (ctx_settings.scale_factor - settings.ui_scale).abs() > f32::EPSILON {
            ctx_settings.scale_factor = settings.ui_scale;
        }
    }
}
