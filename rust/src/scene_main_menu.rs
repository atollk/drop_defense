use crate::node_getter_named;
use godot::classes::{Button, Control, IControl};
use godot::obj::{Base, WithBaseField, WithUserSignals};
use godot::prelude::{GodotClass, godot_api};

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct SceneMainMenu {
    base: Base<Control>,
}

#[godot_api]
impl SceneMainMenu {
    #[signal]
    pub fn start_button_pressed();
}

impl SceneMainMenu {
    node_getter_named!(Button, "VBoxContainer/StartButton", get_node_start_button);
    node_getter_named!(Button, "VBoxContainer/ExitButton", get_node_exit_button);

    fn on_start_button_press(&mut self) {
        self.signals().start_button_pressed().emit();
    }

    fn on_exit_button_press(&self) {
        self.base().get_tree().quit();
    }
}

#[godot_api]
impl IControl for SceneMainMenu {
    fn ready(&mut self) {
        self.get_node_start_button()
            .signals()
            .pressed()
            .connect_other(self, |s| s.on_start_button_press());
        self.get_node_exit_button()
            .signals()
            .pressed()
            .connect_other(self, |s| s.on_exit_button_press());
    }
}
