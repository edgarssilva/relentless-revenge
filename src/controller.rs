use crate::{
    direction::Direction,
    follow::{Follow, FollowTarget},
    stats::Stats,
    KeyMaps,
};
use bevy::{
    math::Vec2,
    prelude::{Commands, Entity, Input, KeyCode, Query, Res, Time, Transform},
};

pub struct PlayerControlled(pub Direction);

//Player Movement
pub fn player_controller(
    mut commands: Commands,
    mut query: Query<(
        &mut Transform,
        Option<&Stats>,
        &mut PlayerControlled,
        Entity,
    )>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mapping: Res<KeyMaps>,
) {
    for (mut transform, stats, mut controller, entity) in query.iter_mut() {
        let mut dir = Vec2::ZERO;

        if keys.pressed(mapping.walk_up) {
            dir += Vec2::Y;
            controller.0 = Direction::NORTH;
        }

        if keys.pressed(mapping.walk_left) {
            dir -= Vec2::X;
            controller.0 = Direction::WEST;
        }

        if keys.pressed(mapping.walk_down) {
            dir -= Vec2::Y;
            controller.0 = Direction::SOUTH;
        }

        if keys.pressed(mapping.walk_right) {
            dir += Vec2::X;
            controller.0 = Direction::EAST;
        }

        if keys.just_pressed(mapping.dash) {
            let dash_dir = if dir == Vec2::ZERO {
                controller.0.vec()
            } else {
                dir
            };

            //TODO: Add dash stats
            let new_pos = transform.translation + (dash_dir * 35.).extend(0.);
            commands.entity(entity).insert(Follow::new(
                FollowTarget::Position(new_pos),
                10.,
                false,
            ));

            return;
        }

        let speed: u32 = if let Some(stats) = stats {
            stats.speed
        } else {
            45 /*TODO: Check default movement speed*/
        };

        let dir = dir.normalize_or_zero() * speed as f32 * time.delta_seconds();

        transform.translation.x += dir.x;
        transform.translation.y += dir.y;
    }
}
