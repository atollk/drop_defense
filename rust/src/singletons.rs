use godot::classes::Engine;
use godot::obj::{Gd, Singleton};
use godot::prelude::{GodotClass, Inherits, Object, UserSingleton};

#[derive(GodotClass, Debug)]
#[class(init, base = Object)]
pub struct GameStats {
    pub max_health: u32,
    pub health: f32,
}

impl Default for GameStats {
    fn default() -> Self {
        GameStats {
            max_health: 100,
            health: 100.,
        }
    }
}

impl UserSingleton for GameStats {}

macro_rules! singleton_list {
    ($($ty:ty),* $(,)?) => {
        const REGISTER_FNS: &[fn()] = &[$(register_singleton::<$ty>),*];
        const UNREGISTER_FNS: &[fn()] = &[$(unregister_singleton::<$ty>),*];
    };
}

singleton_list!(GameStats);

pub fn register_singletons() {
    for f in REGISTER_FNS {
        f();
    }
}

pub fn unregister_singletons() {
    for f in UNREGISTER_FNS {
        f();
    }
}

fn register_singleton<T: UserSingleton + Default + Inherits<Object>>() {
    let obj = Gd::from_object(T::default());
    Engine::singleton().register_singleton(&T::class_id().to_string_name(), &obj);
}

fn unregister_singleton<T: UserSingleton + Inherits<Object>>() {
    let obj = T::singleton();
    Engine::singleton().unregister_singleton(&T::class_id().to_string_name());
    obj.free();
}
