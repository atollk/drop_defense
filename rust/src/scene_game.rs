use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct SceneGame {}

impl SceneGame {}

#[godot_api]
impl INode2D for SceneGame {
    fn init(base: Base<Self::Base>) -> Self {
        SceneGame {}
    }
}