mod animation;
mod attack;
mod collision;
mod controller;
mod enemy;
mod helper;
mod level;
mod map;
mod movement;
mod player;
mod state;
mod stats;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_ecs_tilemap::prelude::*;

use animation::*;
use attack::*;
use collision::CollisionPlugin;
use controller::*;
use enemy::EnemyBehaviourPlugin;
use helper::*;
use leafwing_input_manager::prelude::InputManagerPlugin;
use level::LevelPlugin;
use map::{generation::*, walkable::restrict_movement};
use movement::movement::{Follow, MovementPlugin};
use player::{PlayerActions, PlayerBundle};
use stats::*;

pub const PLAYER_Z: f32 = 39.;
pub const MAP_Z: f32 = 36.;
pub const BACKGROUND_Z: f32 = 1.;
pub const DEBUG_Z: f32 = 100.;

#[derive(Component, Clone, Copy)]
pub struct XP(u32);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(20. / 255., 0. / 255., 25. / 255.)))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(KeyMaps::default())
        // .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(InputManagerPlugin::<PlayerActions>::default())
        // .add_plugin(TiledMapPlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(EnemyBehaviourPlugin)
        .add_plugin(LevelPlugin)
        .add_plugin(MovementPlugin)
        .add_system(set_texture_filters_to_nearest)
        .add_system(helper_camera_controller)
        // .add_system(sprite_animation)
        .add_system(move_player)
        .add_system(dash_ability)
        .add_system_to_stage(CoreStage::PostUpdate, finish_dash)
        .add_system(attack_ability)
        .add_system(attack_system)
        .add_system(death_system)
        .add_system(attack_cooldown_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_on_camera_move)
                .with_system(parallax_system),
        )
        .add_startup_system(setup_map)
        .add_startup_system(setup)
        .add_system(shake_system)
        .add_system(remake_map)
        .add_system(attack_lifetime)
        .add_system(projectile_break)
        .add_system_to_stage(CoreStage::PostUpdate, restrict_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let player_entity = commands
        .spawn_bundle(PlayerBundle::new(asset_server, texture_atlases))
        .id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.15;
    commands
        .spawn_bundle(camera_bundle)
        .insert(Follow::new(player_entity, 3., true, 5.));
}
