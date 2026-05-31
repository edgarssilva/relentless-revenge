use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use crate::floor::{
    FloorClearedMessage, FloorResource, GenerateFloorMessage, SpawnFloorEntitiesMessage,
};

use crate::game_states::loading::GameAssets;
use crate::map::map::generate_map;
use crate::map::map::Tile;
use crate::map::map::TileVariant;
use crate::map::walkable::WalkableTile;

const QUADRANT_SIDE_LENGTH: u32 = 64;
const TILE_SIZE: f32 = 32.0;

#[derive(Component)]
pub struct LevelStartTile;

#[derive(Component)]
pub struct LevelPortalTile;

pub fn setup_map(mut commands: Commands, game_assets: Res<GameAssets>) {
    let tilemap_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };

    let mut tile_storage = TileStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = TilemapGridSize {
        x: TILE_SIZE,
        y: TILE_SIZE / 2.,
    };

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
        texture: TilemapTexture::Single(game_assets.map_texture.clone()),
        tile_size,
        map_type: TilemapType::Isometric(IsoCoordSystem::Diamond),
        render_settings: TilemapRenderSettings {
            render_chunk_size: UVec2::new(32, 1),
            y_sort: true,
        },
        ..Default::default()
    });
}

pub fn remake_map(
    mut event: MessageReader<GenerateFloorMessage>,
    mut tile_query: Query<(Entity, &mut TileTextureIndex)>,
    map_query: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
    tile_storage_query: Query<&TileStorage>,
    mut spawn_writer: MessageWriter<SpawnFloorEntitiesMessage>,
    mut commands: Commands,
    floor: Res<FloorResource>,
) {
    if let Some(domain_data) = &floor.domain {
        for _ in event.read() {
            //Change all tiles to clear texture
            for (entity, mut tile) in tile_query.iter_mut() {
                tile.0 = 8;
                //TODO: Check if it's better to remove all the tiles and then add them back
                commands.entity(entity).remove::<WalkableTile>();
                commands.entity(entity).remove::<LevelStartTile>();
                commands.entity(entity).remove::<LevelPortalTile>();
            }

            if let Ok(tile_storage) = tile_storage_query.single() {
                let map = generate_map(domain_data);
                let tiles: Vec<Tile> = map.into();
                let spawn_event = build_map(
                    tiles,
                    &mut tile_query,
                    &map_query,
                    tile_storage,
                    &mut commands,
                );

                spawn_writer.write(spawn_event);
            }
        }
    }
}

fn build_map(
    tiles: Vec<Tile>,
    tile_query: &mut Query<(Entity, &mut TileTextureIndex)>,
    map_query: &Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapTileSize,
        &TilemapType,
        &TilemapAnchor,
    )>,
    tile_storage: &TileStorage,
    commands: &mut Commands,
) -> SpawnFloorEntitiesMessage {
    let mut player_pos = Vec2::ZERO;
    let mut spawnable_pos = Vec::new();
    let mut portal_pos = Vec2::ZERO;

    for tile in &tiles {
        let tile_pos = TilePos {
            x: tile.pos.x as u32,
            y: tile.pos.y as u32,
        };

        if let Ok((map_size, grid_size, tile_size, map_type, anchor)) = map_query.single() {
            let world_pos =
                tile_pos.center_in_world(map_size, grid_size, tile_size, map_type, anchor);

            //TODO: Build room using neighbors
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                let mut ec = commands.entity(tile_entity);

                if let Ok((_, mut tile_texture)) = tile_query.get_mut(tile_entity) {
                    tile_texture.0 = match tile.variant {
                        TileVariant::Standard => 2,
                        TileVariant::Accented => 0,
                    };
                }

                if tile.spawnable {
                    spawnable_pos.push(world_pos);
                }

                if tile.walkable {
                    ec.insert(WalkableTile);
                }

                if tile.is_center {
                    if tile.firt_room {
                        ec.insert(LevelStartTile);
                        player_pos = world_pos;
                    } else if tile.last_room {
                        ec.insert(LevelPortalTile);
                        portal_pos = world_pos;
                    }
                }
            }
        }
    }

    SpawnFloorEntitiesMessage {
        spawnable_pos,
        player_pos,
        portal_pos,
    }
}

pub fn open_level_portal(
    mut events: MessageReader<FloorClearedMessage>,
    mut tile_query: Query<&mut TileTextureIndex, With<LevelPortalTile>>,
) {
    if !events.is_empty() {
        for mut tile in tile_query.iter_mut() {
            tile.0 = 4;
        }

        events.clear();
    }
}
