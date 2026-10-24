use crate::enemy_weapon::EnemyWeapon;
use crate::portal::portal::{Portal, PortalState};
use crate::scene_game::enemy::draggable::Draggable;
use crate::scene_game::enemy::enemy_sprite::EnemySprite;
use crate::scene_game::enemy::enemy_state::EnemyState;
use crate::scene_game::portal_key::{PortalEntrant, PortalKey};
use crate::singletons::GameStats;
use crate::{node_getter, node_getter_named};
use godot::classes::{CollisionShape2D, IRigidBody2D, Path2D, RayCast2D, RigidBody2D};
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
    #[var(get, set)]
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
        self.get_node_enemy_sprite().bind_mut().set_weapon(&weapon);
        self.weapon = Some(weapon);
    }
}

// General logic
impl Enemy {
    node_getter!(Scaling/GroundCast: Node2D);
    node_getter!(BodyShape: CollisionShape2D);
    node_getter!(Scaling/EnemySprite: EnemySprite);
    node_getter!(Scaling/Feet: PortalKey);
    node_getter_named!(Draggable, "Scaling/Draggable", pub get_node_draggable);

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
        if !self.state.is_moving_along_path() {
            return;
        }
        let Some(walk_path) = self.walk_path.as_mut() else {
            return;
        };
        if matches!(self.state, EnemyState::Walking) {
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
        if self.state == EnemyState::StandingUp {
            self.set_state(EnemyState::Walking);
        }
    }

    pub fn get_state(&self) -> EnemyState {
        self.state
    }

    /// A setter for `self.state` that makes sure the other invariants are updated correctly.
    fn set_state(&mut self, new_state: EnemyState) {
        self.state = new_state;

        // Falling into portal ?
        let portal_is_fully_open = self
            .inside_portal
            .as_ref()
            .map(|portal| portal.bind().get_state() == PortalState::Stable { is_open: true })
            .unwrap_or(false);
        if matches!(self.state, EnemyState::Falling) && portal_is_fully_open {
            self.set_state(EnemyState::FallingIntoPortal);
            return;
        }

        // Draggable
        self.get_node_draggable().bind_mut().can_be_dragged = self.state.is_draggable();

        // Feet
        let feet_enabled = self.state.is_portal_key();
        self.get_node_feet().bind_mut().set_enabled(feet_enabled);

        // Gravity
        // TODO: maybe lower this if swallowed to give the portal a "viscosity"
        let gravity_scale = if self.state.is_falling() {
            FALLING_GRAVITY
        } else {
            0.
        };
        self.base_mut().set_gravity_scale(gravity_scale);

        // Physics
        self.state
            .is_physics_frozen()
            .tap(|&frozen| self.base_mut().set_freeze_enabled(frozen));

        // Animation
        self.get_node_enemy_sprite()
            .bind_mut()
            .play_state_animation(self.state);

        if self.state == EnemyState::Dying {
            self.signals().died().emit();
        }
    }

    fn is_falling_down(&self) -> bool {
        self.state.is_falling() && self.base().get_linear_velocity().y > -1.0
    }

    /// Checks whether a falling enemy has hit the ground.
    fn fell_on_the_ground(&self) -> bool {
        if !self.is_falling_down() {
            return false;
        }
        // Check if any of the "foot" rays detects ground.
        let rays: [_; 3] = self
            .get_node_ground_cast()
            .get_children()
            .iter_shared()
            .map(|node| node.cast::<RayCast2D>())
            .collect_array()
            .unwrap();
        rays.iter().any(|ray| ray.is_colliding())
    }

    fn on_ground_fall(&mut self) {
        let walk_path = self.walk_path.as_ref().unwrap();
        let curve = walk_path
            .get_curve()
            .expect("the path must have a curve defined");
        let offset =
            curve.get_closest_offset(walk_path.to_local(self.base().get_global_position()));
        self.progress = (offset / curve.get_baked_length()) as f64;

        let fall_damage = (self.base().get_global_position().y - self.fall_height) as f64;
        if matches!(self.state, EnemyState::SwallowedByPortal) {
            self.base_mut().queue_free();
        } else if fall_damage > self.health as f64 {
            self.set_state(EnemyState::Dying);
        } else if fall_damage * 5. > self.health as f64 {
            self.set_state(EnemyState::StandingUp);
        } else {
            self.set_state(EnemyState::Walking);
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

        self.get_node_enemy_sprite()
            .signals()
            .animation_ended()
            .connect_other(self, Self::on_sprite_animation_finished);
        self.get_node_enemy_sprite()
            .signals()
            .death_animation_ended()
            .connect_other(self, |s| s.base_mut().queue_free());

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
        if self.fell_on_the_ground() {
            self.on_ground_fall();
        }

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

#[godot_dyn]
impl PortalEntrant for Enemy {
    fn entered_portal(&mut self, portal: &Portal) {
        self.inside_portal = Some(portal.to_gd());
        self.get_node_enemy_sprite()
            .bind_mut()
            .set_inside_portal(self.inside_portal.clone());
    }

    fn exited_portal(&mut self, portal: &Portal) {
        if matches!(self.state, EnemyState::SwallowedByPortal) {
            // if-check to prevent some annoying workaround logic when the signal gets emitted twice.
            return;
        }

        let swallowed_by_portal = matches!(self.get_state(), EnemyState::FallingIntoPortal);

        if swallowed_by_portal {
            self.signals()
                .swallowed_by_portal()
                .emit(portal.portal_index);
            self.set_state(EnemyState::SwallowedByPortal);
        } else {
            self.inside_portal = None;
            self.get_node_enemy_sprite()
                .bind_mut()
                .set_inside_portal(self.inside_portal.clone());
        }
    }
}
