//! Undo/Redo system with trait-based action recording.

use bevy::ecs::component::Mutable;
use bevy::prelude::*;

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

/// Resource that manages the undo/redo stack.
#[derive(Resource)]
pub struct UndoStack {
    undo_stack: Vec<Box<dyn UndoAction>>,
    redo_stack: Vec<Box<dyn UndoAction>>,
    /// Maximum number of undo history entries.
    pub max_history: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 100,
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

    /// Whether there are actions to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there are actions to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Description of the last undo-able action.
    pub fn undo_description(&self) -> Option<&str> {
        self.undo_stack.last().map(|a| a.description())
    }

    /// Description of the last redo-able action.
    pub fn redo_description(&self) -> Option<&str> {
        self.redo_stack.last().map(|a| a.description())
    }
}

/// System that handles Ctrl+Z / Ctrl+Shift+Z keyboard shortcuts.
pub fn undo_input_system(world: &mut World) {
    let ctrl_pressed;
    let shift_pressed;
    let z_just_pressed;

    {
        let input = world.resource::<ButtonInput<KeyCode>>();
        ctrl_pressed = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);
        shift_pressed = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
        z_just_pressed = input.just_pressed(KeyCode::KeyZ);
    }

    if !ctrl_pressed || !z_just_pressed {
        return;
    }

    let mut undo_stack = world.remove_resource::<UndoStack>();
    if let Some(ref mut stack) = undo_stack {
        if shift_pressed {
            stack.redo(world);
        } else {
            stack.undo(world);
        }
    }
    if let Some(stack) = undo_stack {
        world.insert_resource(stack);
    }
}
