//! Minimal bevy_workbench usage example.
//!
//! Shows a workbench editor with menu bar, inspector, console, and game view panels.
//! Uses simple egui panels (like flambe) for reliable rendering.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Workbench — Minimal".into(),
                resolution: (1280u32, 720u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            EguiPrimaryContextPass,
            (apply_theme, menu_bar, main_panel).chain(),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn apply_theme(mut contexts: EguiContexts, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    bevy_workbench::theme::apply_theme_to_ctx(ctx, None);
    *done = true;
}

fn menu_bar(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New").clicked() {
                    ui.close();
                }
                if ui.button("Open...").clicked() {
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                if ui.button("Undo").clicked() {
                    ui.close();
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("▶ Play");
            });
        });
    });
}

fn main_panel(mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::SidePanel::left("inspector")
        .default_width(260.0)
        .show(ctx, |ui| {
            ui.heading("Inspector");
            ui.separator();
            ui.label("No entity selected");
        });
    egui::TopBottomPanel::bottom("console")
        .default_height(150.0)
        .show(ctx, |ui| {
            ui.heading("Console");
            ui.separator();
            ui.label("Ready.");
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Game View");
        ui.separator();
        ui.centered_and_justified(|ui| {
            ui.label("Game View — no scene loaded");
        });
    });
}
