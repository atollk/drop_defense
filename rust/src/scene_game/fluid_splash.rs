use crate::util::range_from_center;
use godot::classes::{ISprite2D, Node2D, Sprite2D, Texture2D};
use godot::global::deg_to_rad;
use godot::obj::{Base, Gd, WithBaseField};
use godot::prelude::*;
use itertools::Itertools;
use std::f32::consts::PI;
use tap::Tap;

#[derive(GodotClass)]
#[class(init, tool, base=Node2D)]
pub struct FluidSplash {
    base: Base<Node2D>,

    /// Random range amount of spawned particles. (min/max)
    #[export]
    pub particle_amount: Vector2i,

    #[export_group(name = "Drops")]

    /// The range (in degree) within which the drops are randomized.
    #[export]
    pub drop_angle_degree_range: u16,

    /// The range of initial velocity within which the drops are randomized. (min/max)
    #[export]
    pub drop_velocity_range: Vector2i,

    /// The range of scale within which the texture size is randomized. (min/max)
    #[export]
    pub drop_scale_range: Vector2,

    /// The texture that is displayed while flying.
    #[export]
    pub drop_texture: Option<Gd<Texture2D>>,

    /// The strength of gravity that is pulling on the drops.
    #[export]
    pub drop_gravity: f32,

    #[export_group(name = "Puddles")]

    /// The texture that is displayed after hitting the ground.
    #[export]
    pub puddle_texture: Option<Gd<Texture2D>>,

    /// The base scale of the puddle texture; this is multiplied with the randomized drop scale.
    #[export]
    pub puddle_base_scale: f32,

    /// The growth in scale over time of each puddle.
    #[export]
    pub puddle_scale_growth: f32,

    #[export_group(name = "")]
    #[export_tool_button(fn = Self::spawn_drops, name = "Spawn Drops", icon = "Add")]
    spawn_drops: PhantomVar<Callable>,

    // Internal fields
    drops: Vec<Gd<SplashDrop>>,
    puddles: Vec<Gd<SplashPuddle>>,
}

#[godot_api]
impl FluidSplash {
    #[signal]
    pub fn animation_ended();
}

impl FluidSplash {
    fn on_drop_hit_ground(&mut self, drop: Gd<SplashDrop>) {
        let i = self.drops.iter().find_position(|x| **x == drop).unwrap().0;
        self.drops.remove(i);
        self.spawn_puddle(drop);
    }

    fn on_puddle_vanish(&mut self, mut puddle: Gd<SplashPuddle>) {
        let i = self
            .puddles
            .iter()
            .find_position(|x| **x == puddle)
            .unwrap()
            .0;
        self.puddles.remove(i);
        puddle.queue_free();
        if self.puddles.is_empty() {
            self.signals().animation_ended().emit();
        }
    }

    pub fn spawn_drops(&mut self) {
        let particle_amount = rand::random_range(self.particle_amount.x..=self.particle_amount.y);
        for _ in 0..particle_amount {
            self.spawn_drop();
        }
    }

    fn spawn_drop(&mut self) {
        let mut drop = Gd::from_init_fn(SplashDrop::init);
        if let Some(texture) = &self.drop_texture {
            drop.set_texture(texture);
            drop.set_scale(
                Vector2::ONE
                    * rand::random_range(self.drop_scale_range.x..=self.drop_scale_range.y),
            );
        }
        drop.bind_mut().tap_mut(|drop| {
            drop.angle = rand::random_range(range_from_center(
                PI * 1.5,
                deg_to_rad(self.drop_angle_degree_range as f64) as f32,
            ));
            drop.velocity =
                rand::random_range(self.drop_velocity_range.x..=self.drop_velocity_range.y) as f32;
            drop.gravity = self.drop_gravity;
        });
        drop.signals()
            .hit_ground()
            .connect_other(self, |s, drop| s.on_drop_hit_ground(drop));
        self.base_mut().add_child(&drop);
        self.drops.push(drop);
    }

    fn spawn_puddle(&mut self, mut drop: Gd<SplashDrop>) {
        let mut puddle = Gd::from_init_fn(SplashPuddle::init);
        puddle.bind_mut().tap_mut(|puddle| {
            puddle.scale_growth = self.puddle_scale_growth;
        });
        puddle.set_texture(self.puddle_texture.as_ref());
        puddle.set_position(drop.get_position());
        puddle.set_scale(drop.get_scale() * self.puddle_base_scale);
        puddle
            .signals()
            .vanish()
            .connect_other(self, |s, puddle| s.on_puddle_vanish(puddle));
        drop.replace_by(&puddle);
        self.puddles.push(puddle);
    }
}

#[derive(GodotClass)]
#[class(tool, base=Sprite2D)]
struct SplashDrop {
    angle: f32,
    velocity: f32,
    gravity: f32,
    position: Vector2,
    base: Base<Sprite2D>,
}

#[godot_api]
impl SplashDrop {
    #[signal]
    fn hit_ground(drop: Gd<SplashDrop>);
}

#[godot_api]
impl ISprite2D for SplashDrop {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            angle: 0.0,
            velocity: 0.0,
            gravity: 0.0,
            position: Vector2::ZERO,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        // Update velocity & position
        let velocity_vector = {
            let mut velocity_vector = self.velocity * Vector2::RIGHT.rotated(self.angle);
            velocity_vector.y += delta as f32 * self.gravity;
            velocity_vector
        };
        self.angle = velocity_vector.angle();
        self.velocity = velocity_vector.length();
        self.position += velocity_vector * delta as f32;

        self.angle
            .tap(|angle| self.base_mut().set_rotation(*angle + PI * 1.5));
        self.position.tap(|pos| self.base_mut().set_position(*pos));

        // Check despawn
        if self.position.y > 10. {
            let s = self.base().clone().cast::<Self>();
            self.signals().hit_ground().emit(&s);
        }
    }
}

#[derive(GodotClass)]
#[class(tool, base=Sprite2D)]
struct SplashPuddle {
    scale_growth: f32,
    base: Base<Sprite2D>,
}

#[godot_api]
impl SplashPuddle {
    #[signal]
    fn vanish(drop: Gd<SplashPuddle>);
}

#[godot_api]
impl ISprite2D for SplashPuddle {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            scale_growth: 0.0,
            base,
        }
    }

    fn process(&mut self, delta: f64) {
        let scale_growth = Vector2::ONE * (delta as f32 * self.scale_growth);
        self.base_mut().tap_mut(|base| {
            let scale = base.get_scale() + scale_growth;
            base.set_scale(scale);

            let mut modulate = base.get_modulate();
            modulate.a -= delta as f32;
            base.set_modulate(modulate);
        });
    }
}
