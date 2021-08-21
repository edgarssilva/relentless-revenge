mod animation;
mod collision;
mod controller;
mod follow;
mod helper;
mod stats;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use heron::{prelude::*, SensorShape};

use animation::*;
use collision::*;
use controller::*;
use follow::*;
use helper::*;
use stats::*;

pub const PLAYER_Z: f32 = 39.;
pub const MAP_Z: f32 = 36.;
pub const BACKGROUND_Z: f32 = 1.;
pub const DEBUG_Z: f32 = 100.;

#[derive(PartialEq)]
pub enum Direction {
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

impl Direction {
    pub fn vec(&self) -> Vec2 {
        match self {
            &Self::NORTH => Vec2::Y,
            &Self::SOUTH => -Vec2::Y,
            &Self::WEST => -Vec2::X,
            &Self::EAST => Vec2::X,
        }
    }

    pub fn values() -> [Self; 4] {
        [Self::NORTH, Self::SOUTH, Self::WEST, Self::EAST]
    }
}

pub struct MeleeSensor {
    pub dir: Direction,
    pub targets: Vec<Entity>,
}

impl MeleeSensor {
    pub fn from(dir: Direction) -> Self {
        Self {
            dir,
            targets: Vec::new(),
        }
    }
}

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(48. / 255., 44. / 255., 46. / 255.)))
        .insert_resource(KeyMaps::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(helper_camera_controller.system())
        .add_system(sprite_animation.system())
        .add_system(player_controller.system())
        .add_system(follow_entity_system.system())
        .add_system(melee_collisions.system())
        .add_system(attack_system.system())
        .add_system(death_system.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_on_camera_move.system())
                .with_system(parallax_system.system()),
        )
        .add_system(shake_system.system())
        .add_startup_system(setup.system())
        .run();
}

fn attack_system(
    player_query: Query<&Stats, With<PlayerControlled>>,
    mut stats_query: Query<&mut Stats, Without<PlayerControlled>>,
    sensors_query: Query<&MeleeSensor>,
    keys: Res<Input<KeyCode>>,
    keymaps: Res<KeyMaps>,
) {
    if !keys.just_pressed(keymaps.attack) {
        return;
    }

    if let Ok(attacker_stats) = player_query.single() {
        for sensor in sensors_query
            .iter()
            //TODO: Change this to the players direction
            .filter(|sensor| sensor.dir == Direction::EAST)
        {
            for &attacked_entity in sensor.targets.iter() {
                if let Ok(mut attacked_stats) = stats_query.get_mut(attacked_entity) {
                    attacked_stats.health -= attacker_stats.damage;
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    //Map Creation
    let map_id = commands.spawn().id();
    let map_handle = asset_server.load("map.tmx");

    commands.entity(map_id).insert_bundle(TiledMapBundle {
        tiled_map: map_handle,
        map: Map::new(0u16, map_id),
        transform: Transform::from_xyz(0.0, 40.0, MAP_Z), //TODO: Find a way to center the map
        ..Default::default()
    });

    //Player Creation
    let player_size = Vec2::new(16., 17.);

    //Load the textures
    let texture_handle = asset_server.load("IsometricTRPGAssetPack_Entities.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, player_size, 4, 33);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    let player_size = player_size / 2.; //Player actual size is half of his sprite (whitespace)

    let player_entity = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 0., PLAYER_Z),
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(player_size.x / 2., player_size.y / 2., 0.),
            border_radius: None,
        })
        .insert(PlayerControlled)
        .insert(
            CollisionLayers::none()
                .with_group(Layers::Player)
                .with_mask(Layers::Enemy),
        )
        .insert(Timer::from_seconds(0.1, true))
        .insert(Stats::new(100, 20, 50))
        .with_children(|children| {
            let offset = player_size.x;
            let width = player_size.x * 1.25;
            let height = player_size.y * 1.25;

            //Add attack sensors
            for dir in Direction::values() {
                children
                    .spawn_bundle((
                        Transform::from_translation((dir.vec() * offset).extend(10.)),
                        GlobalTransform::default(),
                    ))
                    .insert(SensorShape)
                    .insert(CollisionShape::Cuboid {
                        half_extends: Vec3::new(width / 2., height / 2., 0.),
                        border_radius: None,
                    })
                    .insert(
                        CollisionLayers::none()
                            .with_group(Layers::Attack)
                            .with_mask(Layers::Enemy),
                    )
                    .insert(MeleeSensor::from(dir));
            }
        })
        .id();

    //Add enemy
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            material: materials.add(asset_server.load("old/char/iddle_l1.png").into()),
            transform: Transform {
                translation: Vec3::new(30., 5., PLAYER_Z),
                scale: Vec3::splat(0.4),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Stats::new(100, 20, 50))
        .insert(RigidBody::KinematicPositionBased)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(6.4, 8.8, 0.),
            border_radius: None,
        })
        .insert(
            CollisionLayers::none()
                .with_group(Layers::Enemy)
                .with_masks([Layers::Player, Layers::Attack]),
        );

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.15;
    commands
        .spawn_bundle(camera_bundle)
        .insert(Follow {
            target: FollowTarget::Transform(player_entity),
            speed: 5.,
        })
       /*  .insert(Shake {
            strength: 15.,
            duration: 10.,
        }) */;

    //Add parallax planet
    commands
        .spawn()
        .insert(Transform::from_xyz(-75., 30., 0.))
        .insert(GlobalTransform::default())
        .with_children(|parent| {
            parent
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: texture_atlases.add(TextureAtlas::from_grid(
                        asset_server.load("earth2.png"),
                        Vec2::splat(100.),
                        50,
                        50,
                    )),
                    transform: Transform {
                        translation: Vec3::new(0., 0., BACKGROUND_Z + 20.),
                        scale: Vec3::new(0.5, 0.5, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Timer::from_seconds(0.050, true))
                .insert(Parallax);
        });

    //Add space layers with parallax
    let names = vec![
        "background_4.png",
        "background_3.png",
        "background_2.png",
        "background_1.png",
    ];

    for i in 1..5 {
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.add(asset_server.load(names[i - 1]).into()),
                transform: Transform::from_xyz(0., 0., BACKGROUND_Z + (i * 10) as f32),
                ..Default::default()
            })
            .insert(Parallax);
    }
}
