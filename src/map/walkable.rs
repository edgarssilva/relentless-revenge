use bevy::math::Vec3Swizzles;
use bevy::prelude::{Component, EventWriter, Local, Query, Res, Time, Transform, Vec2, With};
use bevy_ecs_tilemap::{
    prelude::{TilemapGridSize, TilemapSize, TilemapType},
    tiles::{TilePos, TileStorage},
};

use crate::floor::TriggerNextFloorEvent;
use crate::map::generation::LevelPortalTile;
use crate::{controller::Controlled, state::State};

#[derive(Component)]
pub struct WalkableTile;

pub fn restrict_movement(
    mut controlled_query: Query<(&Controlled, &mut Transform, Option<&State>)>,
    query: Query<(&TileStorage, &TilemapType, &TilemapSize, &TilemapGridSize)>,
    walkable_tiles_query: Query<&WalkableTile>,
) {
    if let Some((tile_storage, tilemap_type, map_size, grid_size)) = query.iter().next() {
        for (controlled, mut transform, state) in controlled_query.iter_mut() {
            if let Some(move_to) = controlled.move_to {
                let grid_pos = move_to + Vec2::new(0., -16.); //Account for the tile being 32x32 on a
                                                              //32x16 grid

                if let Some(tile_pos) =
                    TilePos::from_world_pos(&grid_pos, map_size, grid_size, tilemap_type)
                {
                    //Don't move if the player doesn't want to move
                    if let Some(state) = state {
                        if !state.equals(State::Walking) {
                            continue;
                        }
                    }

                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if walkable_tiles_query.get(tile_entity).is_ok() {
                            transform.translation = move_to.extend(transform.translation.z);
                        }
                    }
                }
            }
        }
    }
}

pub fn travel_through_portal(
    controlled_query: Query<&Transform, With<Controlled>>,
    query: Query<(&TileStorage, &TilemapType, &TilemapSize, &TilemapGridSize)>,
    portal_query: Query<&LevelPortalTile>,
    mut timer: Local<f32>,
    delta: Res<Time>,
    mut level_writer: EventWriter<TriggerNextFloorEvent>,
) {
    if let Some((tile_storage, tilemap_type, map_size, grid_size)) = query.iter().next() {
        for transform in controlled_query.iter() {
            let pos = transform.translation.xy() + Vec2::new(0., -16.); //Account for the tile being 32x32 on a
                                                                        //32x16 grid
            if let Some(tile_pos) = TilePos::from_world_pos(&pos, map_size, grid_size, tilemap_type)
            {
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if portal_query.get(tile_entity).is_ok() {
                        *timer += delta.delta_seconds();

                        if *timer > 3. {
                            *timer = 0.0;
                            level_writer.send(TriggerNextFloorEvent);
                        }
                    }
                }
            }
        }
    }
}
