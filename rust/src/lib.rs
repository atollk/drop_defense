mod enemy;
mod scene_game;
pub mod singletons;
pub mod util;

use crate::singletons::{register_singletons, unregister_singletons};
use godot::prelude::*;

struct DropDefense;

#[gdextension]
unsafe impl ExtensionLibrary for DropDefense {
    fn on_stage_init(stage: InitStage) {
        // Singleton should be registered before the MainLoop startup;
        // otherwise it won't be recognized by the GDScriptParser.
        if stage == InitStage::Scene {
            register_singletons();
        }
    }

    fn on_stage_deinit(stage: InitStage) {
        if stage == InitStage::Scene {
            unregister_singletons();
        }
    }
}
