use crate::node_getter;
use godot::classes::input::CursorShape;
use godot::classes::{
    CollisionShape2D, IRigidBody2D, Input, InputEvent, InputEventMouse, InputEventMouseButton,
    InputEventMouseMotion, PathFollow2D, RigidBody2D, StaticBody2D,
};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
struct Enemy {
    #[export]
    /// A [PathFollow2D] node that the enemy will continuously move along while not dragged.
    path_follow: Option<Gd<PathFollow2D>>,

    /// The progress (from 0 to 1) along [path_follow].
    progress: f64,

    /// State of the mouse w.r.t. this enemy.
    mouse_state: MouseState,

    base: Base<RigidBody2D>,
}

enum MouseState {
    None,
    Hovering,
    Dragging,
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

    fn on_mouse_entered(&mut self, entered: bool) {
        if !matches!(self.mouse_state, MouseState::Dragging) {
            self.mouse_state = if entered {
                MouseState::Hovering
            } else {
                MouseState::None
            };
        }
        let shape = if entered {
            CursorShape::POINTING_HAND
        } else {
            CursorShape::ARROW
        };
        Input::singleton()
            .set_default_cursor_shape_ex()
            .shape(shape)
            .done();
    }

    fn drag_towards_cursor(&mut self) {
        if !matches!(self.mouse_state, MouseState::Dragging) {
            return;
        }

        const STIFFNESS: f32 = 400.; // how hard are we pulling
        const DAMPING_RATIO: f32 = 0.9;
        const MAX_FORCE: f32 = 8000.;

        let local_mouse_pos = self.base().get_local_mouse_position();
        let mass = self.base().get_mass();
        let k = STIFFNESS * mass;
        let c = 2. * DAMPING_RATIO * (k * mass).sqrt();
        let force = local_mouse_pos * k - self.base().get_linear_velocity() * c;
        self.base_mut()
            .apply_central_force(force.limit_length(Some(MAX_FORCE)));
    }
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy {
            path_follow: None,
            progress: 0.0,
            mouse_state: MouseState::None,
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
    }

    fn physics_process(&mut self, _delta: f64) {
        self.drag_towards_cursor();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() {
            if mouse_button_event.is_pressed() {
                if matches!(self.mouse_state, MouseState::Hovering) {
                    self.mouse_state = MouseState::Dragging;
                }
            } else if mouse_button_event.is_released() {
                self.mouse_state = MouseState::None; // TODO consider hovering
            }
        }
    }
}
