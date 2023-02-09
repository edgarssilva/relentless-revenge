use bevy::asset::Assets;
use bevy::prelude::{App, Camera2dBundle, Commands, CoreStage, Plugin, Res};
use bevy_ecs_tilemap::TilemapPlugin;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::metadata::{GameMeta, PlayerMeta};
use crate::stats::despawn_dead_system;
use crate::ui::setup_ui;
use crate::{
    animation::AnimationPlugin,
    attack::{attack_lifetime, attack_system, projectile_break, tick_cooldown},
    collision::CollisionPlugin,
    controller::{attack_ability, dash_ability, finish_dash, move_player},
    enemy::EnemyBehaviourPlugin,
    helper::{helper_camera_controller, shake_system},
    level::LevelPlugin,
    map::{
        generation::{remake_map, setup_map},
        walkable::restrict_movement,
    },
    movement::movement::{Follow, MovementPlugin},
    player::{PlayerActions, PlayerBundle},
    stats::{death_system, drop_xp_system},
    GameState,
};

pub struct InGamePlugin;

impl Plugin for InGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .add_plugin(InputManagerPlugin::<PlayerActions>::default())
            .add_plugin(CollisionPlugin)
            .add_plugin(AnimationPlugin)
            .add_plugin(EnemyBehaviourPlugin)
            .add_plugin(LevelPlugin)
            .add_plugin(MovementPlugin)
            .add_enter_system(GameState::InGame, setup_game)
            .add_enter_system(GameState::InGame, setup_map)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .label("death_system")
                    .with_system(death_system) //Mark entities for despawn
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .after("death_system")
                    .with_system(setup_ui)
                    .with_system(helper_camera_controller)
                    .with_system(move_player)
                    .with_system(dash_ability)
                    .with_system(attack_ability)
                    .with_system(attack_system)
                    .with_system(drop_xp_system)
                    .with_system(tick_cooldown)
                    //run_on_camera_move  .with_system(parallax_system),
                    .with_system(shake_system)
                    .with_system(remake_map)
                    .with_system(attack_lifetime)
                    .with_system(projectile_break)
                    .into(),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(restrict_movement)
                    .with_system(finish_dash)
                    .with_system(despawn_dead_system)
                    .into(),
            );
    }
}

fn setup_game(
    mut commands: Commands,
    game_meta: Res<GameMeta>,
    player_meta: Res<Assets<PlayerMeta>>,
) {
    println!("InGamePlugin::setup_game");
    let player = player_meta
        .get(&game_meta.player)
        .expect("Player Meta not found");
    let player_entity = commands.spawn(PlayerBundle::new(player)).id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.15;

    commands.spawn((camera_bundle, Follow::new(player_entity, 3., true, 5.)));
}
