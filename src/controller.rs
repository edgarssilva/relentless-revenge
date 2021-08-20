use crate::{stats::Stats, KeyMaps};
use bevy::{
    math::Vec2,
    prelude::{Input, KeyCode, Query, Res, Time, Transform, With},
};
// use bevy_rapier2d::prelude::RigidBodyPosition;

pub struct PlayerControlled;

//Player Movement TODO: Add option to Transform, Collider and RigidBody
pub fn player_controller(
    mut query: Query<(&mut Transform, Option<&Stats>), With<PlayerControlled>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mapping: Res<KeyMaps>,
) {
    for (mut transform, stats) in query.iter_mut() {
        let mut dir = Vec2::ZERO;

        if keys.pressed(mapping.walk_up) {
            dir += Vec2::Y;
        }

        if keys.pressed(mapping.walk_left) {
            dir -= Vec2::X;
        }

        if keys.pressed(mapping.walk_down) {
            dir -= Vec2::Y;
        }

        if keys.pressed(mapping.walk_right) {
            dir += Vec2::X;
        }

        /* if keys.pressed(KeyCode::Space) { dir += Vec3::new(0., 0., 1.); }*/

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
