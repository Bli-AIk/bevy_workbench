//! Top menu bar with mode controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::dock::{TileLayoutState, WorkbenchPanel};
use crate::mode::EditorMode;

/// System that renders the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    mut tile_state: ResMut<TileLayoutState>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("workbench_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // Left side: menus
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    ui.close();
                }
                if ui.button("Open...").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    ui.close();
                }
                if ui.button("Save As...").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Settings").clicked() {
                    tile_state.open_or_focus_panel("settings");
                    ui.close();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    ui.close();
                }
                if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Reset Layout").clicked() {
                    ui.close();
                }
            });

            // Window menu — toggle panel visibility
            let panel_list = tile_state.panel_list();
            ui.menu_button("Window", |ui| {
                for (str_id, title, visible) in &panel_list {
                    let text = if *visible {
                        egui::RichText::new(title)
                    } else {
                        egui::RichText::new(title).weak()
                    };
                    if ui.selectable_label(*visible, text).clicked() {
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
                        .add_sized([button_w, 18.0], egui::Button::new("Play"))
                        .clicked()
                    {
                        next_mode.set(EditorMode::Play);
                    }
                }
                EditorMode::Play => {
                    if ui
                        .add_sized([button_w, 18.0], egui::Button::new("Pause"))
                        .clicked()
                    {
                        next_mode.set(EditorMode::Pause);
                    }
                    if ui
                        .add_sized([button_w, 18.0], egui::Button::new("Stop"))
                        .clicked()
                    {
                        next_mode.set(EditorMode::Edit);
                    }
                }
                EditorMode::Pause => {
                    if ui
                        .add_sized([button_w, 18.0], egui::Button::new("Resume"))
                        .clicked()
                    {
                        next_mode.set(EditorMode::Play);
                    }
                    if ui
                        .add_sized([button_w, 18.0], egui::Button::new("Stop"))
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
    /// Set to true when user clicks Save.
    pub save_requested: bool,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            edited_scale: 1.0,
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
            });

        ui.separator();
        if ui.button("Save").clicked() {
            self.save_requested = true;
        }
    }
}
