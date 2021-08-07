
use bevy::prelude::{Query, With, Res, Input, KeyCode, Time, Vec3};
use heron::Velocity;
use crate::stats::Stats;


pub struct PlayerControlled;

//Player Movement
pub fn player_controller(mut query: Query<(&mut Velocity, Option<&Stats>), With<PlayerControlled>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
    for (mut velocity, stats) in query.iter_mut() {
        let mut dir = Vec3::ZERO;

        if keys.pressed(KeyCode::W) { dir += Vec3::Y; }
        if keys.pressed(KeyCode::A) { dir -= Vec3::X; }
        if keys.pressed(KeyCode::S) { dir -= Vec3::Y; }
        if keys.pressed(KeyCode::D) { dir += Vec3::X; }
        /* if keys.pressed(KeyCode::Space) { dir += Vec3::new(0., 0., 1.); }*/

        let speed: u32 = if let Some(stats) = stats { stats.speed } else { 45 /*TODO: Check default movement speed*/ };

        *velocity = Velocity::from_linear(dir.normalize_or_zero() * speed as f32 * 100. * time.delta_seconds());
    }
}
