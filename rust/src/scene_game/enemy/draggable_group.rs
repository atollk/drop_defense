//! Add this node to your scene to handle drag-and-drop mechanic of enemies.

use crate::scene_game::enemy::draggable::Draggable;
use godot::classes::{INode2D, InputEvent, InputEventMouseButton, Node2D};
use godot::global::MouseButton;
use godot::obj::Gd;
use godot::prelude::{Base, GodotClass, godot_api};
use itertools::Itertools;

/// A helper node that groups together multiple `Draggable` nodes so only one of them can be grabbed at once.
#[derive(GodotClass)]
#[class(base=Node2D, init)]
pub struct DraggableGroup {
    draggables: Vec<Gd<Draggable>>,
    base: Base<Node2D>,
}

impl DraggableGroup {
    fn on_mouse_button_event(&mut self, event: Gd<InputEventMouseButton>) {
        if event.get_button_index() != MouseButton::LEFT {
            return;
        }
        let button_down = if event.is_pressed() {
            true
        } else if event.is_released() {
            false
        } else {
            return;
        };
        Draggable::drag_on_mouse_event(button_down, &mut self.draggables);
    }

    pub fn add_draggable(&mut self, draggable: Gd<Draggable>) {
        self.draggables.push(draggable);
    }

    pub fn remove_draggable(&mut self, draggable: Gd<Draggable>) {
        self.draggables.remove(
            self.draggables
                .iter()
                .find_position(|d| **d == draggable)
                .unwrap()
                .0,
        );
    }
}

#[godot_api]
impl INode2D for DraggableGroup {
    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() {
            self.on_mouse_button_event(mouse_button_event);
        }
    }
}
