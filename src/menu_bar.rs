//! Top menu bar with mode controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::dock::{TileLayoutState, WorkbenchPanel};
use crate::mode::EditorMode;
use crate::theme::gray;

/// System that renders the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    mut tile_state: ResMut<TileLayoutState>,
    i18n: Res<crate::i18n::I18n>,
    mut undo_stack: ResMut<crate::undo::UndoStack>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("workbench_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // Left side: menus
            ui.menu_button(i18n.t("menu-file"), |ui| {
                if ui.button(i18n.t("menu-file-settings")).clicked() {
                    tile_state.request_open_panel("settings");
                    ui.close();
                }
            });

            ui.menu_button(i18n.t("menu-edit"), |ui| {
                let undo_label = if let Some(desc) = undo_stack.undo_description() {
                    format!("{} ({})", i18n.t("menu-edit-undo"), desc)
                } else {
                    i18n.t("menu-edit-undo")
                };
                if ui
                    .add_enabled(undo_stack.can_undo(), egui::Button::new(undo_label))
                    .clicked()
                {
                    undo_stack.undo_requested = true;
                    ui.close();
                }
                let redo_label = if let Some(desc) = undo_stack.redo_description() {
                    format!("{} ({})", i18n.t("menu-edit-redo"), desc)
                } else {
                    i18n.t("menu-edit-redo")
                };
                if ui
                    .add_enabled(undo_stack.can_redo(), egui::Button::new(redo_label))
                    .clicked()
                {
                    undo_stack.redo_requested = true;
                    ui.close();
                }
                ui.separator();
                if ui.button("Keybindings...").clicked() {
                    tile_state.request_open_panel("keybindings");
                    ui.close();
                }
                if ui.button("Undo History").clicked() {
                    tile_state.request_open_panel("undo_history");
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
                            tile_state.request_open_panel(str_id);
                        }
                        ui.close();
                    }
                }
            });
        });
    });

    // Secondary toolbar — centered Play/Pause/Stop
}

/// System that renders the Play/Pause/Stop toolbar.
/// Only added when `WorkbenchConfig::show_toolbar` is `true`.
pub fn toolbar_system(
    mut contexts: EguiContexts,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    i18n: Res<crate::i18n::I18n>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

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
    /// Edited custom font path (None = use embedded).
    pub edited_font_path: Option<String>,
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
            edited_font_path: None,
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
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                self.settings_ui(ui);
            });
    }

    fn default_visible(&self) -> bool {
        false
    }
}

impl SettingsPanel {
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
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

                ui.label("Custom Font:");
                let display = self.edited_font_path.as_deref().unwrap_or("(embedded)");
                if ui.button(display).clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Font", &["otf", "ttf", "ttc"])
                        .pick_file()
                {
                    self.edited_font_path = Some(path.display().to_string());
                }
                ui.end_row();
            });

        ui.separator();
        if ui.button("Save").clicked() {
            self.save_requested = true;
        }
    }
}

/// Keybindings settings panel — allows users to view and modify editor keybindings.
pub struct KeybindingsPanel;

/// Tracks which keybind slot is currently being recorded.
#[derive(Resource, Default)]
pub(crate) struct KeyRecordState {
    /// Which action is being recorded (e.g., "undo", "redo").
    recording: Option<String>,
    /// Which binding index within the slot (None = add new).
    recording_index: Option<usize>,
}

impl WorkbenchPanel for KeybindingsPanel {
    fn id(&self) -> &str {
        "keybindings"
    }

    fn title(&self) -> String {
        "Keybindings".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui) {}

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut bindings = world
            .remove_resource::<crate::keybind::KeyBindings>()
            .unwrap_or_default();
        let mut record_state = world
            .remove_resource::<KeyRecordState>()
            .unwrap_or_default();

        // Detect key press for recording
        if let Some(ref action) = record_state.recording.clone()
            && let Some(input) = world.get_resource::<ButtonInput<KeyCode>>()
        {
            // Check for Escape to cancel
            if input.just_pressed(KeyCode::Escape) {
                record_state.recording = None;
                record_state.recording_index = None;
            } else {
                // Find the first non-modifier key just pressed
                let pressed_key = find_just_pressed_key(input);
                if let Some(key) = pressed_key {
                    let ctrl =
                        input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);
                    let shift =
                        input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
                    let alt = input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight);
                    let new_bind = crate::keybind::KeyBind {
                        key,
                        ctrl,
                        shift,
                        alt,
                    };

                    let slot = match action.as_str() {
                        "undo" => &mut bindings.undo,
                        "redo" => &mut bindings.redo,
                        "play_stop" => &mut bindings.play_stop,
                        "pause_resume" => &mut bindings.pause_resume,
                        _ => {
                            record_state.recording = None;
                            world.insert_resource(bindings);
                            world.insert_resource(record_state);
                            return;
                        }
                    };

                    if let Some(idx) = record_state.recording_index {
                        if idx < slot.bindings.len() {
                            slot.bindings[idx] = new_bind;
                        }
                    } else {
                        slot.bindings.push(new_bind);
                    }

                    record_state.recording = None;
                    record_state.recording_index = None;
                }
            }
        }

        egui::Frame::NONE
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                ui.heading("Keybindings");
                ui.separator();
                ui.label("Click a binding to re-record. Press Esc to cancel.");
                ui.add_space(4.0);

                egui::Grid::new("keybind_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        keybind_row(ui, "Undo", "undo", &mut bindings.undo, &mut record_state);
                        keybind_row(ui, "Redo", "redo", &mut bindings.redo, &mut record_state);
                        keybind_row(
                            ui,
                            "Play / Stop",
                            "play_stop",
                            &mut bindings.play_stop,
                            &mut record_state,
                        );
                        keybind_row(
                            ui,
                            "Pause / Resume",
                            "pause_resume",
                            &mut bindings.pause_resume,
                            &mut record_state,
                        );
                    });

                ui.separator();
                if ui.button("Reset to Defaults").clicked() {
                    bindings = crate::keybind::KeyBindings::default();
                    record_state.recording = None;
                }
            });

        world.insert_resource(bindings);
        world.insert_resource(record_state);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        false
    }
}

/// Helper to draw an editable keybinding row.
fn keybind_row(
    ui: &mut egui::Ui,
    label: &str,
    action_id: &str,
    slot: &mut crate::keybind::KeyBindSlot,
    record_state: &mut KeyRecordState,
) {
    ui.label(label);
    ui.horizontal(|ui| {
        let is_recording = record_state
            .recording
            .as_deref()
            .is_some_and(|r| r == action_id);

        for (i, bind) in slot.bindings.iter().enumerate() {
            if i > 0 {
                ui.label("/");
            }

            let recording_this = is_recording && record_state.recording_index == Some(i);
            let text = if recording_this {
                egui::RichText::new("⏺ Press key...")
                    .monospace()
                    .color(egui::Color32::YELLOW)
                    .background_color(gray::S200)
            } else {
                egui::RichText::new(bind.label())
                    .monospace()
                    .background_color(gray::S300)
            };

            if ui.button(text).clicked() && !is_recording {
                record_state.recording = Some(action_id.to_string());
                record_state.recording_index = Some(i);
            }
        }

        // "+" button to add a new alternative binding
        if !is_recording && ui.small_button("+").clicked() {
            record_state.recording = Some(action_id.to_string());
            record_state.recording_index = None;
        }

        // "×" button to remove the last binding (keep at least 1)
        if !is_recording && slot.bindings.len() > 1 && ui.small_button("×").clicked() {
            slot.bindings.pop();
        }
    });
    ui.end_row();
}

/// Find the first non-modifier key that was just pressed.
fn find_just_pressed_key(input: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    let non_modifier_keys = [
        KeyCode::KeyA,
        KeyCode::KeyB,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
        KeyCode::KeyF,
        KeyCode::KeyG,
        KeyCode::KeyH,
        KeyCode::KeyI,
        KeyCode::KeyJ,
        KeyCode::KeyK,
        KeyCode::KeyL,
        KeyCode::KeyM,
        KeyCode::KeyN,
        KeyCode::KeyO,
        KeyCode::KeyP,
        KeyCode::KeyQ,
        KeyCode::KeyR,
        KeyCode::KeyS,
        KeyCode::KeyT,
        KeyCode::KeyU,
        KeyCode::KeyV,
        KeyCode::KeyW,
        KeyCode::KeyX,
        KeyCode::KeyY,
        KeyCode::KeyZ,
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::F1,
        KeyCode::F2,
        KeyCode::F3,
        KeyCode::F4,
        KeyCode::F5,
        KeyCode::F6,
        KeyCode::F7,
        KeyCode::F8,
        KeyCode::F9,
        KeyCode::F10,
        KeyCode::F11,
        KeyCode::F12,
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Tab,
        KeyCode::Delete,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ];

    non_modifier_keys
        .iter()
        .find(|&&key| input.just_pressed(key))
        .copied()
}
