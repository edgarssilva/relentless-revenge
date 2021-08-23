use crate::{direction::Direction, stats::Stats, KeyMaps};
use bevy::{
    math::Vec2,
    prelude::{Input, KeyCode, Query, Res, Time, Transform, With},
};

pub struct PlayerControlled(pub Direction);

//Player Movement TODO: Add option to Transform, Collider and RigidBody
pub fn player_controller(
    mut query: Query<(&mut Transform, Option<&Stats>, &mut PlayerControlled)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mapping: Res<KeyMaps>,
) {
    for (mut transform, stats, mut controller) in query.iter_mut() {
        let mut dir = Vec2::ZERO;

        if keys.pressed(mapping.walk_up) {
            dir += Vec2::Y;
        }

        if keys.pressed(mapping.walk_left) {
            dir -= Vec2::X;
            controller.0 = Direction::WEST;
        }

        if keys.pressed(mapping.walk_down) {
            dir -= Vec2::Y;
        }

        if keys.pressed(mapping.walk_right) {
            dir += Vec2::X;
            controller.0 = Direction::EAST;
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
