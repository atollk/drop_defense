use godot::classes::{IRigidBody2D, PathFollow2D, RigidBody2D};
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
    pub fn set_scale(&mut self, scale: Vector2) {
        for child in self.base().get_children().iter_shared() {
            if let Ok(mut child) = child.try_cast::<Node2D>() {
                let old_scale = child.get_scale();
                child.set_scale(scale);
                let new_position = child.get_position() * scale / old_scale;
                child.set_position(new_position);
            }
        }
    }

    fn update_path_progress(&mut self, delta: f64) {
        self.progress += delta / 20.;
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
        self.update_path_progress(delta);
        self.set_scale(Vector2::new(0.5, 0.5));
    }
}
