use crate::map::walkable::WalkableTile;
use crate::GameState;
use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{
    in_state, App, Commands, Component, Entity, IntoSystemConfigs, Plugin, Query, Res, Time,
    Transform, Update,
};
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapType};
use bevy_ecs_tilemap::prelude::{TilePos, TileStorage};

use super::easing::ease_to_position;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (follow_entity_system, movement_system, ease_to_position)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Follow {
    pub target: Entity,
    pub speed: f32,
    pub continuous: bool,
    pub threshold: f32,
    pub(crate) on_target: bool,
}

impl Follow {
    pub fn new(target: Entity, speed: f32, continuous: bool, threshold: f32) -> Self {
        Self {
            target,
            speed,
            continuous,
            threshold,
            on_target: false,
        }
    }

    /* pub fn on_target(self: &Self) -> bool {
        self.on_target
    } */
}

//System for an entity to follow another
pub fn follow_entity_system(
    mut commands: Commands,
    mut query_followers: Query<(&mut Follow, Entity)>,
    mut transform_query: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut follow, entity) in query_followers.iter_mut() {
        let pos: Vec2 = if let Ok(transform) = transform_query.get_mut(follow.target) {
            transform.translation.xy()
        } else {
            //Entity was removed or does not exist
            commands.entity(entity).remove::<Follow>();
            continue;
        };

        if let Ok(mut transform) = transform_query.get_mut(entity) {
            //TODO: Check distance threshold (This was added because of Changed<>)
            if transform.translation.xy().distance(pos) > follow.threshold {
                transform.translation = transform
                    .translation
                    .xy()
                    .lerp(pos, follow.speed * time.delta_seconds())
                    .extend(transform.translation.z);

                follow.on_target = false;
            } else {
                follow.on_target = true;

                if !follow.continuous {
                    commands.entity(entity).remove::<Follow>();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Velocity(pub Vec2, pub bool);

pub fn movement_system(
    mut query_velocity: Query<(&Velocity, &mut Transform)>,
    tile_query: Query<(&TileStorage, &TilemapType, &TilemapSize, &TilemapGridSize)>,
    walkable_tiles_query: Query<&WalkableTile>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in query_velocity.iter_mut() {
        let new_pos = transform.translation + velocity.0.extend(0.) * time.delta_seconds();

        if !velocity.1 {
            //If not restricted to walkable tiles
            transform.translation = new_pos;
            continue;
        }

        if let Some((tile_storage, tilemap_type, map_size, grid_size)) = tile_query.iter().next() {
            if let Some(tile_pos) =
                TilePos::from_world_pos(&new_pos.xy(), map_size, grid_size, tilemap_type)
            {
                if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                    if walkable_tiles_query.get(tile_entity).is_ok() {
                        transform.translation = new_pos;
                    }
                }
            }
        }
    }
}
