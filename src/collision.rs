use bevy::prelude::{Commands, DespawnRecursiveExt, Entity, EventReader, Query, With};
use heron::{CollisionData, CollisionEvent, PhysicsLayer};

use crate::{
    attack::MeleeSensor,
    controller::PlayerControlled,
    follow::{Follow, FollowTarget},
    stats::Stats,
    XP,
};

#[derive(PhysicsLayer)]
pub enum Layers {
    Player,
    Enemy,
    Attack,
    XP,
}

pub fn melee_collisions(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut query: Query<&mut MeleeSensor>,
) {
    events.iter().for_each(|e| match e {
        CollisionEvent::Started(d1, d2) => {
            //Enemy entered players attack sensor
            if let Some((sensor, enemy)) = enemy_on_sensor(d1, d2) {
                if let Ok(mut melee_sensor) = query.get_mut(sensor) {
                    melee_sensor.targets.push(enemy);
                }
            } else if let Some((xp, player)) = xp_on_player(d1, d2) {
                commands.entity(xp).insert(Follow {
                    target: FollowTarget::Transform(player),
                    speed: 4.,
                    continous: true,
                    treshhold: 5., //TODO: Check xp hit range
                    ..Default::default()
                });
            }
        }

        CollisionEvent::Stopped(d1, d2) => {
            //Enemy left players attack sensor
            if let Some((sensor, enemy)) = enemy_on_sensor(d1, d2) {
                if let Ok(mut melee_sensor) = query.get_mut(sensor) {
                    melee_sensor.targets.retain(|&entity| entity != enemy);
                }
            }
        }
    });
}

fn enemy_on_sensor(d1: &CollisionData, d2: &CollisionData) -> Option<(Entity, Entity)> {
    if is_attack(d1) && is_enemy(d2) {
        Some((d1.collision_shape_entity(), d2.rigid_body_entity()))
    } else if is_attack(d2) && is_enemy(d1) {
        Some((d2.rigid_body_entity(), d1.collision_shape_entity()))
    } else {
        None
    }
}

fn xp_on_player(d1: &CollisionData, d2: &CollisionData) -> Option<(Entity, Entity)> {
    if is_xp(d1) && is_player(d2) {
        Some((d1.collision_shape_entity(), d2.rigid_body_entity()))
    } else if is_xp(d2) && is_player(d1) {
        Some((d2.rigid_body_entity(), d1.collision_shape_entity()))
    } else {
        None
    }
}

fn is_attack(data: &CollisionData) -> bool {
    data.collision_layers()
        .contains_group(crate::Layers::Attack)
}

fn is_player(data: &CollisionData) -> bool {
    data.collision_layers().contains_group(Layers::Player)
}

fn is_enemy(data: &CollisionData) -> bool {
    data.collision_layers().contains_group(Layers::Enemy)
}

fn is_xp(data: &CollisionData) -> bool {
    data.collision_layers().contains_group(Layers::XP)
}

pub fn xp_system(
    mut commands: Commands,
    query: Query<(&Follow, &XP, Entity)>,
    mut player_query: Query<&mut Stats, With<PlayerControlled>>,
) {
    for (follow, xp, entity) in query.iter() {
        if !follow.on_target() {
            continue;
        }

        player_query.get_single_mut().unwrap().xp += xp.0; //TODO: Prevent panic

        commands.entity(entity).despawn_recursive();
    }
}
