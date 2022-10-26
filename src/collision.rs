use bevy::prelude::{
    App, Camera, Commands, DespawnRecursiveExt, Entity, EventReader, Plugin, Query, With,
};
use bevy_rapier2d::{prelude::*, rapier::prelude::CollisionEventFlags};

use crate::{
    attack::{Damage, Damageable},
    helper::Shake,
    player::Player,
    stats::Stats,
    XP,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_system(xp_system)
            .add_system(damagable_collision);
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

pub fn xp_system(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    query: Query<&XP>,
    mut player_query: Query<&mut Stats, With<Player>>,
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

        //TODO: Prevent panic from XP
        if let Ok(xp) = query.get(*e1) {
            if let Ok(mut stats) = player_query.get_mut(*e2) {
                stats.xp += xp.0;
                commands.entity(*e1).despawn_recursive();
            }
        } else if let Ok(xp) = query.get(*e2) {
            if let Ok(mut stats) = player_query.get_mut(*e1) {
                stats.xp += xp.0;
                commands.entity(*e2).despawn_recursive();
            }
        }
    });
}

pub fn damagable_collision(
    mut events: EventReader<CollisionEvent>,
    damage_query: Query<&Damage>,
    mut damageable_query: Query<&mut Stats, With<Damageable>>,
    camera_query: Query<Entity, With<Camera>>,
    mut commands: Commands,
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

        //TODO: Check what to do when both entities have damage and damageable
        let collision = match (
            damage_query.contains(*e1) && damageable_query.contains(*e2),
            (damage_query.contains(*e2) && damageable_query.contains(*e1)),
        ) {
            (true, false) => Some((
                damage_query.get(*e1).unwrap(),
                damageable_query.get_mut(*e2).unwrap(),
            )),
            (false, true) => Some((
                damage_query.get(*e2).unwrap(),
                damageable_query.get_mut(*e1).unwrap(),
            )),
            _ => None,
        };

        if let Some((attack_damage, mut collider_stats)) = collision {
            collider_stats.damage(attack_damage.0);

            //Switch this into a shake event
            if let Ok(camera) = camera_query.get_single() {
                commands.entity(camera).insert(Shake {
                    duration: 0.25,
                    strength: 7.5,
                });
            }
        }
    });
}
