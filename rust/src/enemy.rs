use crate::node_getter;
use godot::classes::input::CursorShape;
use godot::classes::{
    IRigidBody2D, Input, InputEvent, InputEventMouseButton, Path2D, RayCast2D, RigidBody2D,
};
use godot::prelude::*;
use itertools::Itertools;

const FALLING_GRAVITY: f32 = 1.5;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Enemy {
    #[export]
    /// A [Path2D] node that the enemy will continuously move along while not dragged.
    pub walk_path: Option<Gd<Path2D>>,

    /// The progress (from 0 to 1) along [path].
    progress: f64,

    /// Whether the mouse is inside this enemy's area.
    mouse_is_over: bool,

    /// State of the enemy
    state: EnemyState,

    /// The highest recorded fall velocity in Y direction.
    fall_velocity: f64,

    base: Base<RigidBody2D>,
}

#[derive(Debug, PartialEq, Eq)]
enum EnemyState {
    Walking,
    Dragged,
    Falling,
}

impl Enemy {
    node_getter!(Node2D, "GroundCast");
    node_getter!(Node2D, "GrabPoint");

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
        if self.state != EnemyState::Walking {
            return;
        }
        let Some(path) = self.walk_path.as_mut() else {
            return;
        };
        self.progress += delta;
        let curve = path
            .get_curve()
            .expect("the path must have a curve defined");
        let curve_len = curve.get_baked_length();
        let new_position = curve
            .sample_baked_ex()
            .offset(self.progress as f32 * curve_len)
            .done();
        self.base_mut().set_position(new_position);
    }

    fn update_cursor(&self) {
        let shape = if self.state == EnemyState::Dragged {
            CursorShape::MOVE
        } else {
            if self.mouse_is_over {
                CursorShape::POINTING_HAND
            } else {
                CursorShape::ARROW
            }
        };
        Input::singleton()
            .set_default_cursor_shape_ex()
            .shape(shape)
            .done();
    }

    fn on_mouse_entered(&mut self, entered: bool) {
        self.mouse_is_over = entered;
        self.update_cursor();
    }

    fn drag_towards_cursor(&mut self) {
        const STIFFNESS: f32 = 600.; // how hard are we pulling
        const DAMPING_RATIO: f32 = 0.9;
        const MAX_FORCE: f32 = 8000.;

        let local_mouse_pos =
            self.base().get_local_mouse_position() - self.get_node_grab_point().get_position();
        let mass = self.base().get_mass();
        let k = STIFFNESS * mass;
        let c = 2. * DAMPING_RATIO * (k * mass).sqrt();
        let force = local_mouse_pos * k - self.base().get_linear_velocity() * c;
        self.base_mut()
            .apply_central_force(force.limit_length(Some(MAX_FORCE)));
    }

    /// A setter for `self.state` that makes sure the other invariants are updated correctly.
    fn set_state(&mut self, new_state: EnemyState) {
        self.state = new_state;
        self.update_cursor();
        let gravity_scale = if self.state == EnemyState::Falling {
            FALLING_GRAVITY
        } else {
            0.
        };
        self.base_mut().set_gravity_scale(gravity_scale);
        let freeze_enabled = self.state == EnemyState::Walking;
        self.base_mut().set_freeze_enabled(freeze_enabled);
    }

    /// Checks whether a falling enemy has hit the ground.
    fn check_fall_ground(&mut self) {
        if self.state != EnemyState::Falling {
            return;
        }
        let Some(walk_path) = &self.walk_path else {
            return;
        };

        let rays: [_; 3] = self
            .get_node_ground_cast()
            .get_children()
            .iter_shared()
            .map(|node| node.cast::<RayCast2D>())
            .collect_array()
            .unwrap();
        let hit_ground = rays.iter().any(|ray| ray.is_colliding());
        if !hit_ground {
            return;
        }

        // The enemy has fallen to the ground.
        let curve = walk_path
            .get_curve()
            .expect("the path must have a curve defined");
        let offset =
            curve.get_closest_offset(walk_path.to_local(self.base().get_global_position()));
        self.progress = (offset / curve.get_baked_length()) as f64;
        self.set_state(EnemyState::Walking);
        godot_print!("{}", self.fall_velocity)
    }
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy {
            walk_path: None,
            progress: 0.0,
            mouse_is_over: false,
            state: EnemyState::Walking,
            fall_velocity: 0.0,
            base,
        }
    }

    fn ready(&mut self) {
        self.set_scale(Vector2::new(0.5, 0.5));

        self.base()
            .signals()
            .mouse_entered()
            .connect_other(self, |s| s.on_mouse_entered(true));
        self.base()
            .signals()
            .mouse_exited()
            .connect_other(self, |s| s.on_mouse_entered(false));
    }

    fn process(&mut self, delta: f64) {
        self.update_path_progress(delta / 20.0);
        self.check_fall_ground();
    }

    fn physics_process(&mut self, _delta: f64) {
        if self.state == EnemyState::Dragged {
            self.drag_towards_cursor();
        }

        // Update fall velocity
        if self.state == EnemyState::Falling {
            let current_velocity_y = self.base().get_linear_velocity().y as f64;
            self.fall_velocity = self.fall_velocity.max(current_velocity_y);
        } else {
            self.fall_velocity = 0.0;
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() {
            if mouse_button_event.is_pressed() {
                if self.state == EnemyState::Dragged {
                    unreachable!(
                        "We were dragging an enemy already and sent a mouse press during that."
                    )
                } else {
                    if self.mouse_is_over {
                        self.set_state(EnemyState::Dragged);
                    }
                }
            } else if mouse_button_event.is_released() {
                self.set_state(EnemyState::Falling);
            }
        }
    }
}
