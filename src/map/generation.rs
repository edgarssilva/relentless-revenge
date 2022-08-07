use std::time::Duration;

use crate::animation::Animation;
use crate::collision::BodyLayers;
use crate::controller::PlayerControlled;
use crate::follow::{Follow, FollowTarget};
use crate::map::room::Room;
use crate::stats::Stats;
use bevy::input::Input;
use bevy::math::Vec2;
use bevy::prelude::{
    default, AssetServer, Assets, Commands, Entity, KeyCode, Mut, Query, Res, ResMut, Timer,
    Transform, UVec2, Vec3, With,
};
use bevy::sprite::{SpriteSheetBundle, TextureAtlas};

use bevy_ecs_tilemap::prelude::{
    IsoType, TilemapGridSize, TilemapId, TilemapMeshType, TilemapSize, TilemapTexture,
    TilemapTileSize,
};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTexture};
use bevy_ecs_tilemap::TilemapBundle;
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use rand;
use rand::prelude::*;

use super::bridge::Bridge;

pub fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("tileset.png");

    let tilemap_size = TilemapSize { x: 160, y: 160 };
    let mut tile_storage = TileStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    let tile_size = TilemapTileSize { x: 32.0, y: 16.0 };

    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: TileTexture(3),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: TilemapGridSize { x: 32.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture(texture_handle),
            tile_size,
            mesh_type: TilemapMeshType::Isometric(IsoType::Diamond),
            ..Default::default()
        });
}

pub fn remake_map(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Transform, Entity), With<PlayerControlled>>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut tile_query: Query<&mut TileTexture>,
    tile_storage_query: Query<&TileStorage>,
) {
    if keys.just_released(KeyCode::LControl) {
        let (transform, entity) = player_query.single_mut();

        //Change all tiles to clear texture
        for mut tile in tile_query.iter_mut() {
            tile.0 = 3;
        }

        if let Ok(tile_storage) = tile_storage_query.get_single() {
            build_map(
                &mut commands,
                transform,
                entity,
                texture_atlases,
                asset_server,
                tile_query,
                tile_storage,
            );
        }
    }
}

fn build_map(
    commands: &mut Commands,
    mut player_transform: Mut<Transform>,
    player_entity: Entity,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut tile_query: Query<&mut TileTexture>,
    tile_storage: &TileStorage,
) {
    let mut rng = rand::thread_rng();
    //Load the textures
    let texture_handle = asset_server.load("monster_flesh_eye_sheet.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::splat(256.), 3, 3);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut first_room = true;

    let (rooms, bridges) = generate_level();
    for room in rooms {
        for x in room.pos.x - room.radius..room.pos.x + room.radius {
            for y in room.pos.y - room.radius..room.pos.y + room.radius + 1 {
                //Cut corners
                if UVec2::new(x, y).as_vec2().distance(room.pos.as_vec2()) > room.radius as f32 {
                    continue;
                }

                //TODO: Build room using neighbors
                if let Some(tile_entity) = tile_storage.get(&TilePos { x, y }) {
                    if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                        tile_texture.0 = 2;
                    }
                }

                //TODO: Build a function to move map cords to world cords
                let world_x = (x as f32 - y as f32) * 32. / 2.0;
                let world_y = (x as f32 + y as f32) * 16. / 2.0;
                let world_pos = Vec2::new(world_x, -world_y);

                //TODO: Move enemy spawns to a separate system
                if rng.gen_bool(1. / 40.) {
                    commands
                        .spawn_bundle(SpriteSheetBundle {
                            texture_atlas: texture_atlas_handle.clone(),
                            transform: Transform {
                                translation: world_pos.extend(1.),
                                scale: Vec3::new(0.2, 0.2, 1.),
                                ..default()
                            },
                            ..default()
                        })
                        .insert(Stats::new(100, 20, 20, 2., 5))
                        .insert(Animation {
                            frames: (0..7).collect(),
                            current_frame: 0,
                            timer: Timer::new(Duration::from_millis(250), true),
                        })
                        .insert(RigidBody::KinematicPositionBased)
                        .insert(Collider::cuboid(256. * 0.2, 256. * 0.2))
                        .insert(CollisionGroups::new(
                            BodyLayers::ENEMY,
                            BodyLayers::PLAYER_ATTACK,
                        ))
                        .insert(ActiveEvents::COLLISION_EVENTS)
                        .insert(ActiveCollisionTypes::all())
                        .insert(Follow {
                            target: FollowTarget::Transform(player_entity),
                            speed: 0.05,
                            continous: true,
                            ..default()
                        });
                }

                if first_room && room.pos.to_array() == [x, y] {
                    player_transform.translation.x = world_pos.x;
                    player_transform.translation.y = world_pos.y;
                }
            }
        }
        first_room = false;
    }

    for bridge in bridges {
        for pos in bridge.pos {
            let x = pos.x;
            let y = pos.y;

            if let Some(tile_entity) = tile_storage.get(&TilePos { x, y }) {
                if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                    tile_texture.0 = 2;
                }
            }
        }
    }
}

fn generate_level() -> (Vec<Room>, Vec<Bridge>) {
    let mut rng = rand::thread_rng();

    let mut rooms = Vec::<Room>::new();
    let mut bridges = Vec::<Bridge>::new();

    let num_rooms = rng.gen_range(6..10);
    let room_min_radius = 4;
    let room_max_radius = 6;

    let mut old_room = Room::new(
        UVec2::new(80, 80),
        rng.gen_range(room_min_radius..room_max_radius),
    );

    let main_direction: i32 = rng.gen_range(0..360);
    let angle_range = 130;

    while rooms.len() < num_rooms {
        let radius = rng.gen_range(room_min_radius..room_max_radius);
        let direction = (main_direction + rng.gen_range(-angle_range..angle_range)) as f32;
        let bridge_length = rng.gen_range(1..3);
        let distance = old_room.radius + bridge_length + radius;

        let new_room = Room::new(
            Vec2::new(direction.cos(), direction.sin())
                .round()
                .as_uvec2()
                * distance
                + old_room.pos,
            radius,
        );

        if !rooms.is_empty() {
            bridges.push(generate_bridge(old_room.pos, new_room.pos));
        }

        rooms.push(new_room.clone());
        old_room = new_room;
    }

    (rooms, bridges)
}

fn generate_bridge(from: UVec2, to: UVec2) -> Bridge {
    let mut rng = rand::thread_rng();
    let mut current = from.clone().as_vec2();
    let to = to.as_vec2();

    let mut positions = Vec::<UVec2>::new();

    while current != to {
        let mut dir = (to.y - current.y).atan2(to.x - current.x).to_degrees();
        dir = dir + [-90., 0., 90.].choose(&mut rng).unwrap(); //Add randomness
        dir = (dir / 90.).round() * 90.; //Increments of 90
        dir = dir.to_radians();

        current = current + Vec2::new(dir.cos(), dir.sin()).round();

        positions.push(current.as_uvec2());
    }

    Bridge::new(positions)
}
