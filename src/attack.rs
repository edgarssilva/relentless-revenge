use bevy::prelude::EventReader;
use bevy::{
    prelude::{
        default, BuildChildren, Bundle, Commands, Component, DespawnRecursiveExt, Entity, Handle,
        Quat, Query, Res, Transform, Vec2, Vec3, With,
    },
    sprite::{SpriteSheetBundle, TextureAtlas},
    time::{Time, Timer},
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Sensor,
};

use crate::metadata::AttackMeta;
use crate::{
    collision::BodyLayers,
    movement::direction::Direction,
    movement::movement::Velocity,
    player::Player,
    state::State,
    stats::{Cooldown, Damage},
};

#[derive(Component)]
pub struct Attack;

#[derive(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct Breakable(pub u32);

#[derive(Component)]
pub struct Damageable;

#[derive(Component)]
pub struct Knockback {
    pub force: f32,
    pub direction: Direction,
}

#[derive(Component)]
pub struct AttackPhase {
    pub charge: Timer,
    pub attack: Timer,
    pub recover: Timer,
}

/**
 * Generic attack bundle, missing an transform that can be added alone or with an sprite
 */
#[derive(Bundle)]
struct AttackBundle {
    attack: Attack,
    collider: Collider,
    sensor: Sensor,
    events: ActiveEvents,
    collision_types: ActiveCollisionTypes,
    collision_groups: CollisionGroups,
    lifetime: Lifetime,
    damage: Damage,
}

impl AttackBundle {
    pub fn new(size: Vec2, duration: f32, damage: Damage, is_player_attack: bool) -> Self {
        let collision_groups = if is_player_attack {
            CollisionGroups::new(BodyLayers::PLAYER_ATTACK, BodyLayers::ENEMY)
        } else {
            CollisionGroups::new(BodyLayers::ENEMY_ATTACK, BodyLayers::PLAYER)
        };

        Self {
            attack: Attack,
            collider: Collider::cuboid(size.x / 2., size.y / 2.),
            sensor: Sensor,
            events: ActiveEvents::COLLISION_EVENTS,
            collision_types: ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC,
            collision_groups,
            lifetime: Lifetime(Timer::from_seconds(duration, bevy::time::TimerMode::Once)),
            damage,
        }
    }
}

#[derive(Bundle)]
pub struct MeleeAttackBundle {
    #[bundle]
    attack: AttackBundle,
    #[bundle]
    transform_bundle: TransformBundle,
    knockback: Knockback,
}

impl MeleeAttackBundle {
    pub fn new(
        position: Vec3,
        size: Vec2,
        duration: f32,
        damage: Damage,
        knockback: Knockback,
        is_player_attack: bool,
    ) -> Self {
        Self {
            attack: AttackBundle::new(size, duration, damage, is_player_attack),
            transform_bundle: TransformBundle {
                local: Transform::from_translation(position),
                ..default()
            },
            knockback,
        }
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    #[bundle]
    attack: AttackBundle,
    #[bundle]
    spritesheet_bundle: SpriteSheetBundle,
    velocity: Velocity,
    breakable: Breakable,
}

impl ProjectileBundle {
    pub fn new(
        texture: Handle<TextureAtlas>,
        position: Vec3,
        rotation: f32,
        size: Vec2,
        duration: f32,
        damage: Damage,
        is_player_attack: bool,
        velocity: Velocity,
    ) -> Self {
        Self {
            attack: AttackBundle::new(size, duration, damage, is_player_attack),
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
            velocity,
            breakable: Breakable(1),
        }
    }
}

pub struct SpawnEnemyAttack {
    pub meta: AttackMeta,
    pub position: Vec3,
    pub direction: Vec2,
    pub damage: Damage,
    pub enemy_size: Vec2,
}

pub fn attack_spawner(mut event: EventReader<SpawnEnemyAttack>, mut commands: Commands) {
    for spawn_attack in event.iter() {
        match &spawn_attack.meta {
            AttackMeta::Melee {
                size,
                duration,
                knockback,
            } => {
                let direction = Direction::from_vec2(spawn_attack.direction * -1.)
                    .expect("Bad knockback direction");

                let offset = spawn_attack.direction * spawn_attack.enemy_size / 2.;

                commands.spawn(MeleeAttackBundle::new(
                    spawn_attack.position + offset.extend(0.),
                    *size / 2.,
                    *duration,
                    spawn_attack.damage,
                    Knockback {
                        force: *knockback,
                        direction,
                    },
                    false,
                ));
            }
            AttackMeta::Ranged {
                texture,
                velocity,
                size,
                duration,
            } => {
                commands.spawn(ProjectileBundle::new(
                    texture.atlas_handle.clone(),
                    spawn_attack.position,
                    f32::atan2(spawn_attack.direction.y, spawn_attack.direction.x),
                    *size / 2.,
                    *duration,
                    spawn_attack.damage,
                    false,
                    Velocity(spawn_attack.direction * *velocity, false),
                ));
            }
        }
    }
}

pub fn attack_system(
    mut query: Query<(&mut State, &mut AttackPhase, &Direction, &Damage, Entity), With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut state, mut attack_phase, direction, damage, entity) in query.iter_mut() {
        if !attack_phase.charge.finished() {
            attack_phase.charge.tick(time.delta());

            if attack_phase.charge.just_finished() {
                //TODO: Add player meta so we can get the stats, player size and attack size
                commands.entity(entity).with_children(|children| {
                    let player_size = Vec2::new(32., 24.) * 0.75;
                    let offset = player_size.x * 0.75;

                    children.spawn(MeleeAttackBundle::new(
                        (direction.vec() * offset).extend(10.),
                        player_size,
                        attack_phase.attack.duration().as_secs_f32(),
                        *damage,
                        Knockback {
                            force: 7.,
                            direction: direction.clone(),
                        },
                        true,
                    ));
                });
            }
            return;
        }

        if !attack_phase.attack.finished() {
            attack_phase.attack.tick(time.delta());
            return;
        }

        attack_phase.recover.tick(time.delta());
        if attack_phase.recover.finished() {
            state.set(State::Idle);
            commands.entity(entity).remove::<AttackPhase>();
        }
    }
}

pub fn lifetimes(
    mut commands: Commands,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in lifetimes.iter_mut() {
        lifetime.0.tick(time.delta());

        if lifetime.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

//TODO: Check the collisions in its own system
pub fn projectile_break(mut commands: Commands, query: Query<(Entity, &Breakable)>) {
    for (entity, breakable) in query.iter() {
        if breakable.0 == 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn tick_cooldown(mut query: Query<&mut Cooldown>, time: Res<Time>) {
    for mut cooldown in query.iter_mut() {
        cooldown.update(time.delta());
    }
}
