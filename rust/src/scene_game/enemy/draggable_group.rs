//! Add this node to your scene to handle drag-and-drop mechanic of enemies.

use crate::scene_game::enemy::draggable::Draggable;
use godot::classes::input::CursorShape;
use godot::classes::{INode2D, Input, InputEvent, InputEventMouseButton, Node2D};
use godot::global::MouseButton;
use godot::obj::{Gd, Singleton};
use godot::prelude::{Base, GodotClass, godot_api};
use godot::signal::ConnectHandle;
use itertools::Itertools;

/// A helper node that groups together multiple `Draggable` nodes so only one of them can be grabbed at once.
#[derive(GodotClass)]
#[class(base=Node2D, init)]
pub struct DraggableGroup {
    draggables: Vec<(Gd<Draggable>, ConnectHandle)>,
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
        Draggable::drag_on_mouse_event(
            button_down,
            &mut self.draggables.iter_mut().map(|(a, _)| a),
        );
    }

    pub fn add_draggable(&mut self, draggable: Gd<Draggable>) {
        let signal_handle = draggable
            .signals()
            .cursor_changed()
            .connect_other(self, |s| s.update_cursor());
        self.draggables.push((draggable, signal_handle));
    }

    pub fn remove_draggable(&mut self, draggable: Gd<Draggable>) {
        let position = self
            .draggables
            .iter()
            .find_position(|(d, _)| *d == draggable)
            .unwrap()
            .0;
        let (_, signal_handle) = self.draggables.remove(position);
        signal_handle.disconnect();
    }

    fn update_cursor(&self) {
        let shape = {
            let grabbed = self.draggables.iter().find(|d| d.0.bind().is_grabbed());
            if grabbed.is_some() {
                CursorShape::MOVE
            } else {
                let hovered = self.draggables.iter().find(|d| d.0.bind().is_mouse_over());
                if hovered.is_some() {
                    CursorShape::POINTING_HAND
                } else {
                    CursorShape::ARROW
                }
            }
        };
        Input::singleton()
            .set_default_cursor_shape_ex()
            .shape(shape)
            .done();
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
