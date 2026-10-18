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
    base: Base<Node2D>,
}

// TODO: show Sprite2D icon for nodes of this class

#[godot_api]
impl EnemySprite {
    #[signal]
    pub fn animation_ended(animation_name: StringName);

    pub fn set_weapon(&mut self, weapon: &Gd<EnemyWeapon>) {
        self.get_node_weapon()
            .set_texture(weapon.bind().texture.as_ref());
        self.get_node_weapon()
            .set_offset(weapon.bind().texture_offset);
    }
}

impl EnemySprite {
    node_getter!(FluidSplash, "BloodSplash");
    node_getter!(FluidSplash, "LimbsSplash");
    node_getter!(AnimatedSprite2D, "BodySprite");
    node_getter_named!(Sprite2D, "WeaponHand/Weapon", get_node_weapon);
    node_getter!(AnimationPlayer, "AnimationPlayer");

    pub fn get_shader_material(&self) -> Gd<ShaderMaterial> {
        self.base().get_material().unwrap().cast()
    }

    pub fn set_inside_portal(&mut self, value: bool) {
        let mut material = self.get_shader_material();
        material.set_shader_parameter("falling_into_portal", &value.to_variant());
    }

    pub fn sync_shader_to_portal(&mut self, portal: &Gd<Portal>) {
        let mut material = self.get_shader_material();
        material.set_shader_parameter(
            "portal_center_pos",
            &portal.get_global_position().to_variant(),
        );
        material.set_shader_parameter(
            "portal_radii",
            &portal
                .bind()
                .area_node
                .as_ref()
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
                .connect_other(self, |s| s.base_mut().queue_free());
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
        // TODO: does it make sense to move this here?
        // self.sync_shader_to_portal();
    }
}
