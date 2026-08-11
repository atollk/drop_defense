use crate::node_getter;
use godot::classes::{
    CollisionShape2D, IPath2D, IRigidBody2D, Path2D, PathFollow2D, RigidBody2D, Sprite2D,
};
use godot::global::print;
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
