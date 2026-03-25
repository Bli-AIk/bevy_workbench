//! # settings_panel.rs
//!
//! # settings_panel.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the settings tab that `bevy_workbench` embeds in its menu system. It stores
//! the editable UI-facing copies of workbench settings and renders the controls used to tweak
//! themes, scale, locale, font choice, and application-provided custom sections.
//!
//! 定义了 `bevy_workbench` 菜单系统里的设置面板。它保存面向 UI 的可编辑设置副本，
//! 并渲染用于调整主题、缩放、语言、字体选择以及应用侧自定义设置区块的控件。

use crate::dock::WorkbenchPanel;

/// Settings panel — displayed as a tab in the tile layout.
/// A custom section to inject into the Settings panel.
pub struct SettingsSection {
    /// Section heading.
    pub label: String,
    /// UI rendering callback.
    pub ui_fn: Box<dyn FnMut(&mut egui::Ui) + Send + Sync>,
}

pub struct SettingsPanel {
    /// Edited scale value (not yet saved).
    pub edited_scale: f32,
    /// Edited edit-mode theme.
    pub edited_edit_theme: crate::theme::ThemePreset,
    /// Edited play-mode theme.
    pub edited_play_theme: crate::theme::ThemePreset,
    /// Edited edit-mode brightness.
    pub edited_edit_brightness: f32,
    /// Edited play-mode brightness.
    pub edited_play_brightness: f32,
    /// Edited interface language.
    pub edited_locale: crate::i18n::Locale,
    /// Edited custom font path (None = use embedded).
    pub edited_font_path: Option<String>,
    /// Set to true when user clicks Save.
    pub save_requested: bool,
    /// Custom settings sections injected by downstream applications.
    pub custom_sections: Vec<SettingsSection>,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            edited_scale: 1.0,
            edited_edit_theme: crate::theme::ThemePreset::default(),
            edited_play_theme: crate::theme::ThemePreset::Rerun,
            edited_edit_brightness: 1.0,
            edited_play_brightness: 0.6,
            edited_locale: crate::i18n::Locale::default(),
            edited_font_path: None,
            save_requested: false,
            custom_sections: Vec::new(),
        }
    }
}

impl WorkbenchPanel for SettingsPanel {
    fn id(&self) -> &str {
        "settings"
    }

    fn title(&self) -> String {
        "Settings".to_string()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                settings_panel_ui(self, ui);
            });
    }

    fn default_visible(&self) -> bool {
        false
    }
}

fn settings_panel_ui(panel: &mut SettingsPanel, ui: &mut egui::Ui) {
    ui.heading("Editor Settings");
    ui.separator();

    egui::Grid::new("settings_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            ui.label("UI Scale:");
            ui.add(egui::Slider::new(&mut panel.edited_scale, 0.5..=2.0).step_by(0.25));
            ui.end_row();

            ui.label("Edit Theme:");
            egui::ComboBox::from_id_salt("edit_theme")
                .selected_text(panel.edited_edit_theme.label())
                .show_ui(ui, |ui| {
                    for preset in crate::theme::ThemePreset::ALL {
                        ui.selectable_value(&mut panel.edited_edit_theme, *preset, preset.label());
                    }
                });
            ui.end_row();

            ui.label("Edit Brightness:");
            ui.add(egui::Slider::new(&mut panel.edited_edit_brightness, 0.2..=1.0).step_by(0.05));
            ui.end_row();

            ui.label("Play Theme:");
            egui::ComboBox::from_id_salt("play_theme")
                .selected_text(panel.edited_play_theme.label())
                .show_ui(ui, |ui| {
                    for preset in crate::theme::ThemePreset::ALL {
                        ui.selectable_value(&mut panel.edited_play_theme, *preset, preset.label());
                    }
                });
            ui.end_row();

            ui.label("Play Brightness:");
            ui.add(egui::Slider::new(&mut panel.edited_play_brightness, 0.2..=1.0).step_by(0.05));
            ui.end_row();

            ui.label("Language:");
            egui::ComboBox::from_id_salt("locale")
                .selected_text(panel.edited_locale.label())
                .show_ui(ui, |ui| {
                    for locale in crate::i18n::Locale::ALL {
                        ui.selectable_value(&mut panel.edited_locale, *locale, locale.label());
                    }
                });
            ui.end_row();

            ui.label("Custom Font:");
            let display = panel.edited_font_path.as_deref().unwrap_or("(embedded)");
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button(display).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Font", &["otf", "ttf", "ttc"])
                    .pick_file()
            {
                panel.edited_font_path = Some(path.display().to_string());
            }
            #[cfg(target_arch = "wasm32")]
            {
                ui.label(display);
            }
            ui.end_row();
        });

    ui.separator();
    if ui.button("Save").clicked() {
        panel.save_requested = true;
    }

    for section in &mut panel.custom_sections {
        ui.separator();
        ui.heading(&section.label);
        (section.ui_fn)(ui);
    }
}
