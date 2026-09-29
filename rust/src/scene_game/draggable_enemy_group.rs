//! Add this node to your scene to handle drag-and-drop mechanic of enemies.

use crate::scene_game::enemy::Enemy;
use godot::classes::{INode2D, InputEvent, InputEventMouseButton, Node2D};
use godot::global::MouseButton;
use godot::obj::{Gd, WithBaseField};
use godot::prelude::{Base, GodotClass, godot_api};

#[derive(GodotClass)]
#[class(base=Node2D, init)]
pub struct DraggableEnemyGroup {
    base: Base<Node2D>,
}

impl DraggableEnemyGroup {
    /// Handles mouse button events (for grabbing enemies).
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
        let enemy_group = self.base().get_children();
        let enemies = enemy_group.iter_shared().map(|enemy| enemy.cast::<Enemy>());
        Enemy::drag_on_mouse_event(button_down, enemies);
    }

    pub fn add_enemy(&mut self, enemy: Gd<Enemy>) {
        self.base_mut().add_child(&enemy);
    }
}

#[godot_api]
impl INode2D for DraggableEnemyGroup {
    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() {
            self.on_mouse_button_event(mouse_button_event);
        }
    }
}
