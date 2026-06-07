use bevy::ecs::error::Result;
use bevy::ecs::message::MessageWriter;
use bevy::math::{IVec2, Vec3Swizzles};
use bevy::prelude::Component;
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

    if controlled.move_to.is_some()
        && match map_resource.get_aprox_tile(controlled.move_to.unwrap()) {
            Some(tile) => tile.walkable,
            None => false,
        }
        && state.map_or_else(|| true, |s| s.equals(State::Walking))
    {
        transform.translation = controlled.move_to.unwrap().extend(transform.translation.z);
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
            .get_aprox_tile(transform.translation.xy())
            .map(|tile| tile.last_room)
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
