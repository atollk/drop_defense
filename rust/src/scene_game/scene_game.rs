use crate::enemy_weapon::EnemyWeapon;
use crate::node_getter;
use crate::scene_game::enemy::draggable_group::DraggableGroup;
use crate::scene_game::enemy::enemy::Enemy;
use crate::singletons::GameStats;
use godot::classes::{ColorRect, Path2D, Timer};
use godot::prelude::*;
use rand::prelude::IndexedRandom;

#[derive(GodotClass)]
#[class(init, base=Node2D)]
pub struct SceneGame {
    base: Base<Node2D>,
}

impl SceneGame {
    node_getter!(WalkPath: Path2D);
    node_getter!(Enemies: DraggableGroup);
    node_getter!(UI/LifeBarRed: ColorRect);
    node_getter!(UI/LifeBarGreen: ColorRect);
    node_getter!(SpawnTimer: Timer);

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
                s.on_enemy_swallowed_by_portal(&enemy_node, portal_index);
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

    fn on_enemy_swallowed_by_portal(&mut self, _enemy: &Gd<Enemy>, portal_index: u8) {
        // TODO: implement the logic for an enemy being swallowed
        godot_print!("Portal {}", portal_index);
    }

    /// Updates the health bar.
    fn update_health_bar(&mut self) {
        let max_size = self.get_node_life_bar_red().get_size();
        let game_stats_singleton = GameStats::singleton();
        let game_stats = game_stats_singleton.bind();
        let bar_size = Vector2::new(
            max_size.x * game_stats.health / game_stats.max_health as f32,
            max_size.y,
        );
        self.get_node_life_bar_green().set_size(bar_size);
    }

    /// Spawns a new enemy and resets the timer.
    fn on_spawn_timeout(&mut self) {
        let spawn_time = 50.0;
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
