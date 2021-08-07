use bevy::prelude::{Texture, EventReader, AssetEvent, ResMut, Assets, Transform, Res, Input, KeyCode, Time, Windows, Query};
use bevy::render::{texture::FilterMode, camera::{Camera, OrthographicProjection, CameraProjection}};

//Just a quick helper to make all textures Nearest Neighbour  TODO: Change this to a helper file
pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Texture>>,
    mut textures: ResMut<Assets<Texture>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.sampler.min_filter = FilterMode::Nearest;
                    texture.sampler.mag_filter = FilterMode::Nearest;
                    texture.sampler.mipmap_filter = FilterMode::Nearest;
                }
            }
            _ => (),
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
        if keys.pressed(KeyCode::Up) { transform.translation.y += 150.0 * time.delta_seconds(); }
        if keys.pressed(KeyCode::Left) { transform.translation.x -= 150.0 * time.delta_seconds(); }
        if keys.pressed(KeyCode::Down) { transform.translation.y -= 150.0 * time.delta_seconds(); }
        if keys.pressed(KeyCode::Right) { transform.translation.x += 150.0 * time.delta_seconds(); }

        let scale = projection.scale.clone();

        let w = windows.get(camera.window).unwrap();

        if keys.pressed(KeyCode::Z) { projection.scale -= 0.55 * time.delta_seconds(); }
        if keys.pressed(KeyCode::X) { projection.scale += 0.55 * time.delta_seconds(); }

        if projection.scale != scale {
            projection.update(w.width(), w.height());
            camera.projection_matrix = projection.get_projection_matrix();
            camera.depth_calculation = projection.depth_calculation();
        }
    }
}