use godot::builtin::{Rect2, Vector2};
use godot::classes::{
    CollisionPolygon2D, CollisionShape2D, INode2D, Node2D, Path2D, StaticBody2D,
    WorldBoundaryShape2D,
};
use godot::obj::NewGd;
use godot::obj::{Base, NewAlloc, WithBaseField};
use godot::prelude::{Gd, GodotClass, godot_api};
use tap::Tap;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct GameWalls {
    #[export]
    pub walk_path: Option<Gd<Path2D>>,

    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for GameWalls {
    fn ready(&mut self) {
        let ground_body = create_ground_from_path(
            self.walk_path.as_ref().unwrap(),
            self.base().get_viewport_rect(),
        );

        let left_wall = {
            let shape = {
                let mut shape = WorldBoundaryShape2D::new_gd();
                shape.set_distance(0.0);
                shape.set_normal(Vector2::new(1.0, 0.0));
                shape
            };

            let collision_shape = {
                let mut collision_shape = CollisionShape2D::new_alloc();
                collision_shape.set_shape(&shape);
                collision_shape
            };

            let mut left_wall = StaticBody2D::new_alloc();
            left_wall.add_child(&collision_shape);
            left_wall.set_collision_layer(2);
            left_wall
        };

        let right_wall = {
            let shape = {
                let mut shape = WorldBoundaryShape2D::new_gd();
                shape.set_distance(-self.base().get_viewport_rect().end().x);
                shape.set_normal(Vector2::new(-1.0, 0.0));
                shape
            };

            let collision_shape = {
                let mut collision_shape = CollisionShape2D::new_alloc();
                collision_shape.set_shape(&shape);
                collision_shape
            };

            let mut right_wall = StaticBody2D::new_alloc();
            right_wall.add_child(&collision_shape);
            right_wall.set_collision_layer(2);
            right_wall
        };

        self.base_mut().tap_mut(|base| {
            base.add_child(&ground_body);
            base.add_child(&left_wall);
            base.add_child(&right_wall);
        });
    }
}

/// Creates a collision object of the ground, based on the path.
fn create_ground_from_path(walk_path: &Gd<Path2D>, viewport_rect: Rect2) -> Gd<StaticBody2D> {
    // Get point collection for the ground.
    let ground_points = {
        let mut ground_points = walk_path
            .get_curve()
            .expect("path needs to have a curve")
            .tessellate();
        for x in ground_points.as_mut_slice() {
            *x = walk_path.to_global(*x);
        }
        let lower_right_corner = viewport_rect.end();
        let lower_left_corner = Vector2::new(0., lower_right_corner.y);
        ground_points.push(lower_right_corner);
        ground_points.push(lower_left_corner);
        ground_points
    };

    let ground_shape = {
        let mut ground_shape = CollisionPolygon2D::new_alloc();
        ground_shape.set_polygon(&ground_points);
        ground_shape
    };

    let ground_body = {
        let mut ground_body = StaticBody2D::new_alloc();
        ground_body.add_child(&ground_shape);
        ground_body.set_collision_layer(2);
        ground_body
    };

    ground_body
}
