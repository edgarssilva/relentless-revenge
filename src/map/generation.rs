use crate::controller::PlayerControlled;
use crate::map::room::Room;
use bevy::input::Input;
use bevy::math::{IVec2, Vec2};
use bevy::prelude::{
    AssetServer, Commands, GlobalTransform, KeyCode, Mut, Query, QueryState, Res, Transform, With,
};
use bevy_ecs_tilemap::{
    Chunk, ChunkSize, IsoType, LayerBuilder, LayerSettings, Map, MapQuery, MapSize, TextureSize,
    Tile, TileBundle, TilePos, TileSize, TilemapMeshType,
};
use rand;
use rand::prelude::*;

use super::bridge::Bridge;

pub fn setup_map(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let texture_handle = asset_server.load("tileset.png");

    let mut layer_settings = LayerSettings::new(
        MapSize(1, 1),
        ChunkSize(256, 256),
        TileSize(32.0, 32.0),
        TextureSize(128.0, 128.0),
    );

    layer_settings.grid_size = Vec2::new(32.0, 32.0 / 2.0);
    layer_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    let (mut layer_builder, _) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16);

    layer_builder.set_all(TileBundle {
        tile: Tile {
            texture_index: 4,
            ..Default::default()
        },
        ..Default::default()
    });

    let layer_entity = map_query.build_layer(&mut commands, layer_builder, texture_handle);

    map.add_layer(&mut commands, 0u16, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(0.0, 0., 0.0))
        .insert(GlobalTransform::default());
}

pub fn remake_map(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut map_query: MapQuery,
    mut player_query: Query<&mut Transform, With<PlayerControlled>>,
) {
    if keys.just_released(KeyCode::LControl) {
        let transform = player_query.single_mut();

        map_query.despawn_layer_tiles(&mut commands, 0u16, 0u16);
        build_map(&mut commands, map_query, transform);
    }
}

//TODO: Build on the chunk not the map itself
fn build_map(
    commands: &mut Commands,
    mut map_query: MapQuery,
    mut player_transform: Mut<Transform>,
) {
    /*   for x in 0..256 {
        let tile = Tile {
            texture_index: 0,
            ..Default::default()
        };
        let _ = map_query.set_tile(commands, TilePos(0, x), tile, 0u16, 0u16);
        map_query.notify_chunk_for_tile(TilePos(0, x), 0u16, 0u16);
    } */

    let mut first = true;

    let (rooms, bridges) = generate_level();
    for room in rooms {
        for x in room.pos.x - room.radius..room.pos.x + room.radius {
            for y in room.pos.y - room.radius..room.pos.y + room.radius + 1 {
                //Cut corners
                if IVec2::new(x, y).as_vec2().distance(room.pos.as_vec2()) > room.radius as f32 {
                    continue;
                }

                let tile = Tile {
                    texture_index: 2,
                    ..Default::default()
                };

                let tile_pos = TilePos(x as u32, y as u32);

                if first && room.pos.to_array() == [x, y] {
                    let pos = room.pos.as_vec2();
                    let x = (pos.x - pos.y) * 32. / 2.0;
                    let y = (pos.x + pos.y) * 16. / 2.0;
                    let new = Vec2::new(x, -y);

                    player_transform.translation.x = new.x;
                    player_transform.translation.y = new.y;
                }

                let _ = map_query.set_tile(commands, tile_pos, tile, 0u16, 0u16);
                map_query.notify_chunk_for_tile(tile_pos, 0u16, 0u16);
            }
        }
        first = false;
    }

    for bridge in bridges {
        for pos in bridge.pos {
            let x = pos.x;
            let y = pos.y;

            let tile_pos = TilePos(x as u32, y as u32);

            let tile = Tile {
                texture_index: 2,
                ..Default::default()
            };

            let _ = map_query.set_tile(commands, tile_pos, tile, 0u16, 0u16);
            map_query.notify_chunk_for_tile(tile_pos, 0u16, 0u16);
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
        IVec2::new(128, 128),
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
                .as_ivec2()
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

fn generate_bridge(from: IVec2, to: IVec2) -> Bridge {
    let mut rng = rand::thread_rng();
    let mut current = from.clone().as_vec2();
    let to = to.as_vec2();

    let mut positions = Vec::<IVec2>::new();

    while current != to {
        let mut dir = (to.y - current.y).atan2(to.x - current.x).to_degrees();
        dir = dir + [-90., 0., 90.].choose(&mut rng).unwrap(); //Add randomness
        dir = (dir / 90.).round() * 90.; //Increments of 90
        dir = dir.to_radians();

        current = current + Vec2::new(dir.cos(), dir.sin()).round();

        positions.push(current.as_ivec2());
    }

    Bridge::new(positions)
}
