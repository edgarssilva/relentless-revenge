mod animation;
mod controller;
mod follow;
mod helper;
mod stats;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use heron::prelude::*;

use animation::*;
use bevy::ecs::schedule::ShouldRun;
use bevy::render::camera::Camera;
use controller::*;
use follow::*;
use helper::*;
use stats::*;

struct Parallax;

const PLAYER_Z: f32 = 39.;
const MAP_Z: f32 = 36.;
const BACKGROUND_Z: f32 = 1.;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(48. / 255., 44. / 255., 46. / 255.)))
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(helper_camera_controller.system())
        .add_system(sprite_animation.system())
        .add_system(player_controller.system())
        .add_system(follow_entity_system.system())
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_on_camera_move.system())
                .with_system(parallax_system.system()),
        )
        .add_system(shake_system.system())
        .add_startup_system(setup.system())
        .run();
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

    let player_entity = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0., 0., PLAYER_Z),
            ..Default::default()
        })
        .insert(PlayerControlled)
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: player_size.extend(1.) / 2.,
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        // .insert(PhysicMaterial { friction: 0., restitution: 0., density: 1. })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(Timer::from_seconds(0.1, true))
        .insert(Stats::new(100, 20, 50))
        .id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.15;
    commands.spawn_bundle(camera_bundle)
        .insert(Follow {
            target: FollowTarget::Entity(player_entity),
            speed: 5.,
        })
        .insert(Shake {
            strength: 15.,
            duration: 10.
        });

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

fn run_on_camera_move(query: Query<(), (Changed<Transform>, With<Camera>)>) -> ShouldRun {
    if query.single().is_ok() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn parallax_system(
    cam_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<Parallax>, Without<Camera>)>,
) {
    if let Ok(cam_trans) = cam_query.single() {
        for mut trans in query.iter_mut() {
            trans.translation.x =
                -cam_trans.translation.x * (0.002 * (trans.translation.z - BACKGROUND_Z));
            trans.translation.y =
                -cam_trans.translation.y * (0.001 * (trans.translation.z - BACKGROUND_Z));
        }
    }
}