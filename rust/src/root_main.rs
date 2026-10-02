use crate::scene_game::scene_game::SceneGame;
use crate::scene_main_menu::SceneMainMenu;
use godot::global::Error;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Node)]
struct RootMain {
    base: Base<Node>,
    #[export]
    pub active_scene_type: ActiveSceneType,
}

#[derive(GodotConvert, Var, Export, Clone, Default)]
#[godot(via = GString)]
enum ActiveSceneType {
    #[default]
    MainMenu,
    Game,
}

impl ActiveSceneType {
    fn get_packed_scene(&self) -> Gd<PackedScene> {
        let resource_path = match self {
            ActiveSceneType::MainMenu => "res://scenes/scene_main_menu.tscn",
            ActiveSceneType::Game => "res://scenes/scene_game.tscn",
        };
        load(resource_path)
    }
}

enum ActiveSceneInstance {
    MainMenu(Gd<SceneMainMenu>),
    #[expect(dead_code)]
    Game(Gd<SceneGame>),
}

impl ActiveSceneInstance {
    fn build(typ: &ActiveSceneType, node: Gd<Node>) -> ActiveSceneInstance {
        match typ {
            ActiveSceneType::MainMenu => Self::MainMenu(node.cast()),
            ActiveSceneType::Game => Self::Game(node.cast()),
        }
    }
}

const ACTIVE_SCENE_NODE_NAME: &str = "ActiveScene";

impl RootMain {
    fn try_get_node_active_scene(&self) -> Option<Gd<Node>> {
        self.base().try_get_node_as(ACTIVE_SCENE_NODE_NAME)
    }

    fn on_main_menu_start_button_pressed(&mut self) {
        let ActiveSceneInstance::MainMenu(_) = self.get_active_scene_instance() else {
            panic!("Main Menu button pressed but Main Menu is not active.")
        };
        self.enter_scene(ActiveSceneType::Game);
    }

    fn get_active_scene_instance(&self) -> ActiveSceneInstance {
        ActiveSceneInstance::build(
            &self.active_scene_type,
            self.try_get_node_active_scene().unwrap(),
        )
    }

    fn init_scene(&mut self, scene_instance: ActiveSceneInstance) {
        match scene_instance {
            ActiveSceneInstance::MainMenu(scene) => {
                scene
                    .signals()
                    .start_button_pressed()
                    .connect_other(self, |s| s.on_main_menu_start_button_pressed());
            }
            ActiveSceneInstance::Game(_) => {}
        }
    }

    fn enter_scene(&mut self, new_scene_type: ActiveSceneType) {
        let new_scene = {
            let mut new_scene = new_scene_type.get_packed_scene().instantiate().unwrap();
            new_scene.set_name(ACTIVE_SCENE_NODE_NAME);
            new_scene
        };
        if let Some(old_scene) = self.try_get_node_active_scene() {
            let err = old_scene.get_tree().change_scene_to_node(&new_scene);
            if err != Error::OK {
                panic!("Failed to enter scene with {:?}", err);
            }
        } else {
            self.base_mut().add_child(&new_scene);
        }
        self.init_scene(ActiveSceneInstance::build(&new_scene_type, new_scene));
        self.active_scene_type = new_scene_type;
    }
}

#[godot_api]
impl INode for RootMain {
    fn ready(&mut self) {
        self.enter_scene(self.active_scene_type.clone());
    }
}
