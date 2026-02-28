//! Inspector panel: bridges bevy-inspector-egui for entity inspection.

use bevy::ecs::component::ComponentId;
use bevy::ecs::observer::Observer;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::reflect::PartialReflect;
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

/// Snapshot of an entity's reflected components (for undo).
type ComponentSnapshot = Vec<(ComponentId, Box<dyn PartialReflect>)>;

/// Clone a component snapshot (Box<dyn PartialReflect> uses clone_value()).
fn clone_snapshot(snapshot: &ComponentSnapshot) -> ComponentSnapshot {
    snapshot
        .iter()
        .map(|(id, val)| (*id, val.to_dynamic()))
        .collect()
}

/// Tracks inspector editing for undo (baseline + debounce).
#[derive(Resource, Default)]
pub(crate) struct InspectorUndoState {
    /// Entity being tracked.
    tracked_entity: Option<Entity>,
    /// Baseline snapshot taken when editing starts.
    baseline: Option<ComponentSnapshot>,
    /// Whether the mouse was pressed last frame (for drag detection).
    was_pressing: bool,
}

/// Take a reflected snapshot of an entity's components.
fn snapshot_entity(world: &World, entity: Entity) -> Option<ComponentSnapshot> {
    let entity_ref = world.get_entity(entity).ok()?;
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let mut snapshot = Vec::new();
    for &component_id in entity_ref.archetype().components() {
        let Some(type_id) = world
            .components()
            .get_info(component_id)
            .and_then(|info| info.type_id())
        else {
            continue;
        };
        let Some(registration) = type_registry.get(type_id) else {
            continue;
        };
        let Some(reflect_component) = registration.data::<ReflectComponent>() else {
            continue;
        };
        if let Some(reflected) = reflect_component.reflect(entity_ref) {
            snapshot.push((component_id, reflected.as_partial_reflect().to_dynamic()));
        }
    }
    Some(snapshot)
}

/// Check if two snapshots differ.
fn snapshots_differ(a: &ComponentSnapshot, b: &ComponentSnapshot) -> bool {
    if a.len() != b.len() {
        return true;
    }
    for ((id_a, val_a), (id_b, val_b)) in a.iter().zip(b.iter()) {
        if id_a != id_b {
            return true;
        }
        match val_a.reflect_partial_eq(val_b.as_ref()) {
            Some(true) => {}
            _ => return true,
        }
    }
    false
}

/// Restore an entity's components from a snapshot.
fn restore_snapshot(world: &mut World, entity: Entity, snapshot: &ComponentSnapshot) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    for (component_id, value) in snapshot {
        let Some(type_id) = world
            .components()
            .get_info(*component_id)
            .and_then(|info| info.type_id())
        else {
            continue;
        };
        let Some(registration) = type_registry.get(type_id) else {
            continue;
        };
        let Some(reflect_component) = registration.data::<ReflectComponent>() else {
            continue;
        };
        if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
            reflect_component.apply(&mut entity_mut, value.as_ref());
        }
    }
}

/// Undo action for inspector component changes (uses reflected snapshots).
struct InspectorUndoAction {
    entity: Entity,
    before: ComponentSnapshot,
    after: ComponentSnapshot,
    desc: String,
}

impl crate::undo::UndoAction for InspectorUndoAction {
    fn undo(&self, world: &mut World) {
        restore_snapshot(world, self.entity, &self.before);
    }

    fn redo(&self, world: &mut World) {
        restore_snapshot(world, self.entity, &self.after);
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

// InspectorUndoAction needs Send+Sync but Box<dyn PartialReflect> is Send+Sync already
unsafe impl Send for InspectorUndoAction {}
unsafe impl Sync for InspectorUndoAction {}

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
                    // Inspector undo: track changes
                    let mut undo_state = world
                        .remove_resource::<InspectorUndoState>()
                        .unwrap_or_default();

                    // Take baseline on selection change
                    if undo_state.tracked_entity != Some(entity) {
                        undo_state.tracked_entity = Some(entity);
                        undo_state.baseline = snapshot_entity(world, entity);
                        undo_state.was_pressing = false;
                    }

                    let pressing = ui.input(|i| i.pointer.any_pressed());

                    // Render inspector (may modify components)
                    bevy_inspector::ui_for_entity(world, entity, ui);

                    // On mouse release after pressing, check for changes
                    if undo_state.was_pressing && !pressing {
                        if let Some(baseline) = &undo_state.baseline {
                            if let Some(current) = snapshot_entity(world, entity) {
                                if snapshots_differ(baseline, &current) {
                                    let before = clone_snapshot(baseline);
                                    let desc = format!("Modify entity {entity:?}");
                                    if let Some(mut undo_stack) =
                                        world.get_resource_mut::<crate::undo::UndoStack>()
                                    {
                                        undo_stack.push(InspectorUndoAction {
                                            entity,
                                            before,
                                            after: current,
                                            desc,
                                        });
                                    }
                                    // Update baseline to current state
                                    undo_state.baseline = snapshot_entity(world, entity);
                                }
                            }
                        }
                    }
                    undo_state.was_pressing = pressing;

                    world.insert_resource(undo_state);
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
