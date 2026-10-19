use super::fluid_splash::FluidSplash;
use crate::enemy_weapon::EnemyWeapon;
use crate::portal::portal::Portal;
use crate::scene_game::enemy::enemy_state::EnemyState;
use crate::{node_getter, node_getter_named};
use godot::classes::{AnimatedSprite2D, AnimationPlayer, ShaderMaterial, Sprite2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct EnemySprite {
    /// The portal this enemy is currently in, if any.
    inside_portal: Option<Gd<Portal>>,
    base: Base<Node2D>,
}

#[godot_api]
impl EnemySprite {
    #[signal]
    pub fn animation_ended(animation_name: StringName);

    #[signal]
    pub fn death_animation_ended();

    pub fn set_weapon(&mut self, weapon: &Gd<EnemyWeapon>) {
        self.get_node_weapon()
            .set_texture(weapon.bind().texture.as_ref());
        self.get_node_weapon()
            .set_offset(weapon.bind().texture_offset);
    }
}

impl EnemySprite {
    node_getter!(BloodSplash: FluidSplash);
    node_getter!(LimbsSplash: FluidSplash);
    node_getter!(BodySprite: AnimatedSprite2D);
    node_getter_named!(Sprite2D, "WeaponHand/Weapon", get_node_weapon);
    node_getter!(AnimationPlayer: AnimationPlayer);

    pub fn get_shader_material(&self) -> Gd<ShaderMaterial> {
        self.base().get_material().unwrap().cast()
    }

    pub fn set_inside_portal(&mut self, value: Option<Gd<Portal>>) {
        self.inside_portal = value;
        let mut material = self.get_shader_material();
        material.set_shader_parameter(
            "falling_into_portal",
            &self.inside_portal.is_some().to_variant(),
        );
    }

    fn sync_shader_to_portal(&mut self) {
        let Some(portal) = self.inside_portal.as_ref() else {
            return;
        };
        let mut material = self.get_shader_material();
        material.set_shader_parameter(
            "portal_center_pos",
            &portal.get_global_position().to_variant(),
        );
        material.set_shader_parameter(
            "portal_radii",
            &portal
                .bind()
                .get_area_node()
                .unwrap()
                .get_global_scale()
                .to_variant(),
        );
    }

    pub fn play_state_animation(&mut self, state: EnemyState) {
        self.get_node_animation_player()
            .play_ex()
            .name(state.animation_name())
            .done();
        if state == EnemyState::Dying {
            self.get_node_blood_splash().bind_mut().spawn_drops();
            self.get_node_limbs_splash().bind_mut().spawn_drops();
            self.get_node_animation_player()
                .signals()
                .animation_finished()
                .connect_other(self, |s, _| s.get_node_body_sprite().set_visible(false));
            self.get_node_blood_splash()
                .signals()
                .animation_ended()
                .connect_other(self, |s| s.signals().death_animation_ended().emit());
        }
    }
}

#[godot_api]
impl INode2D for EnemySprite {
    fn ready(&mut self) {
        self.get_node_animation_player()
            .signals()
            .animation_finished()
            .connect_other(self, |s, animation_name| {
                s.signals().animation_ended().emit(&animation_name);
                s.get_node_animation_player().play();
            });
        self.get_node_body_sprite().stop();
    }

    fn process(&mut self, _delta: f64) {
        self.sync_shader_to_portal();
    }
}
