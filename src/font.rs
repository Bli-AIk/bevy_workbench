//! Font management for the workbench editor.
//!
//! Embeds Source Han Sans CN as the default CJK font and provides
//! configuration to override with a custom font file.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

/// Embedded CJK font (Source Han Sans CN Regular, ~8 MB).
const EMBEDDED_CJK_FONT: &[u8] = include_bytes!("../fonts/SourceHanSansCN-Regular.otf");

/// Font configuration stored in settings.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FontConfig {
    /// Optional path to a custom font file. When set, this font is used
    /// instead of the embedded CJK font.
    #[serde(default)]
    pub custom_font_path: Option<String>,
}

/// Resource tracking whether fonts have been installed into the egui context.
#[derive(Resource, Default)]
pub struct FontState {
    pub installed: bool,
}

/// System that installs CJK font into the egui context on first run.
pub fn install_fonts_system(
    mut contexts: EguiContexts,
    settings: Res<crate::config::WorkbenchSettings>,
    mut font_state: ResMut<FontState>,
) {
    if font_state.installed {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let font_data = if let Some(ref path) = settings.font.custom_font_path {
        match std::fs::read(path) {
            Ok(data) => {
                info!("Loaded custom font from: {path}");
                data
            }
            Err(e) => {
                warn!("Failed to load custom font '{path}': {e}, using embedded CJK font");
                EMBEDDED_CJK_FONT.to_vec()
            }
        }
    } else {
        EMBEDDED_CJK_FONT.to_vec()
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "cjk".to_owned(),
        egui::FontData::from_owned(font_data).into(),
    );
    // Append CJK as fallback for Proportional family
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("cjk".to_owned());
    // Also add as fallback for Monospace
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("cjk".to_owned());

    ctx.set_fonts(fonts);
    font_state.installed = true;
    info!("CJK font installed into egui context");
}
