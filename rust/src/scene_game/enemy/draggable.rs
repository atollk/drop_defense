use godot::classes::input::CursorShape;
use godot::classes::{Area2D, CollisionShape2D, Input, RigidBody2D};
use godot::prelude::*;
use itertools::Itertools;

// TODO: enemies can be dragged below the floor and fall out of bounds

const AREA_NODE_NAME: &str = "Area2D";

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct Draggable {
    /// Can currently be dragged?
    pub can_be_dragged: bool,

    /// Whether the mouse is inside this entities' area.
    mouse_is_over: bool,

    /// The position at which this entity was grabbed by the cursor.
    grabbed_at: Option<Vector2>,

    base: Base<Node2D>,
}

#[godot_api]
impl Draggable {
    #[signal]
    pub fn drag_started();

    #[signal]
    pub fn drag_ended();
}

impl Draggable {
    pub fn drag_on_mouse_event(button_down: bool, draggables: &mut [Gd<Self>]) {
        if button_down {
            if draggables.iter().any(|d| d.bind().grabbed_at.is_some()) {
                panic!("We were dragging an enemy already and sent a mouse press during that.")
            }
            const DRAG_ENEMIES_AT_ONCE: usize = 1;
            for mut draggable in draggables
                .iter_mut()
                .map(|d| d.bind_mut())
                .filter(|d| d.can_be_dragged && d.mouse_is_over)
                .take(DRAG_ENEMIES_AT_ONCE)
            {
                draggable.grabbed_at = Some(draggable.base().get_local_mouse_position());
                draggable.signals().drag_started().emit();
            }
        } else {
            for mut draggable in draggables
                .iter_mut()
                .map(|d| d.bind_mut())
                .filter(|d| d.grabbed_at.is_some())
            {
                draggable.grabbed_at = None;
                draggable.signals().drag_ended().emit();
            }
        }
    }

    fn update_cursor(&self) {
        // TODO: there can probably be conflicts if grabbing one enemy and hovering over another
        let shape = if self.grabbed_at.is_some() {
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

    /// Called in [physics_process] while the enemy is being dragged to apply the necessary forces.
    pub fn drag_towards_cursor(&mut self, parent: &mut Gd<RigidBody2D>, _delta: f64) {
        const STIFFNESS: f32 = 600.; // how hard are we pulling
        const DAMPING_RATIO: f32 = 0.9;
        const MAX_FORCE: f32 = 8000.;

        let Some(grabbed_at) = self.grabbed_at else {
            return;
        };

        let local_mouse_pos = self.base().get_local_mouse_position() - grabbed_at;
        let mass = parent.get_mass();
        let k = STIFFNESS * mass;
        let c = 2. * DAMPING_RATIO * (k * mass).sqrt();
        let force = local_mouse_pos * k - parent.get_linear_velocity() * c;
        let limited_force = force.limit_length(Some(MAX_FORCE));
        // No need to use delta: Forces are time-independent
        parent.apply_central_force(limited_force);
    }

    pub fn get_shape(&self) -> Option<Gd<CollisionShape2D>> {
        let area: Gd<Area2D> = self.base().get_node_as(AREA_NODE_NAME);
        if area.get_child_count() > 1 {
            panic!("Draggable area has two children for some reason.");
        }
        area.get_children()
            .iter_shared()
            .exactly_one()
            .ok()
            .map(|node| node.cast())
    }

    pub fn set_shape(&mut self, shape: Option<Gd<CollisionShape2D>>) {
        let mut area: Gd<Area2D> = self.base().get_node_as(AREA_NODE_NAME);
        for mut child in area.get_children().iter_shared() {
            area.remove_child(&child);
            child.queue_free();
        }
        let Some(shape) = shape else { return };
        area.add_child(&shape);
    }
}

#[godot_api]
impl INode2D for Draggable {
    fn ready(&mut self) {
        let area = {
            let mut area = Area2D::new_alloc();
            area.set_name(AREA_NODE_NAME);
            area.signals()
                .mouse_entered()
                .connect_other(self, |s| s.on_mouse_entered(true));
            area.signals()
                .mouse_exited()
                .connect_other(self, |s| s.on_mouse_entered(false));
            area
        };
        self.base_mut().add_child(&area);
    }
}
