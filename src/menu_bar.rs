//! Top menu bar with mode controls.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::dock::TileLayoutState;
use crate::mode::EditorMode;
use crate::theme::gray;

mod keybindings_panel;
mod settings_panel;

pub(crate) use keybindings_panel::KeyRecordState;
pub use keybindings_panel::KeybindingsPanel;
pub use settings_panel::{SettingsPanel, SettingsSection};

/// A custom item to inject into a menu.
pub struct MenuExtItem {
    /// Unique identifier for this action (e.g., "open", "save").
    pub id: &'static str,
    /// Display label (e.g., "Open...", "Save").
    pub label: String,
    /// Whether the item is clickable.
    pub enabled: bool,
}

/// A custom top-level menu to add to the menu bar.
pub struct CustomMenu {
    /// Unique identifier for this menu.
    pub id: &'static str,
    /// Display label for the menu button.
    pub label: String,
    /// Whether the menu is interactable. When `false`, the menu button is grayed out.
    pub enabled: bool,
    /// Items inside this menu.
    pub items: Vec<MenuExtItem>,
}

/// Resource for extending the workbench menu bar with custom items.
#[derive(Resource, Default)]
pub struct MenuBarExtensions {
    /// Custom items prepended to the File menu (before built-in "Settings").
    pub file_items: Vec<MenuExtItem>,
    /// Custom top-level menus rendered after built-in menus (File/Edit/View/Window).
    pub custom_menus: Vec<CustomMenu>,
    /// Info text displayed after all menus (e.g., project title, resolution).
    pub info_text: Option<String>,
}

/// Message sent when a custom menu item is clicked.
#[derive(Message)]
pub struct MenuAction {
    pub id: &'static str,
}

/// System that renders the top menu bar.
pub fn menu_bar_system(
    mut contexts: EguiContexts,
    mut tile_state: ResMut<TileLayoutState>,
    i18n: Res<crate::i18n::I18n>,
    mut undo_stack: ResMut<crate::undo::UndoStack>,
    extensions: Option<Res<MenuBarExtensions>>,
    mut menu_actions: MessageWriter<MenuAction>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::TopBottomPanel::top("workbench_menu_bar").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            // Left side: menus
            ui.menu_button(i18n.t("menu-file"), |ui| {
                file_menu_ui(
                    ui,
                    &mut tile_state,
                    &i18n,
                    extensions.as_deref(),
                    &mut menu_actions,
                );
            });

            ui.menu_button(i18n.t("menu-edit"), |ui| {
                edit_menu_ui(ui, &i18n, &mut undo_stack, &mut tile_state);
            });

            ui.menu_button(i18n.t("menu-view"), |ui| {
                view_menu_ui(ui, &i18n, &mut tile_state);
            });

            // Window menu — toggle panel visibility
            let panel_list = tile_state.panel_list();
            ui.menu_button(i18n.t("menu-window"), |ui| {
                window_menu_ui(ui, &mut tile_state, &panel_list);
            });

            // Custom top-level menus
            if let Some(ref ext) = extensions {
                custom_menus_ui(ui, &ext.custom_menus, &mut menu_actions);
            }

            // Extension info text (e.g., project title)
            if let Some(ref ext) = extensions
                && let Some(ref text) = ext.info_text
            {
                ui.separator();
                ui.label(text);
            }
        });
    });

    // Secondary toolbar — centered Play/Pause/Stop
}

/// System that renders the Play/Pause/Stop toolbar.
/// Only added when `WorkbenchConfig::show_toolbar` is `true`.
pub fn toolbar_system(
    mut contexts: EguiContexts,
    current_mode: Res<State<EditorMode>>,
    mut next_mode: ResMut<NextState<EditorMode>>,
    i18n: Res<crate::i18n::I18n>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    let btn_fill = gray::S250;
    egui::TopBottomPanel::top("workbench_toolbar").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            toolbar_buttons_ui(ui, current_mode.get(), &mut next_mode, &i18n, btn_fill);
        });
    });
}

/// File menu content, extracted to reduce nesting.
fn file_menu_ui(
    ui: &mut egui::Ui,
    tile_state: &mut TileLayoutState,
    i18n: &crate::i18n::I18n,
    extensions: Option<&MenuBarExtensions>,
    menu_actions: &mut MessageWriter<MenuAction>,
) {
    if let Some(ext) = extensions {
        for item in &ext.file_items {
            if ui
                .add_enabled(item.enabled, egui::Button::new(&item.label))
                .clicked()
            {
                menu_actions.write(MenuAction { id: item.id });
                ui.close();
            }
        }
        if !ext.file_items.is_empty() {
            ui.separator();
        }
    }
    if ui.button(i18n.t("menu-file-settings")).clicked() {
        tile_state.request_open_panel("settings");
        ui.close();
    }
}

/// Edit menu content, extracted to reduce nesting.
fn edit_menu_ui(
    ui: &mut egui::Ui,
    i18n: &crate::i18n::I18n,
    undo_stack: &mut crate::undo::UndoStack,
    tile_state: &mut TileLayoutState,
) {
    let undo_label = if let Some(desc) = undo_stack.undo_description() {
        format!("{} ({})", i18n.t("menu-edit-undo"), desc)
    } else {
        i18n.t("menu-edit-undo")
    };
    if ui
        .add_enabled(undo_stack.can_undo(), egui::Button::new(undo_label))
        .clicked()
    {
        undo_stack.undo_requested = true;
        ui.close();
    }
    let redo_label = if let Some(desc) = undo_stack.redo_description() {
        format!("{} ({})", i18n.t("menu-edit-redo"), desc)
    } else {
        i18n.t("menu-edit-redo")
    };
    if ui
        .add_enabled(undo_stack.can_redo(), egui::Button::new(redo_label))
        .clicked()
    {
        undo_stack.redo_requested = true;
        ui.close();
    }
    ui.separator();
    if ui.button("Keybindings...").clicked() {
        tile_state.request_open_panel("keybindings");
        ui.close();
    }
    if ui.button("Undo History").clicked() {
        tile_state.request_open_panel("undo_history");
        ui.close();
    }
}

/// View menu content, extracted to reduce nesting.
fn view_menu_ui(ui: &mut egui::Ui, i18n: &crate::i18n::I18n, tile_state: &mut TileLayoutState) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        if ui.button(i18n.t("menu-view-save-layout")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(i18n.t("dialog-save-layout"))
                .add_filter("JSON", &["json"])
                .set_file_name("layout.json")
                .save_file()
            {
                tile_state.layout_save_path = Some(path);
            }
            ui.close();
        }
        if ui.button(i18n.t("menu-view-load-layout")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(i18n.t("dialog-load-layout"))
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                tile_state.layout_load_path = Some(path);
            }
            ui.close();
        }
        ui.separator();
    }
    if ui.button(i18n.t("menu-view-reset-layout")).clicked() {
        tile_state.layout_reset_requested = true;
        ui.close();
    }
}

/// Window menu content, extracted to reduce nesting.
fn window_menu_ui(
    ui: &mut egui::Ui,
    tile_state: &mut TileLayoutState,
    panel_list: &[(String, String, bool)],
) {
    for (str_id, title, visible) in panel_list {
        let text = if *visible {
            egui::RichText::new(title)
        } else {
            egui::RichText::new(title).weak()
        };
        if ui.button(text).clicked() {
            if *visible
                && let Some(&panel_id) = tile_state.panel_id_map.get(str_id.as_str())
                && let Some(&tile_id) = tile_state.panel_tile_map.get(&panel_id)
            {
                tile_state.hide_tile(tile_id);
            } else if !*visible {
                tile_state.request_open_panel(str_id);
            }
            ui.close();
        }
    }
}

/// Custom top-level menus, extracted to reduce nesting.
fn custom_menus_ui(
    ui: &mut egui::Ui,
    menus: &[CustomMenu],
    menu_actions: &mut MessageWriter<MenuAction>,
) {
    for menu in menus {
        ui.add_enabled_ui(menu.enabled, |ui| {
            ui.menu_button(&menu.label, |ui| {
                custom_menu_items_ui(ui, &menu.items, menu_actions);
            });
        });
    }
}

/// Items inside a single custom menu.
fn custom_menu_items_ui(
    ui: &mut egui::Ui,
    items: &[MenuExtItem],
    menu_actions: &mut MessageWriter<MenuAction>,
) {
    for item in items {
        if ui
            .add_enabled(item.enabled, egui::Button::new(&item.label))
            .clicked()
        {
            menu_actions.write(MenuAction { id: item.id });
            ui.close();
        }
    }
}

/// Toolbar buttons for Play/Pause/Stop, extracted to reduce nesting.
fn toolbar_buttons_ui(
    ui: &mut egui::Ui,
    current_mode: &EditorMode,
    next_mode: &mut NextState<EditorMode>,
    i18n: &crate::i18n::I18n,
    btn_fill: egui::Color32,
) {
    let button_w = 80.0;
    let n_buttons: f32 = match current_mode {
        EditorMode::Edit => 1.0,
        _ => 2.0,
    };
    let total = button_w * n_buttons + 4.0 * (n_buttons - 1.0_f32).max(0.0);
    let pad = ((ui.available_width() - total) / 2.0).max(0.0);
    ui.add_space(pad);

    match current_mode {
        EditorMode::Edit => {
            if ui
                .add_sized(
                    [button_w, 18.0],
                    egui::Button::new(i18n.t("toolbar-play")).fill(btn_fill),
                )
                .clicked()
            {
                next_mode.set(EditorMode::Play);
            }
        }
        EditorMode::Play => {
            if ui
                .add_sized(
                    [button_w, 18.0],
                    egui::Button::new(i18n.t("toolbar-pause")).fill(btn_fill),
                )
                .clicked()
            {
                next_mode.set(EditorMode::Pause);
            }
            if ui
                .add_sized(
                    [button_w, 18.0],
                    egui::Button::new(i18n.t("toolbar-stop")).fill(btn_fill),
                )
                .clicked()
            {
                next_mode.set(EditorMode::Edit);
            }
        }
        EditorMode::Pause => {
            if ui
                .add_sized(
                    [button_w, 18.0],
                    egui::Button::new(i18n.t("toolbar-resume")).fill(btn_fill),
                )
                .clicked()
            {
                next_mode.set(EditorMode::Play);
            }
            if ui
                .add_sized(
                    [button_w, 18.0],
                    egui::Button::new(i18n.t("toolbar-stop")).fill(btn_fill),
                )
                .clicked()
            {
                next_mode.set(EditorMode::Edit);
            }
        }
    }
}
