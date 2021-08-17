use bevy::prelude::{
    AssetEvent, Assets, Commands, Entity, EventReader, Input, KeyCode, Query, Res, ResMut, Texture,
    Time, Transform, Windows,
};
use bevy::render::{
    camera::{Camera, CameraProjection, OrthographicProjection},
    texture::FilterMode,
};
use rand::Rng;

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

        if (projection.scale - scale).abs() > 0.05 {
            projection.update(w.width(), w.height());
            camera.projection_matrix = projection.get_projection_matrix();
            camera.depth_calculation = projection.depth_calculation();
        }
    }
}
