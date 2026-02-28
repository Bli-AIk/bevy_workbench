//! Top menu bar with mode controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::dock::{TileLayoutState, WorkbenchPanel};
use crate::mode::EditorMode;
use crate::theme::gray;

/// System that renders the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    mut tile_state: ResMut<TileLayoutState>,
    i18n: Res<crate::i18n::I18n>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("workbench_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // Left side: menus
            ui.menu_button(i18n.t("menu-file"), |ui| {
                if ui.button(i18n.t("menu-file-new")).clicked() {
                    ui.close();
                }
                if ui.button(i18n.t("menu-file-open")).clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button(i18n.t("menu-file-save")).clicked() {
                    ui.close();
                }
                if ui.button(i18n.t("menu-file-save-as")).clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button(i18n.t("menu-file-settings")).clicked() {
                    tile_state.open_or_focus_panel("settings");
                    ui.close();
                }
            });

            ui.menu_button(i18n.t("menu-edit"), |ui| {
                if ui.button(i18n.t("menu-edit-undo")).clicked() {
                    ui.close();
                }
                if ui.button(i18n.t("menu-edit-redo")).clicked() {
                    ui.close();
                }
            });

            ui.menu_button(i18n.t("menu-view"), |ui| {
                if ui.button(i18n.t("menu-view-save-layout")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title(i18n.t("dialog-save-layout"))
                        .add_filter("JSON", &["json"])
                        .set_file_name("layout.json")
                        .save_file()
                    {
                        tile_state.layout_save_path = Some(path);
                    }
                    ui.close();
                }
                if ui.button(i18n.t("menu-view-load-layout")).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title(i18n.t("dialog-load-layout"))
                        .add_filter("JSON", &["json"])
                        .pick_file()
                    {
                        tile_state.layout_load_path = Some(path);
                    }
                    ui.close();
                }
                ui.separator();
                if ui.button(i18n.t("menu-view-reset-layout")).clicked() {
                    tile_state.layout_reset_requested = true;
                    ui.close();
                }
            });

            // Window menu — toggle panel visibility
            let panel_list = tile_state.panel_list();
            ui.menu_button(i18n.t("menu-window"), |ui| {
                for (str_id, title, visible) in &panel_list {
                    let text = if *visible {
                        egui::RichText::new(title)
                    } else {
                        egui::RichText::new(title).weak()
                    };
                    if ui.button(text).clicked() {
                        if *visible {
                            if let Some(&panel_id) = tile_state.panel_id_map.get(str_id.as_str())
                                && let Some(&tile_id) = tile_state.panel_tile_map.get(&panel_id)
                            {
                                tile_state.hide_tile(tile_id);
                            }
                        } else {
                            tile_state.open_or_focus_panel(str_id);
                        }
                        ui.close();
                    }
                }
            });
        });
    });

    // Secondary toolbar — centered Play/Pause/Stop
    let btn_fill = gray::S250;
    egui::TopBottomPanel::top("workbench_toolbar").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            let button_w = 80.0;
            let n_buttons: f32 = match current_mode.get() {
                EditorMode::Edit => 1.0,
                _ => 2.0,
            };
            let total = button_w * n_buttons + 4.0 * (n_buttons - 1.0_f32).max(0.0);
            let pad = ((ui.available_width() - total) / 2.0).max(0.0);
            ui.add_space(pad);

            match current_mode.get() {
                EditorMode::Edit => {
                    if ui
                        .add_sized(
                            [button_w, 18.0],
                            egui::Button::new(i18n.t("toolbar-play")).fill(btn_fill),
                        )
                        .clicked()
                    {
                        next_mode.set(EditorMode::Play);
                    }
                }
                EditorMode::Play => {
                    if ui
                        .add_sized(
                            [button_w, 18.0],
                            egui::Button::new(i18n.t("toolbar-pause")).fill(btn_fill),
                        )
                        .clicked()
                    {
                        next_mode.set(EditorMode::Pause);
                    }
                    if ui
                        .add_sized(
                            [button_w, 18.0],
                            egui::Button::new(i18n.t("toolbar-stop")).fill(btn_fill),
                        )
                        .clicked()
                    {
                        next_mode.set(EditorMode::Edit);
                    }
                }
                EditorMode::Pause => {
                    if ui
                        .add_sized(
                            [button_w, 18.0],
                            egui::Button::new(i18n.t("toolbar-resume")).fill(btn_fill),
                        )
                        .clicked()
                    {
                        next_mode.set(EditorMode::Play);
                    }
                    if ui
                        .add_sized(
                            [button_w, 18.0],
                            egui::Button::new(i18n.t("toolbar-stop")).fill(btn_fill),
                        )
                        .clicked()
                    {
                        next_mode.set(EditorMode::Edit);
                    }
                }
            }
        });
    });
}

/// Settings panel — displayed as a tab in the tile layout.
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
    /// Set to true when user clicks Save.
    pub save_requested: bool,
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
            save_requested: false,
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
        ui.heading("Editor Settings");
        ui.separator();

        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("UI Scale:");
                ui.add(egui::Slider::new(&mut self.edited_scale, 0.5..=2.0).step_by(0.25));
                ui.end_row();

                ui.label("Edit Theme:");
                egui::ComboBox::from_id_salt("edit_theme")
                    .selected_text(self.edited_edit_theme.label())
                    .show_ui(ui, |ui| {
                        for preset in crate::theme::ThemePreset::ALL {
                            ui.selectable_value(
                                &mut self.edited_edit_theme,
                                *preset,
                                preset.label(),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Edit Brightness:");
                ui.add(
                    egui::Slider::new(&mut self.edited_edit_brightness, 0.2..=1.0).step_by(0.05),
                );
                ui.end_row();

                ui.label("Play Theme:");
                egui::ComboBox::from_id_salt("play_theme")
                    .selected_text(self.edited_play_theme.label())
                    .show_ui(ui, |ui| {
                        for preset in crate::theme::ThemePreset::ALL {
                            ui.selectable_value(
                                &mut self.edited_play_theme,
                                *preset,
                                preset.label(),
                            );
                        }
                    });
                ui.end_row();

                ui.label("Play Brightness:");
                ui.add(
                    egui::Slider::new(&mut self.edited_play_brightness, 0.2..=1.0).step_by(0.05),
                );
                ui.end_row();

                ui.label("Language:");
                egui::ComboBox::from_id_salt("locale")
                    .selected_text(self.edited_locale.label())
                    .show_ui(ui, |ui| {
                        for locale in crate::i18n::Locale::ALL {
                            ui.selectable_value(&mut self.edited_locale, *locale, locale.label());
                        }
                    });
                ui.end_row();
            });

        ui.separator();
        if ui.button("Save").clicked() {
            self.save_requested = true;
        }
    }

    fn default_visible(&self) -> bool {
        false
    }
}
