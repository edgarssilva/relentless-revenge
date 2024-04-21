use crate::movement::movement::Follow;
use bevy::input::ButtonInput;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{
    Commands, Component, Entity, KeyCode, Query, Res, Resource, Time, Transform, With,
};
use bevy::render::camera::OrthographicProjection;
use noisy_bevy::fbm_simplex_2d_seeded;

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
    //Move this a good spot
    const FREQUENCY_SCALE: f32 = 0.55;
    const OCTAVES: usize = 4;
    const LACUNARITY: f32 = 2.;
    const GAIN: f32 = 1.75;

    for (mut trans, mut shake, entity) in query.iter_mut() {
        if shake.duration > 0. {
            //let rand = Rng::new();

            let pos = trans.translation.xy();
            let x_offset = fbm_simplex_2d_seeded(
                pos * FREQUENCY_SCALE,
                OCTAVES,
                LACUNARITY,
                GAIN,
                time.delta_seconds(),
            ) * shake.strength;

            let y_offset = fbm_simplex_2d_seeded(
                pos * FREQUENCY_SCALE,
                OCTAVES,
                LACUNARITY,
                GAIN,
                time.delta_seconds() + 100.0,
            ) * shake.strength;

            let angle_offset = fbm_simplex_2d_seeded(
                pos * FREQUENCY_SCALE,
                OCTAVES,
                LACUNARITY,
                GAIN / 2.,
                time.delta_seconds() + 50.,
            ) * shake.strength
                / 200.;

            trans.translation.x += x_offset * time.delta_seconds();
            trans.translation.y += y_offset * time.delta_seconds();

            trans.rotate_z(angle_offset * time.delta_seconds());

            shake.duration -= time.delta_seconds();
        } else {
            commands.entity(entity).remove::<Shake>();
        }
    }
}

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut OrthographicProjection, &mut Transform), With<Follow>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut projection, mut transform)) = query.get_single_mut() {
        if keys.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 150.0 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 150.0 * time.delta_seconds();
        }

        if keys.pressed(KeyCode::KeyZ) {
            projection.scale -= 1. * time.delta_seconds();
        }
        if keys.pressed(KeyCode::KeyX) {
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
            walk_up: KeyCode::KeyW,
            walk_left: KeyCode::KeyA,
            walk_down: KeyCode::KeyS,
            walk_right: KeyCode::KeyD,
            attack: KeyCode::KeyJ,
            dash: KeyCode::Space,
        }
    }
}
