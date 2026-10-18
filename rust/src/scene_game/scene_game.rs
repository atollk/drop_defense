use crate::enemy_weapon::EnemyWeapon;
use crate::node_getter;
use crate::scene_game::enemy::draggable_group::DraggableGroup;
use crate::scene_game::enemy::enemy::Enemy;
use crate::singletons::GameStats;
use godot::classes::{CollisionPolygon2D, ColorRect, Path2D, Sprite2D, StaticBody2D, Timer};
use godot::prelude::*;
use rand::prelude::IndexedRandom;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct SceneGame {
    base: Base<Node2D>,
}

impl SceneGame {
    node_getter!(Background: Sprite2D);
    node_getter!(WalkPath: Path2D);
    node_getter!(Walls/GroundBody: StaticBody2D);
    node_getter!(Enemies: DraggableGroup);
    node_getter!(UI/LifeBarRed: ColorRect);
    node_getter!(UI/LifeBarGreen: ColorRect);
    node_getter!(SpawnTimer: Timer);

    /// Creates a collision object of the ground, based on the path.
    fn set_ground_from_path(&mut self) {
        // Get point collection for the ground.
        let ground_points = {
            let walk_path = self.get_node_walk_path();
            let mut ground_points = walk_path
                .get_curve()
                .expect("path needs to have a curve")
                .tessellate();
            for x in ground_points.as_mut_slice() {
                *x = walk_path.to_global(*x);
            }
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
            area.get_child(i).unwrap().queue_free();
        }
        area.add_child(&ground);
    }

    /// Spawns a new enemy
    fn spawn_enemy(&mut self) {
        let weapons = [
            load::<EnemyWeapon>("res://resources/weapons/none.tres"),
            load::<EnemyWeapon>("res://resources/weapons/sword.tres"),
        ];
        let enemy_scene = load::<PackedScene>("res://scenes/enemy.tscn");
        let mut enemy_node = enemy_scene.instantiate_as::<Enemy>();
        let on_enemy_death = {
            let enemy_node = enemy_node.clone();
            move |s: &mut Self| {
                s.get_node_enemies()
                    .bind_mut()
                    .remove_draggable(enemy_node.bind().get_node_draggable());
            }
        };
        let on_enemy_swallowed_by_portal = {
            let enemy_node = enemy_node.clone();
            move |s: &mut Self, portal_index: u8| {
                s.get_node_enemies()
                    .bind_mut()
                    .remove_draggable(enemy_node.bind().get_node_draggable());
                // TODO: implement the logic for an enemy being swallowed
                godot_print!("Portal {}", portal_index);
            }
        };
        {
            let mut enemy = enemy_node.bind_mut();
            enemy.walk_path = Some(self.get_node_walk_path());
            enemy.health = 500;
            enemy.set_weapon(weapons.choose(&mut rand::rng()).cloned());
            enemy.signals().died().connect_other(self, on_enemy_death);
            enemy
                .signals()
                .swallowed_by_portal()
                .connect_other(self, on_enemy_swallowed_by_portal);
        }
        self.get_node_enemies().add_child(&enemy_node);
        self.get_node_enemies()
            .bind_mut()
            .add_draggable(enemy_node.bind().get_node_draggable());
    }

    /// Updates the health bar.
    fn update_health_bar(&mut self) {
        let max_size = self.get_node_life_bar_red().get_size();
        let game_stats_singleton = GameStats::singleton();
        let game_stats = game_stats_singleton.bind();
        let bar_size = Vector2::new(
            max_size.x * game_stats.health as f32 / game_stats.max_health as f32,
            max_size.y,
        );
        self.get_node_life_bar_green().set_size(bar_size);
    }

    /// Spawns a new enemy and resets the timer.
    fn on_spawn_timeout(&mut self) {
        let spawn_time = 5.0;
        self.get_node_spawn_timer()
            .start_ex()
            .time_sec(spawn_time)
            .done();
        self.spawn_enemy();
    }
}

#[godot_api]
impl INode2D for SceneGame {
    fn ready(&mut self) {
        // TODO(debug): remove once the web portal placement issue is solved.
        {
            let db = godot::classes::ClassDb::singleton();
            for class in [
                "DropPortal",
                "PortalSwallowArea",
                "PortalKey",
                "Enemy",
                "SceneGame",
            ] {
                godot_print!(
                    "[portal-debug] ClassDb {class}: exists={} can_instantiate={} parent={}",
                    db.class_exists(class),
                    db.can_instantiate(class),
                    db.get_parent_class(class),
                );
            }
            godot_print!(
                "[portal-debug] Node2D inheritors containing Portal/Drop: {:?}",
                db.get_inheriters_from_class("Node2D")
                    .to_vec()
                    .into_iter()
                    .filter(|c| {
                        let c = c.to_string();
                        c.contains("Portal") || c.contains("Drop")
                    })
                    .collect::<Vec<_>>()
            );
        }
        for name in ["Left", "Center", "Right"] {
            let path = format!("Portals/{name}");
            match self.base().get_node_or_null(&path) {
                Some(node) => {
                    godot_print!(
                        "[portal-debug] {path}: class={} script={:?} children={:?}",
                        node.get_class(),
                        node.get_script(),
                        node.get_children()
                            .iter_shared()
                            .map(|c| c.get_class())
                            .collect::<Vec<_>>()
                    );
                }
                None => godot_print!("[portal-debug] {path}: MISSING"),
            }
        }
        self.set_ground_from_path();

        // Connect and start the timer.
        self.get_node_spawn_timer()
            .signals()
            .timeout()
            .connect_other(self, Self::on_spawn_timeout);
        self.on_spawn_timeout();
    }

    fn process(&mut self, _delta: f64) {
        self.update_health_bar();
    }
}
