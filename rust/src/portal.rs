use crate::{node_getter, node_getter_named};
use godot::classes::tween::{EaseType, TransitionType};
use godot::classes::{Area2D, CollisionShape2D, ShaderMaterial, Sprite2D, Tween};
use godot::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

#[derive(GodotClass)]
#[class(tool, base=Node2D)]
pub struct Portal {
    #[export]
    pub transform_duration_ms: u32,

    #[export_tool_button(fn = Self::open, name = "Open")]
    open_fn: PhantomVar<Callable>,
    #[export_tool_button(fn = Self::close, name = "Close")]
    close_fn: PhantomVar<Callable>,

    shader_transforms: ShaderTransforms,
    transform_tween: Option<Gd<Tween>>,

    base: Base<Node2D>,
}

impl Portal {
    node_getter!(Sprite2D, "Sprite2D");
    node_getter!(Area2D, "Area2D");

    fn transform(&mut self, target_state: usize, duration: Duration) {
        let sprite_material = self
            .get_node_sprite2_d()
            .get_material()
            .unwrap()
            .cast::<ShaderMaterial>();

        let shader_param_values = {
            let mut shader_param_values: HashMap<&str, Variant> = HashMap::new();
            macro_rules! add_shader_param {
                ($field:ident) => {
                    if let Some(val) = self.shader_transforms.$field {
                        shader_param_values
                            .insert(stringify!($field), val[target_state].to_variant());
                    }
                };
            }
            add_shader_param!(inner_color);
            add_shader_param!(rim_color);
            add_shader_param!(radius);
            add_shader_param!(rim_width);
            add_shader_param!(softness);
            add_shader_param!(breathing_speed);
            add_shader_param!(breathing_strength);
            add_shader_param!(noise_angle);
            add_shader_param!(noise_speed);
            add_shader_param!(noise_strength);
            shader_param_values
        };

        let tween = {
            let mut tween = self.base_mut().create_tween();
            tween.set_parallel_ex().parallel(true).done();
            tween.set_ease(EaseType::IN_OUT);
            tween.set_trans(TransitionType::CUBIC);
            for (param_name, param_value) in shader_param_values.iter() {
                tween.tween_property(
                    &sprite_material,
                    &format!("shader_parameter/{param_name}"),
                    param_value,
                    duration.as_secs_f64(),
                );
            }
            tween
        };
        self.transform_tween = Some(tween);
    }

    fn open(&mut self) {
        self.transform(1, Duration::from_millis(500));
    }

    fn close(&mut self) {
        self.transform(0, Duration::from_millis(500));
    }
}

type ShaderTransformValue<T> = Option<[T; 2]>;

struct ShaderTransforms {
    // Circle
    inner_color: ShaderTransformValue<Vector3>,
    rim_color: ShaderTransformValue<Vector3>,
    radius: ShaderTransformValue<f32>,
    rim_width: ShaderTransformValue<f32>,
    softness: ShaderTransformValue<f32>,

    // Breathing
    breathing_speed: ShaderTransformValue<f32>,
    breathing_strength: ShaderTransformValue<f32>,

    // Noise
    noise_angle: ShaderTransformValue<f32>,
    noise_speed: ShaderTransformValue<f32>,
    noise_strength: ShaderTransformValue<f32>,
}

#[godot_api]
impl INode2D for Portal {
    fn init(base: Base<Self::Base>) -> Self {
        Portal {
            transform_duration_ms: 0,
            open_fn: Default::default(),
            close_fn: Default::default(),
            shader_transforms: ShaderTransforms {
                inner_color: None,
                rim_color: None,
                radius: Some([0.1, 0.365]),
                rim_width: Some([0.02, 0.065]),
                softness: None,
                breathing_speed: None,
                breathing_strength: Some([0., 0.018]),
                noise_angle: None,
                noise_speed: None,
                noise_strength: None,
            },
            transform_tween: None,
            base,
        }
    }

    fn ready(&mut self) {
        self.close();
        self.get_node_area2_d()
            .signals()
            .mouse_entered()
            .connect_other(self, |s| s.open());
        self.get_node_area2_d()
            .signals()
            .mouse_exited()
            .connect_other(self, |s| s.close());
    }
}
