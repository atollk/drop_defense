use crate::portal::portal::Portal;
use godot::classes::{Area2D, CircleShape2D, CollisionShape2D};
use godot::prelude::*;

/// A PortalKey is a node that is recognized by a portal when nearby and causes the portal to open.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct PortalKey {
    #[export]
    // TODO: emit a warning if this is assigned incorrectly; or remove it and infer it from the scene tree
    pub entrant_ancestor: Option<DynGd<Node2D, dyn PortalEntrant>>,

    #[export(flags_2d_physics)]
    collision_layer: u32,

    #[export]
    #[var(set, get)]
    enabled: PhantomVar<bool>,

    base: Base<Node2D>,
}

const AREA_CHILD_NAME: &str = "Area2D";

impl PortalKey {
    fn get_node_area(&self) -> Gd<Area2D> {
        self.base().get_node_as(AREA_CHILD_NAME)
    }
}

#[godot_api]
impl PortalKey {
    #[func]
    pub fn get_enabled(&self) -> bool {
        self.get_node_area().is_monitorable()
    }

    #[func]
    // TODO: it looks like changing this doesn't trigger exit/entry signals
    pub fn set_enabled(&mut self, value: bool) {
        self.get_node_area()
            .set_deferred("monitorable", &value.to_variant());
    }
}

#[godot_api]
impl INode2D for PortalKey {
    fn ready(&mut self) {
        let shape = {
            let mut shape = CircleShape2D::new_gd();
            shape.set_radius(0.01);
            shape
        };
        let collision_shape = {
            let mut collision_shape = CollisionShape2D::new_alloc();
            collision_shape.set_shape(&shape);
            collision_shape
        };
        let area = {
            let mut area = Area2D::new_alloc();
            area.set_name(AREA_CHILD_NAME);
            area.set_collision_layer(self.collision_layer);
            area.set_collision_mask(0);
            area.add_child(&collision_shape);
            area
        };
        self.base_mut().add_child(&area);
    }
}

pub trait PortalEntrant {
    fn entered_portal(&mut self, portal: &Portal);
    fn exited_portal(&mut self, portal: &Portal);
}
