//! Bevy-specialized UI components for egui.
//!
//! These provide editing widgets for Bevy types that egui doesn't natively support.
//! Basic egui controls (button, slider, checkbox) should be used directly from egui.

use bevy::math::{Vec2, Vec3};
use bevy::transform::components::Transform;
use egui::Ui;

/// Displays an editable Vec2 with X/Y drag values.
/// Returns `true` if the value was changed.
pub fn vec2(ui: &mut Ui, label: &str, value: &mut Vec2) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.label("X");
        if ui
            .add(egui::DragValue::new(&mut value.x).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label("Y");
        if ui
            .add(egui::DragValue::new(&mut value.y).speed(0.1))
            .changed()
        {
            changed = true;
        }
    });
    changed
}

/// Displays an editable Vec3 with X/Y/Z drag values.
/// Returns `true` if the value was changed.
pub fn vec3(ui: &mut Ui, label: &str, value: &mut Vec3) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.label("X");
        if ui
            .add(egui::DragValue::new(&mut value.x).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label("Y");
        if ui
            .add(egui::DragValue::new(&mut value.y).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label("Z");
        if ui
            .add(egui::DragValue::new(&mut value.z).speed(0.1))
            .changed()
        {
            changed = true;
        }
    });
    changed
}

/// Displays a color picker for a Bevy `Color`.
/// Returns `true` if the value was changed.
pub fn color(ui: &mut Ui, label: &str, value: &mut bevy::color::Color) -> bool {
    let linear = value.to_linear();
    let mut rgba = [linear.red, linear.green, linear.blue, linear.alpha];
    let response = ui.horizontal(|ui| {
        ui.label(label);
        ui.color_edit_button_rgba_unmultiplied(&mut rgba)
    });
    if response.inner.changed() {
        *value = bevy::color::Color::LinearRgba(bevy::color::LinearRgba::new(
            rgba[0], rgba[1], rgba[2], rgba[3],
        ));
        true
    } else {
        false
    }
}

/// Displays a full Transform editor with collapsible Position/Rotation/Scale groups.
/// Returns `true` if any value was changed.
pub fn transform(ui: &mut Ui, label: &str, value: &mut Transform) -> bool {
    let mut changed = false;
    egui::CollapsingHeader::new(label)
        .default_open(true)
        .show(ui, |ui| {
            if vec3(ui, "Position", &mut value.translation) {
                changed = true;
            }
            // Rotation as euler angles (degrees)
            let (mut yaw, mut pitch, mut roll) = value.rotation.to_euler(bevy::math::EulerRot::YXZ);
            yaw = yaw.to_degrees();
            pitch = pitch.to_degrees();
            roll = roll.to_degrees();
            let mut rot = Vec3::new(pitch, yaw, roll);
            if vec3(ui, "Rotation", &mut rot) {
                value.rotation = bevy::math::Quat::from_euler(
                    bevy::math::EulerRot::YXZ,
                    rot.y.to_radians(),
                    rot.x.to_radians(),
                    rot.z.to_radians(),
                );
                changed = true;
            }
            if vec3(ui, "Scale", &mut value.scale) {
                changed = true;
            }
        });
    changed
}

/// Displays an entity picker dropdown from a provided list.
/// Returns `true` if the selection changed.
pub fn entity_picker(
    ui: &mut Ui,
    entities: &[bevy::prelude::Entity],
    selected: &mut Option<bevy::prelude::Entity>,
) -> bool {
    let mut changed = false;
    let label_text = match selected {
        Some(e) => format!("Entity {:?}", e),
        None => "None".to_string(),
    };

    egui::ComboBox::from_label("Entity")
        .selected_text(label_text)
        .show_ui(ui, |ui| {
            if ui.selectable_label(selected.is_none(), "None").clicked() {
                *selected = None;
                changed = true;
            }
            for &entity in entities {
                let is_selected = *selected == Some(entity);
                if ui
                    .selectable_label(is_selected, format!("{:?}", entity))
                    .clicked()
                {
                    *selected = Some(entity);
                    changed = true;
                }
            }
        });
    changed
}

/// Displays a read-only list of component names attached to an entity.
pub fn component_list(ui: &mut Ui, world: &bevy::prelude::World, entity: bevy::prelude::Entity) {
    let Some(entity_ref) = world.get_entity(entity).ok() else {
        ui.label("Entity not found");
        return;
    };

    ui.label(format!("Components on {:?}:", entity));
    let archetype = entity_ref.archetype();
    for component_id in archetype.components() {
        if let Some(info) = world.components().get_info(*component_id) {
            ui.label(format!("  • {}", info.name()));
        }
    }
}
