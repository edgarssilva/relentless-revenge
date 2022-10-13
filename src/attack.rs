use std::time::Duration;

use bevy::{
    input::Input,
    prelude::{
        default, Bundle, Commands, Component, DespawnRecursiveExt, Entity, Handle, KeyCode, Quat,
        Query, Res, Transform, Vec2, Vec3, With, Without,
    },
    render::camera::Camera,
    sprite::{SpriteSheetBundle, TextureAtlas},
    time::{Time, Timer},
};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Sensor,
};

use crate::{
    collision::BodyLayers,
    controller::PlayerControlled,
    direction::Direction,
    helper::{KeyMaps, Shake},
    movement::movement::Velocity,
    state::State,
    stats::Stats,
};

#[derive(Component)]
pub struct Attack;

#[derive(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct Damage(pub u32);

#[derive(Component)]
pub struct MeleeSensor {
    pub dir: Direction,
    pub targets: Vec<Entity>,
}

impl MeleeSensor {
    pub fn from(dir: Direction) -> Self {
        Self {
            dir,
            targets: Vec::new(),
        }
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    attack: Attack,
    #[bundle]
    spritesheet_bundle: SpriteSheetBundle,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    duration: Lifetime,
    damage: Damage,
    velocity: Velocity,
}

impl ProjectileBundle {
    pub fn new(
        texture: Handle<TextureAtlas>,
        position: Vec3,
        rotation: f32,
        size: Vec2,
        duration: u64,
        damage: Damage,
        velocity: Velocity,
    ) -> Self {
        Self {
            attack: Attack,
            spritesheet_bundle: SpriteSheetBundle {
                texture_atlas: texture,
                transform: Transform {
                    translation: position,
                    rotation: Quat::from_rotation_z(rotation),
                    // scale: Vec3::new(size.x, size.y, 1.0),
                    ..default()
                },
                ..default()
            },
            collider: Collider::cuboid(size.x / 2., size.y / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY_ATTACK, BodyLayers::PLAYER),
            duration: Lifetime(Timer::new(Duration::from_millis(duration), false)),
            damage,
            velocity,
        }
    }
}

pub fn attack_system(
    mut player_query: Query<(&Stats, &Direction, &mut State), With<PlayerControlled>>,
    mut stats_query: Query<&mut Stats, Without<PlayerControlled>>,
    sensors_query: Query<&MeleeSensor>,
    keys: Res<Input<KeyCode>>,
    keymaps: Res<KeyMaps>,
    camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(keymaps.attack) {
        return;
    }

    if let Ok((attacker_stats, direction, mut state)) = player_query.get_single_mut() {
        if !attacker_stats.can_attack() {
            return;
        }

        state.set(State::ATTACKING);

        for sensor in sensors_query
            .iter()
            .filter(|sensor| sensor.dir == *direction)
        {
            for &attacked_entity in sensor.targets.iter() {
                if let Ok(mut attacked_stats) = stats_query.get_mut(attacked_entity) {
                    if attacked_stats.health < attacker_stats.damage {
                        attacked_stats.health = 0;
                    } else {
                        attacked_stats.health -= attacker_stats.damage;
                    }
                    if let Ok(camera) = camera_query.get_single() {
                        commands.entity(camera).insert(Shake {
                            duration: 0.25,
                            strength: 7.5,
                        });
                    }
                }
            }
        }
    }
}

pub fn attack_lifetime(
    mut commands: Commands,
    mut attacks: Query<(Entity, &mut Lifetime), With<Attack>>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in attacks.iter_mut() {
        lifetime.0.tick(time.delta());

        if lifetime.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
