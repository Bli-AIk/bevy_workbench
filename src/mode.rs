//! Editor mode state machine: Edit / Play / Pause.

use bevy::prelude::*;

/// The current editor mode.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorMode {
    /// Editing mode — panels visible, game paused.
    #[default]
    Edit,
    /// Playing mode — game running, panels optionally hidden.
    Play,
    /// Paused during play.
    Pause,
}

/// Resource controlling mode behavior.
#[derive(Resource, Default)]
pub struct ModeController {
    /// Whether to hide editor panels when entering Play mode.
    pub hide_panels_on_play: bool,
}

/// Syncs Bevy's virtual time with the editor mode.
/// Pauses in Edit/Pause, unpauses in Play.
pub fn mode_time_sync_system(
    mode: Res<State<EditorMode>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    match mode.get() {
        EditorMode::Play => {
            if virtual_time.is_paused() {
                virtual_time.unpause();
            }
        }
        EditorMode::Edit | EditorMode::Pause => {
            if !virtual_time.is_paused() {
                virtual_time.pause();
            }
        }
    }
}

/// System that handles keyboard shortcuts for mode transitions.
pub fn mode_input_system(
    input: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
) {
    let ctrl = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);
    let shift = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);

    // F5 or Ctrl+P: Play/Stop toggle
    if input.just_pressed(KeyCode::F5) || (ctrl && input.just_pressed(KeyCode::KeyP) && !shift) {
        match current_mode.get() {
            EditorMode::Edit => next_mode.set(EditorMode::Play),
            EditorMode::Play | EditorMode::Pause => next_mode.set(EditorMode::Edit),
        }
    }

    // Ctrl+Shift+P: Pause/Resume
    if ctrl && shift && input.just_pressed(KeyCode::KeyP) {
        match current_mode.get() {
            EditorMode::Play => next_mode.set(EditorMode::Pause),
            EditorMode::Pause => next_mode.set(EditorMode::Play),
            _ => {}
        }
    }
}
