use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::{
    AssetEvent, Assets, Changed, Commands, Component, Entity, EventReader, Image, Input, KeyCode,
    Query, Res, ResMut, Time, Transform, With, Without,
};
use bevy::render::camera::{Camera, OrthographicProjection};

use bevy::render::render_resource::TextureUsages;
use rand::Rng;

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
            let mut rng = rand::thread_rng();

            trans.translation.x += rng.gen_range(-1.0..1.0) * shake.strength * time.delta_seconds();
            trans.translation.y += rng.gen_range(-1.0..1.0) * shake.strength * time.delta_seconds();

            shake.duration -= time.delta_seconds();
        } else {
            commands.entity(entity).remove::<Shake>();
        }
    }
}

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
            _ => (),
        }
    }
}

//Helper camera controller
pub fn helper_camera_controller(
    mut query: Query<(&mut OrthographicProjection, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut projection, mut transform) = query.single_mut();

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

pub fn run_on_camera_move(query: Query<(), (Changed<Transform>, With<Camera>)>) -> ShouldRun {
    if query.get_single().is_ok() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn parallax_system(
    cam_query: Query<&Transform, With<Camera>>,
    mut query: Query<&mut Transform, (With<Parallax>, Without<Camera>)>,
) {
    let cam_trans = cam_query.single();

    for mut trans in query.iter_mut() {
        trans.translation.x =
            -cam_trans.translation.x * (0.002 * (trans.translation.z - crate::BACKGROUND_Z));
        trans.translation.y =
            -cam_trans.translation.y * (0.001 * (trans.translation.z - crate::BACKGROUND_Z));
    }
}

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
