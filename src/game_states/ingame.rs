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
use crate::ui::{draw_hud, draw_xp_bar};
use crate::{
    animation::AnimationPlugin,
    attack::{lifetimes, projectile_break, tick_cooldown},
    collision::CollisionPlugin,
    controller::{attack_ability, dash_ability, finish_dash, move_player},
    enemy::EnemyBehaviourPlugin,
    helper::{helper_camera_controller, shake_system},
    floor::FloorPlugin,
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
use crate::stats::level_up;

pub struct InGamePlugin;

//TODO: Refactor this to something meaningful
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum InGameSet {
    Normal,
    Post,
}

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .add_plugin(InputManagerPlugin::<PlayerActions>::default())
            .add_plugin(CollisionPlugin)
            .add_plugin(AnimationPlugin)
            .add_plugin(EnemyBehaviourPlugin)
            .add_plugin(FloorPlugin)
            .add_plugin(MovementPlugin)
            .add_event::<SpawnEnemyAttack>() //TODO: Add attack plugin
            .add_systems((auto_save, statistics).in_set(OnUpdate(GameState::InGame)))
            .add_system(setup_game.in_schedule(OnEnter(GameState::InGame)))
            .add_system(setup_map.in_schedule(OnEnter(GameState::InGame)))
            .configure_set(Normal.before(Post).in_set(OnUpdate(GameState::InGame)))
            .configure_set(Post.after(Normal).in_set(OnUpdate(GameState::InGame)))
            //TODO: Check system ordering and optimize it
            .add_systems(
                (
                    draw_hud,
                    draw_xp_bar,
                    helper_camera_controller,
                    move_player,
                    dash_ability,
                    attack_ability,
                    attack_spawner,
                    combo_system,
                    drop_xp_system,
                    level_up,
                    tick_cooldown,
                    shake_system,
                    remake_map,
                    lifetimes,
                    projectile_break,
                )
                    .in_set(Normal),
            )
            .add_systems(
                (
                    charge_phase_system,
                    attack_phase_system,
                    recover_phase_system,
                )
                    .in_set(Normal),
            )
            .add_systems((restrict_movement, finish_dash, death_system).in_set(Post));
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
            .build(),
    );

    let player = player_meta
        .get(&game_meta.player)
        .expect("Player Meta not found");
    let player_entity = commands.spawn(PlayerBundle::new(player)).id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.25;

    commands.spawn((camera_bundle, Follow::new(player_entity, 3., true, 5.)));
}
