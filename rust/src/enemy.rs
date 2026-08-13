use crate::node_getter;
use godot::classes::{
    CollisionShape2D, IRigidBody2D, Input, InputEvent, InputEventMouseMotion, PathFollow2D,
    RigidBody2D, StaticBody2D,
};
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
    node_getter!(StaticBody2D, "FollowMouseCursor");

    /// RigidBody2D does not support settings its scale directly, so we have to use this workaround
    /// by setting the scale of every child individually.
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

    /// Move the enemy along the specified PathFollow2D by the given amount.
    fn update_path_progress(&mut self, delta: f64) {
        let Some(path) = self.path_follow.as_mut() else {
            return;
        };
        self.progress += delta;
        path.set_progress_ratio(self.progress as f32);
        let new_position = path.get_position();
        self.base_mut().set_position(new_position);
    }

    fn on_mouse_entered(&mut self) {
        // TODO
    }

    fn on_mouse_exited(&mut self) {
        // TODO
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

    fn ready(&mut self) {
        self.set_scale(Vector2::new(0.5, 0.5));

        // TODO
        /*self.base()
        .signals()
        .mouse_entered()
        .connect_self(|s: &mut RigidBody2D| s.on_mouse_entered());*/
    }

    fn process(&mut self, delta: f64) {
        self.update_path_progress(delta / 20.0);
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        // Update mouse follow marker
        let Ok(event) = event.try_cast::<InputEventMouseMotion>() else {
            return;
        };
        let mouse_pos = event.get_global_position();
        self.get_node_follow_mouse_cursor()
            .set_global_position(mouse_pos);
    }
}
