//! Rerun-inspired dark theme for the workbench editor.
//!
//! Colors extracted from rerun's `re_ui` design tokens (dark_theme.ron + color_table.ron).

use bevy::prelude::*;
use egui::{Color32, Stroke, Vec2, epaint::Shadow};

/// Rerun gray scale.
pub mod gray {
    use egui::Color32;
    pub const S0: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
    pub const S100: Color32 = Color32::from_rgb(0x0d, 0x10, 0x11);
    pub const S125: Color32 = Color32::from_rgb(0x11, 0x14, 0x15);
    pub const S150: Color32 = Color32::from_rgb(0x14, 0x18, 0x19);
    pub const S200: Color32 = Color32::from_rgb(0x1c, 0x21, 0x23);
    pub const S250: Color32 = Color32::from_rgb(0x26, 0x2b, 0x2e);
    pub const S300: Color32 = Color32::from_rgb(0x31, 0x38, 0x3b);
    pub const S325: Color32 = Color32::from_rgb(0x37, 0x3f, 0x42);
    pub const S350: Color32 = Color32::from_rgb(0x3e, 0x46, 0x4a);
    pub const S500: Color32 = Color32::from_rgb(0x6c, 0x79, 0x7f);
    pub const S550: Color32 = Color32::from_rgb(0x7d, 0x8c, 0x92);
    pub const S700: Color32 = Color32::from_rgb(0xae, 0xc2, 0xca);
    pub const S775: Color32 = Color32::from_rgb(0xca, 0xd8, 0xde);
    pub const S800: Color32 = Color32::from_rgb(0xd3, 0xde, 0xe3);
    pub const S1000: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
}

pub mod blue {
    use egui::Color32;
    pub const S350: Color32 = Color32::from_rgb(0x00, 0x3d, 0xa1);
    pub const S400: Color32 = Color32::from_rgb(0x00, 0x4b, 0xc2);
    pub const S450: Color32 = Color32::from_rgb(0x00, 0x5a, 0xe6);
    pub const S500: Color32 = Color32::from_rgb(0x2a, 0x6c, 0xff);
    pub const S750: Color32 = Color32::from_rgb(0xc2, 0xcc, 0xff);
    pub const S900: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xff);
}

// Color constants for UI panels.
pub const PANEL_BG: Color32 = gray::S100;
pub const HEADER_BG: Color32 = gray::S150;
pub const ROW_EVEN_BG: Color32 = gray::S100;
pub const ROW_ODD_BG: Color32 = gray::S125;
pub const ROW_SELECTED_BG: Color32 = Color32::from_rgb(0x00, 0x25, 0x69);
pub const BAR_COLOR: Color32 = blue::S400;
pub const SEPARATOR_COLOR: Color32 = gray::S250;
pub const TEXT_SUBDUED: Color32 = gray::S550;
pub const TEXT_DEFAULT: Color32 = gray::S775;
pub const TEXT_STRONG: Color32 = gray::S1000;

/// Resource holding the current theme configuration.
#[derive(Resource, Default)]
pub struct ThemeState {
    /// Minimum interactive element size in points (larger on touch devices).
    pub interact_size: Option<Vec2>,
}

impl ThemeState {
    /// Create a theme state optimized for touch devices (Android).
    pub fn touch() -> Self {
        Self {
            interact_size: Some(Vec2::new(44.0, 44.0)),
        }
    }
}

/// Darken a Color32 by a factor (0.0 = black, 1.0 = unchanged).
fn dim_color(c: Color32, factor: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r() as f32 * factor) as u8,
        (c.g() as f32 * factor) as u8,
        (c.b() as f32 * factor) as u8,
        c.a(),
    )
}

fn dim_stroke(s: Stroke, factor: f32) -> Stroke {
    Stroke::new(s.width, dim_color(s.color, factor))
}

/// Apply the Rerun-inspired dark theme to an egui context.
/// `brightness` = 1.0 for normal, < 1.0 to dim (e.g. 0.6 in Play mode).
pub fn apply_theme_to_ctx(
    ctx: &egui::Context,
    interact_size_override: Option<Vec2>,
    brightness: f32,
) {
    let mut style = (*ctx.style()).clone();

    // Typography
    let font_size = 12.0;
    for text_style in [
        egui::TextStyle::Body,
        egui::TextStyle::Monospace,
        egui::TextStyle::Button,
    ] {
        if let Some(font_id) = style.text_styles.get_mut(&text_style) {
            font_id.size = font_size;
        }
    }
    if let Some(font_id) = style.text_styles.get_mut(&egui::TextStyle::Heading) {
        font_id.size = 16.0;
    }
    if let Some(font_id) = style.text_styles.get_mut(&egui::TextStyle::Small) {
        font_id.size = 10.0;
    }
    style.spacing.interact_size.y = 15.0;

    if let Some(size) = interact_size_override {
        style.spacing.interact_size = size;
    }

    // Spacing
    style.visuals.button_frame = true;

    style.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    style.visuals.widgets.open.bg_stroke = Stroke::NONE;

    style.visuals.widgets.hovered.expansion = 2.0;
    style.visuals.widgets.active.expansion = 2.0;
    style.visuals.widgets.open.expansion = 2.0;

    let window_radius = egui::CornerRadius::same(6);
    let small_radius = egui::CornerRadius::same(4);
    style.visuals.window_corner_radius = window_radius;
    style.visuals.menu_corner_radius = window_radius;
    style.visuals.widgets.noninteractive.corner_radius = small_radius;
    style.visuals.widgets.inactive.corner_radius = small_radius;
    style.visuals.widgets.hovered.corner_radius = small_radius;
    style.visuals.widgets.active.corner_radius = small_radius;
    style.visuals.widgets.open.corner_radius = small_radius;

    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.menu_margin = egui::Margin::same(12);
    style.spacing.menu_spacing = 1.0;
    style.visuals.clip_rect_margin = 0.0;
    style.visuals.striped = false;
    style.visuals.indent_has_left_vline = false;
    style.spacing.button_padding = Vec2::new(1.0, 0.0);
    style.spacing.indent = 14.0;
    style.spacing.combo_width = 8.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.tooltip_width = 600.0;
    style.visuals.image_loading_spinners = false;

    // Colors
    let b = brightness;
    style.visuals.dark_mode = true;
    style.visuals.faint_bg_color = dim_color(gray::S150, b);
    style.visuals.extreme_bg_color = dim_color(gray::S0, b);

    style.visuals.widgets.noninteractive.weak_bg_fill = dim_color(gray::S100, b);
    style.visuals.widgets.noninteractive.bg_fill = dim_color(gray::S100, b);
    style.visuals.text_edit_bg_color = Some(dim_color(gray::S200, b));

    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_fill = dim_color(gray::S300, b);

    let hovered = dim_color(gray::S325, b);
    style.visuals.widgets.hovered.weak_bg_fill = hovered;
    style.visuals.widgets.hovered.bg_fill = hovered;
    style.visuals.widgets.active.weak_bg_fill = hovered;
    style.visuals.widgets.active.bg_fill = hovered;
    style.visuals.widgets.open.weak_bg_fill = hovered;
    style.visuals.widgets.open.bg_fill = hovered;

    style.visuals.selection.bg_fill = dim_color(blue::S350, b);
    style.visuals.selection.stroke.color = dim_color(blue::S900, b);

    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, dim_color(gray::S250, b));

    let subdued = dim_color(gray::S550, b);
    let default_text = dim_color(gray::S775, b);
    let strong = dim_color(gray::S1000, b);

    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, subdued);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, default_text);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, strong);
    style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, strong);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, default_text);

    style.visuals.selection.stroke = dim_stroke(Stroke::new(2.0, blue::S900), b);

    let shadow = Shadow {
        offset: [0, 15],
        blur: 50,
        spread: 0,
        color: Color32::from_black_alpha(128),
    };
    style.visuals.popup_shadow = shadow;
    style.visuals.window_shadow = shadow;

    style.visuals.window_fill = dim_color(gray::S200, b);
    style.visuals.window_stroke = Stroke::NONE;
    style.visuals.panel_fill = dim_color(gray::S100, b);

    style.visuals.hyperlink_color = default_text;
    style.visuals.error_fg_color = dim_color(Color32::from_rgb(0xAB, 0x01, 0x16), b);
    style.visuals.warn_fg_color = dim_color(Color32::from_rgb(0xFF, 0x7A, 0x0C), b);

    ctx.set_style(style);
}

/// System that applies the theme to the egui context (once on startup, then on changes).
pub fn apply_theme_system(
    mut contexts: bevy_egui::EguiContexts,
    theme: Res<ThemeState>,
    mode: Res<State<crate::mode::EditorMode>>,
    mut applied: Local<bool>,
    mut prev_mode: Local<Option<crate::mode::EditorMode>>,
) {
    let mode_changed = *prev_mode != Some(*mode.get());
    if *applied && !theme.is_changed() && !mode_changed {
        return;
    }
    *prev_mode = Some(*mode.get());
    let Ok(ctx) = contexts.ctx_mut() else { return };
    let brightness = match mode.get() {
        crate::mode::EditorMode::Edit => 1.0,
        crate::mode::EditorMode::Play | crate::mode::EditorMode::Pause => 0.6,
    };
    apply_theme_to_ctx(ctx, theme.interact_size, brightness);
    *applied = true;
}
