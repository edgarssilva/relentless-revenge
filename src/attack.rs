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

use crate::{
    collision::BodyLayers, movement::direction::Direction, movement::movement::Velocity,
    player::Player, state::State, stats::Stats,
};

#[derive(Component)]
pub struct Attack;

#[derive(Component)]
pub struct Lifetime(pub Timer);

#[derive(Component)]
pub struct Damage(pub u32);

#[derive(Component)]
pub struct Damageable;

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
    pub fn new(size: Vec2, lifetime: Lifetime, damage: Damage, is_player_attack: bool) -> Self {
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
            lifetime,
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
}

impl MeleeAttackBundle {
    pub fn new(
        position: Vec3,
        size: Vec2,
        lifetime: Lifetime,
        damage: Damage,
        is_player_attack: bool,
    ) -> Self {
        Self {
            attack: AttackBundle::new(size, lifetime, damage, is_player_attack),
            transform_bundle: TransformBundle {
                local: Transform::from_translation(position),
                ..default()
            },
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
            attack: AttackBundle::new(
                size,
                Lifetime(Timer::from_seconds(duration, false)),
                damage,
                is_player_attack,
            ),
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
        }
    }
}

pub fn attack_system(
    mut query: Query<(&mut State, &mut AttackPhase, &Direction, &Stats, Entity), With<Player>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut state, mut attack_phase, direction, stats, entity) in query.iter_mut() {
        if !attack_phase.charge.finished() {
            attack_phase.charge.tick(time.delta());

            if attack_phase.charge.just_finished() {
                //TODO: Add player meta so we can get the stats, player size and attack size
                commands.entity(entity).with_children(|children| {
                    let player_size = Vec2::new(64., 64.) / 3.75;
                    let offset = player_size.x * 0.75;

                    children.spawn_bundle(MeleeAttackBundle::new(
                        (direction.vec() * offset).extend(10.),
                        player_size,
                        Lifetime(attack_phase.attack.clone()),
                        Damage(stats.damage),
                        true,
                    ));
                });
            } else {
                return;
            }
        }

        if !attack_phase.attack.finished() {
            attack_phase.attack.tick(time.delta());
            return;
        }

        attack_phase.recover.tick(time.delta());
        if attack_phase.recover.finished() {
            state.set(State::IDLE);
            commands.entity(entity).remove::<AttackPhase>();
        }
    }
}

//TODO: Change attack lifetime to anything lifetime
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
