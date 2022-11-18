use bevy::{
    prelude::{App, AssetServer, Assets, Camera2dBundle, Commands, CoreStage, Plugin, Res, ResMut},
    sprite::TextureAtlas,
};
use bevy_ecs_tilemap::TilemapPlugin;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{
    animation::AnimationPlugin,
    attack::{attack_lifetime, attack_system, projectile_break, tick_cooldown},
    collision::CollisionPlugin,
    controller::{attack_ability, dash_ability, finish_dash, move_player},
    enemy::EnemyBehaviourPlugin,
    helper::{helper_camera_controller, set_texture_filters_to_nearest, shake_system},
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
                    .with_system(set_texture_filters_to_nearest)
                    .with_system(helper_camera_controller)
                    .with_system(move_player)
                    .with_system(dash_ability)
                    .with_system(attack_ability)
                    .with_system(attack_system)
                    .with_system(death_system)
                    .with_system(drop_xp_system)
                    .with_system(tick_cooldown)
                    //run_on_camera_move  .with_system(parallax_system),
                    .with_system(shake_system)
                    .with_system(remake_map)
                    .with_system(attack_lifetime)
                    .with_system(projectile_break)
                    .into(),
            )
            .add_system_to_stage(CoreStage::PostUpdate, restrict_movement)
            .add_system_to_stage(CoreStage::PostUpdate, finish_dash);
    }
}

fn setup_game(
    mut commands: Commands,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
) {
    let player_entity = commands
        .spawn(PlayerBundle::new(asset_server, texture_atlases))
        .id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.15;

    commands.spawn((camera_bundle, Follow::new(player_entity, 3., true, 5.)));
}
