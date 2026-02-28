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

/// Tracks elapsed time within the current game session.
/// Reset on each Play; paused during Pause; stopped during Edit.
#[derive(Resource)]
pub struct GameClock {
    /// Seconds elapsed since the current Play session started.
    pub elapsed: f32,
    /// The previous editor mode (for distinguishing fresh Play vs Resume).
    pub(crate) previous_mode: EditorMode,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            previous_mode: EditorMode::Edit,
        }
    }
}

/// Runs the [`GameSchedule`] when in [`EditorMode::Play`],
/// advancing [`GameClock`] each frame.
pub fn run_game_schedule_system(world: &mut World) {
    let mode = world.resource::<State<EditorMode>>().get().to_owned();
    if mode == EditorMode::Play {
        let dt = world.resource::<Time>().delta_secs();
        world.resource_mut::<GameClock>().elapsed += dt;
        world.run_schedule(GameSchedule);
    }
}

/// Resets the [`GameClock`] when entering Play from Edit (not Resume from Pause).
pub fn on_enter_play(mut clock: ResMut<GameClock>) {
    if clock.previous_mode == EditorMode::Edit {
        clock.elapsed = 0.0;
    }
    clock.previous_mode = EditorMode::Play;
}

/// Tracks that we entered Pause.
pub fn on_enter_pause(mut clock: ResMut<GameClock>) {
    clock.previous_mode = EditorMode::Pause;
}

/// Tracks that we returned to Edit.
pub fn on_enter_edit(mut clock: ResMut<GameClock>) {
    clock.previous_mode = EditorMode::Edit;
}

/// Run condition: true only for fresh Play (from Edit), not Resume (from Pause).
/// Use with `OnEnter(EditorMode::Play)` to gate game setup systems.
pub fn on_fresh_play(clock: Res<GameClock>) -> bool {
    clock.previous_mode == EditorMode::Edit
}

/// System that handles keyboard shortcuts for mode transitions.
pub fn mode_input_system(
    input: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    bindings: Option<Res<super::keybind::KeyBindings>>,
) {
    let default_bindings = super::keybind::KeyBindings::default();
    let bindings = bindings.as_deref().unwrap_or(&default_bindings);

    // Play/Stop toggle
    if bindings.play_stop.just_pressed(&input) {
        match current_mode.get() {
            EditorMode::Edit => next_mode.set(EditorMode::Play),
            EditorMode::Play | EditorMode::Pause => next_mode.set(EditorMode::Edit),
        }
    }

    // Pause/Resume
    if bindings.pause_resume.just_pressed(&input) {
        match current_mode.get() {
            EditorMode::Play => next_mode.set(EditorMode::Pause),
            EditorMode::Pause => next_mode.set(EditorMode::Play),
            _ => {}
        }
    }
}
