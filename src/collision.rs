use bevy::prelude::{
    App, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, Plugin, Query, With,
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{
    attack::{Attack, Damage, MeleeSensor},
    stats::Stats,
    XP,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_event::<HandleCollisionEvent>()
            .add_system(collision_events)
            .add_system(xp_system)
            .add_system(player_attack_system)
            .add_system(enemy_projectile_system);
    }
}

#[derive(Copy, Clone)]
pub struct BodyLayers;

impl BodyLayers {
    pub const PLAYER: Group = Group::GROUP_1;
    pub const ENEMY: Group = Group::GROUP_2;
    pub const PLAYER_ATTACK: Group = Group::GROUP_3;
    pub const ENEMY_ATTACK: Group = Group::GROUP_4;
    pub const XP_LAYER: Group = Group::GROUP_5;
}

pub enum CollisionType {
    PlayerAttack(Entity, Entity),
    XPOnPlayer(Entity, Entity),
    EnemyAttack(Entity, Entity),
}

pub struct HandleCollisionEvent {
    pub collision_type: CollisionType,
    pub started: bool,
}

pub fn collision_events(
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
                (BodyLayers::ENEMY_ATTACK, BodyLayers::PLAYER) => {
                    Some(CollisionType::EnemyAttack(*e1, *e2))
                }
                (BodyLayers::PLAYER, BodyLayers::ENEMY_ATTACK) => {
                    Some(CollisionType::EnemyAttack(*e2, *e1))
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

//TODO: Refractor for generic attack system
pub fn player_attack_system(
    mut events: EventReader<HandleCollisionEvent>,
    mut query: Query<&mut MeleeSensor>,
) {
    for e in events.iter() {
        match e.collision_type {
            CollisionType::PlayerAttack(e_attacker, e_defender) => {
                if let Ok(mut sensor) = query.get_mut(e_attacker) {
                    if e.started {
                        sensor.targets.push(e_defender);
                    } else {
                        sensor.targets.retain(|&e| e == e_defender);
                    }
                }
            }
            _ => {}
        }
    }
}

//TODO: Refractor for generic attack system
pub fn enemy_projectile_system(
    mut commands: Commands,
    mut events: EventReader<HandleCollisionEvent>,
    query: Query<&Damage, With<Attack>>,
    mut stats_query: Query<&mut Stats>,
) {
    for e in events.iter().filter(|e| e.started) {
        match e.collision_type {
            CollisionType::EnemyAttack(e_projectile, e_defender) => {
                if let Ok(damage) = query.get(e_projectile) {
                    if let Ok(mut stats) = stats_query.get_mut(e_defender) {
                        stats.health -= damage.0;
                        commands.entity(e_projectile).despawn_recursive();
                    }
                }
            }
            _ => {}
        }
    }
}
