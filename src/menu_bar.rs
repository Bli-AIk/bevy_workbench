//! Top menu bar with mode controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::mode::EditorMode;

/// System that renders the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("workbench_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    ui.close();
                }
                if ui.button("Open...").clicked() {
                    ui.close();
                }
                ui.separator();
                if ui.button("Save").clicked() {
                    ui.close();
                }
                if ui.button("Save As...").clicked() {
                    ui.close();
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo (Ctrl+Z)").clicked() {
                    ui.close();
                }
                if ui.button("Redo (Ctrl+Shift+Z)").clicked() {
                    ui.close();
                }
            });

            ui.menu_button("View", |ui| {
                if ui.button("Reset Layout").clicked() {
                    ui.close();
                }
            });

            // Spacer
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| match current_mode.get() {
                    EditorMode::Edit => {
                        if ui.button("▶ Play").clicked() {
                            next_mode.set(EditorMode::Play);
                        }
                    }
                    EditorMode::Play => {
                        if ui.button("⏸ Pause").clicked() {
                            next_mode.set(EditorMode::Pause);
                        }
                        if ui.button("⏹ Stop").clicked() {
                            next_mode.set(EditorMode::Edit);
                        }
                    }
                    EditorMode::Pause => {
                        if ui.button("▶ Resume").clicked() {
                            next_mode.set(EditorMode::Play);
                        }
                        if ui.button("⏹ Stop").clicked() {
                            next_mode.set(EditorMode::Edit);
                        }
                    }
                },
            );
        });
    });
}
