use crate::node_getter;
use crate::portal::portal_swallow_area::PortalSwallowArea;
use godot::classes::notify::CanvasItemNotification;
use godot::classes::tween::{EaseType, TransitionType};
use godot::classes::{Area2D, CircleShape2D, CollisionShape2D, ShaderMaterial, Sprite2D, Tween};
use godot::prelude::*;
use itertools::Itertools;
use std::collections::HashMap;
use std::time::Duration;

#[derive(GodotClass)]
#[class(tool, init, base=Node2D)]
pub struct Portal {
    #[export]
    pub transform_duration_ms: u32,

    #[export]
    pub portal_index: u8,

    #[export_tool_button(fn = Self::open, name = "Open")]
    open_fn: PhantomVar<Callable>,
    #[export_tool_button(fn = Self::close, name = "Close")]
    close_fn: PhantomVar<Callable>,

    shader_transforms: ShaderTransforms,
    transform_tween: Option<Gd<Tween>>,
    pub area_node: Option<Gd<Area2D>>,

    base: Base<Node2D>,
}

impl Portal {
    node_getter!(Sprite2D: Sprite2D);

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
            // Tween setup
            let mut tween = self.base_mut().create_tween();
            tween.set_parallel_ex().parallel(true).done();
            tween.set_ease(EaseType::IN_OUT);
            tween.set_trans(TransitionType::CUBIC);

            // Transition the shaders parameters
            for (param_name, param_value) in shader_param_values.iter() {
                tween.tween_property(
                    &sprite_material,
                    &format!("shader_parameter/{param_name}"),
                    param_value,
                    duration.as_secs_f64(),
                );
            }

            // Transition the area scale to match the visual size
            if let Some(param_value) = shader_param_values.get("radius") {
                let sprite_size = self.get_sprite_size();
                tween.tween_property(
                    self.area_node.as_ref().unwrap(),
                    "scale",
                    &Vector2::new(
                        param_value.to::<real>() * sprite_size.x,
                        param_value.to::<real>() * sprite_size.y,
                    )
                    .to_variant(),
                    duration.as_secs_f64(),
                );
            }

            tween
        };
        self.transform_tween = Some(tween);
    }

    pub fn get_sprite_size(&self) -> Vector2 {
        let texture_size = self.get_node_sprite2_d().get_texture().unwrap().get_size();
        let local_scale = self.get_node_sprite2_d().get_scale();
        Vector2::new(
            texture_size.x * local_scale.x,
            texture_size.y * local_scale.y,
        )
    }

    pub(crate) fn open(&mut self) {
        self.transform(1, Duration::from_millis(500));
    }

    pub(crate) fn close(&mut self) {
        self.transform(0, Duration::from_millis(500));
    }

    fn find_swallow_area(&self) -> Result<Gd<PortalSwallowArea>, String> {
        self.base()
            .get_children()
            .iter_shared()
            .filter_map(|child| child.try_cast::<PortalSwallowArea>().ok())
            .exactly_one()
            .map_err(|e| e.to_string())
    }

    fn connect_signals(&mut self) {
        let Some(mut swallow_area) = self.find_swallow_area().ok() else {
            return;
        };
        swallow_area.bind_mut().connect_signals(self);
    }

    /// Spawns a new Area2D node that covers the visible area of the portal.
    fn spawn_portal_area_node(&mut self) {
        let shape = {
            let mut shape = CircleShape2D::new_gd();
            shape.set_radius(1.0);
            shape
        };
        let shape_node = {
            let mut shape_node = CollisionShape2D::new_alloc();
            shape_node.set_name("CollisionShape2D");
            shape_node.set_shape(&shape);
            shape_node
        };
        let area_node = {
            let mut area_node = Area2D::new_alloc();
            area_node.set_name("PortalArea");
            area_node.add_child(&shape_node);
            area_node.set_collision_layer(0);
            area_node.set_collision_mask(0);
            area_node
        };
        self.base_mut().add_child(&area_node);
        self.area_node = Some(area_node);
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

impl Default for ShaderTransforms {
    fn default() -> Self {
        ShaderTransforms {
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
        }
    }
}

#[godot_api]
impl INode2D for Portal {
    fn ready(&mut self) {
        // TODO(debug): remove once the web portal placement issue is solved.
        godot_print!("[portal-debug] ready: enter");
        self.spawn_portal_area_node();
        godot_print!("[portal-debug] ready: area spawned");
        self.close();
        godot_print!("[portal-debug] ready: closed");
        self.connect_signals();
        godot_print!("[portal-debug] ready: signals connected");
        self.base_mut().update_configuration_warnings();
        godot_print!(
            "[portal-debug] {} pos={:?} global_pos={:?} scale={:?} sprite_size={:?} area_scale={:?}",
            self.base().get_path(),
            self.base().get_position(),
            self.base().get_global_position(),
            self.base().get_scale(),
            self.get_sprite_size(),
            self.area_node.as_ref().map(|a| a.get_scale()),
        );
    }

    fn on_notification(&mut self, what: CanvasItemNotification) {
        // https://godot-rust.github.io/book/register/signals.html#signals-in-the-editor--hot-reload-interaction
        if what == CanvasItemNotification::EXTENSION_RELOADED {
            self.connect_signals();
            self.base_mut().update_configuration_warnings();
        }
    }

    fn get_configuration_warnings(&self) -> PackedStringArray {
        let mut warnings = PackedStringArray::new();
        let area_child = self.find_swallow_area();
        if let Err(e) = area_child {
            warnings.push(&format!(
                "Portal needs to have exactly one PortalSwallowArea child for opening/closing proximity. {e}"
            ));
        }
        warnings
    }
}
