//! Responsive layout: desktop (landscape) and portrait (mobile) modes.

use bevy::prelude::*;

/// Layout mode configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Auto-detect based on window aspect ratio.
    Auto,
    /// Force desktop landscape layout.
    Desktop,
    /// Force portrait layout (Android / mobile).
    Portrait,
}

/// The currently active layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveLayout {
    Desktop,
    Portrait,
}

/// Resource holding the current layout state.
#[derive(Resource)]
pub struct LayoutState {
    pub mode: LayoutMode,
    pub active: ActiveLayout,
}

impl LayoutState {
    pub fn new(mode: LayoutMode) -> Self {
        Self {
            mode,
            active: match mode {
                LayoutMode::Desktop => ActiveLayout::Desktop,
                LayoutMode::Portrait => ActiveLayout::Portrait,
                LayoutMode::Auto => ActiveLayout::Desktop,
            },
        }
    }
}

/// System that detects the current layout based on window dimensions.
pub fn detect_layout_system(windows: Query<&Window>, mut layout: ResMut<LayoutState>) {
    if layout.mode != LayoutMode::Auto {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let ratio = window.width() / window.height();
    let new_layout = if ratio < 1.0 {
        ActiveLayout::Portrait
    } else {
        ActiveLayout::Desktop
    };

    if layout.active != new_layout {
        layout.active = new_layout;
    }
}
