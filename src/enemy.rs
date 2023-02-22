use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use seldom_state::prelude::StateMachine;

use crate::{
    animation::Animation,
    attack::Damageable,
    collision::BodyLayers,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
};
use crate::metadata::EnemyMeta;

mod state_machine;

pub struct EnemyBehaviourPlugin;

impl Plugin for EnemyBehaviourPlugin {
    fn build(&self, app: &mut App) {
        state_machine::register(app);
    }
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    #[bundle]
    pub spritesheet: SpriteSheetBundle,
    #[bundle]
    pub stats: StatsBundle,
    pub damageable: Damageable,
    pub animation: Animation,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    // finding_player: FindingPLayer,
    state_machine: StateMachine,
}

impl EnemyBundle {
    pub fn new(meta: &EnemyMeta, translation: Vec3) -> Self {
        Self {
            enemy: Enemy,
            spritesheet: SpriteSheetBundle {
                texture_atlas: meta.texture.atlas_handle.clone(),
                transform: Transform {
                    translation,
                    scale: Vec3::new(0.25, 0.25, 1.0),
                    ..default()
                },
                ..default()
            },
            stats: StatsBundle {
                health: Health::new(meta.health),
                damage: Damage::new(meta.damage),
                speed: MovementSpeed::new(meta.speed),
                xp: XP::new(meta.xp),
                cooldown: Cooldown::new(meta.cooldown),
            },
            damageable: Damageable,
            animation: Animation {
                frames: meta.texture.frames.clone(),
                current_frame: 0,
                timer: Timer::new(
                    Duration::from_millis(meta.texture.duration),
                    TimerMode::Repeating,
                ),
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(meta.hitbox.x / 2., meta.hitbox.y / 2.),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
            state_machine: state_machine::get_state_machine(),
        }
    }
}
