use crate::{stats::Stats, KeyMaps};
use bevy::prelude::{Input, KeyCode, Query, Res, Time, Transform, Vec3, With};

pub struct PlayerControlled;

//Player Movement
pub fn player_controller(
    mut query: Query<(&mut Transform, Option<&Stats>), With<PlayerControlled>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mapping: Res<KeyMaps>,
) {
    for (mut transform, stats) in query.iter_mut() {
        let mut dir = Vec3::ZERO;

        if keys.pressed(mapping.walk_up) {
            dir += Vec3::Y;
        }

        if keys.pressed(mapping.walk_left) {
            dir -= Vec3::X;
        }

        if keys.pressed(mapping.walk_down) {
            dir -= Vec3::Y;
        }

        if keys.pressed(mapping.walk_right) {
            dir += Vec3::X;
        }

        /* if keys.pressed(KeyCode::Space) { dir += Vec3::new(0., 0., 1.); }*/

        let speed: u32 = if let Some(stats) = stats {
            stats.speed
        } else {
            45 /*TODO: Check default movement speed*/
        };

        transform.translation += dir.normalize_or_zero() * speed as f32 * time.delta_seconds();
    }
}
