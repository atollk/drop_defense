use crate::portal::portal::Portal;
use crate::scene_game::portal_key::PortalKey;
use godot::classes::Area2D;
use godot::obj::{Base, WithBaseField};
use godot::prelude::{Gd, GodotClass};
use godot::signal::ConnectHandle;
use std::collections::HashMap;

#[derive(GodotClass)]
#[class(tool, init, base=Area2D)]
pub struct PortalSwallowArea {
    portal_key_signal_handles: HashMap<Gd<PortalKey>, ConnectHandle>,

    base: Base<Area2D>,
}

impl PortalSwallowArea {
    pub fn connect_signals(&mut self, portal: &Gd<Portal>) {
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
        {
            let portal = portal.clone();
            self.base()
                .signals()
                .area_entered()
                .connect_other(self, move |s, portal_key_area| {
                    s.on_portal_key_entered(&portal, portal_key_area.get_parent().unwrap().cast());
                });
        }
        {
            let portal = portal.clone();
            self.base()
                .signals()
                .area_exited()
                .connect_other(self, move |s, portal_key_area| {
                    s.on_portal_key_exited(&portal, portal_key_area.get_parent().unwrap().cast());
                });
        }
    }

    fn on_portal_key_entered(&mut self, portal: &Gd<Portal>, portal_key: Gd<PortalKey>) {
        let mut entrant = portal_key.bind().entrant_ancestor.clone().unwrap();
        if portal_key.bind().get_enabled() {
            entrant.dyn_bind_mut().entered_portal(&portal.bind());
        }
        // These signals only trigger when the areas are moving. Changing things like 'monitorable'
        // is not guaranteed to trigger a signal on itself. So we have to hand-build our own
        // solution for that with this and watch the enabling/disabling of PortalKeys while they
        // are inside the portal.
        let signal_fn = {
            let portal = portal.clone();
            move |s: &mut Self, enabled: bool| {
                if enabled {
                    entrant.dyn_bind_mut().entered_portal(&portal.bind());
                } else {
                    entrant.dyn_bind_mut().exited_portal(&portal.bind());
                }
            }
        };
        let handle = portal_key
            .signals()
            .enabled_changed()
            .connect_other(self, signal_fn);
        self.portal_key_signal_handles.insert(portal_key, handle);
    }

    fn on_portal_key_exited(&mut self, portal: &Gd<Portal>, portal_key: Gd<PortalKey>) {
        let mut entrant = portal_key.bind().entrant_ancestor.clone().unwrap();
        entrant.dyn_bind_mut().exited_portal(&portal.bind());
        self.portal_key_signal_handles.remove(&portal_key);
    }
}
