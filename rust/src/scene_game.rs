use crate::node_getter;
use godot::classes::{CollisionPolygon2D, Path2D, Sprite2D, StaticBody2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct SceneGame {
    base: Base<Node2D>,
}

impl SceneGame {
    node_getter!(Sprite2D, "Background");
    node_getter!(Path2D, "Path2D");
    node_getter!(StaticBody2D, "GroundBody");

    /// Creates a collision object of the ground, based on the path.
    fn set_ground_from_path(&mut self) {
        // Get point collection for the ground.
        let ground_points = {
            let mut ground_points = self
                .get_node_path2_d()
                .get_curve()
                .expect("path needs to have a curve")
                .tessellate();
            let background_sprite = self.get_node_background();
            let lower_right_corner =
                background_sprite.get_texture().unwrap().get_size() * background_sprite.get_scale();
            let lower_left_corner = Vector2::new(0., lower_right_corner.y);
            ground_points.push(lower_right_corner);
            ground_points.push(lower_left_corner);
            ground_points
        };

        // Create ground object.
        let mut ground = CollisionPolygon2D::new_alloc();
        ground.set_polygon(&ground_points);

        // Clear all old children from the area and set the new one.
        let mut area = self.get_node_ground_body();
        for i in 0..area.get_child_count() {
            let child = area.get_child(i).unwrap();
            area.remove_child(&child);
        }
        area.add_child(&ground);
    }
}

#[godot_api]
impl INode2D for SceneGame {
    fn init(base: Base<Self::Base>) -> Self {
        SceneGame { base }
    }

    fn ready(&mut self) {
        self.set_ground_from_path();
    }
}
