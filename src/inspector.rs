//! Inspector panel: bridges bevy-inspector-egui for entity inspection.

use bevy::prelude::*;

use crate::dock::WorkbenchPanel;

/// Resource tracking the currently selected entity for inspection.
#[derive(Resource, Default)]
pub struct InspectorSelection {
    pub selected: Option<Entity>,
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
            ui.label("No entity selected\n(Inspector requires ECS access — coming soon)");
        });
    }

    fn closable(&self) -> bool {
        true
    }
}
