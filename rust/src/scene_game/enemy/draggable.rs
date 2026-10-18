use godot::classes::{Area2D, CollisionShape2D, RigidBody2D};
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

    #[signal]
    pub fn cursor_changed();
}

impl Draggable {
    pub fn is_grabbed(&self) -> bool {
        self.grabbed_at.is_some()
    }

    pub fn is_mouse_over(&self) -> bool {
        self.mouse_is_over
    }

    pub fn drag_on_mouse_event<'a>(
        button_down: bool,
        draggables: impl IntoIterator<Item = &'a mut Gd<Self>>,
    ) {
        let draggables = draggables.into_iter().map(|d| d.bind_mut());
        if button_down {
            const DRAG_ENEMIES_AT_ONCE: usize = 1;
            let mut grabbed_count = 0;
            for mut draggable in draggables {
                assert!(
                    !draggable.is_grabbed(),
                    "We were dragging an enemy already and sent a mouse press during that."
                );
                if !draggable.can_be_dragged || !draggable.mouse_is_over {
                    continue;
                }
                if grabbed_count >= DRAG_ENEMIES_AT_ONCE {
                    continue;
                }
                grabbed_count += 1;
                draggable.grabbed_at = Some(draggable.base().get_local_mouse_position());
                draggable.signals().drag_started().emit();
            }
        } else {
            for mut draggable in draggables {
                if draggable.is_grabbed() {
                    draggable.grabbed_at = None;
                    draggable.signals().drag_ended().emit();
                }
            }
        }
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
            area.signals().mouse_entered().connect_other(self, |s| {
                s.mouse_is_over = true;
                s.signals().cursor_changed().emit();
            });
            area.signals().mouse_exited().connect_other(self, |s| {
                s.mouse_is_over = false;
                s.signals().cursor_changed().emit();
            });
            area
        };
        self.base_mut().add_child(&area);
    }
}
