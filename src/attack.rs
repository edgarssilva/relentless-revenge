use bevy::ecs::observer::On;
use bevy::image::Image;
use bevy::image::TextureAtlas;
use bevy::image::TextureAtlasLayout;
use bevy::prelude::Bundle;
use bevy::prelude::Event;
use bevy::reflect::Reflect;
use bevy::sprite::Sprite;
use bevy::time::TimerMode;
use bevy::{
    prelude::{
        default, Commands, Component, Entity, Handle, Quat, Query, Res, Transform, Vec2, Vec3,
    },
    time::{Time, Timer},
};
use bevy_rapier2d::prelude::CollisionGroups;
use bevy_rapier2d::prelude::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor};
use seldom_state::prelude::{Done, StateMachine};
use seldom_state::trigger::done;

use crate::layers::local_z;
use crate::manifest::AttackData;
use crate::{
    collision::BodyLayers,
    movement::direction::Direction,
    movement::movement::Velocity,
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

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
pub struct ChargePhase(pub Timer, pub f32); // Timer, attackphase duration for spawning attack

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
pub struct AttackPhase(pub Timer);

#[derive(Component, Clone, Reflect)]
#[component(storage = "SparseSet")]
pub struct RecoverPhase(pub Timer);

pub fn attack_phase(_charge: f32, attack: f32, recover: f32) -> StateMachine {
    StateMachine::default()
        .trans::<ChargePhase, _>(
            done(Some(Done::Success)),
            AttackPhase(Timer::from_seconds(attack, TimerMode::Once)),
        )
        .trans::<AttackPhase, _>(
            done(Some(Done::Success)),
            RecoverPhase(Timer::from_seconds(recover, TimerMode::Once)),
        )
}

#[derive(Component)]
pub struct EntitiesHit(pub Vec<Entity>);

/**
 * Generic attack bundle, missing an transform that can be added alone or with an sprite
 */
#[derive(Bundle)]
struct AttackBundle {
    attack: Attack,
    entities_hit: EntitiesHit,
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
            entities_hit: EntitiesHit(Vec::new()),
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
    attack: AttackBundle,
    transform: Transform,
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
            transform: Transform::from_translation(position),
            knockback,
        }
    }
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    attack: AttackBundle,
    sprite: Sprite,
    transform: Transform,
    velocity: Velocity,
    breakable: Breakable,
}

impl ProjectileBundle {
    pub fn new(
        texture: Handle<Image>,
        atlas: Handle<TextureAtlasLayout>,
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
            sprite: Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: atlas,
                    index: 0,
                },
            ),
            transform: Transform {
                translation: position,
                rotation: Quat::from_rotation_z(rotation),
                // scale: Vec3::new(size.x, size.y, 1.0),
                ..default()
            },
            velocity,
            breakable: Breakable(1),
        }
    }
}

#[derive(Event)]
pub struct SpawnEnemyAttack {
    pub data: AttackData,
    pub position: Vec3,
    pub direction: Vec2,
    pub damage: Damage,
    pub enemy_size: Vec2,
}

pub fn attack_spawner_observer(spawn_attack: On<SpawnEnemyAttack>, mut commands: Commands) {
    match &spawn_attack.data {
        AttackData::Melee {
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
        AttackData::Ranged {
            texture,
            velocity,
            size,
            duration,
            atlas,
        } => {
            commands.spawn(ProjectileBundle::new(
                texture.clone(),
                atlas.clone(),
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

pub fn charge_phase_system(
    mut query: Query<(&mut ChargePhase, &Direction, &Damage, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut charge_phase, direction, damage, entity) in query.iter_mut() {
        if charge_phase.0.is_finished() {
            commands.entity(entity).with_children(|children| {
                let player_size = Vec2::new(32., 24.) * 0.75;
                let offset = player_size.x * 0.75;

                children.spawn(MeleeAttackBundle::new(
                    (direction.vec() * offset).extend(local_z::PLAYER_MELEE_ATTACK),
                    player_size,
                    charge_phase.1,
                    *damage,
                    Knockback {
                        force: 7.,
                        direction: *direction,
                    },
                    true,
                ));
            });

            commands.entity(entity).insert(Done::Success);
        } else {
            charge_phase.0.tick(time.delta());
        }
    }
}

pub fn attack_phase_system(
    mut query: Query<(&mut AttackPhase, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut attack_phase, entity) in query.iter_mut() {
        if attack_phase.0.is_finished() {
            commands.entity(entity).insert(Done::Success);
        } else {
            attack_phase.0.tick(time.delta());
        }
    }
}

pub fn recover_phase_system(
    mut query: Query<(&mut RecoverPhase, &mut State, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut recover_phase, mut state, entity) in query.iter_mut() {
        if recover_phase.0.is_finished() {
            state.set(State::Idle);
            commands
                .entity(entity)
                .insert(Done::Success)
                .remove::<StateMachine>()
                .remove::<RecoverPhase>();
        } else {
            recover_phase.0.tick(time.delta());
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

        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

//TODO: Check the collisions in its own system
pub fn projectile_break(mut commands: Commands, query: Query<(Entity, &Breakable)>) {
    for (entity, breakable) in query.iter() {
        if breakable.0 == 0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn tick_cooldown(mut query: Query<&mut Cooldown>, time: Res<Time>) {
    for mut cooldown in query.iter_mut() {
        cooldown.update(time.delta());
    }
}
