use crate::controller::PlayerControlled;
use crate::enemy::EnemyBundle;
use crate::map::room::Room;
use bevy::input::Input;
use bevy::math::Vec2;
use bevy::prelude::{
    default, AssetServer, Assets, Commands, KeyCode, Mut, Query, Res, ResMut, Transform, UVec2,
    With,
};
use bevy::sprite::TextureAtlas;

use bevy_ecs_tilemap::prelude::{
    TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType,
};
use bevy_ecs_tilemap::tiles::{TileBundle, TilePos, TileStorage, TileTexture};
use bevy_ecs_tilemap::TilemapBundle;

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
    let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

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
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: grid_size,
            size: tilemap_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size,
            map_type: TilemapType::isometric_diamond(true),
            ..Default::default()
        });
}

pub fn remake_map(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Transform, With<PlayerControlled>>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    asset_server: Res<AssetServer>,
    mut tile_query: Query<&mut TileTexture>,
    tile_storage_query: Query<&TileStorage>,
) {
    if keys.just_released(KeyCode::LControl) {
        let transform = player_query.single_mut();

        //Change all tiles to clear texture
        for mut tile in tile_query.iter_mut() {
            tile.0 = 3;
        }

        if let Ok(tile_storage) = tile_storage_query.get_single() {
            build_map(
                &mut commands,
                transform,
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

                let tile_pos = TilePos { x, y };

                //TODO: Build room using neighbors
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                        tile_texture.0 = 2;
                    }
                }

                //TODO: Get the grid-size and map type from the current map
                let world_pos = tile_pos.center_in_world(
                    &TilemapGridSize { x: 32., y: 16. },
                    &TilemapType::isometric_diamond(true),
                );

                //TODO: Move enemy spawns to a separate system
                if rng.gen_bool(1. / 40.) {
                    commands.spawn_bundle(EnemyBundle::new(
                        texture_atlas_handle.clone(),
                        world_pos.extend(1.0),
                    ));
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
