mod scene_game;
mod enemy;
pub mod util;

use godot::prelude::*;

struct DropDefense;

#[gdextension]
unsafe impl ExtensionLibrary for DropDefense {}
