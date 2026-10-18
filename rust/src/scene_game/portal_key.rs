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
    layers_2d_physics: u32,

    base: Base<Node2D>,
}

const AREA_CHILD_NAME: &str = "Area2D";

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
            area.set_collision_layer(self.layers_2d_physics);
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
