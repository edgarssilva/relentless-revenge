use bevy::prelude::{
    Commands, Component, Entity, Input, KeyCode, Query, Res, Resource, Time, Transform, With,
};
use bevy::render::camera::OrthographicProjection;
use turborand::rng::Rng;
use turborand::TurboRand;

use crate::movement::movement::Follow;

#[derive(Component)]
pub struct Parallax;

#[derive(Component)]
pub struct Shake {
    pub strength: f32,
    pub duration: f32,
}

pub fn shake_system(
    mut commands: Commands,
    mut query: Query<(&mut Transform, &mut Shake, Entity)>,
    time: Res<Time>,
) {
    for (mut trans, mut shake, entity) in query.iter_mut() {
        if shake.duration > 0. {
            let rand = Rng::new();

            trans.translation.x += (rand.i32(-100..=100) as f32 / 100.) * shake.strength * time.delta_seconds();
            trans.translation.y += (rand.i32(-100..=100) as f32 / 100.) * shake.strength * time.delta_seconds();

            shake.duration -= time.delta_seconds();
        } else {
            commands.entity(entity).remove::<Shake>();
        }
    }
}

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut OrthographicProjection, &mut Transform), With<Follow>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut projection, mut transform)) = query.get_single_mut() {
        if keys.pressed(KeyCode::Up) {
            transform.translation.y += 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::Left) {
            transform.translation.x -= 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::Down) {
            transform.translation.y -= 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::Right) {
            transform.translation.x += 150.0 * time.delta_seconds();
        }

        if keys.pressed(KeyCode::Z) {
            projection.scale -= 1. * time.delta_seconds();
        }
        if keys.pressed(KeyCode::X) {
            projection.scale += 1. * time.delta_seconds();
        }
    }
}

#[derive(Resource)]
pub struct KeyMaps {
    pub walk_up: KeyCode,
    pub walk_left: KeyCode,
    pub walk_down: KeyCode,
    pub walk_right: KeyCode,
    pub attack: KeyCode,
    pub dash: KeyCode,
}

impl Default for KeyMaps {
    fn default() -> Self {
        KeyMaps {
            walk_up: KeyCode::W,
            walk_left: KeyCode::A,
            walk_down: KeyCode::S,
            walk_right: KeyCode::D,
            attack: KeyCode::K,
            dash: KeyCode::Space,
        }
    }
}
