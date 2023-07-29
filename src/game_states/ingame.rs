use bevy::asset::Assets;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_persistent::prelude::*;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::attack::{
    attack_phase_system, attack_spawner, charge_phase_system, recover_phase_system,
    SpawnEnemyAttack,
};
use crate::controller::combo_system;
use crate::game_states::ingame::InGameSet::{Normal, Post};
use crate::metadata::{GameMeta, PlayerMeta};
use crate::stats::{level_up, revenge_mode};
use crate::ui::{draw_hud, draw_revenge_bar, draw_xp_bar};
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
    stats::{death_system, drop_xp_system},
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
            .add_event::<SpawnEnemyAttack>() //TODO: Add attack plugin
            .add_systems(
                Update,
                (auto_save, statistics).run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnEnter(GameState::InGame), (setup_game, setup_map))
            //TODO: Check system ordering and optimize it
            .add_systems(
                Update,
                (
                    draw_hud,
                    draw_xp_bar,
                    draw_revenge_bar,
                    helper_camera_controller,
                    move_player,
                    dash_ability,
                    attack_ability,
                    attack_spawner,
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
                (restrict_movement, finish_dash, death_system)
                    .in_set(Post)
                    .after(Normal)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn setup_game(
    mut commands: Commands,
    game_meta: Res<GameMeta>,
    player_meta: Res<Assets<PlayerMeta>>,
) {
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

    let player = player_meta
        .get(&game_meta.player)
        .expect("Player Meta not found");
    let player_entity = commands.spawn(PlayerBundle::new(player)).id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;

    commands.spawn((camera_bundle, Follow::new(player_entity, 2.5, true, 2.)));
}
