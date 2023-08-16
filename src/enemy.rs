use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use seldom_state::prelude::StateMachine;

use crate::effects::Shadow;
use crate::metadata::EnemyMeta;
use crate::sorting::{self, FeetOffset, YSort};
use crate::{
    animation::Animation,
    attack::Damageable,
    collision::BodyLayers,
    stats::{Cooldown, Damage, Health, MovementSpeed, StatsBundle, XP},
};

pub mod state_machine;

pub struct EnemyBehaviourPlugin;

impl Plugin for EnemyBehaviourPlugin {
    fn build(&self, app: &mut App) {
        state_machine::register(app);
    }
}

#[derive(Component)]
pub struct Enemy(pub EnemyMeta);

#[derive(Bundle)]
pub struct EnemyBundle {
    enemy: Enemy,
    pub spritesheet: SpriteSheetBundle,
    pub stats: StatsBundle,
    pub damageable: Damageable,
    pub animation: Animation,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    pub ysort: YSort,
    pub shadow: Shadow,
    pub feet_offset: FeetOffset,
    // finding_player: FindingPLayer,
    state_machine: StateMachine,
}

impl EnemyBundle {
    pub fn new(meta: &EnemyMeta, translation: Vec3) -> Self {
        Self {
            enemy: Enemy(meta.clone()),
            spritesheet: SpriteSheetBundle {
                texture_atlas: meta.texture.atlas_handle.clone(),
                transform: Transform {
                    translation,
                    scale: meta.scale.extend(1.),
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
            ysort: YSort(sorting::ENTITIES_LAYER),
            feet_offset: FeetOffset(meta.feet_offset.unwrap_or_default()),
            shadow: Shadow,
        }
    }
}
