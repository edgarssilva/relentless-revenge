use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::{
    AssetEvent, Assets, Changed, Commands, Entity, EventReader, Input, KeyCode, Query, Res, ResMut,
    Texture, Time, Transform, Windows, With, Without,
};
use bevy::render::{
    camera::{Camera, CameraProjection, OrthographicProjection},
    texture::FilterMode,
};

use rand::Rng;

pub struct Parallax;
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
            let mut rng = rand::thread_rng();

            trans.translation.x += rng.gen_range(-1.0..1.0) * shake.strength * time.delta_seconds();
            trans.translation.y += rng.gen_range(-1.0..1.0) * shake.strength * time.delta_seconds();

            shake.duration -= time.delta_seconds();
        } else {
            commands.entity(entity).remove::<Shake>();
        }
    }
}

//Just a quick helper to make all textures Nearest Neighbour  TODO: Change this to a helper file
pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        if let AssetEvent::Created { handle } = event {
            if let Some(mut texture) = textures.get_mut(handle) {
                texture.sampler.min_filter = FilterMode::Nearest;
                texture.sampler.mag_filter = FilterMode::Nearest;
                texture.sampler.mipmap_filter = FilterMode::Nearest;
            }
        }
    }
}

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut Camera, &mut OrthographicProjection, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    if let Ok((mut camera, mut projection, mut transform)) = query.single_mut() {
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

        let scale = projection.scale;

        let w = windows.get(camera.window).unwrap();

        if keys.pressed(KeyCode::Z) {
            projection.scale -= 0.55 * time.delta_seconds();
        }
        if keys.pressed(KeyCode::X) {
            projection.scale += 0.55 * time.delta_seconds();
        }

        if (projection.scale - scale).abs() > f32::EPSILON {
            projection.update(w.width(), w.height());
            camera.projection_matrix = projection.get_projection_matrix();
            camera.depth_calculation = projection.depth_calculation();
        }
    }
}

pub fn run_on_camera_move(query: Query<(), (Changed<Transform>, With<Camera>)>) -> ShouldRun {
    if query.single().is_ok() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn parallax_system(
    cam_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<Parallax>, Without<Camera>)>,
) {
    if let Ok(cam_trans) = cam_query.single() {
        for mut trans in query.iter_mut() {
            trans.translation.x =
                -cam_trans.translation.x * (0.002 * (trans.translation.z - crate::BACKGROUND_Z));
            trans.translation.y =
                -cam_trans.translation.y * (0.001 * (trans.translation.z - crate::BACKGROUND_Z));
        }
    }
}

pub struct KeyMaps {
    pub walk_up: KeyCode,
    pub walk_left: KeyCode,
    pub walk_down: KeyCode,
    pub walk_right: KeyCode,
    pub attack: KeyCode,
}

impl Default for KeyMaps {
    fn default() -> Self {
        KeyMaps {
            walk_up: KeyCode::W,
            walk_left: KeyCode::A,
            walk_down: KeyCode::S,
            walk_right: KeyCode::D,
            attack: KeyCode::K,
        }
    }
}
