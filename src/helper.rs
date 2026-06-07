use crate::movement::movement::Follow;
use crate::player::Player;
use bevy::camera::Projection;
use bevy::input::ButtonInput;
use bevy::math::Vec3Swizzles;
use bevy::prelude::{
    Camera, Commands, Component, Entity, KeyCode, Query, Res, Resource, Time, Transform, Vec3,
    With, Without,
};
use noisy_bevy::fbm_simplex_2d_seeded;

#[derive(Component)]
pub struct Parallax;

#[derive(Component)]
pub struct Shake {
    pub strength: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct IsometricCameraFollow {
    pub offset: Vec3,
    pub smoothness: f32,
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
                time.delta_secs(),
            ) * shake.strength;

            let y_offset = fbm_simplex_2d_seeded(
                pos * FREQUENCY_SCALE,
                OCTAVES,
                LACUNARITY,
                GAIN,
                time.delta_secs() + 100.0,
            ) * shake.strength;

            let angle_offset = fbm_simplex_2d_seeded(
                pos * FREQUENCY_SCALE,
                OCTAVES,
                LACUNARITY,
                GAIN / 2.,
                time.delta_secs() + 50.,
            ) * shake.strength
                / 200.;

            trans.translation.x += x_offset * time.delta_secs();
            trans.translation.y += y_offset * time.delta_secs();

            trans.rotate_z(angle_offset * time.delta_secs());

            shake.duration -= time.delta_secs();
        } else {
            commands.entity(entity).remove::<Shake>();
        }
    }
}

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut Projection, &mut Transform), With<Follow>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut projection, mut transform)) = query.single_mut() {
        if keys.pressed(KeyCode::ArrowUp) {
            transform.translation.y += 150.0 * time.delta_secs();
        }
        if keys.pressed(KeyCode::ArrowLeft) {
            transform.translation.x -= 150.0 * time.delta_secs();
        }
        if keys.pressed(KeyCode::ArrowDown) {
            transform.translation.y -= 150.0 * time.delta_secs();
        }
        if keys.pressed(KeyCode::ArrowRight) {
            transform.translation.x += 150.0 * time.delta_secs();
        }

        if let Projection::Orthographic(ortho_projection) = projection.as_mut() {
            if keys.pressed(KeyCode::KeyZ) {
                ortho_projection.scale -= 1. * time.delta_secs();
            }
            if keys.pressed(KeyCode::KeyX) {
                ortho_projection.scale += 1. * time.delta_secs();
            }
        }
    }
}

pub fn follow_player_camera(
    mut camera_query: Query<(&mut Transform, &IsometricCameraFollow), (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut camera_transform, follow) in camera_query.iter_mut() {
        let target_pos = player_transform.translation + follow.offset;
        let lerp_factor = (follow.smoothness * time.delta_secs()).clamp(0.0, 1.0);
        camera_transform.translation = camera_transform.translation.lerp(target_pos, lerp_factor);
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
