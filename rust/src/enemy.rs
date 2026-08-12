use crate::node_getter;
use godot::classes::{AnimatedSprite2D, CollisionShape2D, IRigidBody2D, PathFollow2D, RigidBody2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Enemy {
    #[export]
    path_follow: Option<Gd<PathFollow2D>>,
    progress: f64,
    base: Base<RigidBody2D>,
}

impl Enemy {
    node_getter!(CollisionShape2D, "CollisionShape2D");
    node_getter!(AnimatedSprite2D, "AnimatedSprite2D");

    fn get_all_children(&self) -> impl Iterator<Item = Gd<Node2D>> {
        [
            self.get_node_animated_sprite2_d().upcast::<Node2D>(),
            self.get_node_collision_shape2_d().upcast(),
        ]
        .into_iter()
    }

    pub fn set_scale(&mut self, scale: Vector2) {
        for mut child in self.get_all_children() {
            child.set_scale(scale);
        }
    }
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy {
            path_follow: None,
            progress: 0.0,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        self.progress += delta / 20.;
        godot_print!("{}", self.progress);
        let new_position = {
            let path = self
                .path_follow
                .as_mut()
                .expect("Enemy.path must be provided");
            path.set_progress_ratio(self.progress as f32);
            path.get_position()
        };
        self.base_mut().set_position(new_position);
    }
}
