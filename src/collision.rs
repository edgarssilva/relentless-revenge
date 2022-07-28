use bevy::prelude::{
    App, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, Plugin, Query,
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{stats::Stats, XP};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_event::<HandleCollisionEvent>()
            .add_system(melee_collisions)
            .add_system(xp_system);
    }
}

#[derive(Copy, Clone)]
pub struct BodyLayers;

impl BodyLayers {
    pub const PLAYER: u32 = 1 << 0; // 0b00000001
    pub const ENEMY: u32 = 1 << 1; // 0b00000010
    pub const PLAYER_ATTACK: u32 = 1 << 2; // 0b00000100
                                           // pub const ENEMY_ATTACK: u32 = 1 << 3; // 0b00001000
    pub const XP_LAYER: u32 = 1 << 4; // 0b00010000
                                      // pub const ALL: u32 = u32::MAX; // 0b11111111
}

pub enum CollisionType {
    PlayerAttack(Entity, Entity),
    XPOnPlayer(Entity, Entity),
}

pub struct HandleCollisionEvent {
    pub collision_type: CollisionType,
    pub started: bool,
}

pub fn melee_collisions(
    mut events: EventReader<CollisionEvent>,
    mut handle_writer: EventWriter<HandleCollisionEvent>,
    query: Query<&CollisionGroups>,
) {
    events.iter().for_each(|e| {
        let (e1, e2, started, flags) = match e {
            CollisionEvent::Started(e1, e2, flags) => (e1, e2, true, flags),
            CollisionEvent::Stopped(e1, e2, flags) => (e1, e2, false, flags),
        };

        //If entity removed from world, don't handle collision
        if !started && *flags == CollisionEventFlags::REMOVED {
            return;
        }

        let collision_groups = (query.get(*e1), query.get(*e2));

        if let (Ok(cg1), Ok(cg2)) = collision_groups {
            let collision_type = match (cg1.memberships, cg2.memberships) {
                (BodyLayers::PLAYER_ATTACK, BodyLayers::ENEMY) => {
                    Some(CollisionType::PlayerAttack(*e1, *e2))
                }
                (BodyLayers::ENEMY, BodyLayers::PLAYER_ATTACK) => {
                    Some(CollisionType::PlayerAttack(*e2, *e1))
                }
                (BodyLayers::XP_LAYER, BodyLayers::PLAYER) => {
                    Some(CollisionType::XPOnPlayer(*e1, *e2))
                }
                (BodyLayers::PLAYER, BodyLayers::XP_LAYER) => {
                    Some(CollisionType::XPOnPlayer(*e2, *e1))
                }
                _ => None,
            };

            if let Some(collision_type) = collision_type {
                handle_writer.send(HandleCollisionEvent {
                    collision_type,
                    started,
                });
            }
        }
    });
}

pub fn xp_system(
    mut commands: Commands,
    mut events: EventReader<HandleCollisionEvent>,
    query: Query<&XP>,
    mut player_query: Query<&mut Stats>,
) {
    for e in events.iter() {
        match e.collision_type {
            CollisionType::XPOnPlayer(e_exp, e_player) => {
                //TODO: Prevent panic from unwrap
                if let Ok(xp) = query.get(e_exp) {
                    let mut stats = player_query.get_mut(e_player).unwrap();
                    stats.xp += xp.0;
                    commands.entity(e_exp).despawn_recursive();
                }
            }
            _ => {}
        }
    }
}
