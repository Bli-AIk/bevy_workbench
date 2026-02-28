//! Inspector panel: bridges bevy-inspector-egui for entity inspection.

use bevy::ecs::observer::Observer;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::window::Monitor;
use bevy_inspector_egui::bevy_inspector::{
    self,
    hierarchy::{Hierarchy, SelectedEntities},
};

use crate::dock::WorkbenchPanel;
use crate::i18n::I18n;

/// Marker component for entities created/managed by the workbench editor.
/// These are hidden in the inspector hierarchy by default.
#[derive(Component)]
pub struct WorkbenchInternal;

/// Resource tracking the currently selected entity for inspection.
#[derive(Resource, Default)]
pub struct InspectorSelection {
    pub selected: SelectedEntities,
    /// When true, show internal (workbench + Bevy) entities in the hierarchy.
    pub show_internal: bool,
}

/// Built-in inspector panel using bevy-inspector-egui.
pub struct InspectorPanel;

impl WorkbenchPanel for InspectorPanel {
    fn id(&self) -> &str {
        "workbench_inspector"
    }

    fn title(&self) -> String {
        "Inspector".to_string()
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.label("Inspector requires World access");
        });
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let mut selected = world
            .remove_resource::<InspectorSelection>()
            .unwrap_or_default();

        // Pre-fetch translated strings before borrowing world mutably
        let (s_hierarchy, s_components, s_select_hint) = {
            let i18n = world.get_resource::<I18n>();
            let t = |id: &str| i18n.map_or_else(|| id.to_string(), |i| i.t(id));
            (
                t("inspector-hierarchy"),
                t("inspector-components"),
                t("inspector-select-hint"),
            )
        };

        // Two-column layout: hierarchy on left, components on right
        egui::SidePanel::left("inspector_hierarchy")
            .resizable(true)
            .default_width(180.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(&s_hierarchy);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut selected.show_internal, "🔧");
                    });
                });
                ui.separator();
                egui::ScrollArea::both().show(ui, |ui| {
                    let show_internal = selected.show_internal;
                    let mut hierarchy = Hierarchy {
                        world,
                        selected: &mut selected.selected,
                        context_menu: None,
                        shortcircuit_entity: None,
                        extra_state: &mut (),
                    };
                    if show_internal {
                        hierarchy.show::<()>(ui);
                    } else {
                        hierarchy.show::<Without<WorkbenchInternal>>(ui);
                    }
                });
            });

        // Right side: selected entity components
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(&s_components);
            ui.separator();
            egui::ScrollArea::both().show(ui, |ui| match selected.selected.as_slice() {
                &[entity] => {
                    bevy_inspector::ui_for_entity(world, entity, ui);
                }
                entities if !entities.is_empty() => {
                    bevy_inspector::ui_for_entities_shared_components(world, entities, ui);
                }
                _ => {
                    ui.weak(&s_select_hint);
                }
            });
        });

        world.insert_resource(selected);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn closable(&self) -> bool {
        true
    }
}

/// Marks Bevy-internal entities (Window, Monitor, Pointer, Observer) with
/// [`WorkbenchInternal`] so the inspector hides them by default.
#[allow(clippy::type_complexity)]
pub fn mark_internal_entities_system(
    mut commands: Commands,
    unmarked: Query<
        Entity,
        (
            Or<(With<Window>, With<Monitor>, With<PointerId>, With<Observer>)>,
            Without<WorkbenchInternal>,
        ),
    >,
) {
    for entity in &unmarked {
        commands.entity(entity).insert(WorkbenchInternal);
    }
}
