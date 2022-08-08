mod animation;
mod attack;
mod collision;
mod controller;
mod direction;
mod enemy;
mod follow;
mod helper;
mod map;
mod state;
mod stats;

use bevy::{prelude::*, render::texture::ImageSettings, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::prelude::Sensor;
use bevy_rapier2d::prelude::*;

use animation::*;
use attack::*;
use collision::{BodyLayers, CollisionPlugin};
use controller::*;
use direction::Direction;
use follow::*;
use helper::*;
use map::generation::*;
use state::State;
use stats::*;

pub const PLAYER_Z: f32 = 39.;
pub const MAP_Z: f32 = 36.;
pub const BACKGROUND_Z: f32 = 1.;
pub const DEBUG_Z: f32 = 100.;

#[derive(Component, Clone, Copy)]
pub struct XP(u32);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(48. / 255., 44. / 255., 46. / 255.)))
        .insert_resource(ImageSettings::default_nearest())
        .insert_resource(KeyMaps::default())
        // .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        // .add_plugin(TiledMapPlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(AnimationPlugin)
        .add_system(set_texture_filters_to_nearest)
        .add_system(helper_camera_controller)
        // .add_system(sprite_animation)
        .add_system(player_controller)
        .add_system(follow_entity_system)
        .add_system(attack_system)
        .add_system(death_system)
        .add_system(attack_cooldown_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_on_camera_move)
                .with_system(parallax_system),
        )
        .add_system(shake_system)
        .add_system(remake_map)
        .add_startup_system(setup_map)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //Player Creation
    let player_size = Vec2::new(64., 64.);

    //Load the textures
    let texture_handle = asset_server.load("tiny_hero.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, player_size, 8, 8);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let player_size = player_size / 3.75;

    let mut player_animations = HashMap::new();

    let mut idle_animations = HashMap::new();
    idle_animations.insert(Direction::SOUTH, (16..19).collect());
    idle_animations.insert(Direction::NORTH, (28..31).collect());
    idle_animations.insert(Direction::EAST, (20..23).collect());
    idle_animations.insert(Direction::WEST, (24..27).collect());

    let mut walk_animations = HashMap::new();
    walk_animations.insert(Direction::SOUTH, (48..51).collect());
    walk_animations.insert(Direction::NORTH, (60..63).collect());
    walk_animations.insert(Direction::EAST, (56..59).collect());
    walk_animations.insert(Direction::WEST, (52..55).collect());

    let mut attack_animations = HashMap::new();
    attack_animations.insert(Direction::SOUTH, (0..3).collect());
    attack_animations.insert(Direction::NORTH, (13..16).collect());
    attack_animations.insert(Direction::EAST, (8..12).collect());
    attack_animations.insert(Direction::WEST, (4..7).collect());

    player_animations.insert(State::IDLE, idle_animations);
    player_animations.insert(State::WALKING, walk_animations);
    player_animations.insert(State::ATTACKING, attack_animations);

    let player_entity = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform {
                translation: Vec3::new(0., 0., PLAYER_Z),
                scale: Vec3::new(1.25, 1.25, 1.),
                ..default()
            },
            ..default()
        })
        .insert(PlayerControlled)
        .insert(Direction::SOUTH)
        .insert(AnimationState::new(player_animations, 200, true))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(player_size.x / 2., player_size.y / 2.))
        .insert(CollisionGroups::new(
            BodyLayers::PLAYER,
            BodyLayers::XP_LAYER,
        ))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::all())
        // .insert(Timer::from_seconds(0.1, true))
        .insert(Stats::new(100, 20, 70, 3., 0))
        .insert(State::IDLE)
        .with_children(|children| {
            let offset = player_size.x * 0.75;
            let width = player_size.x;
            let height = player_size.y;

            //Add attack sensors
            for dir in Direction::values() {
                children
                    .spawn_bundle((
                        Transform::from_translation((dir.vec() * offset).extend(10.)),
                        GlobalTransform::default(),
                    ))
                    .insert(Sensor) //TODO: Uncomment this line to enable sensors
                    .insert(Collider::cuboid(width / 2., height / 2.))
                    .insert(CollisionGroups::new(
                        BodyLayers::PLAYER_ATTACK,
                        BodyLayers::ENEMY,
                    ))
                    .insert(MeleeSensor::from(dir))
                    .insert(ActiveEvents::COLLISION_EVENTS);
            }
        })
        .id();
    //Add Camera after so we can give it the player entity
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.15;
    commands.spawn_bundle(camera_bundle).insert(Follow::new(
        FollowTarget::Transform(player_entity),
        5.,
        true,
    ));
}
