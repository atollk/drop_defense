use crate::node_getter;
use crate::scene_game::enemy::enemy::Enemy;
use crate::scene_game::portal_key::PortalKey;
use godot::classes::node::InternalMode;
use godot::classes::notify::CanvasItemNotification;
use godot::classes::tween::{EaseType, TransitionType};
use godot::classes::{
    Area2D, CanvasItem, CircleShape2D, CollisionShape2D, ShaderMaterial, Sprite2D, Tween,
};
use godot::prelude::*;
use itertools::Itertools;
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

    fn find_swallow_area(&self) -> Result<Gd<Area2D>, String> {
        // The node from `spawn_portal_area_node` is an internal node and thus implicitly excluded here.
        self.base()
            .get_children()
            .iter_shared()
            .filter_map(|child| child.try_cast::<Area2D>().ok())
            .exactly_one()
            .map_err(|e| e.to_string())
    }

    fn connect_signals(&mut self) {
        let Some(swallow_area) = self.find_swallow_area().ok() else {
            return;
        };
        // Connections for the open/close animation.
        swallow_area
            .signals()
            .area_entered()
            .connect_other(self, |s, _| {
                s.open();
            });
        swallow_area
            .signals()
            .area_exited()
            .connect_other(self, |s, _| {
                s.close();
            });
        // Connections to the enemy.
        swallow_area
            .signals()
            .area_entered()
            .connect_self(|swallow_area, portal_key_area| {
                // TODO: it would be nicer if this was using an interface rather than hardcoded for Enemies
                let portal_key = portal_key_area.get_parent().unwrap().cast::<PortalKey>();
                let mut enemy = portal_key.bind().enemy_parent.clone().unwrap();
                enemy.bind_mut().entered_portal(swallow_area);
            });
        swallow_area
            .signals()
            .area_exited()
            .connect_other(self, |s, portal_key_area| {
                // TODO: it would be nicer if this was using an interface rather than hardcoded for Enemies
                let portal_key = portal_key_area.get_parent().unwrap().cast::<PortalKey>();
                let mut enemy = portal_key.bind().enemy_parent.clone().unwrap();
                enemy.bind_mut().exited_portal(s);
            });
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
            shape_node.set_scale(Vector2::ONE); // TODO
            shape_node
        };
        let area_node = {
            let mut area_node = Area2D::new_alloc();
            area_node.set_name("_PortalArea");
            area_node.add_child(&shape_node);
            area_node.set_collision_layer(0);
            area_node.set_collision_mask(0);
            area_node
        };
        self.base_mut()
            .add_child_ex(&area_node)
            .internal(InternalMode::FRONT)
            .done();
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
        self.connect_signals();
        self.base_mut().update_configuration_warnings();
        self.spawn_portal_area_node();
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
                "Portal needs to have exactly one Area2D child for opening/closing proximity. {e}"
            ));
        }
        warnings
    }
}
