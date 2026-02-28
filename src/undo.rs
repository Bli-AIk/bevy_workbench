//! Undo/Redo system with trait-based action recording.

use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy_egui::egui;

/// Trait for undo/redo actions.
pub trait UndoAction: Send + Sync + 'static {
    /// Undo this action.
    fn undo(&self, world: &mut World);
    /// Redo this action.
    fn redo(&self, world: &mut World);
    /// Human-readable description for UI display.
    fn description(&self) -> &str;
}

/// Undo action for a component change on a mutable component.
struct ComponentUndoAction<T: Component<Mutability = Mutable> + Clone + 'static> {
    entity: Entity,
    old_value: T,
    new_value: T,
    desc: String,
}

impl<T: Component<Mutability = Mutable> + Clone + 'static> UndoAction for ComponentUndoAction<T> {
    fn undo(&self, world: &mut World) {
        if let Some(mut component) = world.get_mut::<T>(self.entity) {
            *component = self.old_value.clone();
        }
    }

    fn redo(&self, world: &mut World) {
        if let Some(mut component) = world.get_mut::<T>(self.entity) {
            *component = self.new_value.clone();
        }
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

/// Undo action for a resource change.
struct ResourceUndoAction<T: Resource + Clone + 'static> {
    old_value: T,
    new_value: T,
    desc: String,
}

impl<T: Resource + Clone + 'static> UndoAction for ResourceUndoAction<T> {
    fn undo(&self, world: &mut World) {
        world.insert_resource(self.old_value.clone());
    }

    fn redo(&self, world: &mut World) {
        world.insert_resource(self.new_value.clone());
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

/// Undo action that groups multiple actions into one.
/// Undo/redo applies all actions in reverse/forward order.
pub struct GroupUndoAction {
    actions: Vec<Box<dyn UndoAction>>,
    desc: String,
}

impl GroupUndoAction {
    pub fn new(desc: impl Into<String>, actions: Vec<Box<dyn UndoAction>>) -> Self {
        Self {
            actions,
            desc: desc.into(),
        }
    }
}

impl UndoAction for GroupUndoAction {
    fn undo(&self, world: &mut World) {
        for action in self.actions.iter().rev() {
            action.undo(world);
        }
    }

    fn redo(&self, world: &mut World) {
        for action in &self.actions {
            action.redo(world);
        }
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

/// A closure-based undo action for custom one-off operations.
pub struct ClosureUndoAction {
    undo_fn: Box<dyn Fn(&mut World) + Send + Sync>,
    redo_fn: Box<dyn Fn(&mut World) + Send + Sync>,
    desc: String,
}

impl ClosureUndoAction {
    pub fn new(
        desc: impl Into<String>,
        undo_fn: impl Fn(&mut World) + Send + Sync + 'static,
        redo_fn: impl Fn(&mut World) + Send + Sync + 'static,
    ) -> Self {
        Self {
            undo_fn: Box::new(undo_fn),
            redo_fn: Box::new(redo_fn),
            desc: desc.into(),
        }
    }
}

impl UndoAction for ClosureUndoAction {
    fn undo(&self, world: &mut World) {
        (self.undo_fn)(world);
    }

    fn redo(&self, world: &mut World) {
        (self.redo_fn)(world);
    }

    fn description(&self) -> &str {
        &self.desc
    }
}

/// Resource that manages the undo/redo stack.
#[derive(Resource)]
pub struct UndoStack {
    undo_stack: Vec<Box<dyn UndoAction>>,
    redo_stack: Vec<Box<dyn UndoAction>>,
    /// Maximum number of undo history entries.
    pub max_history: usize,
    /// Set to true to request undo on next frame (for menu buttons).
    pub undo_requested: bool,
    /// Set to true to request redo on next frame (for menu buttons).
    pub redo_requested: bool,
    /// Set to request jumping to a specific history index.
    pub jump_requested: Option<usize>,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
            undo_requested: false,
            redo_requested: false,
            jump_requested: None,
        }
    }
}

impl UndoStack {
    /// Record a component change as an undo action.
    pub fn record_component<T: Component<Mutability = Mutable> + Clone + 'static>(
        &mut self,
        entity: Entity,
        old_value: T,
        new_value: T,
    ) {
        let desc = format!("Modify {} on {:?}", std::any::type_name::<T>(), entity);
        self.push(ComponentUndoAction {
            entity,
            old_value,
            new_value,
            desc,
        });
    }

    /// Record a resource change as an undo action.
    pub fn record_resource<T: Resource + Clone + 'static>(&mut self, old_value: T, new_value: T) {
        let desc = format!("Modify {}", std::any::type_name::<T>());
        self.push(ResourceUndoAction {
            old_value,
            new_value,
            desc,
        });
    }

    /// Push a custom undo action.
    pub fn push(&mut self, action: impl UndoAction) {
        self.redo_stack.clear();
        self.undo_stack.push(Box::new(action));
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Push a boxed undo action.
    pub fn push_boxed(&mut self, action: Box<dyn UndoAction>) {
        self.redo_stack.clear();
        self.undo_stack.push(action);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last action.
    pub fn undo(&mut self, world: &mut World) {
        if let Some(action) = self.undo_stack.pop() {
            action.undo(world);
            self.redo_stack.push(action);
        }
    }

    /// Redo the last undone action.
    pub fn redo(&mut self, world: &mut World) {
        if let Some(action) = self.redo_stack.pop() {
            action.redo(world);
            self.undo_stack.push(action);
        }
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Whether there are actions to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there are actions to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Number of actions in the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of actions in the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Description of the last undo-able action.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|a| a.description())
    }

    /// Description of the last redo-able action.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|a| a.description())
    }

    /// Returns descriptions of all undo entries (oldest first).
    pub fn undo_history(&self) -> Vec<&str> {
        self.undo_stack.iter().map(|a| a.description()).collect()
    }

    /// Returns descriptions of all redo entries (next-to-redo first).
    pub fn redo_history(&self) -> Vec<&str> {
        self.redo_stack
            .iter()
            .rev()
            .map(|a| a.description())
            .collect()
    }

    /// Jump to a specific state by index.
    /// Index 0 = initial state (undo everything), index == undo_count = current state.
    pub fn jump_to(&mut self, target_index: usize, world: &mut World) {
        let current = self.undo_stack.len();
        if target_index < current {
            // Undo forward (current → target)
            for _ in 0..(current - target_index) {
                if let Some(action) = self.undo_stack.pop() {
                    action.undo(world);
                    self.redo_stack.push(action);
                }
            }
        } else if target_index > current {
            // Redo forward (current → target)
            let steps = target_index - current;
            for _ in 0..steps {
                if let Some(action) = self.redo_stack.pop() {
                    action.redo(world);
                    self.undo_stack.push(action);
                }
            }
        }
    }
}

/// System that handles undo/redo keyboard shortcuts and menu requests.
pub fn undo_input_system(world: &mut World) {
    let bindings = world
        .get_resource::<super::keybind::KeyBindings>()
        .cloned()
        .unwrap_or_default();
    let input = world.resource::<ButtonInput<KeyCode>>();

    let do_undo = bindings.undo.just_pressed(input);
    let do_redo = bindings.redo.just_pressed(input);

    // Also check request flags from menu buttons
    let (menu_undo, menu_redo, jump_target) = world
        .get_resource::<UndoStack>()
        .map(|s| (s.undo_requested, s.redo_requested, s.jump_requested))
        .unwrap_or_default();

    let want_undo = do_undo || menu_undo;
    let want_redo = do_redo || menu_redo;

    if !want_undo && !want_redo && jump_target.is_none() {
        return;
    }

    let mut undo_stack = world.remove_resource::<UndoStack>();
    if let Some(ref mut stack) = undo_stack {
        stack.undo_requested = false;
        stack.redo_requested = false;
        stack.jump_requested = None;

        if let Some(target) = jump_target {
            stack.jump_to(target, world);
        } else if want_redo {
            stack.redo(world);
        } else if want_undo {
            stack.undo(world);
        }
    }
    if let Some(stack) = undo_stack {
        world.insert_resource(stack);
    }
}

/// Panel that shows undo/redo history as a clickable list.
pub struct UndoHistoryPanel;

impl crate::dock::WorkbenchPanel for UndoHistoryPanel {
    fn id(&self) -> &str {
        "undo_history"
    }

    fn title(&self) -> String {
        "Undo History".to_string()
    }

    fn ui(&mut self, _ui: &mut egui::Ui) {}

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        let Some(mut stack) = world.remove_resource::<UndoStack>() else {
            ui.label("No undo stack");
            return;
        };

        let undo_descs: Vec<String> = stack
            .undo_stack
            .iter()
            .map(|a| a.description().to_string())
            .collect();
        let redo_descs: Vec<String> = stack
            .redo_stack
            .iter()
            .rev()
            .map(|a| a.description().to_string())
            .collect();
        let current_index = undo_descs.len();

        egui::Frame::NONE
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "History: {} undo, {} redo",
                        undo_descs.len(),
                        redo_descs.len()
                    ));
                    if ui.small_button("Clear").clicked() {
                        stack.clear();
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Initial state
                        let is_current = current_index == 0;
                        let label = if is_current {
                            egui::RichText::new("▸ (initial state)")
                                .strong()
                                .color(egui::Color32::WHITE)
                        } else {
                            egui::RichText::new("  (initial state)").color(egui::Color32::GRAY)
                        };
                        if ui.selectable_label(is_current, label).clicked() && !is_current {
                            stack.jump_requested = Some(0);
                        }

                        // Undo entries (past actions)
                        for (i, desc) in undo_descs.iter().enumerate() {
                            let idx = i + 1;
                            let is_current = idx == current_index;
                            let label = if is_current {
                                egui::RichText::new(format!("▸ {desc}"))
                                    .strong()
                                    .color(egui::Color32::WHITE)
                            } else {
                                egui::RichText::new(format!("  {desc}"))
                            };
                            if ui.selectable_label(is_current, label).clicked() && !is_current {
                                stack.jump_requested = Some(idx);
                            }
                        }

                        // Redo entries (future actions, grayed out)
                        for (i, desc) in redo_descs.iter().enumerate() {
                            let idx = current_index + 1 + i;
                            let label = egui::RichText::new(format!("  {desc}"))
                                .color(egui::Color32::from_gray(100));
                            if ui.selectable_label(false, label).clicked() {
                                stack.jump_requested = Some(idx);
                            }
                        }
                    });
            });

        world.insert_resource(stack);
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        false
    }
}
