use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, RigidBody,
};

use crate::{animation::Animation, collision::BodyLayers, stats::Stats};

#[derive(Bundle)]
pub struct EnemyBundle {
    #[bundle]
    pub sprisheet: SpriteSheetBundle,
    pub stats: Stats,
    pub animation: Animation,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
}

impl EnemyBundle {
    pub fn new(texture_handle: Handle<TextureAtlas>, translation: Vec3) -> Self {
        Self {
            sprisheet: SpriteSheetBundle {
                texture_atlas: texture_handle,
                transform: Transform {
                    translation,
                    scale: Vec3::new(0.25, 0.25, 1.0),
                    ..default()
                },
                ..default()
            },
            stats: Stats::new(100, 20, 20, 2., 5),
            animation: Animation {
                frames: (0..7).collect(),
                current_frame: 0,
                timer: Timer::new(Duration::from_millis(250), true),
            },
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::cuboid(256. * 0.2, 256. * 0.2),
            collision_groups: CollisionGroups::new(BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK),
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
        }
    }
}
