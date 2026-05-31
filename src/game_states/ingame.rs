use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_egui::EguiPrimaryContextPass;
use bevy_persistent::prelude::*;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::attack::{
    attack_phase_system, attack_spawner_observer, charge_phase_system, recover_phase_system,
};
use crate::controller::combo_system;
use crate::effects::spawn_shadows;
use crate::game_states::ingame::InGameSet::{Normal, Post};
use crate::manifest::player::PlayerManifest;
use crate::sorting::ysort;
use crate::stats::{level_up, revenge_mode};
use crate::ui::boss::{draw_boss_health, draw_domain_name};
use crate::ui::player::{draw_hud, draw_revenge_bar, draw_xp_bar};
use crate::{
    animation::AnimationPlugin,
    attack::{lifetimes, projectile_break, tick_cooldown},
    collision::CollisionPlugin,
    controller::{attack_ability, dash_ability, finish_dash, move_player},
    enemy::EnemyBehaviourPlugin,
    floor::FloorPlugin,
    helper::{helper_camera_controller, shake_system},
    map::{
        generation::{remake_map, setup_map},
        walkable::restrict_movement,
    },
    movement::movement::{Follow, MovementPlugin},
    player::{PlayerActions, PlayerBundle},
    statistics::{auto_save, statistics, Statistics},
    stats::{drop_xp_system, trigger_enemy_death},
    GameState,
};

pub struct InGamePlugin;

//TODO: Refactor this to something meaningful
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum InGameSet {
    Normal,
    Post,
}

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_plugins(InputManagerPlugin::<PlayerActions>::default())
            .add_plugins(CollisionPlugin)
            .add_plugins(AnimationPlugin)
            .add_plugins(EnemyBehaviourPlugin)
            .add_plugins(FloorPlugin)
            .add_plugins(MovementPlugin)
            .add_systems(
                Update,
                (auto_save, statistics).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::InGame), (setup_game, setup_map))
            //TODO: Check system ordering and optimize it
            .add_systems(
                EguiPrimaryContextPass,
                (
                    draw_hud,
                    draw_domain_name,
                    draw_xp_bar,
                    draw_revenge_bar,
                    draw_boss_health,
                )
                    .in_set(Normal)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_observer(attack_spawner_observer)
            .add_systems(
                Update,
                (
                    helper_camera_controller,
                    move_player,
                    dash_ability,
                    attack_ability,
                    combo_system,
                    tick_cooldown,
                    shake_system,
                    remake_map,
                    lifetimes,
                    projectile_break,
                    drop_xp_system,
                    level_up,
                    revenge_mode,
                    charge_phase_system,
                    attack_phase_system,
                    recover_phase_system,
                )
                    .in_set(Normal)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                (
                    spawn_shadows,
                    restrict_movement,
                    finish_dash,
                    trigger_enemy_death,
                    ysort,
                )
                    .in_set(Post)
                    .after(Normal)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_game(mut commands: Commands, player_manifest: Res<PlayerManifest>) {
    let dir = dirs::data_dir().unwrap().join("relentless_revenge");

    commands.insert_resource(
        Persistent::<Statistics>::builder()
            .name("statistics")
            .format(StorageFormat::Bincode)
            .path(dir.join("statistic.bin"))
            .default(Statistics::default())
            .build()
            .expect("Failed to create persistent statistics"),
    );

    let player_data = &player_manifest.player_data;
    let player_entity = commands.spawn(PlayerBundle::new(&player_data)).id();

    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: 0.25,
            ..OrthographicProjection::default_2d()
        }),
        Follow::new(player_entity, 2.5, true, 2.),
    ));
}
