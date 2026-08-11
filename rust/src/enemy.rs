use crate::node_getter;
use godot::classes::{CollisionShape2D, IRigidBody2D, RigidBody2D, Sprite2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Enemy {
    base: Base<RigidBody2D>,
}

impl Enemy {
    node_getter!(CollisionShape2D, "CollisionShape2D");
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy { base }
    }
}
