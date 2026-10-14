use godot::classes::{Resource, Texture2D};
use godot::obj::Base;
use godot::prelude::{Gd, GodotClass};

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct EnemyWeapon {
    #[export]
    pub base_damage: i32,
    #[export]
    pub texture: Option<Gd<Texture2D>>,
    base: Base<Resource>,
}
