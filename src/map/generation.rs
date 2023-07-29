use bevy::math::Vec2;
use bevy::prelude::Commands;
use bevy::prelude::With;
use bevy::prelude::{
    Component, Entity, EventReader, EventWriter, IVec2, Mut, Query, Res, Transform,
};
use bevy_ecs_tilemap::prelude::*;
use turborand::prelude::Rng;
use turborand::TurboRand;

use crate::floor::{FloorClearedEvent, FloorResource, GenerateFloorEvent, SpawnFloorEntitiesEvent};
use crate::game_states::loading::TextureAssets;
use crate::map::room::Room;
use crate::metadata::FloorMeta;
use crate::player::Player;

use super::bridge::Bridge;
use super::walkable::WalkableTile;

#[derive(Component)]
pub struct LevelPortalTile;

pub fn setup_map(mut commands: Commands, texture_assets: Res<TextureAssets>) {
    let tilemap_size = TilemapSize { x: 160, y: 160 };
    let mut tile_storage = TileStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = TilemapGridSize { x: 32.0, y: 16.0 };

    fill_tilemap(
        TileTextureIndex(8),
        tilemap_size,
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: tilemap_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_assets.map_texture.clone()),
        tile_size,
        map_type: TilemapType::Isometric(IsoCoordSystem::Diamond),
        ..Default::default()
    });
}

pub fn remake_map(
    mut event: EventReader<GenerateFloorEvent>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut tile_query: Query<(Entity, &mut TileTextureIndex)>,
    tile_storage_query: Query<&TileStorage>,
    mut spawn_writer: EventWriter<SpawnFloorEntitiesEvent>,
    mut commands: Commands,
    level: Res<FloorResource>,
) {
    if let Some(level_meta) = &level.meta {
        for _ in event.iter() {
            let transform = player_query.single_mut();

            //Change all tiles to clear texture
            for (entity, mut tile) in tile_query.iter_mut() {
                tile.0 = 8;
                //TODO: Check if it's better to remove all the tiles and then add them back
                commands.entity(entity).remove::<WalkableTile>();
                commands.entity(entity).remove::<LevelPortalTile>();
            }

            if let Ok(tile_storage) = tile_storage_query.get_single() {
                let spawnable_tiles = build_map(
                    level_meta,
                    transform,
                    &mut tile_query,
                    tile_storage,
                    &mut commands,
                );
                spawn_writer.send(SpawnFloorEntitiesEvent(spawnable_tiles));
            }
        }
    }
}

fn build_map(
    level_meta: &FloorMeta,
    mut player_transform: Mut<Transform>,
    tile_query: &mut Query<(Entity, &mut TileTextureIndex)>,
    tile_storage: &TileStorage,
    commands: &mut Commands,
) -> Vec<Vec<Vec2>> {
    let mut rand = Rng::new();

    let (rooms, bridges) = generate_level(level_meta, &mut rand);

    let mut spawnable_room_tiles = Vec::new();

    for (i, room) in rooms.iter().enumerate() {
        let mut spawnable_tiles = Vec::new();

        for x in room.pos.x - room.radius..room.pos.x + room.radius {
            for y in room.pos.y - room.radius..room.pos.y + room.radius + 1 {
                let tile_pos = TilePos {
                    x: x as u32,
                    y: y as u32,
                };

                //Cut corners
                if IVec2::new(x, y).as_vec2().distance(room.pos.as_vec2()) > room.radius as f32 {
                    continue;
                }

                //TODO: Get the grid-size and map type from the current map
                let world_pos = tile_pos.center_in_world(
                    &TilemapGridSize { x: 32., y: 16. },
                    &TilemapType::Isometric(IsoCoordSystem::Diamond),
                );

                //TODO: Build room using neighbors
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    commands.entity(tile_entity).insert(WalkableTile);
                    if let Ok((_, mut tile_texture)) = tile_query.get_mut(tile_entity) {
                        tile_texture.0 = 2;

                        //Check if the tile is in the outer edge of the room
                        if x == room.pos.x - room.radius
                            || x == room.pos.x + room.radius - 1
                            || y == room.pos.y - room.radius
                            || y == room.pos.y + room.radius
                        {
                            //tile_texture.0 = 0;
                        } else {
                            spawnable_tiles.push(world_pos);
                        }
                    }
                }

                if i == 0 && room.pos.to_array() == [x, y] {
                    //TODO: Move player position out
                    player_transform.translation.x = world_pos.x;
                    player_transform.translation.y = world_pos.y;
                    spawnable_tiles.pop();
                }

                if i == rooms.len() - 1 && room.pos.to_array() == [x, y] {
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        commands.entity(tile_entity).insert(LevelPortalTile);
                    }
                }
            }
        }
        spawnable_room_tiles.push(spawnable_tiles);
    }

    for bridge in bridges {
        for pos in bridge.pos {
            let x = pos.x;
            let y = pos.y;

            if let Some(tile_entity) = tile_storage.get(&TilePos {
                x: x as u32,
                y: y as u32,
            }) {
                commands.entity(tile_entity).insert(WalkableTile);
                if let Ok((_, mut tile_texture)) = tile_query.get_mut(tile_entity) {
                    tile_texture.0 = 2;
                }
            }
        }
    }
    spawnable_room_tiles
}

pub fn open_level_portal(
    mut events: EventReader<FloorClearedEvent>,
    mut tile_query: Query<&mut TileTextureIndex, With<LevelPortalTile>>,
) {
    if !events.is_empty() {
        for mut tile in tile_query.iter_mut() {
            tile.0 = 0;
        }

        events.clear();
    }
}

fn generate_level(level_meta: &FloorMeta, rand: &mut Rng) -> (Vec<Room>, Vec<Bridge>) {
    let mut rooms = Vec::<Room>::new();
    let mut bridges = Vec::<Bridge>::new();

    let num_rooms = rand.u32(level_meta.rooms.0..=level_meta.rooms.1);
    let room_min_radius = level_meta.room_size.0;
    let room_max_radius = level_meta.room_size.1;

    let mut old_room = Room::new(
        IVec2::new(80, 80),
        rand.u32(room_min_radius..=room_max_radius) as i32,
    );

    let angle_range = 180;
    let main_direction = rand.i32(0..360);

    while rooms.len() < num_rooms as usize {
        let radius = rand.u32(room_min_radius..=room_max_radius) as i32;
        let direction = main_direction + rand.i32(-angle_range..angle_range);
        let direction = (direction as f32).to_radians();
        let bridge_length = rand.i32(1..=3);
        let distance = old_room.radius + bridge_length + (radius / 2);

        let new_room = Room::new(
            (Vec2::new(direction.cos(), direction.sin()) * distance as f32)
                .round()
                .as_ivec2()
                + old_room.pos,
            radius,
        );

        if !rooms.is_empty() {
            bridges.push(generate_bridge(old_room.pos, new_room.pos, rand));
        }

        rooms.push(new_room.clone());
        old_room = new_room;
    }

    (rooms, bridges)
}

fn generate_bridge(from: IVec2, to: IVec2, rand: &mut Rng) -> Bridge {
    let mut current = from.clone().as_vec2();
    let to = to.as_vec2();

    let mut positions = Vec::<IVec2>::new();
    let directions = [-90., 0., 90.];

    while current != to {
        let mut dir = (to.y - current.y).atan2(to.x - current.x).to_degrees();
        dir += *rand.sample(&directions).expect("No direction");
        dir = (dir / 90.).round() * 90.; //Increments of 90
        dir = dir.to_radians();

        current += Vec2::new(dir.cos(), dir.sin()).round();

        positions.push(current.as_ivec2());
    }

    Bridge::new(positions)
}
