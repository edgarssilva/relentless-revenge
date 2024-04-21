use bevy::math::UVec2;
use bevy::math::Vec2;
use bevy::prelude::Commands;
use bevy::prelude::With;
use bevy::prelude::{Component, Entity, EventReader, EventWriter, Query, Res};
use bevy_ecs_tilemap::prelude::*;

use crate::floor::{FloorClearedEvent, FloorResource, GenerateFloorEvent, SpawnFloorEntitiesEvent};

use crate::game_states::loading::GameAssets;
use crate::map::map::generate_map;
use crate::map::map::Tile;
use crate::map::map::TileVariant;
use crate::map::walkable::WalkableTile;

#[derive(Component)]
pub struct LevelStartTile;

#[derive(Component)]
pub struct LevelPortalTile;

pub fn setup_map(mut commands: Commands, game_assets: Res<GameAssets>) {
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
    mut event: EventReader<GenerateFloorEvent>,
    mut tile_query: Query<(Entity, &mut TileTextureIndex)>,
    tile_storage_query: Query<&TileStorage>,
    mut spawn_writer: EventWriter<SpawnFloorEntitiesEvent>,
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

            if let Ok(tile_storage) = tile_storage_query.get_single() {
                let map = generate_map(domain_data);
                let tiles: Vec<Tile> = map.into();
                let spawn_event = build_map(tiles, &mut tile_query, tile_storage, &mut commands);

                spawn_writer.send(spawn_event);
            }
        }
    }
}

fn build_map(
    tiles: Vec<Tile>,
    tile_query: &mut Query<(Entity, &mut TileTextureIndex)>,
    tile_storage: &TileStorage,
    commands: &mut Commands,
) -> SpawnFloorEntitiesEvent {
    let mut player_pos = Vec2::ZERO;
    let mut spawnable_pos = Vec::new();

    for tile in &tiles {
        let tile_pos = TilePos {
            x: tile.pos.x as u32,
            y: tile.pos.y as u32,
        };

        //TODO: Get the grid-size and map type from the current map
        let world_pos = tile_pos.center_in_world(
            &TilemapGridSize { x: 32., y: 16. },
            &TilemapType::Isometric(IsoCoordSystem::Diamond),
        );

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
                }
            }
        }
    }

    SpawnFloorEntitiesEvent {
        spawnable_pos,
        player_pos,
    }
}

pub fn open_level_portal(
    mut events: EventReader<FloorClearedEvent>,
    mut tile_query: Query<&mut TileTextureIndex, With<LevelPortalTile>>,
) {
    if !events.is_empty() {
        for mut tile in tile_query.iter_mut() {
            tile.0 = 4;
        }

        events.clear();
    }
}
