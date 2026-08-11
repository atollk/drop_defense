mod enemy;
mod scene_game;
pub mod util;

use godot::prelude::*;

struct DropDefense;

#[gdextension]
unsafe impl ExtensionLibrary for DropDefense {}
