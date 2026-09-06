use godot::classes::{INode2D, ISprite2D, Node2D, Sprite2D, Texture2D};
use godot::obj::{Base, Gd, NewAlloc, OnReady, Singleton, WithBaseField};
use godot::prelude::*;
use tap::Tap;

#[derive(GodotClass)]
#[class(init, tool, base=Node2D)]
struct FluidSplash {
    base: Base<Node2D>,

    #[export_group(name = "Drops")]
    #[export]
    particle_amount: u32,

    #[export]
    texture: Option<Gd<Texture2D>>,

    #[export_tool_button(fn = Self::spawn_drops, name = "Spawn Sprite", icon = "Add")]
    spawn_sprite: PhantomVar<Callable>,

    // Internal fields
    drops: Vec<Gd<SplashDrop>>,
}

impl FluidSplash {
    fn spawn_drops(&mut self) {
        for _ in 0..self.particle_amount {
            let mut drop = SplashDrop::new();
            if let Some(texture) = &self.texture {
                drop.set_texture(texture);
                drop.set_scale(Vector2::ONE * 0.1);
            }
            self.base_mut().add_child(&drop);
            self.drops.push(drop);
        }
    }
}

#[godot_api]
impl FluidSplash {}

#[godot_api]
impl INode2D for FluidSplash {
    fn ready(&mut self) {}
}

#[derive(GodotClass)]
#[class(init, tool, base=Sprite2D)]
struct SplashDrop {
    angle: f32,
    velocity: f32,
    position: Vector2,
    base: Base<Sprite2D>,
}

impl SplashDrop {
    fn new() -> Gd<Self> {
        let pi = std::f32::consts::PI;
        Gd::from_init_fn(|base| Self {
            angle: rand::random_range((pi * 1.4)..=(pi * 1.6)),
            velocity: rand::random_range(20.0..50.0),
            position: Vector2::ZERO,
            base,
        })
    }
}

#[godot_api]
impl ISprite2D for SplashDrop {
    fn process(&mut self, delta: f64) {
        // Update velocity & position
        let d = delta as f32 * 10.;
        let velocity_vector = {
            let mut velocity_vector = self.velocity * Vector2::RIGHT.rotated(self.angle);
            velocity_vector.y += d * 2.;
            velocity_vector
        };
        self.angle = velocity_vector.angle();
        self.velocity = velocity_vector.length();
        self.position += velocity_vector * d;

        self.angle.tap(|angle| {
            self.base_mut()
                .set_rotation(*angle + std::f32::consts::PI * 1.5)
        });
        self.position.tap(|pos| self.base_mut().set_position(*pos));

        // Check despawn
        if self.position.y > 100. {
            self.base_mut().queue_free();
        }
    }
}

/*

@export_group("Drops")
@export var drop_count: int = 14
## Launch speed in px/s.
@export var speed_range := Vector2(70.0, 160.0)
## Launch angle above the ground plane, in degrees. 90 = straight up.
@export var elevation_range := Vector2(45.0, 85.0)
@export var drop_radius_range := Vector2(1.5, 3.0)
@export var gravity: float = 600.0
## Squash factor for the ground plane. 1.0 = pure side view, ~0.5 = top-down
## with a tilted camera, 0.0 = strict top-down.
@export var ground_squash: float = 0.5

@export_group("Puddles")
@export var puddle_lifetime_range := Vector2(0.7, 1.6)
## Puddle radius as a multiple of the drop that made it.
@export var puddle_spread_range := Vector2(1.8, 3.2)
## Fraction of lifetime spent expanding to full size.
@export var puddle_grow_ratio: float = 0.18

@export_group("Look")
@export var color: Color = Color(0.45, 0.72, 0.95, 0.9)
@export var draw_shadows: bool = true
@export var shadow_color: Color = Color(0.0, 0.0, 0.0, 0.25)

@export_group("Behaviour")
@export var auto_start: bool = true
## Off if you're pooling these instead of instancing per splash.
@export var free_when_done: bool = true

 */
