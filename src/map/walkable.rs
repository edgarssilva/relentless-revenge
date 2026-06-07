use bevy::ecs::error::Result;
use bevy::ecs::message::MessageWriter;
use bevy::math::{Vec2, Vec3Swizzles};
use bevy::prelude::{Local, Query, Res, Time, With};
use bevy::transform::components::Transform;

use crate::controller::Controlled;
use crate::floor::TriggerNextFloorMessage;
use crate::map::generation::MapResource;
use crate::player::Player;
use crate::state::State;

pub fn restrict_movement(
    mut controlled_query: Query<(&Controlled, &mut Transform, Option<&State>)>,
    map_resource: Res<MapResource>,
) -> Result {
    let (controlled, mut transform, state) = controlled_query.single_mut()?;

    if let Some(move_to) = controlled.move_to {
        if state.map_or_else(|| true, |s| s.equals(State::Walking)) {
            let current_pos = transform.translation.xy();

            if map_resource
                .get_aprox_tile(&move_to)
                .map(|tile| tile.walkable)
                .unwrap_or(false)
            {
                transform.translation.x = move_to.x;
                transform.translation.y = move_to.y;
            } else {
                let x_move = Vec2::new(move_to.x, current_pos.y);
                let y_move = Vec2::new(current_pos.x, move_to.y);

                if map_resource
                    .get_aprox_tile(&x_move)
                    .map(|tile| tile.walkable)
                    .unwrap_or(false)
                {
                    transform.translation.x = move_to.x;
                }

                if map_resource
                    .get_aprox_tile(&y_move)
                    .map(|tile| tile.walkable)
                    .unwrap_or(false)
                {
                    transform.translation.y = move_to.y;
                }
            }
        }
    }

    Ok(())
}

pub fn travel_through_portal(
    player_query: Query<&Transform, With<Player>>,
    map_resource: Res<MapResource>,
    mut timer: Local<f32>,
    delta: Res<Time>,
    mut level_writer: MessageWriter<TriggerNextFloorMessage>,
) {
    for transform in player_query.iter() {
        if map_resource
            .get_aprox_tile(&transform.translation.xy())
            .map(|tile| tile.last_room && tile.is_center)
            .unwrap_or(false)
        {
            *timer += delta.delta_secs();

            if *timer > 3. {
                *timer = 0.0;
                level_writer.write(TriggerNextFloorMessage);
            }
        } else {
            *timer = 0.0;
        }
    }
}
