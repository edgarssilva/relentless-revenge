use bevy::math::Vec2;
use bevy::prelude::{
    Commands, Component, EventReader, EventWriter, IVec2, Mut, Query, Res, Transform, With,
};
use bevy_ecs_tilemap::prelude::*;
use rand;
use rand::prelude::*;

use crate::game_states::loading::TextureAssets;
use crate::level::{GenerateMapEvent, OpenLevelPortalEvent, SpawnEnemiesEvent};
use crate::map::room::Room;
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

    let tile_size = TilemapTileSize { x: 32.0, y: 16.0 };
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
    event: EventReader<GenerateMapEvent>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut tile_query: Query<&mut TileTextureIndex>,
    tile_storage_query: Query<&TileStorage>,
    mut spawn_enemies_writer: EventWriter<SpawnEnemiesEvent>,
    mut commands: Commands,
) {
    if !event.is_empty() {
        let transform = player_query.single_mut();

        //Change all tiles to clear texture
        for mut tile in tile_query.iter_mut() {
            tile.0 = 8;
        }

        let mut enemies = SpawnEnemiesEvent {
            positions: Vec::new(),
        };

        if let Ok(tile_storage) = tile_storage_query.get_single() {
            build_map(
                transform,
                tile_query,
                tile_storage,
                &mut enemies,
                &mut commands,
            );
        }

        spawn_enemies_writer.send(enemies);
        event.clear();
    }
}

fn build_map(
    mut player_transform: Mut<Transform>,
    mut tile_query: Query<&mut TileTextureIndex>,
    tile_storage: &TileStorage,
    enemies: &mut SpawnEnemiesEvent,
    commands: &mut Commands,
) {
    let mut rng = thread_rng();

    let (rooms, bridges) = generate_level();

    let max_enemies_per_room = 3;

    for (i, room) in rooms.iter().enumerate() {
        let mut num_enemies = 0;
        for x in room.pos.x - room.radius..room.pos.x + room.radius {
            for y in room.pos.y - room.radius..room.pos.y + room.radius + 1 {
                let tile_pos = TilePos {
                    x: x as u32,
                    y: y as u32,
                };

                //TODO: Get the grid-size and map type from the current map
                let world_pos = tile_pos.center_in_world(
                    &TilemapGridSize { x: 32., y: 16. },
                    &TilemapType::Isometric(IsoCoordSystem::Diamond),
                );

                //Cut corners
                if IVec2::new(x, y).as_vec2().distance(room.pos.as_vec2()) > room.radius as f32 {
                    continue;
                }

                //TODO: Build room using neighbors
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    commands.entity(tile_entity).insert(WalkableTile);
                    if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                        tile_texture.0 = 3;
                    }
                }

                //TODO: Move enemy spawns to a separate system
                if rng.gen_bool(1. / 75.) && num_enemies < max_enemies_per_room {
                    enemies.positions.push(world_pos);
                    num_enemies += 1;
                }

                if i == 0 && room.pos.to_array() == [x, y] {
                    player_transform.translation.x = world_pos.x;
                    player_transform.translation.y = world_pos.y;
                }

                if i == rooms.len() - 1 && room.pos.to_array() == [x, y] {
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        commands.entity(tile_entity).insert(LevelPortalTile);
                    }
                }
            }
        }
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
                if let Ok(mut tile_texture) = tile_query.get_mut(tile_entity) {
                    tile_texture.0 = 3;
                }
            }
        }
    }
}

pub fn open_level_portal(
    events: EventReader<OpenLevelPortalEvent>,
    mut tile_query: Query<&mut TileTextureIndex, With<LevelPortalTile>>,
) {
    if !events.is_empty() {
        for mut tile in tile_query.iter_mut() {
            tile.0 = 0;
        }

        events.clear();
    }
}

fn generate_level() -> (Vec<Room>, Vec<Bridge>) {
    let mut rng = thread_rng();

    let mut rooms = Vec::<Room>::new();
    let mut bridges = Vec::<Bridge>::new();

    let num_rooms = rng.gen_range(6..12);
    let room_min_radius = 4;
    let room_max_radius = 6;

    let mut old_room = Room::new(
        IVec2::new(80, 80),
        rng.gen_range(room_min_radius..room_max_radius),
    );

    let main_direction: f32 = rng.gen_range(0.0..360.);
    let angle_range = 180.;

    while rooms.len() < num_rooms {
        let radius = rng.gen_range(room_min_radius..room_max_radius);
        let direction = main_direction + rng.gen_range(-angle_range..angle_range);
        let direction = direction.to_radians();
        let bridge_length = rng.gen_range(1..4);
        let distance = old_room.radius + bridge_length + (radius / 2);

        let new_room = Room::new(
            (Vec2::new(direction.cos(), direction.sin()) * distance as f32)
                .round()
                .as_ivec2()
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
    let mut rng = thread_rng();
    let mut current = from.clone().as_vec2();
    let to = to.as_vec2();

    let mut positions = Vec::<IVec2>::new();

    while current != to {
        let mut dir = (to.y - current.y).atan2(to.x - current.x).to_degrees();
        dir += [-90., 0., 90.].choose(&mut rng).unwrap(); //Add randomness
        dir = (dir / 90.).round() * 90.; //Increments of 90
        dir = dir.to_radians();

        current += Vec2::new(dir.cos(), dir.sin()).round();

        positions.push(current.as_ivec2());
    }

    Bridge::new(positions)
}
