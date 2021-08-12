mod follow;
mod helper;
mod controller;
mod animation;
mod stats;

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use heron::prelude::*;

use follow::*;
use helper::*;
use controller::*;
use animation::*;
use stats::*;

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::rgb(0.005, 0.03, 0.08)))
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(helper_camera_controller.system())
        .add_system(sprite_animation.system())
        .add_system(player_controller.system())
        .add_system(follow_entity_system.system())
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands, mut texture_atlases: ResMut<Assets<TextureAtlas>>, asset_server: Res<AssetServer>) {
    //Map Creation
    let map_id = commands.spawn().id();
    let map_handle = asset_server.load("map.tmx");

    commands
        .entity(map_id)
        .insert_bundle(TiledMapBundle {
            tiled_map: map_handle,
            map: Map::new(0u16, map_id),
            transform: Transform::from_xyz(0.0, 40.0, 0.0), //TODO: Find a way to center the map
            ..Default::default()
        });

    //Player Creation
    let player_size = Vec2::new(16., 17.);

    //Load the textures
    let texture_handle = asset_server.load("IsometricTRPGAssetPack_Entities.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, player_size, 4, 33);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let player_entity =
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform: Transform::from_xyz(0., 0., 10.),
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
    camera_bundle.orthographic_projection.scale = 0.1;
    commands.spawn_bundle(camera_bundle).insert(FollowEntity { entity: player_entity, lerp_speed: 7. });
}

