//! # keybindings_panel.rs
//!
//! # keybindings_panel.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file implements the keybindings editor panel for `bevy_workbench`. It owns the temporary
//! recording state for rebinding shortcuts and renders the world-aware UI that lets users inspect,
//! replace, add, or reset keybind slots.
//!
//! 这个文件实现了 `bevy_workbench` 的快捷键编辑面板。它负责管理重新录制快捷键时的临时状态，
//! 并渲染需要访问 `World` 的 UI，让用户查看、替换、追加或重置各个按键槽位。

use crate::dock::WorkbenchPanel;
use crate::theme::gray;
use bevy::prelude::*;

/// Keybindings settings panel — allows users to view and modify editor keybindings.
pub struct KeybindingsPanel;

/// Tracks which keybind slot is currently being recorded.
#[derive(Resource, Default)]
pub(crate) struct KeyRecordState {
    /// Which action is being recorded (e.g., "undo", "redo").
    pub(crate) recording: Option<String>,
    /// Which binding index within the slot (None = add new).
    pub(crate) recording_index: Option<usize>,
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

        handle_key_recording(world, &mut record_state, &mut bindings);

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

fn handle_key_recording(
    world: &World,
    record_state: &mut KeyRecordState,
    bindings: &mut crate::keybind::KeyBindings,
) {
    let Some(action) = record_state.recording.clone() else {
        return;
    };
    let Some(input) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };

    if input.just_pressed(KeyCode::Escape) {
        record_state.recording = None;
        record_state.recording_index = None;
        return;
    }

    let Some(key) = find_just_pressed_key(input) else {
        return;
    };

    let ctrl = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);
    let shift = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
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

        if !is_recording && ui.small_button("+").clicked() {
            record_state.recording = Some(action_id.to_string());
            record_state.recording_index = None;
        }

        if !is_recording && slot.bindings.len() > 1 && ui.small_button("×").clicked() {
            slot.bindings.pop();
        }
    });
    ui.end_row();
}

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
