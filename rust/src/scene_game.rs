use crate::enemy::{Enemy, EnemyState};
use crate::singletons::GameStats;
use crate::{node_getter, node_getter_named};
use godot::classes::{
    CollisionPolygon2D, ColorRect, InputEvent, InputEventMouseButton, Path2D, Sprite2D,
    StaticBody2D, Timer,
};
use godot::prelude::*;
use itertools::Itertools;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct SceneGame {
    base: Base<Node2D>,
}

impl SceneGame {
    node_getter!(Sprite2D, "Background");
    node_getter!(Path2D, "WalkPath");
    node_getter!(StaticBody2D, "GroundBody");
    node_getter!(Node2D, "Enemies");
    node_getter_named!(ColorRect, "UI/LifeBarRed", get_node_life_bar_red);
    node_getter_named!(ColorRect, "UI/LifeBarGreen", get_node_life_bar_green);
    node_getter!(Timer, "SpawnTimer");

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
        let enemy_scene = load::<PackedScene>("res://scenes/enemy.tscn");
        let mut enemy_node = enemy_scene.instantiate_as::<Enemy>();
        {
            let mut enemy = enemy_node.bind_mut();
            enemy.walk_path = Some(self.get_node_walk_path());
            enemy.health = 500;
        }
        self.get_node_enemies().add_child(&enemy_node);
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
        let spawn_time = 1.0;
        self.get_node_spawn_timer()
            .start_ex()
            .time_sec(spawn_time)
            .done();
        self.spawn_enemy();
    }

    /// Handles mouse button events (for grabbing enemies).
    fn on_mouse_button_event(&mut self, event: Gd<InputEventMouseButton>) {
        let enemies_by_state = self
            .get_node_enemies()
            .get_children()
            .iter_shared()
            .map(|enemy| enemy.cast::<Enemy>())
            .map(|enemy| {
                let state = enemy.bind().get_state();
                (state, enemy)
            })
            .into_group_map();
        if event.is_pressed() {
            if enemies_by_state.get(&EnemyState::Dragged).is_some() {
                panic!("We were dragging an enemy already and sent a mouse press during that.")
            }
            const DRAGGABLE_STATES: [EnemyState; 4] = [
                EnemyState::Walking,
                EnemyState::Attacking,
                EnemyState::StandingUp,
                EnemyState::Falling,
            ];
            let draggable_enemies = DRAGGABLE_STATES
                .iter()
                .filter_map(|state| enemies_by_state.get(state))
                .flatten();
            const DRAG_ENEMIES_AT_ONCE: usize = 1;
            let mut dragged_enemies = draggable_enemies
                .filter(|enemy| enemy.bind().has_mouse_is_over())
                .take(DRAG_ENEMIES_AT_ONCE)
                .cloned()
                .collect_vec();
            for enemy in dragged_enemies.iter_mut() {
                enemy.bind_mut().start_dragged();
            }
        } else if event.is_released() {
            if let Some(dragged_enemies) = enemies_by_state.get(&EnemyState::Dragged) {
                for mut enemy in dragged_enemies.iter().cloned() {
                    enemy.bind_mut().stop_dragged();
                }
            }
        }
    }
}

#[godot_api]
impl INode2D for SceneGame {
    fn init(base: Base<Self::Base>) -> Self {
        SceneGame { base }
    }

    fn ready(&mut self) {
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

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() {
            self.on_mouse_button_event(mouse_button_event);
        }
    }
}
