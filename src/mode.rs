//! Editor mode state machine: Edit / Play / Pause.

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

/// The current editor mode.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditorMode {
    /// Editing mode — panels visible, game stopped.
    #[default]
    Edit,
    /// Playing mode — game running.
    Play,
    /// Paused during play.
    Pause,
}

/// Schedule for game logic. Only runs during [`EditorMode::Play`].
///
/// Users should add their game systems here instead of `Update`:
/// ```rust,ignore
/// app.add_systems(GameSchedule, (move_player, spawn_enemies));
/// ```
///
/// Entities spawned from `OnEnter(EditorMode::Play)` should include
/// `DespawnOnExit(EditorMode::Play)` for automatic cleanup on Stop.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GameSchedule;

/// Resource controlling mode behavior.
#[derive(Resource, Default)]
pub struct ModeController {
    /// Whether to hide editor panels when entering Play mode.
    pub hide_panels_on_play: bool,
}

/// Runs the [`GameSchedule`] when in [`EditorMode::Play`].
pub fn run_game_schedule_system(world: &mut World) {
    let mode = world.resource::<State<EditorMode>>().get().to_owned();
    if mode == EditorMode::Play {
        world.run_schedule(GameSchedule);
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
