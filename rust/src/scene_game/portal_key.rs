use crate::scene_game::portal::Portal;
use godot::classes::{Area2D, CircleShape2D, CollisionShape2D};
use godot::prelude::*;

/// A PortalKey is a node that is recognized by a portal when nearby and causes the portal to open.
#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct PortalKey {
    base: Base<Node2D>,
}

const AREA_CHILD_NAME: &str = "Area2D";

#[godot_api]
impl PortalKey {
    #[signal]
    fn portal_opened(portal: Gd<Portal>);

    #[signal]
    fn portal_closed(portal: Gd<Portal>);

    #[signal]
    fn portal_moved(portal: Gd<Portal>);
}

#[godot_api]
impl INode2D for PortalKey {
    fn ready(&mut self) {
        let collision_shape = {
            let mut shape = CircleShape2D::new_gd();
            shape.set_radius(0.01);
            let mut collision_shape = CollisionShape2D::new_alloc();
            collision_shape.set_shape(&shape);
            collision_shape
        };
        let area = {
            let mut area = Area2D::new_alloc();
            area.set_name(AREA_CHILD_NAME);
            area.add_child(&collision_shape);
            area
        };
        self.base_mut().add_child(&area);
    }
}
