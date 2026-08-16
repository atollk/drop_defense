use crate::node_getter;
use godot::classes::input::CursorShape;
use godot::classes::{
    AnimatedSprite2D, IRigidBody2D, Input, InputEvent, InputEventMouseButton, Path2D, RayCast2D,
    RigidBody2D,
};
use godot::prelude::*;
use itertools::Itertools;

const FALLING_GRAVITY: f32 = 1.5;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct Enemy {
    /// A [Path2D] node that the enemy will continuously move along while not dragged.
    #[export]
    pub walk_path: Option<Gd<Path2D>>,

    /// The max fall damage this enemy can take before dying.
    #[export]
    pub health: u32,

    /// The progress (from 0 to 1) along [path].
    progress: f64,

    /// Whether the mouse is inside this enemy's area.
    mouse_is_over: bool,

    /// State of the enemy
    state: EnemyState,

    /// The Y coordinate at which falling started.
    fall_height: f32,

    base: Base<RigidBody2D>,
}

#[derive(Debug, PartialEq, Eq)]
enum EnemyState {
    Walking,
    Dragged,
    Falling,
    Attacking,
    StandingUp,
    Dying,
}

impl EnemyState {
    fn animation_name(&self) -> &'static str {
        match self {
            EnemyState::Walking => "walk",
            EnemyState::Dragged => "stand",
            EnemyState::Falling => "stand",
            EnemyState::Attacking => "attack",
            EnemyState::StandingUp => "standup",
            EnemyState::Dying => "death",
        }
    }
}

#[godot_api]
impl Enemy {
    #[signal]
    pub fn died();
}

impl Enemy {
    node_getter!(Node2D, "GroundCast");
    node_getter!(Node2D, "GrabPoint");
    node_getter!(AnimatedSprite2D, "Sprite");

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
        if self.state != EnemyState::Walking
            && self.state != EnemyState::Attacking
            && self.state != EnemyState::StandingUp
            && self.state != EnemyState::Dying
        {
            return;
        }
        let Some(path) = self.walk_path.as_mut() else {
            return;
        };
        if self.state == EnemyState::Walking {
            self.progress = (self.progress + delta).min(1.0);
        }
        let curve = path
            .get_curve()
            .expect("the path must have a curve defined");
        let curve_len = curve.get_baked_length();
        let new_position = curve
            .sample_baked_ex()
            .offset(self.progress as f32 * curve_len)
            .done();
        self.base_mut().set_position(new_position);

        if self.progress == 1.0 && self.state == EnemyState::Walking {
            self.set_state(EnemyState::Attacking);
        }
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

    /// Connected to a signal whenever the mouse cursor enters or leaves the hitbox.
    fn on_mouse_entered(&mut self, entered: bool) {
        self.mouse_is_over = entered;
        self.update_cursor();
    }

    /// Connected to a signal whenever the sprite animation finishes.
    fn on_sprite_animation_finished(&mut self) {
        if self.state == EnemyState::Dying {
            self.signals().died().emit();
            self.base_mut().queue_free();
        }
        if self.state == EnemyState::StandingUp {
            self.set_state(EnemyState::Walking);
        }
    }

    /// Called in [physics_process] while the enemy is being dragged to apply the necessary forces.
    fn drag_towards_cursor(&mut self, _delta: f64) {
        const STIFFNESS: f32 = 600.; // how hard are we pulling
        const DAMPING_RATIO: f32 = 0.9;
        const MAX_FORCE: f32 = 8000.;

        let local_mouse_pos =
            self.base().get_local_mouse_position() - self.get_node_grab_point().get_position();
        let mass = self.base().get_mass();
        let k = STIFFNESS * mass;
        let c = 2. * DAMPING_RATIO * (k * mass).sqrt();
        let force = local_mouse_pos * k - self.base().get_linear_velocity() * c;
        // No need to use delta: Forces are time-independent
        self.base_mut()
            .apply_central_force(force.limit_length(Some(MAX_FORCE)));
    }

    /// A setter for `self.state` that makes sure the other invariants are updated correctly.
    fn set_state(&mut self, new_state: EnemyState) {
        self.state = new_state;

        // Mouse cursor
        self.update_cursor();

        // Gravity
        let gravity_scale = if self.state == EnemyState::Falling {
            FALLING_GRAVITY
        } else {
            0.
        };
        self.base_mut().set_gravity_scale(gravity_scale);

        // Physics
        let freeze_enabled = self.state == EnemyState::Walking;
        self.base_mut().set_freeze_enabled(freeze_enabled);

        // Animation
        self.get_node_sprite()
            .set_animation(self.state.animation_name());
        self.get_node_sprite().play();
    }

    /// Checks whether a falling enemy has hit the ground.
    fn check_fall_ground(&mut self) {
        if self.state != EnemyState::Falling {
            return;
        }
        let Some(walk_path) = &self.walk_path else {
            return;
        };
        if self.base().get_linear_velocity().y.is_sign_positive() {
            return;
        }

        // Check if any of the "foot" rays detects ground.
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

        let fall_damage = (self.base().get_global_position().y - self.fall_height) as f64;
        godot_print!("FALL DAMAGE: {}", fall_damage);
        if fall_damage > self.health as f64 {
            self.set_state(EnemyState::Dying);
        } else {
            self.set_state(EnemyState::StandingUp);
        }
    }
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy {
            walk_path: None,
            health: 0,
            progress: 0.0,
            mouse_is_over: false,
            state: EnemyState::Walking,
            fall_height: f32::INFINITY,
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
        self.get_node_sprite()
            .signals()
            .animation_finished()
            .connect_other(self, Self::on_sprite_animation_finished);
    }

    fn process(&mut self, delta: f64) {
        self.update_path_progress(delta / 20.0);
        self.check_fall_ground();
    }

    fn physics_process(&mut self, delta: f64) {
        if self.state == EnemyState::Dragged {
            self.drag_towards_cursor(delta);
        }

        // Update fall height
        if self.state == EnemyState::Falling {
            self.fall_height = self.fall_height.min(self.base().get_global_position().y);
        } else {
            self.fall_height = f32::INFINITY;
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
