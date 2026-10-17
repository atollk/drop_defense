use super::fluid_splash::FluidSplash;
use crate::enemy_weapon::EnemyWeapon;
use crate::node_getter_named;
use crate::portal::portal::Portal;
use crate::scene_game::enemy::draggable::Draggable;
use crate::singletons::GameStats;
use godot::classes::{
    AnimatedSprite2D, AnimationPlayer, Area2D, CollisionShape2D, IRigidBody2D, Path2D, RayCast2D,
    RigidBody2D, ShaderMaterial, Sprite2D,
};
use godot::prelude::*;
use itertools::Itertools;
use tap::Tap;

const FALLING_GRAVITY: f32 = 1.5;

#[derive(GodotClass)]
#[class(base=RigidBody2D)]
pub struct Enemy {
    /// A [Path2D] node that the enemy will continuously move along while not dragged.
    #[export]
    pub walk_path: Option<Gd<Path2D>>,

    /// The max fall damage this enemy can take before dying.
    #[export]
    pub health: u32,

    /// Movement speed while walking.
    #[export]
    pub speed: u32,

    /// The weapon used by this enemy.
    #[export()]
    #[var(get=get_weapon, set=set_weapon)]
    weapon: Option<Gd<EnemyWeapon>>,

    /// The progress (from 0 to 1) along [path].
    progress: f64,

    /// State of the enemy
    state: EnemyState,

    /// The Y coordinate at which falling started.
    fall_height: f32,

    /// The portal this enemy is currently in, if any.
    inside_portal: Option<Gd<Portal>>,

    base: Base<RigidBody2D>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum EnemyState {
    Walking,
    Dragged,
    Falling,
    Attacking,
    StandingUp,
    Dying,
}

impl EnemyState {
    const fn animation_name(&self) -> &'static str {
        match self {
            EnemyState::Walking => "Walk",
            EnemyState::Dragged => "Stand",
            EnemyState::Falling => "Stand",
            EnemyState::Attacking => "Attack",
            EnemyState::StandingUp => "StandUp",
            EnemyState::Dying => "Death",
        }
    }
}

#[godot_api]
impl Enemy {
    #[signal]
    pub fn died();

    #[signal]
    pub fn swallowed_by_portal(portal_index: u8);

    #[func]
    pub fn get_weapon(&self) -> Option<Gd<EnemyWeapon>> {
        self.weapon.clone()
    }

    #[func]
    pub fn set_weapon(&mut self, value: Option<Gd<EnemyWeapon>>) {
        let Some(weapon) = value else { return };
        self.get_node_weapon_sprite()
            .set_texture(weapon.bind().texture.as_ref());
        self.get_node_weapon_sprite()
            .set_offset(weapon.bind().texture_offset);
        self.weapon = Some(weapon);
    }
}

// General logic
impl Enemy {
    node_getter_named!(Node2D, "Scaling/GroundCast", get_node_ground_cast);
    node_getter_named!(CollisionShape2D, "BodyShape", get_node_body_shape);
    node_getter_named!(AnimatedSprite2D, "Scaling/BodySprite", get_node_body_sprite);
    node_getter_named!(
        AnimationPlayer,
        "Scaling/AnimationPlayer",
        get_node_animation_player
    );
    node_getter_named!(FluidSplash, "Scaling/BloodSplash", get_node_blood_splash);
    node_getter_named!(FluidSplash, "Scaling/LimbsSplash", get_node_limbs_splash);
    node_getter_named!(Draggable, "Scaling/Draggable", pub get_node_draggable);
    node_getter_named!(
        Sprite2D,
        "Scaling/WeaponHand/Weapon",
        get_node_weapon_sprite
    );

    /// RigidBody2D does not support settings its scale directly, so we have to use this workaround
    /// by setting the scale of every child individually.
    pub fn set_scale_recursively(&mut self, scale: Vector2) {
        let children: [_; 2] = self
            .base()
            .get_children()
            .iter_shared()
            .collect_array()
            .expect("expected enemy to have exactly two children");
        for child in children {
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
        if self.state != EnemyState::Walking
            && self.state != EnemyState::Attacking
            && self.state != EnemyState::StandingUp
            && self.state != EnemyState::Dying
        {
            return;
        }
        let Some(walk_path) = self.walk_path.as_mut() else {
            return;
        };
        if self.state == EnemyState::Walking {
            self.progress = (self.progress + delta).min(1.0);
        }
        let curve = walk_path
            .get_curve()
            .expect("the path must have a curve defined");
        let curve_len = curve.get_baked_length();
        let new_position = walk_path.to_global(
            curve
                .sample_baked_ex()
                .offset(self.progress as f32 * curve_len)
                .done(),
        );
        self.base_mut().set_global_position(new_position);

        if self.progress == 1.0 && self.state == EnemyState::Walking {
            self.set_state(EnemyState::Attacking);
        }
    }

    /// Connected to a signal whenever the sprite animation finishes.
    fn on_sprite_animation_finished(&mut self, _animation_name: StringName) {
        if self.state == EnemyState::Dying {
            self.get_node_body_sprite().set_visible(false);
        }
        if self.state == EnemyState::StandingUp {
            self.set_state(EnemyState::Walking);
        }
        self.get_node_animation_player().play();
    }

    pub fn get_state(&self) -> EnemyState {
        self.state
    }

    /// A setter for `self.state` that makes sure the other invariants are updated correctly.
    fn set_state(&mut self, new_state: EnemyState) {
        self.state = new_state;

        // Draggable
        {
            let mut draggable = self.get_node_draggable();
            let mut draggable = draggable.bind_mut();
            draggable.can_be_dragged = matches!(
                self.state,
                EnemyState::Walking
                    | EnemyState::Attacking
                    | EnemyState::StandingUp
                    | EnemyState::Falling
            );
        }

        // Gravity
        let gravity_scale = if matches!(self.state, EnemyState::Falling) {
            FALLING_GRAVITY
        } else {
            0.
        };
        self.base_mut().set_gravity_scale(gravity_scale);

        // Physics
        let freeze_enabled = self.state == EnemyState::Walking;
        self.base_mut().set_freeze_enabled(freeze_enabled);

        // Animation
        self.get_node_animation_player()
            .play_ex()
            .name(self.state.animation_name())
            .done();
        if self.state == EnemyState::Dying {
            self.signals().died().emit();
            self.get_node_blood_splash().bind_mut().spawn_drops();
            self.get_node_limbs_splash().bind_mut().spawn_drops();
            self.get_node_blood_splash()
                .signals()
                .animation_ended()
                .connect_other(self, |s| s.base_mut().queue_free());
        }
    }

    fn is_falling_down(&self) -> bool {
        matches!(self.state, EnemyState::Falling) && self.base().get_linear_velocity().y > -1.0
    }

    /// Checks whether a falling enemy has hit the ground.
    fn check_fall_ground(&mut self) {
        if !self.is_falling_down() {
            return;
        }
        let Some(walk_path) = &self.walk_path else {
            return;
        };

        // Check if any of the "foot" rays detects ground.
        let rays: [_; 3] = self
            .get_node_ground_cast()
            .get_children()
            .iter_shared()
            .map(|node| node.cast::<RayCast2D>())
            .collect_array()
            .unwrap();
        let hit_ground = rays.iter().any(|ray| ray.is_colliding());
        if !hit_ground {
            return;
        }

        // The enemy has fallen to the ground.
        let curve = walk_path
            .get_curve()
            .expect("the path must have a curve defined");
        let offset =
            curve.get_closest_offset(walk_path.to_local(self.base().get_global_position()));
        self.progress = (offset / curve.get_baked_length()) as f64;

        let fall_damage = (self.base().get_global_position().y - self.fall_height) as f64;
        if fall_damage > self.health as f64 {
            self.set_state(EnemyState::Dying);
        } else if fall_damage * 5. > self.health as f64 {
            self.set_state(EnemyState::StandingUp);
        } else {
            self.set_state(EnemyState::Walking);
        }
    }

    pub fn get_shader_material(&self) -> Gd<ShaderMaterial> {
        self.base()
            .find_child("Scaling")
            .unwrap()
            .cast::<Node2D>()
            .get_material()
            .unwrap()
            .cast()
    }

    fn sync_shader_to_portal(&mut self) {
        let Some(portal) = self.inside_portal.clone() else {
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
                .area_node
                .as_ref()
                .unwrap()
                .get_global_scale()
                .to_variant(),
        );
    }

    pub fn entered_portal(&mut self, portal: &Portal) {
        self.inside_portal = Some(portal.to_gd());
        let mut material = self.get_shader_material();
        material.set_shader_parameter("falling_into_portal", &true.to_variant());
    }

    pub fn exited_portal(&mut self, portal: &Portal) {
        if self.is_falling_down() {
            self.signals()
                .swallowed_by_portal()
                .emit(portal.portal_index);
            self.base_mut().queue_free();
        } else {
            let mut material = self.get_shader_material();
            material.set_shader_parameter("falling_into_portal", &false.to_variant());
            self.inside_portal = None;
        }
    }
}

#[godot_api]
impl IRigidBody2D for Enemy {
    fn init(base: Base<Self::Base>) -> Self {
        Enemy {
            walk_path: None,
            health: 0,
            speed: 10,
            weapon: None,
            progress: 0.0,
            state: EnemyState::Dying, // has to be overwritten on ready
            fall_height: f32::INFINITY,
            inside_portal: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.get_node_draggable().tap_mut(|draggable| {
            draggable
                .signals()
                .drag_started()
                .connect_other(self, |s| s.set_state(EnemyState::Dragged));
            draggable
                .signals()
                .drag_ended()
                .connect_other(self, |s| s.set_state(EnemyState::Falling));
            draggable
                .bind_mut()
                .set_shape(Some(self.get_node_body_shape().duplicate_node()));
        });

        self.get_node_animation_player()
            .signals()
            .animation_finished()
            .connect_other(self, Self::on_sprite_animation_finished);
        self.get_node_body_sprite().stop();
        self.set_state(EnemyState::Walking);

        if self.base().get_scale() != Vector2::ONE {
            godot_warn!(
                "The scale of this Enemy is not 1. That's an issue because the physics will probably break. It should be set via Enemy.set_scale_recursively."
            )
        }
        self.set_scale_recursively(Vector2::new(0.5, 0.5));
    }

    fn process(&mut self, delta: f64) {
        self.update_path_progress(delta / 100.0 * self.speed as f64);
        self.check_fall_ground();
        self.sync_shader_to_portal();

        if self.state == EnemyState::Attacking {
            let mut game_stats_singleton = GameStats::singleton();
            let mut game_stats = game_stats_singleton.bind_mut();
            let damage = delta / 100.0 * self.weapon.as_ref().unwrap().bind().base_damage as f64;
            game_stats.health = (game_stats.health - damage as f32).max(0.0);
        }
    }

    fn physics_process(&mut self, delta: f64) {
        self.get_node_draggable()
            .bind_mut()
            .drag_towards_cursor(&mut *self.base_mut(), delta);

        // Update fall height
        if matches!(self.state, EnemyState::Falling) {
            self.fall_height = self.fall_height.min(self.base().get_global_position().y);
        } else {
            self.fall_height = f32::INFINITY;
        }
    }
}
