use bevy::prelude::{Component, Query, Transform};
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::{TilePos, TileStorage},
};

use crate::controller::Controlled;

#[derive(Component)]
pub struct WalkableTile;

pub fn restrict_movement(
    mut controlled_query: Query<(&Controlled, &mut Transform)>,
    query: Query<(&TileStorage, &TilemapType, &TilemapSize, &TilemapGridSize)>,
    walkable_tiles_query: Query<&WalkableTile>,
) {
    for (tile_storage, tilemap_type, map_size, grid_size) in query.iter() {
        for (controlled, mut transform) in controlled_query.iter_mut() {
            if let Some(move_to) = controlled.move_to {
                if let Some(tile_pos) =
                    TilePos::from_world_pos(&move_to, map_size, grid_size, tilemap_type)
                {
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if walkable_tiles_query.get(tile_entity).is_ok() {
                            transform.translation = move_to.extend(transform.translation.z);
                        }
                    }
                }
            }
        }
        return;
    }
}
