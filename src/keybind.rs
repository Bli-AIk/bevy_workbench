//! Configurable keybindings for editor actions.

use bevy::prelude::*;

/// A single key binding: a primary key plus optional modifiers.
#[derive(Debug, Clone)]
pub struct KeyBind {
    pub key: KeyCode,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBind {
    /// Simple key without modifiers.
    pub const fn key(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Ctrl + key.
    pub const fn ctrl(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    /// Ctrl + Shift + key.
    pub const fn ctrl_shift(key: KeyCode) -> Self {
        Self {
            key,
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    /// Check if this binding was just pressed.
    pub fn just_pressed(&self, input: &ButtonInput<KeyCode>) -> bool {
        if !input.just_pressed(self.key) {
            return false;
        }
        let ctrl_ok = if self.ctrl {
            input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight)
        } else {
            !input.pressed(KeyCode::ControlLeft) && !input.pressed(KeyCode::ControlRight)
        };
        let shift_ok = if self.shift {
            input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)
        } else {
            !input.pressed(KeyCode::ShiftLeft) && !input.pressed(KeyCode::ShiftRight)
        };
        let alt_ok = if self.alt {
            input.pressed(KeyCode::AltLeft) || input.pressed(KeyCode::AltRight)
        } else {
            !input.pressed(KeyCode::AltLeft) && !input.pressed(KeyCode::AltRight)
        };
        ctrl_ok && shift_ok && alt_ok
    }

    /// Human-readable label for UI display.
    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        parts.push(key_label(self.key));
        parts.join("+")
    }
}

/// A keybinding slot that supports multiple alternative bindings.
#[derive(Debug, Clone)]
pub struct KeyBindSlot {
    pub bindings: Vec<KeyBind>,
}

impl KeyBindSlot {
    pub fn single(bind: KeyBind) -> Self {
        Self {
            bindings: vec![bind],
        }
    }

    pub fn from(binds: Vec<KeyBind>) -> Self {
        Self { bindings: binds }
    }

    /// Check if any binding in this slot was just pressed.
    pub fn just_pressed(&self, input: &ButtonInput<KeyCode>) -> bool {
        self.bindings.iter().any(|b| b.just_pressed(input))
    }

    /// Human-readable label showing all alternatives.
    pub fn label(&self) -> String {
        self.bindings
            .iter()
            .map(|b| b.label())
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

/// All configurable keybindings for the editor.
#[derive(Resource, Debug, Clone)]
pub struct KeyBindings {
    /// Undo (default: Ctrl+Z)
    pub undo: KeyBindSlot,
    /// Redo (default: Ctrl+Shift+Z)
    pub redo: KeyBindSlot,
    /// Play/Stop toggle (default: F5, Ctrl+P)
    pub play_stop: KeyBindSlot,
    /// Pause/Resume (default: F6, Ctrl+Shift+P)
    pub pause_resume: KeyBindSlot,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            undo: KeyBindSlot::from(vec![KeyBind::ctrl(KeyCode::KeyZ)]),
            redo: KeyBindSlot::from(vec![KeyBind::ctrl_shift(KeyCode::KeyZ)]),
            play_stop: KeyBindSlot::from(vec![
                KeyBind::key(KeyCode::F5),
                KeyBind::ctrl(KeyCode::KeyP),
            ]),
            pause_resume: KeyBindSlot::from(vec![
                KeyBind::key(KeyCode::F6),
                KeyBind::ctrl_shift(KeyCode::KeyP),
            ]),
        }
    }
}

fn key_label(key: KeyCode) -> &'static str {
    match key {
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0",
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Esc",
        KeyCode::Backspace => "Backspace",
        KeyCode::Tab => "Tab",
        KeyCode::Delete => "Del",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::ArrowUp => "↑",
        KeyCode::ArrowDown => "↓",
        KeyCode::ArrowLeft => "←",
        KeyCode::ArrowRight => "→",
        _ => "?",
    }
}
