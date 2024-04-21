use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};
use seldom_state::prelude::StateMachine;

use crate::effects::Shadow;
use crate::manifest::enemy::EnemyData;
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
pub struct Enemy(String);

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
    pub fn new(data: &EnemyData, translation: Vec3) -> Self {
        Self {
            enemy: Enemy(data.name.clone()),
            spritesheet: SpriteSheetBundle {
                texture: data.texture.clone(),
                atlas: TextureAtlas {
                    layout: data.atlas.clone(),
                    index: 0,
                },
                transform: Transform {
                    translation,
                    scale: data.scale.extend(1.),
                    ..default()
                },
                ..default()
            },
            stats: StatsBundle {
                health: Health::new(data.health),
                damage: Damage::new(data.damage),
                speed: MovementSpeed::new(data.speed),
                xp: XP::new(data.xp),
                cooldown: Cooldown::new(data.cooldown),
            },
            damageable: Damageable,
            animation: Animation {
                frames: data.frames.clone(),
                current_frame: 0,
                timer: Timer::new(
                    Duration::from_millis(data.frame_duration),
                    TimerMode::Repeating,
                ),
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(data.hitbox.x / 2., data.hitbox.y / 2.),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
            state_machine: state_machine::get_state_machine(),
            ysort: YSort(sorting::ENTITIES_LAYER),
            feet_offset: FeetOffset(data.feet_offset.unwrap_or_default()),
            shadow: Shadow,
        }
    }
}
