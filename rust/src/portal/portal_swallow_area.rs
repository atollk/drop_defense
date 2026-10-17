use crate::portal::portal::Portal;
use crate::scene_game::portal_key::PortalKey;
use godot::classes::Area2D;
use godot::obj::{Base, WithBaseField};
use godot::prelude::GodotClass;

#[derive(GodotClass)]
#[class(tool, init, base=Area2D)]
pub struct PortalSwallowArea {
    base: Base<Area2D>,
}

impl PortalSwallowArea {
    pub fn connect_signals(&mut self, portal: &Portal) {
        // Connections for the open/close animation.
        self.base()
            .signals()
            .area_entered()
            .connect_other(portal, |s, _| {
                s.open();
            });
        self.base()
            .signals()
            .area_exited()
            .connect_other(portal, |s, _| {
                s.close();
            });
        // Connections to the enemy.
        self.base()
            .signals()
            .area_entered()
            .connect_other(portal, |s, portal_key_area| {
                // TODO: it would be nicer if this was using an interface rather than hardcoded for Enemies
                let portal_key = portal_key_area.get_parent().unwrap().cast::<PortalKey>();
                let mut enemy = portal_key.bind().enemy_parent.clone().unwrap();
                enemy.bind_mut().entered_portal(s);
            });
        self.base()
            .signals()
            .area_exited()
            .connect_other(portal, |s, portal_key_area| {
                // TODO: it would be nicer if this was using an interface rather than hardcoded for Enemies
                let portal_key = portal_key_area.get_parent().unwrap().cast::<PortalKey>();
                let mut enemy = portal_key.bind().enemy_parent.clone().unwrap();
                enemy.bind_mut().exited_portal(s);
            });
    }
}
