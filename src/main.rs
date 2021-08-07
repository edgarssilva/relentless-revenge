use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::render::texture::FilterMode;
use bevy::render::camera::{Camera, OrthographicProjection, CameraProjection};
use heron::prelude::*;
use bevy::math::Vec3Swizzles;

struct PlayerControlled;

pub struct FollowEntity {
    pub entity: Entity,
    pub lerp_speed: f32,
}

pub struct Stats {
    pub health: u32,
    pub damage: u32,
    pub speed: u32,
}

impl Stats {
    fn new(health: u32, damage: u32, speed: u32) -> Self {
        Stats { health, damage, speed }
    }
}


fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_system(set_texture_filters_to_nearest.system())
        .add_system(helper_camera_controller.system())
        .add_system(sprite_animation.system())
        .add_system(player_controller.system())
        .add_system(follow_entity_system.system())
        .add_startup_system(setup.system())
        .run();
}


fn setup(mut commands: Commands, mut texture_atlases: ResMut<Assets<TextureAtlas>>, asset_server: Res<AssetServer>) {
    //Map Creation
    let map_id = commands.spawn().id();
    let map_handle = asset_server.load("map_old.tmx");

    commands
        .entity(map_id)
        .insert_bundle(TiledMapBundle {
            tiled_map: map_handle,
            map: Map::new(0u16, map_id),
            transform: Transform::from_xyz(0.0, 80.0, 0.0), //TODO: Find a way to center the map
            ..Default::default()
        });

    //Player Creation

    //Using weird transform math because of test sprite
    let mut transform = Transform::from_xyz(0.0, 0.0, 10.0);
    transform.scale = Vec3::new(0.25, 0.25, 1.0);

    let unscaled_size = Vec2::new(60.0, 110.0);

    //Load the textures
    let texture_handle = asset_server.load("base.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, unscaled_size, 8, 8);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let player_size = unscaled_size.extend(0.) * transform.scale;

    let player_entity =
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                transform,
                ..Default::default()
            })
            .insert(PlayerControlled)
            .insert(RigidBody::Dynamic)
            .insert(CollisionShape::Cuboid {
                half_extends: player_size / 2.,
                border_radius: None,
            })
            .insert(RotationConstraints::lock())
            // .insert(PhysicMaterial { friction: 0., restitution: 0., density: 1. })
            .insert(Velocity::from_linear(Vec3::ZERO))
            .insert(Timer::from_seconds(0.1, true))
            .insert(Stats::new(100, 20, 50))
            .id();

    //Add Camera after so we can give it the player entity
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.2;
    commands.spawn_bundle(camera_bundle).insert(FollowEntity { entity: player_entity, lerp_speed: 7. });
}

//Cycles through all the animations TODO: Nees different animations
fn sprite_animation(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

//Player Movement
fn player_controller(mut query: Query<(&mut Velocity, Option<&Stats>), With<PlayerControlled>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
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

//System for an entity to follow another
pub fn follow_entity_system(
    mut query: Query<(&mut Transform, &FollowEntity)>,
    transform_query: Query<&Transform, Without<FollowEntity>>,
    time: Res<Time>,
) {
    for (mut transform, follow_entity) in query.iter_mut() {
        if let Ok(follow_transform) = transform_query.get(follow_entity.entity) {
            transform.translation =
                transform.translation.xy()
                    .lerp(follow_transform.translation.xy(), follow_entity.lerp_speed * time.delta_seconds())
                    .extend(transform.translation.z);
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